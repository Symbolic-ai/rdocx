//! Deterministic allocation and serialization of tagged PDF structure.

use std::collections::{BTreeMap, HashMap};

use oxml_layout::{DocumentStructure, PageFrame, PositionedElement, StructureId, StructureRole};
use pdf_writer::types::{StructRole, TableHeaderScope};
use pdf_writer::{Finish, Name, Pdf, Ref, TextStr};

#[derive(Debug, Clone, Copy)]
struct MarkOccurrence {
    page_index: usize,
    mcid: i32,
}

#[derive(Debug, Clone, Copy)]
enum StructureKid {
    Element(Ref),
    MarkedContent(MarkOccurrence),
}

pub(crate) struct PreparedStructure<'a> {
    structure: &'a DocumentStructure,
    pub(crate) root_ref: Ref,
    node_refs: BTreeMap<StructureId, Ref>,
    list_body_refs: BTreeMap<StructureId, Ref>,
    page_parent_refs: Vec<Ref>,
    occurrences: BTreeMap<StructureId, Vec<MarkOccurrence>>,
}

impl<'a> PreparedStructure<'a> {
    pub(crate) fn new(
        structure: &'a DocumentStructure,
        pages: &[std::sync::Arc<PageFrame>],
        alloc: &mut impl FnMut() -> Ref,
    ) -> Option<Self> {
        if !valid_structure(structure, pages) {
            return None;
        }
        let root_ref = alloc();
        let node_refs = structure
            .nodes
            .iter()
            .map(|node| (node.id, alloc()))
            .collect();
        let list_body_refs = structure
            .nodes
            .iter()
            .filter(|node| node.role == StructureRole::ListItem)
            .map(|node| (node.id, alloc()))
            .collect();
        let page_parent_refs = pages.iter().map(|_| alloc()).collect();
        let mut occurrences = BTreeMap::new();
        for (page_index, page) in pages.iter().enumerate() {
            let mut mcid = 0;
            collect_marks(&page.elements, page_index, &mut mcid, &mut occurrences);
        }
        Some(Self {
            structure,
            root_ref,
            node_refs,
            list_body_refs,
            page_parent_refs,
            occurrences,
        })
    }

    pub(crate) fn node_ref(&self, id: StructureId) -> Option<Ref> {
        self.node_refs.get(&id).copied()
    }

    pub(crate) fn role_name(&self, id: StructureId) -> Option<&'static [u8]> {
        self.structure.node(id).map(|node| role_name(node.role))
    }

    pub(crate) fn write(&self, pdf: &mut Pdf, page_ids: &[Ref]) {
        let Some(document_ref) = self.node_ref(self.structure.root) else {
            return;
        };

        {
            let mut root = pdf.indirect(self.root_ref).dict();
            root.pair(Name(b"Type"), Name(b"StructTreeRoot"));
            root.pair(Name(b"K"), document_ref);
            root.pair(
                Name(b"ParentTreeNextKey"),
                self.page_parent_refs.len() as i32,
            );
            let mut parent_tree = root.insert(Name(b"ParentTree")).dict();
            let mut nums = parent_tree.insert(Name(b"Nums")).array();
            for (page_index, parent_ref) in self.page_parent_refs.iter().enumerate() {
                nums.item(page_index as i32);
                nums.item(*parent_ref);
            }
            nums.finish();
            parent_tree.finish();
            root.finish();
        }

        let parents = parent_map(self.structure);
        for node in &self.structure.nodes {
            let Some(node_ref) = self.node_ref(node.id) else {
                continue;
            };
            let parent_ref = parents.get(&node.id).map_or(self.root_ref, |parent| {
                self.list_body_refs
                    .get(parent)
                    .copied()
                    .or_else(|| self.node_ref(*parent))
                    .unwrap_or(self.root_ref)
            });
            let mut element = pdf.struct_element(node_ref);
            element.kind(pdf_role(node.role));
            element.parent(parent_ref);
            if let Some(alt) = node.alternate_text.as_deref() {
                element.alt(TextStr(alt));
            }
            if node.role == StructureRole::TableHeaderCell {
                let mut attributes = element.attributes();
                attributes.push().table().scope(TableHeaderScope::Column);
                attributes.finish();
            }
            let marks = self.occurrences.get(&node.id);
            if !node.children.is_empty() || marks.is_some_and(|marks| !marks.is_empty()) {
                let mut children = element.children();
                if node.role == StructureRole::ListItem {
                    if let Some(body_ref) = self.list_body_refs.get(&node.id) {
                        children.struct_element(*body_ref);
                    }
                } else {
                    for kid in self.ordered_kids(node.id) {
                        match kid {
                            StructureKid::Element(reference) => {
                                children.struct_element(reference);
                            }
                            StructureKid::MarkedContent(mark) => {
                                let Some(page_ref) = page_ids.get(mark.page_index).copied() else {
                                    continue;
                                };
                                children
                                    .marked_content_ref()
                                    .page(page_ref)
                                    .marked_content_id(mark.mcid);
                            }
                        }
                    }
                }
                if node.role == StructureRole::ListItem {
                    for mark in marks.into_iter().flatten() {
                        let Some(page_ref) = page_ids.get(mark.page_index).copied() else {
                            continue;
                        };
                        children
                            .marked_content_ref()
                            .page(page_ref)
                            .marked_content_id(mark.mcid);
                    }
                }
                children.finish();
            }
            element.finish();

            if let Some(body_ref) = self.list_body_refs.get(&node.id) {
                let mut body = pdf.struct_element(*body_ref);
                body.kind(StructRole::LBody);
                body.parent(node_ref);
                if !node.children.is_empty() {
                    let mut children = body.children();
                    for child in &node.children {
                        if let Some(child_ref) = self.node_ref(*child) {
                            children.struct_element(child_ref);
                        }
                    }
                    children.finish();
                }
                body.finish();
            }
        }

        for (page_index, parent_ref) in self.page_parent_refs.iter().enumerate() {
            let mut by_mcid = Vec::new();
            for (id, marks) in &self.occurrences {
                for mark in marks.iter().filter(|mark| mark.page_index == page_index) {
                    if let Some(reference) = self.node_ref(*id) {
                        by_mcid.push((mark.mcid, reference));
                    }
                }
            }
            by_mcid.sort_unstable_by_key(|(mcid, _)| *mcid);
            let mut array = pdf.indirect(*parent_ref).array();
            for (_, reference) in by_mcid {
                array.item(reference);
            }
            array.finish();
        }
    }

    fn ordered_kids(&self, id: StructureId) -> Vec<StructureKid> {
        let Some(node) = self.structure.node(id) else {
            return Vec::new();
        };
        if self
            .occurrences
            .get(&id)
            .is_none_or(|marks| marks.is_empty())
        {
            return node
                .children
                .iter()
                .filter_map(|child| self.node_ref(*child).map(StructureKid::Element))
                .collect();
        }
        let mut kids = Vec::new();
        for (index, child) in node.children.iter().enumerate() {
            if let Some(reference) = self.node_ref(*child) {
                kids.push((
                    self.first_occurrence(*child)
                        .unwrap_or((usize::MAX, i32::MAX)),
                    index,
                    StructureKid::Element(reference),
                ));
            }
        }
        let mark_offset = node.children.len();
        for (index, mark) in self
            .occurrences
            .get(&id)
            .into_iter()
            .flatten()
            .copied()
            .enumerate()
        {
            kids.push((
                (mark.page_index, mark.mcid),
                mark_offset + index,
                StructureKid::MarkedContent(mark),
            ));
        }
        kids.sort_by_key(|(order, stable_index, _)| (*order, *stable_index));
        kids.into_iter().map(|(_, _, kid)| kid).collect()
    }

    fn first_occurrence(&self, id: StructureId) -> Option<(usize, i32)> {
        let own = self
            .occurrences
            .get(&id)
            .and_then(|marks| marks.first())
            .map(|mark| (mark.page_index, mark.mcid));
        let child = self
            .structure
            .node(id)?
            .children
            .iter()
            .filter_map(|child| self.first_occurrence(*child))
            .min();
        own.into_iter().chain(child).min()
    }
}

fn valid_structure(structure: &DocumentStructure, pages: &[std::sync::Arc<PageFrame>]) -> bool {
    if structure.nodes.is_empty()
        || structure
            .nodes
            .iter()
            .enumerate()
            .any(|(index, node)| node.id.get() as usize != index + 1)
        || structure
            .node(structure.root)
            .is_none_or(|root| root.role != StructureRole::Document)
    {
        return false;
    }

    let mut parents = HashMap::new();
    for node in &structure.nodes {
        for child in &node.children {
            if structure.node(*child).is_none() || parents.insert(*child, node.id).is_some() {
                return false;
            }
        }
    }
    if parents.contains_key(&structure.root)
        || structure
            .nodes
            .iter()
            .any(|node| node.id != structure.root && !parents.contains_key(&node.id))
    {
        return false;
    }

    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![structure.root];
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            return false;
        }
        let Some(node) = structure.node(id) else {
            return false;
        };
        stack.extend(node.children.iter().copied());
    }
    if seen.len() != structure.nodes.len() {
        return false;
    }

    pages
        .iter()
        .all(|page| marked_ids_exist(&page.elements, structure))
}

fn marked_ids_exist(elements: &[PositionedElement], structure: &DocumentStructure) -> bool {
    elements.iter().all(|element| match element {
        PositionedElement::MarkedContent {
            structure: Some(id),
            children,
        } => structure.node(*id).is_some() && marked_ids_exist(children, structure),
        PositionedElement::MarkedContent { children, .. }
        | PositionedElement::Group(oxml_layout::GroupElement { children, .. }) => {
            marked_ids_exist(children, structure)
        }
        _ => true,
    })
}

fn collect_marks(
    elements: &[PositionedElement],
    page_index: usize,
    next_mcid: &mut i32,
    occurrences: &mut BTreeMap<StructureId, Vec<MarkOccurrence>>,
) {
    for element in elements {
        match element {
            PositionedElement::MarkedContent {
                structure: Some(id),
                children,
            } if has_content_stream_output(children) => {
                occurrences.entry(*id).or_default().push(MarkOccurrence {
                    page_index,
                    mcid: *next_mcid,
                });
                *next_mcid += 1;
                collect_marks(children, page_index, next_mcid, occurrences);
            }
            PositionedElement::MarkedContent { children, .. }
            | PositionedElement::Group(oxml_layout::GroupElement { children, .. }) => {
                collect_marks(children, page_index, next_mcid, occurrences);
            }
            _ => {}
        }
    }
}

pub(crate) fn has_content_stream_output(elements: &[PositionedElement]) -> bool {
    elements.iter().any(|element| match element {
        PositionedElement::Text(run) => !(run.text.is_empty() && run.glyph_ids.is_empty()),
        PositionedElement::LinkAnnotation { .. } => false,
        PositionedElement::Group(group) => has_content_stream_output(&group.children),
        PositionedElement::MarkedContent { children, .. } => has_content_stream_output(children),
        _ => true,
    })
}

fn parent_map(structure: &DocumentStructure) -> HashMap<StructureId, StructureId> {
    structure
        .nodes
        .iter()
        .flat_map(|node| node.children.iter().map(move |child| (*child, node.id)))
        .collect()
}

pub(crate) fn role_name(role: StructureRole) -> &'static [u8] {
    match role {
        StructureRole::Document => b"Document",
        StructureRole::Paragraph => b"P",
        StructureRole::Heading(1) => b"H1",
        StructureRole::Heading(2) => b"H2",
        StructureRole::Heading(3) => b"H3",
        StructureRole::Heading(4) => b"H4",
        StructureRole::Heading(5) => b"H5",
        StructureRole::Heading(_) => b"H6",
        StructureRole::List => b"L",
        StructureRole::ListItem => b"LI",
        StructureRole::Table => b"Table",
        StructureRole::TableRow => b"TR",
        StructureRole::TableHeaderCell => b"TH",
        StructureRole::TableCell => b"TD",
        StructureRole::Figure => b"Figure",
        _ => b"Span",
    }
}

fn pdf_role(role: StructureRole) -> StructRole {
    match role {
        StructureRole::Document => StructRole::Document,
        StructureRole::Paragraph => StructRole::P,
        StructureRole::Heading(1) => StructRole::H1,
        StructureRole::Heading(2) => StructRole::H2,
        StructureRole::Heading(3) => StructRole::H3,
        StructureRole::Heading(4) => StructRole::H4,
        StructureRole::Heading(5) => StructRole::H5,
        StructureRole::Heading(_) => StructRole::H6,
        StructureRole::List => StructRole::L,
        StructureRole::ListItem => StructRole::LI,
        StructureRole::Table => StructRole::Table,
        StructureRole::TableRow => StructRole::TR,
        StructureRole::TableHeaderCell => StructRole::TH,
        StructureRole::TableCell => StructRole::TD,
        StructureRole::Figure => StructRole::Figure,
        _ => StructRole::Span,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oxml_layout::{Color, Rect};

    use super::*;

    fn id(value: u32) -> StructureId {
        StructureId::new(value).expect("non-zero structure id")
    }

    fn paint() -> PositionedElement {
        PositionedElement::FilledRect {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            color: Color::BLACK,
        }
    }

    #[test]
    fn structure_kids_follow_marked_content_document_order() {
        let paragraph = id(2);
        let figure = id(3);
        let structure = DocumentStructure {
            root: id(1),
            nodes: vec![
                oxml_layout::StructureNode {
                    id: id(1),
                    role: StructureRole::Document,
                    children: vec![paragraph],
                    alternate_text: None,
                },
                oxml_layout::StructureNode {
                    id: paragraph,
                    role: StructureRole::Paragraph,
                    children: vec![figure],
                    alternate_text: None,
                },
                oxml_layout::StructureNode {
                    id: figure,
                    role: StructureRole::Figure,
                    children: Vec::new(),
                    alternate_text: Some("chart".to_owned()),
                },
            ],
        };
        let page = PageFrame::new(
            1,
            10.0,
            10.0,
            vec![
                PositionedElement::MarkedContent {
                    structure: Some(paragraph),
                    children: vec![paint()],
                },
                PositionedElement::MarkedContent {
                    structure: Some(figure),
                    children: vec![paint()],
                },
                PositionedElement::MarkedContent {
                    structure: Some(paragraph),
                    children: vec![paint()],
                },
            ],
        );
        let mut next_ref = 0;
        let prepared = PreparedStructure::new(&structure, &[Arc::new(page)], &mut || {
            next_ref += 1;
            Ref::new(next_ref)
        })
        .expect("valid structure");

        let kids = prepared.ordered_kids(paragraph);
        assert!(matches!(
            kids.as_slice(),
            [
                StructureKid::MarkedContent(MarkOccurrence { mcid: 0, .. }),
                StructureKid::Element(_),
                StructureKid::MarkedContent(MarkOccurrence { mcid: 2, .. })
            ]
        ));
    }

    #[test]
    fn empty_marked_content_does_not_allocate_an_mcid() {
        let paragraph = id(2);
        let structure = DocumentStructure {
            root: id(1),
            nodes: vec![
                oxml_layout::StructureNode {
                    id: id(1),
                    role: StructureRole::Document,
                    children: vec![paragraph],
                    alternate_text: None,
                },
                oxml_layout::StructureNode {
                    id: paragraph,
                    role: StructureRole::Paragraph,
                    children: Vec::new(),
                    alternate_text: None,
                },
            ],
        };
        let page = PageFrame::new(
            1,
            10.0,
            10.0,
            vec![PositionedElement::MarkedContent {
                structure: Some(paragraph),
                children: Vec::new(),
            }],
        );
        let mut next_ref = 0;
        let prepared = PreparedStructure::new(&structure, &[Arc::new(page)], &mut || {
            next_ref += 1;
            Ref::new(next_ref)
        })
        .expect("valid structure");

        assert!(!prepared.occurrences.contains_key(&paragraph));
    }

    #[test]
    fn contentless_nodes_keep_semantic_source_order() {
        let empty = id(2);
        let visible = id(3);
        let structure = DocumentStructure {
            root: id(1),
            nodes: vec![
                oxml_layout::StructureNode {
                    id: id(1),
                    role: StructureRole::Document,
                    children: vec![empty, visible],
                    alternate_text: None,
                },
                oxml_layout::StructureNode {
                    id: empty,
                    role: StructureRole::Paragraph,
                    children: Vec::new(),
                    alternate_text: None,
                },
                oxml_layout::StructureNode {
                    id: visible,
                    role: StructureRole::Paragraph,
                    children: Vec::new(),
                    alternate_text: None,
                },
            ],
        };
        let page = PageFrame::new(
            1,
            10.0,
            10.0,
            vec![PositionedElement::MarkedContent {
                structure: Some(visible),
                children: vec![paint()],
            }],
        );
        let mut next_ref = 0;
        let prepared = PreparedStructure::new(&structure, &[Arc::new(page)], &mut || {
            next_ref += 1;
            Ref::new(next_ref)
        })
        .expect("valid structure");

        let kids = prepared.ordered_kids(structure.root);
        assert!(matches!(
            kids.as_slice(),
            [StructureKid::Element(first), StructureKid::Element(second)]
                if *first == prepared.node_ref(empty).unwrap()
                    && *second == prepared.node_ref(visible).unwrap()
        ));
    }

    #[test]
    fn invalid_public_structure_graph_is_rejected() {
        let structure = DocumentStructure {
            root: id(1),
            nodes: vec![
                oxml_layout::StructureNode {
                    id: id(1),
                    role: StructureRole::Document,
                    children: vec![id(2)],
                    alternate_text: None,
                },
                oxml_layout::StructureNode {
                    id: id(2),
                    role: StructureRole::Paragraph,
                    children: vec![id(1)],
                    alternate_text: None,
                },
            ],
        };
        let mut next_ref = 0;
        let prepared = PreparedStructure::new(&structure, &[], &mut || {
            next_ref += 1;
            Ref::new(next_ref)
        });

        assert!(prepared.is_none());
    }
}

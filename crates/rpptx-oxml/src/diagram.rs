//! Bounded DrawingML diagram models with raw preservation.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::ops::Range;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::namespace::A_NS;
use oxml_drawing::text::CT_TextBody;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::namespace::{NamespaceBindings, R_NS, all_attributes};
use crate::relmap::rewrite_exact_rel_ids_with_namespaces;

/// DrawingML diagram namespace URI.
pub const DGM_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/diagram";
const DSP_NS: &str = "http://schemas.microsoft.com/office/drawing/2008/diagram";

type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;
type RawBoundaryChildren = Vec<Vec<Vec<u8>>>;
type NamedDescendant = (String, Vec<u8>, Vec<(String, String)>);

/// The bounded semantic kind of a diagram point.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagramPointKind {
    Node,
    Assistant,
    Presentation,
    Other(String),
}

/// The bounded semantic kind of a diagram connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagramConnectionKind {
    ParentOf,
    PresentationOf,
    PresentationParentOf,
    Other(String),
}

/// The supported layout family inferred from a diagram layout definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagramLayoutFamily {
    List,
    Hierarchy,
    Cycle,
    Relationship,
    Matrix,
    Pyramid,
    Unsupported(String),
}

/// One data-model point with optional DrawingML text.
#[derive(Clone, Debug, PartialEq)]
pub struct DiagramPoint {
    pub model_id: String,
    pub kind: DiagramPointKind,
    pub text: Option<CT_TextBody>,
    attributes: RawAttributes,
    text_attributes: RawAttributes,
    text_source_xml: Option<Vec<u8>>,
    text_inherited_namespaces: Vec<(String, String)>,
    raw_before_text: Vec<Vec<u8>>,
    raw_after_text: Vec<Vec<u8>>,
}

/// One directed data-model connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagramConnection {
    pub model_id: String,
    pub source_id: String,
    pub destination_id: String,
    pub kind: DiagramConnectionKind,
    pub source_order: u32,
    pub destination_order: u32,
    raw_xml: Vec<u8>,
}

/// Relationship identifiers carried by a `dgm:relIds` graphic payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagramRelationshipIds {
    pub data: String,
    pub layout: String,
    pub style: String,
    pub colors: String,
    pub drawing: Option<String>,
    raw_xml: Vec<u8>,
    inherited_namespaces: Vec<(String, String)>,
}

impl DiagramRelationshipIds {
    /// Parses a self-contained `dgm:relIds` element.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_xml_with_namespaces(xml, &[])
    }

    pub(crate) fn from_xml_with_namespaces(
        xml: &[u8],
        inherited: &[(String, String)],
    ) -> Result<Self> {
        let inherited_bindings = NamespaceBindings::from_entries(inherited);
        let (start, _, _) =
            root_and_children_impl(xml, b"relIds", Some(DGM_NS), &inherited_bindings)?;
        let namespaces = inherited_bindings.with_start(&start)?;
        let data = relationship_attribute(&start, &namespaces, b"dm")?;
        let layout = relationship_attribute(&start, &namespaces, b"lo")?;
        let style = relationship_attribute(&start, &namespaces, b"qs")?;
        let colors = relationship_attribute(&start, &namespaces, b"cs")?;
        Ok(Self {
            data,
            layout,
            style,
            colors,
            drawing: None,
            raw_xml: xml.to_vec(),
            inherited_namespaces: inherited.to_vec(),
        })
    }

    /// Returns the preserved relationship payload bytes.
    pub fn to_xml(&self) -> Vec<u8> {
        self.raw_xml.clone()
    }

    /// Rewrites relationship ids after a relationship scope is copied.
    pub fn remap(&mut self, mapping: &HashMap<String, String>) -> Result<()> {
        self.raw_xml = rewrite_exact_rel_ids_with_namespaces(
            &self.raw_xml,
            mapping,
            &self.inherited_namespaces,
        )?;
        for value in [
            &mut self.data,
            &mut self.layout,
            &mut self.style,
            &mut self.colors,
        ] {
            if let Some(replacement) = mapping.get(value) {
                *value = replacement.clone();
            }
        }
        if let Some(value) = &mut self.drawing
            && let Some(replacement) = mapping.get(value)
        {
            *value = replacement.clone();
        }
        Ok(())
    }
}

/// Typed projection of a `dgm:dataModel` part.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_DiagramData {
    points: Vec<DiagramPoint>,
    connections: Vec<DiagramConnection>,
    point_list_attributes: RawAttributes,
    point_list_raw_children: RawBoundaryChildren,
    connection_list_attributes: RawAttributes,
    connection_list_raw_children: RawBoundaryChildren,
    drawing_relationship_id: Option<String>,
    namespaces: Vec<(String, String)>,
    raw_xml: Vec<u8>,
    attributes: RawAttributes,
    raw_children: [Vec<Vec<u8>>; 6],
    dirty: bool,
}

impl CT_DiagramData {
    /// Parses a complete diagram data-model part.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let (start, children) = root_and_children(xml, b"dataModel")?;
        let namespaces = NamespaceBindings::default().with_start(&start)?;
        let drawing_relationship_id = data_model_drawing_relationship_id(&children, &namespaces)?;
        let attributes = retained_root_attributes(&start)?;
        let mut points = Vec::new();
        let mut connections = Vec::new();
        let mut point_list_attributes = Vec::new();
        let mut point_list_raw_children = vec![Vec::new()];
        let mut connection_list_attributes = Vec::new();
        let mut connection_list_raw_children = vec![Vec::new()];
        let mut raw_children: [Vec<Vec<u8>>; 6] = Default::default();
        let mut boundary = 0usize;
        let mut seen_point_list = false;
        let mut seen_connection_list = false;
        let mut seen_background = false;
        let mut seen_whole = false;
        let mut seen_extensions = false;
        for raw in children {
            let local = first_local_name(&raw)?;
            match (
                local.as_deref(),
                first_element_uri(&raw, &namespaces)?.as_deref(),
            ) {
                (Some(b"ptLst"), Some(DGM_NS)) => {
                    if seen_point_list || boundary > 0 {
                        return Err(order("dgm:ptLst is duplicated or out of schema order"));
                    }
                    let parsed = parse_points(&raw, &namespaces)?;
                    points = parsed.0;
                    point_list_attributes = parsed.1;
                    point_list_raw_children = parsed.2;
                    seen_point_list = true;
                    boundary = 1;
                }
                (Some(b"cxnLst"), Some(DGM_NS)) => {
                    if !seen_point_list || seen_connection_list || boundary > 1 {
                        return Err(order("dgm:cxnLst is duplicated or out of schema order"));
                    }
                    let parsed = parse_connections(&raw, &namespaces)?;
                    connections = parsed.0;
                    connection_list_attributes = parsed.1;
                    connection_list_raw_children = parsed.2;
                    seen_connection_list = true;
                    boundary = 2;
                }
                (Some(b"bg"), Some(DGM_NS)) => {
                    if !seen_point_list || seen_background || boundary > 2 {
                        return Err(order("dgm:bg is duplicated or out of schema order"));
                    }
                    raw_children[2].push(raw);
                    seen_background = true;
                    boundary = 3;
                }
                (Some(b"whole"), Some(DGM_NS)) => {
                    if !seen_point_list || seen_whole || boundary > 3 {
                        return Err(order("dgm:whole is duplicated or out of schema order"));
                    }
                    raw_children[3].push(raw);
                    seen_whole = true;
                    boundary = 4;
                }
                (Some(b"extLst"), Some(DGM_NS)) => {
                    if !seen_point_list || seen_extensions || boundary > 4 {
                        return Err(order("dgm:extLst is duplicated or out of schema order"));
                    }
                    raw_children[4].push(raw);
                    seen_extensions = true;
                    boundary = 5;
                }
                _ => raw_children[boundary].push(raw),
            }
        }
        if !seen_point_list {
            return Err(missing("dgm:ptLst"));
        }
        Ok(Self {
            points,
            connections,
            point_list_attributes,
            point_list_raw_children,
            connection_list_attributes,
            connection_list_raw_children,
            drawing_relationship_id,
            namespaces: namespaces.entries(),
            raw_xml: xml.to_vec(),
            attributes,
            raw_children,
            dirty: false,
        })
    }

    /// Returns the exact opened bytes until a supported field is changed.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        if !self.dirty {
            return Ok(self.raw_xml.clone());
        }
        let root_namespaces = NamespaceBindings::from_entries(&self.namespaces);
        reject_diagram_writer_conflicts(&root_namespaces)?;
        let point_list_namespaces =
            namespaces_with_attributes(&root_namespaces, &self.point_list_attributes)?;
        reject_diagram_writer_conflicts(&point_list_namespaces)?;
        for point in &self.points {
            reject_diagram_writer_conflicts(&namespaces_with_attributes(
                &point_list_namespaces,
                &point.attributes,
            )?)?;
        }
        let connection_list_namespaces =
            namespaces_with_attributes(&root_namespaces, &self.connection_list_attributes)?;
        reject_diagram_writer_conflicts(&connection_list_namespaces)?;
        let mut writer = Writer::new(Vec::new());
        let mut start = diagram_root("dgm:dataModel");
        push_attributes(&mut start, &self.attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(&mut writer, &self.raw_children[0])?;
        let mut point_list = BytesStart::new("dgm:ptLst");
        push_attributes(&mut point_list, &self.point_list_attributes);
        writer.write_event(Event::Start(point_list))?;
        for (index, point) in self.points.iter().enumerate() {
            emit_raw(&mut writer, &self.point_list_raw_children[index])?;
            point.write_xml(&mut writer)?;
        }
        emit_raw(
            &mut writer,
            &self.point_list_raw_children[self.points.len()],
        )?;
        writer.write_event(Event::End(BytesEnd::new("dgm:ptLst")))?;
        emit_raw(&mut writer, &self.raw_children[1])?;
        let mut connection_list = BytesStart::new("dgm:cxnLst");
        push_attributes(&mut connection_list, &self.connection_list_attributes);
        writer.write_event(Event::Start(connection_list))?;
        for (index, connection) in self.connections.iter().enumerate() {
            emit_raw(&mut writer, &self.connection_list_raw_children[index])?;
            writer.get_mut().write_all(&connection.raw_xml)?;
        }
        emit_raw(
            &mut writer,
            &self.connection_list_raw_children[self.connections.len()],
        )?;
        writer.write_event(Event::End(BytesEnd::new("dgm:cxnLst")))?;
        emit_raw(&mut writer, &self.raw_children[2])?;
        emit_raw(&mut writer, &self.raw_children[3])?;
        emit_raw(&mut writer, &self.raw_children[4])?;
        emit_raw(&mut writer, &self.raw_children[5])?;
        writer.write_event(Event::End(BytesEnd::new("dgm:dataModel")))?;
        Ok(writer.into_inner())
    }

    /// Returns points in data-model order.
    pub fn points(&self) -> &[DiagramPoint] {
        &self.points
    }

    /// Returns directed connections in source order.
    pub fn connections(&self) -> &[DiagramConnection] {
        &self.connections
    }

    /// Returns the producing-scope cached drawing relationship id.
    pub fn drawing_relationship_id(&self) -> Option<&str> {
        self.drawing_relationship_id.as_deref()
    }

    /// Remaps the cached drawing id while preserving every other source byte.
    pub fn remap_drawing_relationship(&mut self, mapping: &HashMap<String, String>) -> Result<()> {
        let xml = self.to_xml()?;
        let rewritten = rewrite_data_model_drawing_relationship(&xml, mapping)?;
        *self = Self::from_xml(&rewritten)?;
        Ok(())
    }

    /// Returns the preserved background subtree, when present.
    pub fn background_xml(&self) -> Option<&[u8]> {
        let namespaces = NamespaceBindings::from_entries(&self.namespaces);
        self.raw_children[2]
            .iter()
            .find(|raw| {
                first_local_name(raw).ok().flatten().as_deref() == Some(b"bg")
                    && first_element_uri(raw, &namespaces)
                        .ok()
                        .flatten()
                        .as_deref()
                        == Some(DGM_NS)
            })
            .map(Vec::as_slice)
    }

    /// Replaces the text of a supported node, retaining its remaining XML.
    pub fn set_node_text(&mut self, model_id: &str, text: &str) -> Result<()> {
        self.validate_unique_model_ids()?;
        let point = self
            .points
            .iter_mut()
            .find(|point| point.model_id == model_id)
            .ok_or_else(|| OxmlError::InvalidValue(format!("unknown diagram point {model_id}")))?;
        if !matches!(
            point.kind,
            DiagramPointKind::Node | DiagramPointKind::Assistant
        ) {
            return Err(OxmlError::InvalidValue(format!(
                "diagram point {model_id} is not an editable node"
            )));
        }
        let body = point
            .text
            .as_mut()
            .ok_or_else(|| OxmlError::MissingElement(format!("diagram point {model_id} text")))?;
        body.set_text(text);
        self.dirty = true;
        Ok(())
    }

    /// Moves one point in data-model order after validating both indices.
    pub fn move_point(&mut self, from: usize, to: usize) -> Result<()> {
        self.validate_unique_model_ids()?;
        if from >= self.points.len() || to >= self.points.len() {
            return Err(OxmlError::InvalidValue(format!(
                "diagram point move {from} to {to} exceeds {} points",
                self.points.len()
            )));
        }
        if from != to {
            let point = self.points.remove(from);
            self.points.insert(to, point);
            self.dirty = true;
        }
        Ok(())
    }

    fn validate_unique_model_ids(&self) -> Result<()> {
        let mut point_ids = HashSet::new();
        for point in &self.points {
            if !point_ids.insert(point.model_id.as_str()) {
                return Err(OxmlError::InvalidValue(format!(
                    "duplicate diagram point modelId {}",
                    point.model_id
                )));
            }
        }
        let mut connection_ids = HashSet::new();
        for connection in &self.connections {
            if !connection_ids.insert(connection.model_id.as_str()) {
                return Err(OxmlError::InvalidValue(format!(
                    "duplicate diagram connection modelId {}",
                    connection.model_id
                )));
            }
        }
        Ok(())
    }
}

impl DiagramPoint {
    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("dgm:pt");
        start.push_attribute(("modelId", self.model_id.as_str()));
        let lexical_kind = point_kind_lexical(&self.kind);
        if lexical_kind != "node" {
            start.push_attribute(("type", lexical_kind.as_str()));
        }
        push_unmodelled_attributes(&mut start, &self.attributes, &["modelId", "type"]);
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, &self.raw_before_text)?;
        if let Some(text) = &self.text {
            if let Some(source) = &self.text_source_xml {
                validate_diagram_text_namespaces(
                    source,
                    &NamespaceBindings::from_entries(&self.text_inherited_namespaces),
                )?;
            }
            write_diagram_text(writer, text, &self.text_attributes)?;
        }
        emit_raw(writer, &self.raw_after_text)?;
        writer.write_event(Event::End(BytesEnd::new("dgm:pt")))?;
        Ok(())
    }
}

/// Typed projection of a `dgm:layoutDef` part.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_DiagramLayoutDefinition {
    pub unique_id: Option<String>,
    pub title: Option<String>,
    pub family: DiagramLayoutFamily,
    pub algorithms: Vec<String>,
    pub constraints: Vec<String>,
    pub categories: Vec<String>,
    raw_xml: Vec<u8>,
}

impl CT_DiagramLayoutDefinition {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let (start, children) = root_and_children(xml, b"layoutDef")?;
        let namespaces = NamespaceBindings::default().with_start(&start)?;
        let attributes = all_attributes(&start)?;
        let unique_id = unqualified_attribute(&attributes, "uniqueId");
        let title = direct_child_attribute(&children, &namespaces, DGM_NS, b"title", "val")?;
        let categories = direct_list_attribute_values(
            &children,
            &namespaces,
            DGM_NS,
            b"catLst",
            b"cat",
            "type",
        )?;
        let mut algorithms = Vec::new();
        let mut constraints = Vec::new();
        for raw in direct_children(&children, &namespaces, DGM_NS, b"layoutNode")? {
            collect_layout_projection(&raw, &namespaces, &mut algorithms, &mut constraints)?;
        }
        let family = infer_layout_family(
            unique_id.as_deref(),
            title.as_deref(),
            &categories,
            &algorithms,
        );
        Ok(Self {
            unique_id,
            title,
            family,
            algorithms,
            constraints,
            categories,
            raw_xml: xml.to_vec(),
        })
    }

    pub fn to_xml(&self) -> Vec<u8> {
        self.raw_xml.clone()
    }
}

/// One named quick-style label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagramStyleLabel {
    pub name: String,
    pub shape_style: Option<DiagramShapeStyle>,
}

/// Theme matrix references applied by one quick-style label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagramShapeStyle {
    pub line_reference: Option<u32>,
    pub fill_reference: Option<u32>,
    pub effect_reference: Option<u32>,
    pub font_reference: Option<String>,
}

/// Typed projection of a `dgm:styleDef` part.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_DiagramStyleDefinition {
    pub unique_id: Option<String>,
    pub labels: Vec<DiagramStyleLabel>,
    raw_xml: Vec<u8>,
}

impl CT_DiagramStyleDefinition {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let (start, children) = root_and_children(xml, b"styleDef")?;
        let namespaces = NamespaceBindings::default().with_start(&start)?;
        let unique_id = unqualified_attribute(&all_attributes(&start)?, "uniqueId");
        let labels = named_direct_children(&children, &namespaces, DGM_NS, b"styleLbl")?
            .into_iter()
            .map(|(name, raw, namespaces)| DiagramStyleLabel {
                name,
                shape_style: parse_shape_style(&raw, &namespaces).ok().flatten(),
            })
            .collect();
        Ok(Self {
            unique_id,
            labels,
            raw_xml: xml.to_vec(),
        })
    }

    pub fn to_xml(&self) -> Vec<u8> {
        self.raw_xml.clone()
    }
}

/// One named diagram colour label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagramColorLabel {
    pub name: String,
    pub colors: Vec<String>,
    pub fill_colors: Vec<String>,
    pub line_colors: Vec<String>,
    pub effect_colors: Vec<String>,
    pub text_fill_colors: Vec<String>,
}

/// Typed projection of a `dgm:colorsDef` part.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_DiagramColorsDefinition {
    pub unique_id: Option<String>,
    pub labels: Vec<DiagramColorLabel>,
    raw_xml: Vec<u8>,
}

impl CT_DiagramColorsDefinition {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let (start, children) = root_and_children(xml, b"colorsDef")?;
        let namespaces = NamespaceBindings::default().with_start(&start)?;
        let unique_id = unqualified_attribute(&all_attributes(&start)?, "uniqueId");
        let labels = named_direct_children(&children, &namespaces, DGM_NS, b"styleLbl")?
            .into_iter()
            .map(|(name, raw, namespaces)| {
                let fill_colors =
                    colors_in_list(&raw, &namespaces, b"fillClrLst").unwrap_or_default();
                let line_colors =
                    colors_in_list(&raw, &namespaces, b"linClrLst").unwrap_or_default();
                let effect_colors =
                    colors_in_list(&raw, &namespaces, b"effectClrLst").unwrap_or_default();
                let text_fill_colors =
                    colors_in_list(&raw, &namespaces, b"txFillClrLst").unwrap_or_default();
                let colors = fill_colors
                    .iter()
                    .chain(&line_colors)
                    .chain(&effect_colors)
                    .chain(&text_fill_colors)
                    .cloned()
                    .collect();
                DiagramColorLabel {
                    name,
                    colors,
                    fill_colors,
                    line_colors,
                    effect_colors,
                    text_fill_colors,
                }
            })
            .collect();
        Ok(Self {
            unique_id,
            labels,
            raw_xml: xml.to_vec(),
        })
    }

    pub fn to_xml(&self) -> Vec<u8> {
        self.raw_xml.clone()
    }
}

/// Typed root for a cached `dsp:drawing` diagram shape tree.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_DiagramDrawing {
    pub shape_count: usize,
    raw_xml: Vec<u8>,
}

impl CT_DiagramDrawing {
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let (start, children, _) =
            root_and_children_impl(xml, b"drawing", Some(DSP_NS), &NamespaceBindings::default())?;
        let namespaces = NamespaceBindings::default().with_start(&start)?;
        let mut shape_count = 0;
        for shape_tree in direct_children(&children, &namespaces, DSP_NS, b"spTree")? {
            let (start, children, _) =
                root_and_children_impl(&shape_tree, b"spTree", Some(DSP_NS), &namespaces)?;
            let shape_tree_namespaces = namespaces.with_start(&start)?;
            shape_count += direct_children(&children, &shape_tree_namespaces, DSP_NS, b"sp")?.len();
        }
        Ok(Self {
            shape_count,
            raw_xml: xml.to_vec(),
        })
    }

    pub fn to_xml(&self) -> Vec<u8> {
        self.raw_xml.clone()
    }
}

fn parse_points(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<(Vec<DiagramPoint>, RawAttributes, RawBoundaryChildren)> {
    let (start, children) = root_and_children_or_empty_any_namespace(xml, b"ptLst")?;
    let namespaces = inherited.with_start(&start)?;
    let attributes = all_attributes(&start)?;
    let mut points = Vec::new();
    let mut raw_children = vec![Vec::new()];
    for raw in children {
        if first_local_name(&raw)?.as_deref() == Some(b"pt")
            && first_element_uri(&raw, &namespaces)?.as_deref() == Some(DGM_NS)
        {
            points.push(parse_point(&raw, &namespaces)?);
            raw_children.push(Vec::new());
        } else {
            raw_children
                .last_mut()
                .ok_or_else(|| missing("diagram point-list raw boundary"))?
                .push(raw);
        }
    }
    Ok((points, attributes, raw_children))
}

fn parse_point(xml: &[u8], inherited: &NamespaceBindings) -> Result<DiagramPoint> {
    let (start, children) = root_and_children_in_any_namespace(xml, b"pt")?;
    let namespaces = inherited.with_start(&start)?;
    let attributes = all_attributes(&start)?;
    let model_id = required_unqualified_attribute(&attributes, "modelId")?;
    let kind = point_kind(
        unqualified_attribute(&attributes, "type")
            .as_deref()
            .unwrap_or("node"),
    );
    let mut text = None;
    let mut text_attributes = Vec::new();
    let mut text_source_xml = None;
    let mut text_inherited_namespaces = Vec::new();
    let mut raw_before_text = Vec::new();
    let mut raw_after_text = Vec::new();
    let mut passed_text = false;
    for raw in children {
        if first_local_name(&raw)?.as_deref() == Some(b"t")
            && first_element_uri(&raw, &namespaces)?.as_deref() == Some(DGM_NS)
            && text.is_none()
        {
            passed_text = true;
            if validate_diagram_text_namespaces(&raw, &namespaces).is_ok() {
                let (text_start, _) = root_and_children_in_any_namespace(&raw, b"t")?;
                text_attributes = all_attributes(&text_start)?;
                text = Some(
                    CT_TextBody::from_xml_as(&raw, b"t")
                        .map_err(|error| OxmlError::InvalidValue(error.to_string()))?,
                );
                text_source_xml = Some(raw);
                text_inherited_namespaces = namespaces.entries();
            } else {
                raw_before_text.push(raw);
            }
        } else if passed_text {
            raw_after_text.push(raw);
        } else {
            raw_before_text.push(raw);
        }
    }
    Ok(DiagramPoint {
        model_id,
        kind,
        text,
        attributes,
        text_attributes,
        text_source_xml,
        text_inherited_namespaces,
        raw_before_text,
        raw_after_text,
    })
}

fn parse_connections(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<(Vec<DiagramConnection>, RawAttributes, RawBoundaryChildren)> {
    let (start, children) = root_and_children_or_empty_any_namespace(xml, b"cxnLst")?;
    let namespaces = inherited.with_start(&start)?;
    let attributes = all_attributes(&start)?;
    let mut connections = Vec::new();
    let mut raw_children = vec![Vec::new()];
    for raw in children {
        if first_local_name(&raw)?.as_deref() == Some(b"cxn")
            && first_element_uri(&raw, &namespaces)?.as_deref() == Some(DGM_NS)
        {
            let (start, _) = root_and_children_or_empty_any_namespace(&raw, b"cxn")?;
            let attributes = all_attributes(&start)?;
            connections.push(DiagramConnection {
                model_id: required_unqualified_attribute(&attributes, "modelId")?,
                source_id: required_unqualified_attribute(&attributes, "srcId")?,
                destination_id: required_unqualified_attribute(&attributes, "destId")?,
                kind: connection_kind(
                    unqualified_attribute(&attributes, "type")
                        .as_deref()
                        .unwrap_or("parOf"),
                ),
                source_order: parsed_unqualified_attribute(&attributes, "srcOrd")?
                    .unwrap_or_default(),
                destination_order: parsed_unqualified_attribute(&attributes, "destOrd")?
                    .unwrap_or_default(),
                raw_xml: raw,
            });
            raw_children.push(Vec::new());
        } else {
            raw_children
                .last_mut()
                .ok_or_else(|| missing("diagram connection-list raw boundary"))?
                .push(raw);
        }
    }
    Ok((connections, attributes, raw_children))
}

fn root_and_children(xml: &[u8], expected: &[u8]) -> Result<(BytesStart<'static>, Vec<Vec<u8>>)> {
    let (start, children, empty) =
        root_and_children_impl(xml, expected, Some(DGM_NS), &NamespaceBindings::default())?;
    if empty {
        return Err(missing(&format!(
            "children of {}",
            String::from_utf8_lossy(expected)
        )));
    }
    Ok((start, children))
}

fn root_and_children_in_any_namespace(
    xml: &[u8],
    expected: &[u8],
) -> Result<(BytesStart<'static>, Vec<Vec<u8>>)> {
    let (start, children, empty) =
        root_and_children_impl(xml, expected, None, &NamespaceBindings::default())?;
    if empty {
        return Err(missing(&format!(
            "children of {}",
            String::from_utf8_lossy(expected)
        )));
    }
    Ok((start, children))
}

fn root_and_children_or_empty_any_namespace(
    xml: &[u8],
    expected: &[u8],
) -> Result<(BytesStart<'static>, Vec<Vec<u8>>)> {
    let (start, children, _) =
        root_and_children_impl(xml, expected, None, &NamespaceBindings::default())?;
    Ok((start, children))
}

fn root_and_children_impl(
    xml: &[u8],
    expected: &[u8],
    required_namespace: Option<&str>,
    inherited: &NamespaceBindings,
) -> Result<(BytesStart<'static>, Vec<Vec<u8>>, bool)> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().check_comments = true;
    let mut buffer = Vec::new();
    let mut declaration_allowed = true;
    let mut seen_declaration = false;
    let mut seen_doctype = false;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Decl(_) if declaration_allowed && !seen_declaration => {
                seen_declaration = true;
                declaration_allowed = false;
            }
            Event::DocType(_) if !seen_doctype => {
                seen_doctype = true;
                declaration_allowed = false;
            }
            Event::Text(text) if text.as_ref().iter().all(u8::is_ascii_whitespace) => {
                declaration_allowed = false;
            }
            Event::Comment(_) | Event::PI(_) => {
                declaration_allowed = false;
            }
            Event::Start(start) => {
                let namespaces = inherited.with_start(&start)?;
                if local_name(start.name().as_ref()) != expected
                    || required_namespace.is_some_and(|uri| {
                        namespaces.element_uri(start.name().as_ref()) != Some(uri)
                    })
                {
                    return Err(unexpected(&start));
                }
                let owned = start.into_owned();
                let mut children = Vec::new();
                loop {
                    buffer.clear();
                    match reader.read_event_into(&mut buffer)? {
                        Event::Start(child) => children.push(capture_element(&mut reader, &child)?),
                        Event::Empty(child) => children.push(capture_empty_element(&child)?),
                        Event::End(end) if local_name(end.name().as_ref()) == expected => {
                            ensure_document_end(&mut reader, &mut buffer)?;
                            return Ok((owned, children, false));
                        }
                        Event::Decl(_) | Event::DocType(_) => {
                            return Err(OxmlError::InvalidValue(
                                "diagram XML declaration or document type is inside the root"
                                    .to_owned(),
                            ));
                        }
                        Event::Eof => return Err(missing("closing diagram root")),
                        event => children.push(capture_direct_event(event)?),
                    }
                }
            }
            Event::Empty(start) => {
                let namespaces = inherited.with_start(&start)?;
                if local_name(start.name().as_ref()) != expected
                    || required_namespace.is_some_and(|uri| {
                        namespaces.element_uri(start.name().as_ref()) != Some(uri)
                    })
                {
                    return Err(unexpected(&start));
                }
                let owned = start.into_owned();
                ensure_document_end(&mut reader, &mut buffer)?;
                return Ok((owned, Vec::new(), true));
            }
            Event::Eof => return Err(missing(&String::from_utf8_lossy(expected))),
            _ => {
                return Err(OxmlError::InvalidValue(
                    "content is not allowed outside the diagram root".to_owned(),
                ));
            }
        }
        buffer.clear();
    }
}

fn ensure_document_end(reader: &mut Reader<&[u8]>, buffer: &mut Vec<u8>) -> Result<()> {
    loop {
        buffer.clear();
        match reader.read_event_into(buffer)? {
            Event::Text(text) if text.as_ref().iter().all(u8::is_ascii_whitespace) => {}
            Event::Comment(_) | Event::PI(_) => {}
            Event::Eof => return Ok(()),
            _ => {
                return Err(OxmlError::InvalidValue(
                    "content is not allowed after the diagram root".to_owned(),
                ));
            }
        }
    }
}

fn capture_direct_event(event: Event<'_>) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    writer.write_event(event.into_owned())?;
    Ok(writer.into_inner())
}

fn first_local_name(xml: &[u8]) -> Result<Option<Vec<u8>>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) | Event::Empty(start) => {
                return Ok(Some(local_name(start.name().as_ref()).to_vec()));
            }
            Event::Eof => return Ok(None),
            _ => {}
        }
        buffer.clear();
    }
}

fn first_element_uri(xml: &[u8], inherited: &NamespaceBindings) -> Result<Option<String>> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) | Event::Empty(start) => {
                let namespaces = inherited.with_start(&start)?;
                return Ok(namespaces
                    .element_uri(start.name().as_ref())
                    .map(str::to_owned));
            }
            Event::Eof => return Ok(None),
            _ => {}
        }
        buffer.clear();
    }
}

fn direct_children(
    children: &[Vec<u8>],
    inherited: &NamespaceBindings,
    wanted_uri: &str,
    wanted: &[u8],
) -> Result<Vec<Vec<u8>>> {
    let mut values = Vec::new();
    for raw in children {
        if first_local_name(raw)?.as_deref() == Some(wanted)
            && first_element_uri(raw, inherited)?.as_deref() == Some(wanted_uri)
        {
            values.push(raw.clone());
        }
    }
    Ok(values)
}

fn named_direct_children(
    children: &[Vec<u8>],
    inherited: &NamespaceBindings,
    wanted_uri: &str,
    wanted: &[u8],
) -> Result<Vec<NamedDescendant>> {
    direct_children(children, inherited, wanted_uri, wanted)?
        .into_iter()
        .map(|raw| {
            let (start, _, _) = root_and_children_impl(&raw, wanted, Some(wanted_uri), inherited)?;
            let scope = inherited.with_start(&start)?;
            let name = unqualified_attribute(&all_attributes(&start)?, "name").unwrap_or_default();
            Ok((name, raw, scope.entries()))
        })
        .collect()
}

fn direct_child_attribute(
    children: &[Vec<u8>],
    inherited: &NamespaceBindings,
    wanted_uri: &str,
    wanted: &[u8],
    attribute: &str,
) -> Result<Option<String>> {
    let Some(raw) = direct_children(children, inherited, wanted_uri, wanted)?
        .into_iter()
        .next()
    else {
        return Ok(None);
    };
    let (start, _, _) = root_and_children_impl(&raw, wanted, Some(wanted_uri), inherited)?;
    Ok(unqualified_attribute(&all_attributes(&start)?, attribute))
}

fn direct_list_attribute_values(
    children: &[Vec<u8>],
    inherited: &NamespaceBindings,
    list_uri: &str,
    list_name: &[u8],
    item_name: &[u8],
    attribute: &str,
) -> Result<Vec<String>> {
    let mut values = Vec::new();
    for list in direct_children(children, inherited, list_uri, list_name)? {
        let (start, item_children, _) =
            root_and_children_impl(&list, list_name, Some(list_uri), inherited)?;
        let scope = inherited.with_start(&start)?;
        for item in direct_children(&item_children, &scope, list_uri, item_name)? {
            let (item_start, _, _) =
                root_and_children_impl(&item, item_name, Some(list_uri), &scope)?;
            if let Some(value) = unqualified_attribute(&all_attributes(&item_start)?, attribute) {
                values.push(value);
            }
        }
    }
    Ok(values)
}

fn data_model_drawing_relationship_id(
    children: &[Vec<u8>],
    inherited: &NamespaceBindings,
) -> Result<Option<String>> {
    for extension_list in direct_children(children, inherited, DGM_NS, b"extLst")? {
        let (start, extensions, _) =
            root_and_children_impl(&extension_list, b"extLst", Some(DGM_NS), inherited)?;
        let extension_scope = inherited.with_start(&start)?;
        for extension in direct_children(&extensions, &extension_scope, A_NS, b"ext")? {
            let (start, payloads, _) =
                root_and_children_impl(&extension, b"ext", Some(A_NS), &extension_scope)?;
            let payload_scope = extension_scope.with_start(&start)?;
            if let Some(value) =
                direct_child_attribute(&payloads, &payload_scope, DSP_NS, b"dataModelExt", "relId")?
            {
                return Ok(Some(value));
            }
        }
    }
    Ok(None)
}

fn collect_layout_projection(
    xml: &[u8],
    inherited: &NamespaceBindings,
    algorithms: &mut Vec<String>,
    constraints: &mut Vec<String>,
) -> Result<()> {
    let root = first_local_name(xml)?.ok_or_else(|| missing("diagram layout container"))?;
    let (start, children, _) = root_and_children_impl(xml, &root, Some(DGM_NS), inherited)?;
    let scope = inherited.with_start(&start)?;
    for raw in children {
        let local = first_local_name(&raw)?;
        let uri = first_element_uri(&raw, &scope)?;
        if uri.as_deref() != Some(DGM_NS) {
            continue;
        }
        match local.as_deref() {
            Some(b"alg") => {
                let (item, _, _) = root_and_children_impl(&raw, b"alg", Some(DGM_NS), &scope)?;
                if let Some(value) = unqualified_attribute(&all_attributes(&item)?, "type") {
                    algorithms.push(value);
                }
            }
            Some(b"constrLst") => constraints.extend(direct_list_attribute_values(
                std::slice::from_ref(&raw),
                &scope,
                DGM_NS,
                b"constrLst",
                b"constr",
                "type",
            )?),
            Some(b"layoutNode") | Some(b"forEach") | Some(b"choose") | Some(b"if")
            | Some(b"else") => {
                collect_layout_projection(&raw, &scope, algorithms, constraints)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn colors_in_list(
    xml: &[u8],
    inherited: &[(String, String)],
    wanted: &[u8],
) -> Result<Vec<String>> {
    let inherited = NamespaceBindings::from_entries(inherited);
    let (start, children, _) = root_and_children_impl(xml, b"styleLbl", Some(DGM_NS), &inherited)?;
    let scope = inherited.with_start(&start)?;
    let Some(list) = direct_children(&children, &scope, DGM_NS, wanted)?
        .into_iter()
        .next()
    else {
        return Ok(Vec::new());
    };
    let (list_start, color_children, _) =
        root_and_children_impl(&list, wanted, Some(DGM_NS), &scope)?;
    let list_scope = scope.with_start(&list_start)?;
    let mut values = Vec::new();
    for raw in color_children {
        let Some(local) = first_local_name(&raw)? else {
            continue;
        };
        if !matches!(
            local.as_slice(),
            b"srgbClr" | b"schemeClr" | b"sysClr" | b"prstClr"
        ) || first_element_uri(&raw, &list_scope)?.as_deref() != Some(A_NS)
        {
            continue;
        }
        let (color, _, _) = root_and_children_impl(&raw, &local, Some(A_NS), &list_scope)?;
        if let Some(value) = unqualified_attribute(&all_attributes(&color)?, "val") {
            values.push(value);
        }
    }
    Ok(values)
}

fn parse_shape_style(
    xml: &[u8],
    inherited: &[(String, String)],
) -> Result<Option<DiagramShapeStyle>> {
    let inherited = NamespaceBindings::from_entries(inherited);
    let (start, children, _) = root_and_children_impl(xml, b"styleLbl", Some(DGM_NS), &inherited)?;
    let scope = inherited.with_start(&start)?;
    let Some(style) = direct_children(&children, &scope, DGM_NS, b"style")?
        .into_iter()
        .next()
    else {
        return Ok(None);
    };
    let (style_start, style_children, _) =
        root_and_children_impl(&style, b"style", Some(DGM_NS), &scope)?;
    let style_scope = scope.with_start(&style_start)?;
    let first = |wanted: &[u8]| -> Result<Option<String>> {
        direct_child_attribute(&style_children, &style_scope, A_NS, wanted, "idx")
    };
    let line_reference = first(b"lnRef")?.and_then(|value| value.parse().ok());
    let fill_reference = first(b"fillRef")?.and_then(|value| value.parse().ok());
    let effect_reference = first(b"effectRef")?.and_then(|value| value.parse().ok());
    let font_reference = first(b"fontRef")?;
    Ok((line_reference.is_some()
        || fill_reference.is_some()
        || effect_reference.is_some()
        || font_reference.is_some())
    .then_some(DiagramShapeStyle {
        line_reference,
        fill_reference,
        effect_reference,
        font_reference,
    }))
}

fn relationship_attribute(
    start: &BytesStart<'_>,
    namespaces: &NamespaceBindings,
    local: &[u8],
) -> Result<String> {
    optional_relationship_attribute(start, namespaces, local)?
        .ok_or_else(|| missing(&format!("r:{}", String::from_utf8_lossy(local))))
}

fn optional_relationship_attribute(
    start: &BytesStart<'_>,
    namespaces: &NamespaceBindings,
    local: &[u8],
) -> Result<Option<String>> {
    for attribute in start.attributes() {
        let attribute = attribute?;
        if local_name(attribute.key.as_ref()) == local
            && namespaces.attribute_uri(attribute.key.as_ref()) == Some(R_NS)
        {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(
                        quick_xml::XmlVersion::Implicit1_0,
                        start.decoder(),
                    )?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn retained_root_attributes(start: &BytesStart<'_>) -> Result<RawAttributes> {
    Ok(all_attributes(start)?
        .into_iter()
        .filter(|(name, _)| !matches!(name.as_str(), "xmlns:dgm" | "xmlns:a" | "xmlns:r"))
        .collect())
}

fn diagram_root(name: &str) -> BytesStart<'static> {
    let mut start = BytesStart::new(name.to_owned());
    start.push_attribute(("xmlns:dgm", DGM_NS));
    start.push_attribute(("xmlns:a", A_NS));
    start.push_attribute(("xmlns:r", R_NS));
    start
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &RawAttributes) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn push_unmodelled_attributes(
    start: &mut BytesStart<'_>,
    attributes: &RawAttributes,
    modelled: &[&str],
) {
    for (name, value) in attributes {
        if !modelled.contains(&name.as_str()) {
            start.push_attribute((name.as_str(), value.as_str()));
        }
    }
}

fn namespaces_with_attributes(
    inherited: &NamespaceBindings,
    attributes: &RawAttributes,
) -> Result<NamespaceBindings> {
    let mut start = BytesStart::new("diagram");
    push_attributes(&mut start, attributes);
    inherited.with_start(&start)
}

fn reject_diagram_writer_conflicts(namespaces: &NamespaceBindings) -> Result<()> {
    for (prefix, expected) in [("dgm", DGM_NS), ("a", A_NS), ("r", R_NS)] {
        let qualified = format!("{prefix}:element");
        if let Some(actual) = namespaces.element_uri(qualified.as_bytes())
            && actual != expected
        {
            return Err(OxmlError::InvalidValue(format!(
                "xmlns:{prefix} conflicts with the fixed diagram writer namespace"
            )));
        }
    }
    Ok(())
}

fn validate_diagram_text_namespaces(xml: &[u8], inherited: &NamespaceBindings) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut scopes = vec![inherited.clone()];
    let mut root_seen = false;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let scope = current_namespace_scope(&scopes)?.with_start(&start)?;
                reject_diagram_writer_conflicts(&scope)?;
                if root_seen {
                    if scope.element_uri(start.name().as_ref()) != Some(A_NS) {
                        return Err(OxmlError::InvalidValue(
                            "diagram text contains a non-DrawingML descendant".to_owned(),
                        ));
                    }
                } else {
                    root_seen = true;
                    if local_name(start.name().as_ref()) != b"t"
                        || scope.element_uri(start.name().as_ref()) != Some(DGM_NS)
                    {
                        return Err(unexpected(&start));
                    }
                }
                scopes.push(scope);
            }
            Event::Empty(start) => {
                let scope = current_namespace_scope(&scopes)?.with_start(&start)?;
                reject_diagram_writer_conflicts(&scope)?;
                if root_seen {
                    if scope.element_uri(start.name().as_ref()) != Some(A_NS) {
                        return Err(OxmlError::InvalidValue(
                            "diagram text contains a non-DrawingML descendant".to_owned(),
                        ));
                    }
                } else {
                    root_seen = true;
                    if local_name(start.name().as_ref()) != b"t"
                        || scope.element_uri(start.name().as_ref()) != Some(DGM_NS)
                    {
                        return Err(unexpected(&start));
                    }
                }
            }
            Event::End(_) => {
                if scopes.len() > 1 {
                    scopes.pop();
                }
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
        buffer.clear();
    }
}

fn write_diagram_text<W: Write>(
    writer: &mut Writer<W>,
    text: &CT_TextBody,
    attributes: &RawAttributes,
) -> Result<()> {
    const OPEN: &[u8] = b"<dgm:t>";
    const CLOSE: &[u8] = b"</dgm:t>";
    let mut canonical = Writer::new(Vec::new());
    text.write_xml_as(&mut canonical, "dgm:t")
        .map_err(|error| OxmlError::InvalidValue(error.to_string()))?;
    let canonical = canonical.into_inner();
    if !canonical.starts_with(OPEN) || !canonical.ends_with(CLOSE) {
        return Err(OxmlError::InvalidValue(
            "canonical diagram text wrapper is malformed".to_owned(),
        ));
    }
    let mut start = BytesStart::new("dgm:t");
    push_attributes(&mut start, attributes);
    writer.write_event(Event::Start(start))?;
    writer
        .get_mut()
        .write_all(&canonical[OPEN.len()..canonical.len() - CLOSE.len()])?;
    writer.write_event(Event::End(BytesEnd::new("dgm:t")))?;
    Ok(())
}

fn emit_raw<W: Write>(writer: &mut Writer<W>, children: &[Vec<u8>]) -> Result<()> {
    for child in children {
        writer.get_mut().write_all(child)?;
    }
    Ok(())
}

fn required_unqualified_attribute(attributes: &RawAttributes, name: &str) -> Result<String> {
    unqualified_attribute(attributes, name).ok_or_else(|| missing(name))
}

fn parsed_unqualified_attribute<T: std::str::FromStr>(
    attributes: &RawAttributes,
    name: &str,
) -> Result<Option<T>> {
    unqualified_attribute(attributes, name)
        .map(|value| {
            value.parse().map_err(|_| {
                OxmlError::InvalidValue(format!("invalid diagram {name} value {value}"))
            })
        })
        .transpose()
}

fn unqualified_attribute(attributes: &RawAttributes, wanted: &str) -> Option<String> {
    attributes
        .iter()
        .find(|(name, _)| name == wanted)
        .map(|(_, value)| value.clone())
}

fn current_namespace_scope(scopes: &[NamespaceBindings]) -> Result<&NamespaceBindings> {
    scopes
        .last()
        .ok_or_else(|| missing("diagram namespace scope"))
}

fn rewrite_data_model_drawing_relationship(
    raw: &[u8],
    mapping: &HashMap<String, String>,
) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(raw);
    let mut buffer = Vec::new();
    let mut scopes = vec![NamespaceBindings::default()];
    let mut path: Vec<(Option<String>, Vec<u8>)> = Vec::new();
    let mut replacements: Vec<(Range<usize>, String)> = Vec::new();
    loop {
        let event_start = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let scope = current_namespace_scope(&scopes)?.with_start(&start)?;
                if is_schema_data_model_extension(&path, &start, &scope) {
                    collect_unqualified_attribute_replacement(
                        raw,
                        event_start,
                        &start,
                        &scope,
                        DSP_NS,
                        b"dataModelExt",
                        b"relId",
                        mapping,
                        &mut replacements,
                    )?;
                }
                path.push((
                    scope.element_uri(start.name().as_ref()).map(str::to_owned),
                    local_name(start.name().as_ref()).to_vec(),
                ));
                scopes.push(scope);
            }
            Event::Empty(start) => {
                let scope = current_namespace_scope(&scopes)?.with_start(&start)?;
                if is_schema_data_model_extension(&path, &start, &scope) {
                    collect_unqualified_attribute_replacement(
                        raw,
                        event_start,
                        &start,
                        &scope,
                        DSP_NS,
                        b"dataModelExt",
                        b"relId",
                        mapping,
                        &mut replacements,
                    )?;
                }
            }
            Event::End(_) => {
                if scopes.len() > 1 {
                    scopes.pop();
                }
                path.pop();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if replacements.is_empty() {
        return Ok(raw.to_vec());
    }
    let mut rewritten = Vec::with_capacity(raw.len());
    let mut copied_through = 0usize;
    for (range, value) in replacements {
        if range.start < copied_through || range.end > raw.len() {
            return Err(OxmlError::InvalidValue(
                "diagram relationship replacement ranges overlap".to_owned(),
            ));
        }
        rewritten.extend_from_slice(&raw[copied_through..range.start]);
        rewritten.extend_from_slice(value.as_bytes());
        copied_through = range.end;
    }
    rewritten.extend_from_slice(&raw[copied_through..]);
    Ok(rewritten)
}

fn is_schema_data_model_extension(
    path: &[(Option<String>, Vec<u8>)],
    start: &BytesStart<'_>,
    scope: &NamespaceBindings,
) -> bool {
    path.len() == 3
        && path[0].0.as_deref() == Some(DGM_NS)
        && path[0].1.as_slice() == b"dataModel"
        && path[1].0.as_deref() == Some(DGM_NS)
        && path[1].1.as_slice() == b"extLst"
        && path[2].0.as_deref() == Some(A_NS)
        && path[2].1.as_slice() == b"ext"
        && scope.element_uri(start.name().as_ref()) == Some(DSP_NS)
        && local_name(start.name().as_ref()) == b"dataModelExt"
}

#[allow(clippy::too_many_arguments)]
fn collect_unqualified_attribute_replacement(
    raw: &[u8],
    event_start: usize,
    start: &BytesStart<'_>,
    namespaces: &NamespaceBindings,
    element_uri: &str,
    element_local: &[u8],
    attribute_name: &[u8],
    mapping: &HashMap<String, String>,
    replacements: &mut Vec<(Range<usize>, String)>,
) -> Result<()> {
    if local_name(start.name().as_ref()) != element_local
        || namespaces.element_uri(start.name().as_ref()) != Some(element_uri)
    {
        return Ok(());
    }
    for attribute in start.attributes() {
        let attribute = attribute?;
        if attribute.key.as_ref() != attribute_name {
            continue;
        }
        let decoded =
            attribute.decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?;
        let Some(replacement) = mapping.get(decoded.as_ref()) else {
            continue;
        };
        let value = attribute.value.as_ref();
        let relative_start = (value.as_ptr() as usize)
            .checked_sub(start.as_ptr() as usize)
            .filter(|offset| {
                offset
                    .checked_add(value.len())
                    .is_some_and(|end| end <= start.len())
            })
            .ok_or_else(|| {
                OxmlError::InvalidValue(
                    "diagram relationship attribute is outside its start tag".to_owned(),
                )
            })?;
        let range_start = event_start
            .checked_add(1)
            .and_then(|offset| offset.checked_add(relative_start))
            .ok_or_else(|| {
                OxmlError::InvalidValue(
                    "diagram relationship attribute range overflowed".to_owned(),
                )
            })?;
        let range_end = range_start.checked_add(value.len()).ok_or_else(|| {
            OxmlError::InvalidValue("diagram relationship attribute range overflowed".to_owned())
        })?;
        if raw.get(range_start..range_end) != Some(value) {
            return Err(OxmlError::InvalidValue(
                "diagram relationship attribute range did not match source".to_owned(),
            ));
        }
        replacements.push((
            range_start..range_end,
            quick_xml::escape::escape(replacement).into_owned(),
        ));
    }
    Ok(())
}

fn point_kind(value: &str) -> DiagramPointKind {
    match value {
        "node" => DiagramPointKind::Node,
        "asst" => DiagramPointKind::Assistant,
        "pres" => DiagramPointKind::Presentation,
        other => DiagramPointKind::Other(other.to_owned()),
    }
}

fn point_kind_lexical(kind: &DiagramPointKind) -> String {
    match kind {
        DiagramPointKind::Node => "node".to_owned(),
        DiagramPointKind::Assistant => "asst".to_owned(),
        DiagramPointKind::Presentation => "pres".to_owned(),
        DiagramPointKind::Other(value) => value.clone(),
    }
}

fn connection_kind(value: &str) -> DiagramConnectionKind {
    match value {
        "parOf" => DiagramConnectionKind::ParentOf,
        "presOf" => DiagramConnectionKind::PresentationOf,
        "presParOf" => DiagramConnectionKind::PresentationParentOf,
        other => DiagramConnectionKind::Other(other.to_owned()),
    }
}

fn infer_layout_family(
    unique_id: Option<&str>,
    title: Option<&str>,
    categories: &[String],
    algorithms: &[String],
) -> DiagramLayoutFamily {
    let evidence = std::iter::empty()
        .chain(unique_id)
        .chain(title)
        .chain(categories.iter().map(String::as_str))
        .chain(algorithms.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    for (needle, family) in [
        ("hier", DiagramLayoutFamily::Hierarchy),
        ("cycle", DiagramLayoutFamily::Cycle),
        ("relationship", DiagramLayoutFamily::Relationship),
        ("venn", DiagramLayoutFamily::Relationship),
        ("matrix", DiagramLayoutFamily::Matrix),
        ("pyramid", DiagramLayoutFamily::Pyramid),
        ("list", DiagramLayoutFamily::List),
        ("process", DiagramLayoutFamily::List),
    ] {
        if evidence.contains(needle) {
            return family;
        }
    }
    DiagramLayoutFamily::Unsupported(
        unique_id
            .or(title)
            .or_else(|| algorithms.first().map(String::as_str))
            .unwrap_or("unknown")
            .to_owned(),
    )
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn missing(name: &str) -> OxmlError {
    OxmlError::MissingElement(name.to_owned())
}

fn unexpected(start: &BytesStart<'_>) -> OxmlError {
    OxmlError::UnexpectedElement(String::from_utf8_lossy(start.name().as_ref()).into_owned())
}

fn order(message: &str) -> OxmlError {
    OxmlError::InvalidValue(message.to_owned())
}

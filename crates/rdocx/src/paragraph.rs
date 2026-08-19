//! Paragraph — a block-level container for runs of text.

use rdocx_oxml::borders::{CT_BorderEdge, CT_PBdr, CT_TabStop, CT_Tabs};
use rdocx_oxml::document::CT_SectPr;
use rdocx_oxml::properties::{CT_PPr, CT_Shd};
use rdocx_oxml::shared::{
    ST_Border, ST_Jc, ST_PageOrientation, ST_SectionType, ST_TabJc, ST_TabLeader,
};
use rdocx_oxml::text::{
    BreakType, CT_P, CT_R, CommentRangeMarker, FieldType, HyperlinkSpan, RunContent,
    hyperlink_revision_index,
};
use rdocx_oxml::units::Twips;

use crate::run::{FieldKind, Run, RunRef};
use crate::unsupported_xml::{UnsupportedXmlRef, WORD_NAMESPACE};
use crate::{ContentControlRef, Length, RevisionRef};

/// Paragraph alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justify,
}

/// One direct child of a paragraph, in source order.
pub enum ParagraphItemRef<'a> {
    /// A direct run.
    Run(RunRef<'a>),
    /// A hyperlink and the runs or preserved XML it contains.
    Hyperlink(HyperlinkRef<'a>),
    /// A paragraph-level content control.
    ContentControl(ContentControlRef<'a>),
    /// A tracked insertion, deletion, or move.
    Revision(RevisionRef<'a>),
    /// The start of a comment range.
    CommentRangeStart { id: i32, has_child_content: bool },
    /// The end of a comment range.
    CommentRangeEnd { id: i32, has_child_content: bool },
    /// The start of a bookmark.
    BookmarkStart {
        /// The bookmark ID, when present.
        id: Option<i32>,
        /// The bookmark name, when present.
        name: Option<&'a str>,
        has_child_content: bool,
    },
    /// The end of a bookmark.
    BookmarkEnd {
        /// The bookmark ID, when present.
        id: Option<i32>,
        has_child_content: bool,
    },
    /// A preserved paragraph child that rdocx does not model.
    UnsupportedXml(&'a [u8]),
}

/// One paragraph child exposed through the compatibility reader facade.
pub enum ParagraphContentRef<'a> {
    /// A direct run.
    Run(RunRef<'a>),
    /// A paragraph-level hyperlink.
    Hyperlink(HyperlinkRef<'a>),
    /// A simple Word field.
    SimpleField(SimpleFieldRef<'a>),
    /// A tracked insertion.
    Insertion(InsertionRef<'a>),
    /// Content that the compatibility facade does not model.
    UnsupportedXml(UnsupportedXmlRef<'a>),
}

/// One child inside a hyperlink or simple field through the compatibility facade.
pub enum InlineContentRef<'a> {
    /// A run.
    Run(RunRef<'a>),
    /// A simple Word field.
    SimpleField(SimpleFieldRef<'a>),
    /// Content that the compatibility facade does not model.
    UnsupportedXml(UnsupportedXmlRef<'a>),
}

/// A simple field projection retained by the current reader model.
#[derive(Clone, Copy)]
pub struct SimpleFieldRef<'a> {
    run: &'a CT_R,
    field_type: &'a FieldType,
    display: &'a str,
}

impl<'a> SimpleFieldRef<'a> {
    /// The parsed field instruction kind.
    pub fn kind(&self) -> FieldKind<'a> {
        match self.field_type {
            FieldType::Page => FieldKind::Page,
            FieldType::NumPages => FieldKind::NumPages,
            FieldType::Ref { instruction, .. } | FieldType::PageRef { instruction, .. } => {
                FieldKind::Other(instruction)
            }
            FieldType::Other(instruction) => FieldKind::Other(instruction),
        }
    }

    /// The original field instruction.
    pub fn instruction(&self) -> &'a str {
        match self.field_type {
            FieldType::Page => " PAGE ",
            FieldType::NumPages => " NUMPAGES ",
            FieldType::Ref { instruction, .. }
            | FieldType::PageRef { instruction, .. }
            | FieldType::Other(instruction) => instruction,
        }
    }

    /// The cached display result.
    pub fn cached_text(&self) -> String {
        self.display.to_owned()
    }

    /// Whether Word stored a non-empty cached result.
    pub fn has_cached_content(&self) -> bool {
        !self.display.is_empty()
    }

    /// Whether Word marked the cached result as stale.
    pub fn dirty(&self) -> Option<bool> {
        self.run.simple_field.as_ref().and_then(|field| field.dirty)
    }

    /// The compact field projection cannot preserve inner result-run structure.
    pub fn has_unmodeled_semantic_attributes(&self) -> bool {
        self.run
            .simple_field
            .as_ref()
            .is_none_or(|field| field.has_unmodeled_attributes)
    }

    /// Inner field content is deliberately reported as unsupported until its
    /// formatting and nested semantics are represented losslessly.
    pub fn content(&self) -> impl Iterator<Item = InlineContentRef<'a>> {
        std::iter::once(InlineContentRef::Run(RunRef { inner: self.run }))
    }
}

/// A tracked insertion projection.
#[derive(Clone, Copy)]
pub struct InsertionRef<'a> {
    revision: RevisionRef<'a>,
}

impl<'a> InsertionRef<'a> {
    /// Iterate over the visible result runs in a tracked insertion.
    pub fn content(&self) -> impl Iterator<Item = ParagraphContentRef<'a>> {
        let mut content = Vec::new();
        match self.revision.inner.content() {
            rdocx_oxml::RevisionContent::Runs(runs) => {
                if let Some(paragraph) = self.revision.inner.content_paragraph() {
                    let paragraph = ParagraphRef { inner: paragraph };
                    content.extend(paragraph.content());
                } else {
                    content.extend(runs.iter().map(|run| {
                        let run = RunRef { inner: run };
                        simple_field_ref(run.inner)
                            .map(ParagraphContentRef::SimpleField)
                            .unwrap_or(ParagraphContentRef::Run(run))
                    }));
                }
            }
            _ => content.push(ParagraphContentRef::UnsupportedXml(
                UnsupportedXmlRef::modeled(WORD_NAMESPACE, "ins"),
            )),
        }
        content.into_iter()
    }
}

/// One child within a hyperlink, in source order.
pub enum HyperlinkItemRef<'a> {
    /// A run in the hyperlink.
    Run(RunRef<'a>),
    /// A tracked insertion, deletion, or move in the hyperlink.
    Revision(RevisionRef<'a>),
    /// A preserved hyperlink child that rdocx does not model.
    UnsupportedXml(&'a [u8]),
}

/// An immutable paragraph-level hyperlink.
#[derive(Clone, Copy)]
pub struct HyperlinkRef<'a> {
    paragraph: &'a CT_P,
    index: usize,
}

impl<'a> HyperlinkRef<'a> {
    fn inner(&self) -> &'a HyperlinkSpan {
        &self.paragraph.hyperlinks[self.index]
    }

    /// The relationship ID for this hyperlink, when it has an external target.
    pub fn relationship_id(&self) -> Option<&'a str> {
        self.inner().rel_id.as_deref()
    }

    /// The bookmark anchor for this hyperlink, when it has an internal target.
    pub fn anchor(&self) -> Option<&'a str> {
        self.inner().anchor.as_deref()
    }

    /// The optional hyperlink tooltip.
    pub fn tooltip(&self) -> Option<&'a str> {
        self.inner().tooltip.as_deref()
    }

    /// The optional hyperlink document location.
    pub fn doc_location(&self) -> Option<&'a str> {
        self.inner().doc_location.as_deref()
    }

    /// Whether the hyperlink has attributes outside the semantic facade.
    pub fn has_unmodeled_semantic_attributes(&self) -> bool {
        !self.inner().extra_attributes.is_empty()
    }

    /// Get the combined text of the hyperlink runs.
    pub fn text(&self) -> String {
        let hyperlink = self.inner();
        self.paragraph.runs[hyperlink.run_start..hyperlink.run_end]
            .iter()
            .map(CT_R::text)
            .collect()
    }

    /// Iterate over hyperlink children in source order.
    pub fn items(&self) -> impl Iterator<Item = HyperlinkItemRef<'a>> {
        let hyperlink = self.inner();
        let mut items = Vec::new();
        for relative_index in 0..=hyperlink.run_end - hyperlink.run_start {
            let run_index = hyperlink.run_start + relative_index;
            let revisions = self
                .paragraph
                .revisions
                .iter()
                .filter(|(at, slot, _)| {
                    *at == run_index && hyperlink_revision_index(*slot) == Some(self.index)
                })
                .map(|(_, _, revision)| revision)
                .collect::<Vec<_>>();
            for revision_index in 0..=revisions.len() {
                items.extend(
                    hyperlink
                        .extra_xml
                        .iter()
                        .filter(|(at, before, _)| {
                            *at == relative_index
                                && (*before).min(revisions.len()) == revision_index
                        })
                        .map(|(_, _, raw)| HyperlinkItemRef::UnsupportedXml(raw.as_slice())),
                );
                if let Some(revision) = revisions.get(revision_index) {
                    items.push(HyperlinkItemRef::Revision(RevisionRef { inner: revision }));
                }
            }
            if relative_index < hyperlink.run_end - hyperlink.run_start {
                let run = &self.paragraph.runs[run_index];
                items.push(HyperlinkItemRef::Run(RunRef { inner: run }));
            }
        }
        items.into_iter()
    }

    /// Iterate over hyperlink content through the compatibility facade.
    pub fn content(&self) -> impl Iterator<Item = InlineContentRef<'a>> {
        self.items().map(move |item| match item {
            HyperlinkItemRef::Run(run) => simple_field_ref(run.inner)
                .map(InlineContentRef::SimpleField)
                .unwrap_or(InlineContentRef::Run(run)),
            HyperlinkItemRef::Revision(_) => InlineContentRef::UnsupportedXml(
                UnsupportedXmlRef::modeled(WORD_NAMESPACE, "revision"),
            ),
            HyperlinkItemRef::UnsupportedXml(raw) => {
                InlineContentRef::UnsupportedXml(UnsupportedXmlRef::new(raw))
            }
        })
    }
}

fn simple_field_ref(run: &CT_R) -> Option<SimpleFieldRef<'_>> {
    let [
        RunContent::Field {
            field_type,
            display,
        },
    ] = run.content.as_slice()
    else {
        return None;
    };
    Some(SimpleFieldRef {
        run,
        field_type,
        display,
    })
}

impl Alignment {
    fn to_st_jc(self) -> ST_Jc {
        match self {
            Alignment::Left => ST_Jc::Left,
            Alignment::Center => ST_Jc::Center,
            Alignment::Right => ST_Jc::Right,
            Alignment::Justify => ST_Jc::Both,
        }
    }

    fn from_st_jc(jc: ST_Jc) -> Self {
        match jc {
            ST_Jc::Center => Alignment::Center,
            ST_Jc::Right | ST_Jc::End => Alignment::Right,
            ST_Jc::Both | ST_Jc::Distribute => Alignment::Justify,
            _ => Alignment::Left,
        }
    }
}

/// Border style for paragraph borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Single,
    Thick,
    Double,
    Dotted,
    Dashed,
    DotDash,
    Wave,
}

impl BorderStyle {
    /// Convert to the OXML `ST_Border` type. Visible to the table module,
    /// which builds cell borders from the same enum.
    pub(crate) fn to_st(self) -> ST_Border {
        match self {
            Self::None => ST_Border::None,
            Self::Single => ST_Border::Single,
            Self::Thick => ST_Border::Thick,
            Self::Double => ST_Border::Double,
            Self::Dotted => ST_Border::Dotted,
            Self::Dashed => ST_Border::Dashed,
            Self::DotDash => ST_Border::DotDash,
            Self::Wave => ST_Border::Wave,
        }
    }
}

/// An immutable paragraph border edge.
#[derive(Debug, Clone, Copy)]
pub struct ParagraphBorderRef<'a> {
    inner: &'a CT_BorderEdge,
}

impl ParagraphBorderRef<'_> {
    /// The OOXML border style name.
    pub fn style(self) -> &'static str {
        self.inner.val.to_str()
    }

    /// Border width in eighths of a point.
    pub fn size_eighths_pt(self) -> Option<u32> {
        self.inner.sz
    }

    /// Space between the border and content in points.
    pub fn space_points(self) -> Option<u32> {
        self.inner.space
    }

    /// Border color, normally a six-digit RGB hex value or `auto`.
    pub fn color(&self) -> Option<&str> {
        self.inner.color.as_deref()
    }
}

/// Tab stop alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAlignment {
    Left,
    Center,
    Right,
    Decimal,
}

impl TabAlignment {
    fn to_st(self) -> ST_TabJc {
        match self {
            Self::Left => ST_TabJc::Left,
            Self::Center => ST_TabJc::Center,
            Self::Right => ST_TabJc::Right,
            Self::Decimal => ST_TabJc::Decimal,
        }
    }
}

/// Tab leader character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabLeader {
    None,
    Dot,
    Hyphen,
    Underscore,
}

impl TabLeader {
    fn to_st(self) -> ST_TabLeader {
        match self {
            Self::None => ST_TabLeader::None,
            Self::Dot => ST_TabLeader::Dot,
            Self::Hyphen => ST_TabLeader::Hyphen,
            Self::Underscore => ST_TabLeader::Underscore,
        }
    }
}

/// A mutable reference to a paragraph in a document.
pub struct Paragraph<'a> {
    pub(crate) inner: &'a mut CT_P,
}

impl<'a> Paragraph<'a> {
    /// Get the combined text of all runs.
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Append a footnote reference run (`<w:footnoteReference w:id="..."/>`).
    /// The footnote content itself is added via `Document::add_footnote`.
    pub fn add_footnote_ref(&mut self, id: i32) {
        use rdocx_oxml::text::{CT_R, RunContent};
        let mut r = CT_R::new("");
        r.content = vec![RunContent::FootnoteRef { id }];
        self.inner.runs.push(r);
    }

    /// Add a run with the given text and return a mutable reference for chaining.
    pub fn add_run(&mut self, text: &str) -> Run<'_> {
        self.inner.runs.push(CT_R::new(text));
        Run {
            inner: self.inner.runs.last_mut().unwrap(),
        }
    }

    /// Add a line break in its own run.
    pub fn add_line_break(&mut self) {
        let mut run = CT_R::new("");
        run.content = vec![RunContent::Break(BreakType::Line)];
        self.inner.runs.push(run);
    }

    /// Add a run wrapped in an external hyperlink relationship.
    ///
    /// Obtain `relationship_id` from
    /// [`crate::Document::add_hyperlink_relationship`]. Returning the run
    /// allows the hyperlink text to receive the same direct formatting as any
    /// other run.
    pub fn add_hyperlink(&mut self, text: &str, relationship_id: &str) -> Run<'_> {
        self.add_hyperlink_with_tooltip(text, relationship_id, None)
    }

    /// Add a hyperlink run and optionally set its user-facing hover tooltip.
    pub fn add_hyperlink_with_tooltip(
        &mut self,
        text: &str,
        relationship_id: &str,
        tooltip: Option<&str>,
    ) -> Run<'_> {
        let run_start = self.inner.runs.len();
        self.inner.runs.push(CT_R::new(text));
        self.inner.hyperlinks.push(HyperlinkSpan {
            rel_id: Some(relationship_id.to_string()),
            anchor: None,
            tooltip: tooltip.map(str::to_owned),
            doc_location: None,
            run_start,
            run_end: run_start + 1,
            extra_attributes: Vec::new(),
            extra_xml: Vec::new(),
            preserved_raw_before: None,
        });
        Run {
            inner: self.inner.runs.last_mut().unwrap(),
        }
    }

    /// Get the number of runs in this paragraph.
    pub fn run_count(&self) -> usize {
        self.inner.runs.len()
    }

    /// Get an immutable run by index.
    pub fn run(&self, index: usize) -> Option<RunRef<'_>> {
        self.inner.runs.get(index).map(|inner| RunRef { inner })
    }

    /// Get a mutable run by index.
    pub fn run_mut(&mut self, index: usize) -> Option<Run<'_>> {
        self.inner.runs.get_mut(index).map(|inner| Run { inner })
    }

    /// Get an iterator over immutable run references.
    pub fn runs(&self) -> impl Iterator<Item = RunRef<'_>> {
        self.inner.runs.iter().map(|r| RunRef { inner: r })
    }

    /// Set the paragraph alignment.
    pub fn alignment(mut self, align: Alignment) -> Self {
        self.set_alignment(align);
        self
    }

    /// Set the paragraph alignment in place.
    pub fn set_alignment(&mut self, align: Alignment) {
        self.set_alignment_value(Some(align));
    }

    /// Set or clear direct paragraph alignment.
    pub fn set_alignment_value(&mut self, align: Option<Alignment>) {
        if align.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().jc = align.map(Alignment::to_st_jc);
    }

    /// Set the paragraph style by ID.
    pub fn style(mut self, style_id: &str) -> Self {
        self.set_style(style_id);
        self
    }

    /// Set the paragraph style by ID in place.
    pub fn set_style(&mut self, style_id: &str) {
        self.ensure_ppr().style_id = Some(style_id.to_string());
    }

    /// Attach this paragraph to a list definition as a list item.
    ///
    /// `num_id` comes from [`crate::Document::add_list_definition`] (or the
    /// shared definitions behind `add_bullet_list_item` /
    /// `add_numbered_list_item`); `level` is the 0-based indentation level.
    pub fn numbering(mut self, num_id: u32, level: u32) -> Self {
        let _ = self.set_numbering(num_id, level);
        self
    }

    /// Attach this paragraph to a list definition in place.
    ///
    /// Returns `false` without mutation when `level` is outside Word's
    /// supported range of 0 through 8.
    pub fn set_numbering(&mut self, num_id: u32, level: u32) -> bool {
        self.set_numbering_value(Some((num_id, level)))
    }

    /// Set or clear this paragraph's list numbering.
    ///
    /// Returns `false` without mutation for an out-of-range level.
    pub fn set_numbering_value(&mut self, numbering: Option<(u32, u32)>) -> bool {
        if numbering.is_some_and(|(_, level)| level > 8) {
            return false;
        }
        if numbering.is_none() && self.inner.properties.is_none() {
            return true;
        }
        let ppr = self.ensure_ppr();
        ppr.num_id = numbering.map(|(num_id, _)| num_id);
        ppr.num_ilvl = numbering.map(|(_, level)| level);
        true
    }

    /// Set space before the paragraph.
    pub fn space_before(mut self, length: Length) -> Self {
        self.set_space_before(length);
        self
    }

    /// Set space before the paragraph in place.
    pub fn set_space_before(&mut self, length: Length) {
        self.set_space_before_value(Some(length));
    }

    /// Set or clear direct space before the paragraph.
    pub fn set_space_before_value(&mut self, length: Option<Length>) {
        if length.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().space_before = length.map(Length::as_twips);
    }

    /// Set space after the paragraph.
    pub fn space_after(mut self, length: Length) -> Self {
        self.set_space_after(length);
        self
    }

    /// Set space after the paragraph in place.
    pub fn set_space_after(&mut self, length: Length) {
        self.set_space_after_value(Some(length));
    }

    /// Set or clear direct space after the paragraph.
    pub fn set_space_after_value(&mut self, length: Option<Length>) {
        if length.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().space_after = length.map(Length::as_twips);
    }

    /// Set left indentation.
    pub fn indent_left(mut self, length: Length) -> Self {
        self.set_indent_left(length);
        self
    }

    /// Set left indentation in place.
    pub fn set_indent_left(&mut self, length: Length) {
        self.set_indent_left_value(Some(length));
    }

    /// Set or clear direct left indentation.
    pub fn set_indent_left_value(&mut self, length: Option<Length>) {
        if length.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().ind_left = length.map(Length::as_twips);
    }

    /// Set right indentation.
    pub fn indent_right(mut self, length: Length) -> Self {
        self.set_indent_right(length);
        self
    }

    /// Set right indentation in place.
    pub fn set_indent_right(&mut self, length: Length) {
        self.set_indent_right_value(Some(length));
    }

    /// Set or clear direct right indentation.
    pub fn set_indent_right_value(&mut self, length: Option<Length>) {
        if length.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().ind_right = length.map(Length::as_twips);
    }

    /// Set first line indent.
    pub fn first_line_indent(mut self, length: Length) -> Self {
        self.set_first_line_indent(length);
        self
    }

    /// Set first line indent in place.
    pub fn set_first_line_indent(&mut self, length: Length) {
        self.ensure_ppr().ind_first_line = Some(length.as_twips());
    }

    /// Set or clear the signed first-line indentation.
    ///
    /// Negative values are stored as a positive hanging indentation.
    pub fn set_signed_first_line_indent_value(&mut self, length: Option<Length>) {
        if length.is_none() && self.inner.properties.is_none() {
            return;
        }
        let ppr = self.ensure_ppr();
        match length.map(Length::to_twips) {
            Some(twips) if twips < 0 => {
                ppr.ind_first_line = None;
                ppr.ind_hanging = Some(Twips(twips.saturating_abs()));
            }
            Some(twips) => {
                ppr.ind_first_line = Some(Twips(twips));
                ppr.ind_hanging = None;
            }
            None => {
                ppr.ind_first_line = None;
                ppr.ind_hanging = None;
            }
        }
    }

    /// Set hanging indent.
    pub fn hanging_indent(mut self, length: Length) -> Self {
        self.set_hanging_indent(length);
        self
    }

    /// Set hanging indent in place.
    pub fn set_hanging_indent(&mut self, length: Length) {
        self.ensure_ppr().ind_hanging = Some(length.as_twips());
    }

    /// Set keep with next paragraph.
    pub fn keep_with_next(mut self, val: bool) -> Self {
        self.set_keep_with_next(val);
        self
    }

    /// Set keep with next paragraph in place.
    pub fn set_keep_with_next(&mut self, val: bool) {
        self.set_keep_with_next_value(Some(val));
    }

    /// Set or clear direct keep-with-next formatting.
    pub fn set_keep_with_next_value(&mut self, val: Option<bool>) {
        if val.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().keep_next = val;
    }

    /// Set keep lines together.
    pub fn keep_together(mut self, val: bool) -> Self {
        self.set_keep_together(val);
        self
    }

    /// Set keep lines together in place.
    pub fn set_keep_together(&mut self, val: bool) {
        self.set_keep_together_value(Some(val));
    }

    /// Set or clear direct keep-together formatting.
    pub fn set_keep_together_value(&mut self, val: Option<bool>) {
        if val.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().keep_lines = val;
    }

    /// Set page break before.
    pub fn page_break_before(mut self, val: bool) -> Self {
        self.set_page_break_before(val);
        self
    }

    /// Set page break before in place.
    pub fn set_page_break_before(&mut self, val: bool) {
        self.set_page_break_before_value(Some(val));
    }

    /// Set or clear direct page-break-before formatting.
    pub fn set_page_break_before_value(&mut self, val: Option<bool>) {
        if val.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().page_break_before = val;
    }

    /// Set widow/orphan control.
    pub fn widow_control(mut self, val: bool) -> Self {
        self.set_widow_control(val);
        self
    }

    /// Set widow/orphan control in place.
    pub fn set_widow_control(&mut self, val: bool) {
        self.set_widow_control_value(Some(val));
    }

    /// Set or clear direct widow-control formatting.
    pub fn set_widow_control_value(&mut self, val: Option<bool>) {
        if val.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_ppr().widow_control = val;
    }

    /// Set line spacing in points with "exact" rule.
    pub fn line_spacing(mut self, pt: f64) -> Self {
        self.set_line_spacing(pt);
        self
    }

    /// Set exact line spacing in place.
    pub fn set_line_spacing(&mut self, pt: f64) {
        let ppr = self.ensure_ppr();
        ppr.line_spacing = Some(Twips::from_pt(pt));
        ppr.line_rule = Some("exact".to_string());
    }

    /// Clear direct line spacing.
    pub fn clear_line_spacing(&mut self) {
        if let Some(ppr) = self.inner.properties.as_mut() {
            ppr.line_spacing = None;
            ppr.line_rule = None;
        }
    }

    /// Set line spacing with a multiplier (1.0 = single, 1.5, 2.0 = double, etc.).
    pub fn line_spacing_multiple(mut self, multiple: f64) -> Self {
        self.set_line_spacing_multiple(multiple);
        self
    }

    /// Set multiplied line spacing in place.
    pub fn set_line_spacing_multiple(&mut self, multiple: f64) {
        let ppr = self.ensure_ppr();
        // In "auto" mode, line spacing is in 240ths of a line (240 = single)
        ppr.line_spacing = Some(Twips((multiple * 240.0) as i32));
        ppr.line_rule = Some("auto".to_string());
    }

    /// Set a background/shading fill color (hex string, e.g. "FFFF00").
    pub fn shading(mut self, fill_color: &str) -> Self {
        self.set_shading(fill_color);
        self
    }

    /// Set a background/shading fill color in place.
    pub fn set_shading(&mut self, fill_color: &str) {
        self.ensure_ppr().shading = Some(CT_Shd {
            val: "clear".to_string(),
            color: Some("auto".to_string()),
            fill: Some(fill_color.to_string()),
        });
    }

    /// Add a border to all sides.
    pub fn border_all(mut self, style: BorderStyle, size_eighths_pt: u32, color: &str) -> Self {
        self.set_border_all(style, size_eighths_pt, color);
        self
    }

    /// Add a border to all sides in place.
    pub fn set_border_all(&mut self, style: BorderStyle, size_eighths_pt: u32, color: &str) {
        let edge = CT_BorderEdge {
            val: style.to_st(),
            sz: Some(size_eighths_pt),
            space: Some(1),
            color: Some(color.to_string()),
        };
        self.ensure_ppr().borders = Some(CT_PBdr {
            top: Some(edge.clone()),
            bottom: Some(edge.clone()),
            left: Some(edge.clone()),
            right: Some(edge),
            between: None,
            bar: None,
        });
    }

    /// Add a bottom border only.
    pub fn border_bottom(mut self, style: BorderStyle, size_eighths_pt: u32, color: &str) -> Self {
        self.set_border_bottom(style, size_eighths_pt, color);
        self
    }

    /// Add a bottom border in place.
    pub fn set_border_bottom(&mut self, style: BorderStyle, size_eighths_pt: u32, color: &str) {
        let edge = CT_BorderEdge {
            val: style.to_st(),
            sz: Some(size_eighths_pt),
            space: Some(1),
            color: Some(color.to_string()),
        };
        let borders = self
            .ensure_ppr()
            .borders
            .get_or_insert_with(CT_PBdr::default);
        borders.bottom = Some(edge);
    }

    /// Add a tab stop.
    pub fn add_tab_stop(mut self, alignment: TabAlignment, position: Length) -> Self {
        self.set_add_tab_stop(alignment, position);
        self
    }

    /// Add a tab stop in place.
    pub fn set_add_tab_stop(&mut self, alignment: TabAlignment, position: Length) {
        let tabs = self
            .ensure_ppr()
            .tabs
            .get_or_insert_with(|| CT_Tabs { tabs: Vec::new() });
        tabs.tabs.push(CT_TabStop {
            val: alignment.to_st(),
            pos: position.as_twips(),
            leader: None,
            source_occurrence: None,
        });
    }

    /// Add a tab stop with a leader character.
    pub fn add_tab_stop_with_leader(
        mut self,
        alignment: TabAlignment,
        position: Length,
        leader: TabLeader,
    ) -> Self {
        self.set_add_tab_stop_with_leader(alignment, position, leader);
        self
    }

    /// Add a tab stop with a leader character in place.
    pub fn set_add_tab_stop_with_leader(
        &mut self,
        alignment: TabAlignment,
        position: Length,
        leader: TabLeader,
    ) {
        let tabs = self
            .ensure_ppr()
            .tabs
            .get_or_insert_with(|| CT_Tabs { tabs: Vec::new() });
        tabs.tabs.push(CT_TabStop {
            val: alignment.to_st(),
            pos: position.as_twips(),
            leader: Some(leader.to_st()),
            source_occurrence: None,
        });
    }

    /// Set outline level (0–8, used for TOC generation).
    pub fn outline_level(mut self, level: u32) -> Self {
        self.set_outline_level(level);
        self
    }

    /// Set outline level in place.
    pub fn set_outline_level(&mut self, level: u32) {
        self.ensure_ppr().outline_lvl = Some(level);
    }

    /// Add a section break after this paragraph.
    ///
    /// This creates a `<w:sectPr>` inside the paragraph's properties,
    /// ending the current section at this paragraph.
    pub fn section_break(mut self, break_type: SectionBreak) -> Self {
        self.set_section_break(break_type);
        self
    }

    /// Add a section break after this paragraph in place.
    pub fn set_section_break(&mut self, break_type: SectionBreak) {
        let sect = self.ensure_sect_pr();
        sect.section_type = Some(break_type.to_st());
    }

    /// Set the section ending at this paragraph to landscape orientation.
    ///
    /// Sets page dimensions to 11" x 8.5" (US Letter landscape).
    /// Must be combined with `section_break()` to create a section break.
    pub fn section_landscape(mut self) -> Self {
        self.set_section_landscape();
        self
    }

    /// Set the section ending at this paragraph to landscape in place.
    pub fn set_section_landscape(&mut self) {
        let sect = self.ensure_sect_pr();
        sect.orientation = Some(ST_PageOrientation::Landscape);
        sect.page_width = Some(Twips(15840)); // 11"
        sect.page_height = Some(Twips(12240)); // 8.5"
    }

    /// Set the section ending at this paragraph to portrait orientation.
    ///
    /// Sets page dimensions to 8.5" x 11" (US Letter portrait).
    /// Must be combined with `section_break()` to create a section break.
    pub fn section_portrait(mut self) -> Self {
        self.set_section_portrait();
        self
    }

    /// Set the section ending at this paragraph to portrait in place.
    pub fn set_section_portrait(&mut self) {
        let sect = self.ensure_sect_pr();
        sect.orientation = Some(ST_PageOrientation::Portrait);
        sect.page_width = Some(Twips(12240)); // 8.5"
        sect.page_height = Some(Twips(15840)); // 11"
    }

    /// Set custom page dimensions for the section ending at this paragraph.
    pub fn section_page_size(mut self, width: crate::Length, height: crate::Length) -> Self {
        self.set_section_page_size(width, height);
        self
    }

    /// Set custom page dimensions for the section in place.
    pub fn set_section_page_size(&mut self, width: crate::Length, height: crate::Length) {
        let sect = self.ensure_sect_pr();
        sect.page_width = Some(width.as_twips());
        sect.page_height = Some(height.as_twips());
    }

    fn ensure_ppr(&mut self) -> &mut CT_PPr {
        self.inner.properties.get_or_insert_with(CT_PPr::default)
    }

    fn ensure_sect_pr(&mut self) -> &mut CT_SectPr {
        let ppr = self.ensure_ppr();
        ppr.sect_pr.get_or_insert_with(CT_SectPr::default_letter)
    }
}

/// Section break type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionBreak {
    /// Start a new section on the next page.
    NextPage,
    /// Start a new section on the same page (continuous).
    Continuous,
    /// Start a new section on the next even-numbered page.
    EvenPage,
    /// Start a new section on the next odd-numbered page.
    OddPage,
}

impl SectionBreak {
    fn to_st(self) -> ST_SectionType {
        match self {
            SectionBreak::NextPage => ST_SectionType::NextPage,
            SectionBreak::Continuous => ST_SectionType::Continuous,
            SectionBreak::EvenPage => ST_SectionType::EvenPage,
            SectionBreak::OddPage => ST_SectionType::OddPage,
        }
    }
}

/// An immutable reference to a paragraph.
pub struct ParagraphRef<'a> {
    pub(crate) inner: &'a CT_P,
}

impl<'a> ParagraphRef<'a> {
    /// Get the combined text of all runs.
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Iterate over paragraph content through the compatibility facade.
    pub fn content(&self) -> impl Iterator<Item = ParagraphContentRef<'a>> {
        self.items().map(paragraph_content_item)
    }
}

fn paragraph_content_item<'a>(item: ParagraphItemRef<'a>) -> ParagraphContentRef<'a> {
    match item {
        ParagraphItemRef::Run(run) => simple_field_ref(run.inner)
            .map(ParagraphContentRef::SimpleField)
            .unwrap_or(ParagraphContentRef::Run(run)),
        ParagraphItemRef::Hyperlink(hyperlink) => ParagraphContentRef::Hyperlink(hyperlink),
        ParagraphItemRef::Revision(revision)
            if revision.kind() == rdocx_oxml::RevisionKind::Insertion =>
        {
            ParagraphContentRef::Insertion(InsertionRef { revision })
        }
        ParagraphItemRef::Revision(revision)
            if revision.kind() == rdocx_oxml::RevisionKind::Deletion =>
        {
            ParagraphContentRef::UnsupportedXml(UnsupportedXmlRef::modeled(WORD_NAMESPACE, "del"))
        }
        ParagraphItemRef::ContentControl(_) => {
            ParagraphContentRef::UnsupportedXml(UnsupportedXmlRef::modeled(WORD_NAMESPACE, "sdt"))
        }
        ParagraphItemRef::Revision(_) => ParagraphContentRef::UnsupportedXml(
            UnsupportedXmlRef::modeled(WORD_NAMESPACE, "revision"),
        ),
        ParagraphItemRef::CommentRangeStart {
            has_child_content, ..
        } => ParagraphContentRef::UnsupportedXml(UnsupportedXmlRef::modeled_with_child_content(
            WORD_NAMESPACE,
            "commentRangeStart",
            has_child_content,
        )),
        ParagraphItemRef::CommentRangeEnd {
            has_child_content, ..
        } => ParagraphContentRef::UnsupportedXml(UnsupportedXmlRef::modeled_with_child_content(
            WORD_NAMESPACE,
            "commentRangeEnd",
            has_child_content,
        )),
        ParagraphItemRef::BookmarkStart {
            has_child_content, ..
        } => ParagraphContentRef::UnsupportedXml(UnsupportedXmlRef::modeled_with_child_content(
            WORD_NAMESPACE,
            "bookmarkStart",
            has_child_content,
        )),
        ParagraphItemRef::BookmarkEnd {
            has_child_content, ..
        } => ParagraphContentRef::UnsupportedXml(UnsupportedXmlRef::modeled_with_child_content(
            WORD_NAMESPACE,
            "bookmarkEnd",
            has_child_content,
        )),
        ParagraphItemRef::UnsupportedXml(raw) => {
            ParagraphContentRef::UnsupportedXml(UnsupportedXmlRef::new(raw))
        }
    }
}

impl<'a> ParagraphRef<'a> {
    /// Iterate over direct paragraph items in source order.
    ///
    /// Unlike [`Self::runs`], this retains hyperlinks, content controls,
    /// revisions, comment ranges, bookmarks, and preserved unmodelled XML.
    pub fn items(&self) -> impl Iterator<Item = ParagraphItemRef<'a>> {
        let mut items = Vec::new();
        let mut run_index = 0;
        while run_index <= self.inner.runs.len() {
            let extras = self
                .inner
                .extra_xml
                .iter()
                .enumerate()
                .filter(|(_, (at, _))| *at == run_index)
                .map(|(source_index, (_, raw))| (source_index, raw))
                .collect::<Vec<_>>();
            for raw_index in 0..=extras.len() {
                let markers = self
                    .inner
                    .comment_ranges
                    .iter()
                    .filter(|marker| match marker {
                        CommentRangeMarker::Start {
                            run_index: at,
                            raw_before,
                            ..
                        }
                        | CommentRangeMarker::End {
                            run_index: at,
                            raw_before,
                            ..
                        } => *at == run_index && (*raw_before).min(extras.len()) == raw_index,
                    })
                    .collect::<Vec<_>>();
                for marker_index in 0..=markers.len() {
                    items.extend(
                        self.inner
                            .content_controls
                            .iter()
                            .filter(|(at, raw_before, markers_before, _)| {
                                *at == run_index
                                    && (*raw_before).min(extras.len()) == raw_index
                                    && (*markers_before).min(markers.len()) == marker_index
                            })
                            .map(|(_, _, _, control)| {
                                ParagraphItemRef::ContentControl(ContentControlRef {
                                    inner: control,
                                })
                            }),
                    );
                    if let Some(marker) = markers.get(marker_index) {
                        items.push(match marker {
                            CommentRangeMarker::Start { id, .. } => {
                                ParagraphItemRef::CommentRangeStart {
                                    id: *id,
                                    has_child_content: extras.get(raw_index).is_some_and(
                                        |(_, raw)| UnsupportedXmlRef::new(raw).has_child_content(),
                                    ),
                                }
                            }
                            CommentRangeMarker::End { id, .. } => {
                                ParagraphItemRef::CommentRangeEnd {
                                    id: *id,
                                    has_child_content: extras.get(raw_index).is_some_and(
                                        |(_, raw)| UnsupportedXmlRef::new(raw).has_child_content(),
                                    ),
                                }
                            }
                        });
                    }
                }
                if let Some((source_index, raw)) = extras.get(raw_index)
                    && !self
                        .inner
                        .omitted_proof_error_raw_indices
                        .contains(source_index)
                {
                    let revision =
                        self.inner
                            .revisions
                            .iter()
                            .find_map(|(at, before, revision)| {
                                (*at == run_index
                                    && hyperlink_revision_index(*before).is_none()
                                    && *before == raw_index)
                                    .then_some(revision)
                            });
                    let bookmark = self.inner.bookmark_markers.iter().find(|bookmark| {
                        bookmark.run_index() == run_index && bookmark.raw_before() == raw_index
                    });
                    let empty_hyperlink =
                        self.inner.hyperlinks.iter().enumerate().find(|(_, link)| {
                            link.run_start == run_index
                                && link.run_end == run_index
                                && link.preserved_raw_before == Some(raw_index)
                        });
                    items.push(if let Some(revision) = revision {
                        ParagraphItemRef::Revision(RevisionRef { inner: revision })
                    } else if let Some(bookmark) = bookmark {
                        if bookmark.is_start() {
                            ParagraphItemRef::BookmarkStart {
                                id: bookmark.id(),
                                name: bookmark.name(),
                                has_child_content: UnsupportedXmlRef::new(raw).has_child_content(),
                            }
                        } else {
                            ParagraphItemRef::BookmarkEnd {
                                id: bookmark.id(),
                                has_child_content: UnsupportedXmlRef::new(raw).has_child_content(),
                            }
                        }
                    } else if let Some((index, _)) = empty_hyperlink {
                        ParagraphItemRef::Hyperlink(HyperlinkRef {
                            paragraph: self.inner,
                            index,
                        })
                    } else {
                        ParagraphItemRef::UnsupportedXml(raw.as_slice())
                    });
                }
            }

            if run_index == self.inner.runs.len() {
                break;
            }
            if let Some((index, hyperlink)) =
                self.inner
                    .hyperlinks
                    .iter()
                    .enumerate()
                    .find(|(_, hyperlink)| {
                        hyperlink.run_start == run_index && hyperlink.run_end > run_index
                    })
            {
                items.push(ParagraphItemRef::Hyperlink(HyperlinkRef {
                    paragraph: self.inner,
                    index,
                }));
                run_index = hyperlink.run_end;
            } else {
                items.push(ParagraphItemRef::Run(RunRef {
                    inner: &self.inner.runs[run_index],
                }));
                run_index += 1;
            }
        }
        items.into_iter()
    }

    /// Get the number of runs in this paragraph.
    pub fn run_count(&self) -> usize {
        self.inner.runs.len()
    }

    /// Get an immutable run by index.
    pub fn run(&self, index: usize) -> Option<RunRef<'_>> {
        self.inner.runs.get(index).map(|inner| RunRef { inner })
    }

    /// Get the paragraph style ID, if set.
    pub fn style_id(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.style_id.as_deref())
    }

    /// Check if a page break is set before this paragraph.
    pub fn is_page_break_before(&self) -> bool {
        self.page_break_before_value().unwrap_or(false)
    }

    /// Get direct keep-with-next formatting without collapsing inheritance.
    pub fn keep_with_next_value(&self) -> Option<bool> {
        self.inner.properties.as_ref().and_then(|ppr| ppr.keep_next)
    }

    /// Get direct keep-together formatting without collapsing inheritance.
    pub fn keep_together_value(&self) -> Option<bool> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.keep_lines)
    }

    /// Get direct page-break-before formatting without collapsing inheritance.
    pub fn page_break_before_value(&self) -> Option<bool> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.page_break_before)
    }

    /// Get direct widow-control formatting without collapsing inheritance.
    pub fn widow_control_value(&self) -> Option<bool> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.widow_control)
    }

    /// Line spacing as a multiplier of single spacing (1.0, 1.5, 2.0…)
    /// when the spacing rule is "auto" (the w:line value counts 240ths of
    /// a line). Returns None when unset or when the rule is exact/atLeast
    /// point spacing.
    pub fn line_spacing_multiple(&self) -> Option<f64> {
        let ppr = self.inner.properties.as_ref()?;
        let line = ppr.line_spacing?;
        match ppr.line_rule.as_deref() {
            None | Some("auto") => Some(line.0 as f64 / 240.0),
            _ => None,
        }
    }

    /// Get exact or at-least line spacing as a length.
    pub fn line_spacing(&self) -> Option<Length> {
        let ppr = self.inner.properties.as_ref()?;
        let line = ppr.line_spacing?;
        match ppr.line_rule.as_deref() {
            Some("exact") | Some("atLeast") => Some(Length::twips(line.0)),
            _ => None,
        }
    }

    /// Hyperlink spans in this paragraph as (run_start, run_end, rel_id).
    /// Resolve rel_id to a URL with `Document::hyperlink_url`.
    pub fn hyperlink_spans(&self) -> Vec<(usize, usize, Option<&str>)> {
        self.inner
            .hyperlinks
            .iter()
            .map(|h| (h.run_start, h.run_end, h.rel_id.as_deref()))
            .collect()
    }

    /// Get list numbering as (num_id, level) if this paragraph is a list
    /// item. Resolve bullet-vs-numbered via `Document::numbering_is_bullet`.
    pub fn numbering(&self) -> Option<(u32, u32)> {
        let ppr = self.inner.properties.as_ref()?;
        Some((ppr.num_id?, ppr.num_ilvl.unwrap_or(0)))
    }

    /// Get the alignment, if set.
    pub fn alignment(&self) -> Option<Alignment> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.jc)
            .map(Alignment::from_st_jc)
    }

    /// Get direct space before the paragraph.
    pub fn space_before(&self) -> Option<Length> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.space_before)
            .map(|twips| Length::twips(twips.0))
    }

    /// Get direct space after the paragraph.
    pub fn space_after(&self) -> Option<Length> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.space_after)
            .map(|twips| Length::twips(twips.0))
    }

    /// Get direct left indentation.
    pub fn indent_left(&self) -> Option<Length> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.ind_left)
            .map(|twips| Length::twips(twips.0))
    }

    /// Get direct right indentation.
    pub fn indent_right(&self) -> Option<Length> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.ind_right)
            .map(|twips| Length::twips(twips.0))
    }

    /// Get signed first-line indentation, negative for hanging indentation.
    pub fn first_line_indent(&self) -> Option<Length> {
        let ppr = self.inner.properties.as_ref()?;
        if let Some(twips) = ppr.ind_first_line {
            return Some(Length::twips(twips.0));
        }
        ppr.ind_hanging
            .map(|twips| Length::twips(twips.0.saturating_neg()))
    }

    /// Get an iterator over immutable run references.
    pub fn runs(&self) -> impl Iterator<Item = RunRef<'_>> {
        self.inner.runs.iter().map(|r| RunRef { inner: r })
    }

    /// Check if paragraph has borders.
    pub fn has_borders(&self) -> bool {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.borders.as_ref())
            .map(|b| !b.is_empty())
            .unwrap_or(false)
    }

    /// Get the direct bottom border, if present.
    pub fn bottom_border(&self) -> Option<ParagraphBorderRef<'_>> {
        self.inner
            .properties
            .as_ref()?
            .borders
            .as_ref()?
            .bottom
            .as_ref()
            .map(|inner| ParagraphBorderRef { inner })
    }

    /// Count direct paragraph border edges.
    pub fn border_count(&self) -> usize {
        let Some(borders) = self
            .inner
            .properties
            .as_ref()
            .and_then(|properties| properties.borders.as_ref())
        else {
            return 0;
        };

        [
            borders.top.as_ref(),
            borders.bottom.as_ref(),
            borders.left.as_ref(),
            borders.right.as_ref(),
            borders.between.as_ref(),
            borders.bar.as_ref(),
        ]
        .into_iter()
        .flatten()
        .count()
    }

    /// Get the number of tab stops defined.
    pub fn tab_stop_count(&self) -> usize {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.tabs.as_ref())
            .map(|t| t.tabs.len())
            .unwrap_or(0)
    }

    /// Get the shading fill color, if set.
    pub fn shading_fill(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|ppr| ppr.shading.as_ref())
            .and_then(|shd| shd.fill.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::{Reader, events::Event};
    use rdocx_oxml::namespace::{R_NS, W_NS};

    #[test]
    fn reader_exposes_hyperlink_tooltip_and_document_location() {
        let mut paragraph = CT_P::new();
        paragraph.add_run("linked");
        paragraph.hyperlinks.push(HyperlinkSpan {
            rel_id: Some("rId1".to_owned()),
            anchor: None,
            tooltip: Some("Open link".to_owned()),
            doc_location: Some("section-two".to_owned()),
            run_start: 0,
            run_end: 1,
            extra_attributes: Vec::new(),
            extra_xml: Vec::new(),
            preserved_raw_before: None,
        });

        let hyperlink = HyperlinkRef {
            paragraph: &paragraph,
            index: 0,
        };
        assert_eq!(hyperlink.tooltip(), Some("Open link"));
        assert_eq!(hyperlink.doc_location(), Some("section-two"));
        assert!(!hyperlink.has_unmodeled_semantic_attributes());
    }

    #[test]
    fn reader_preserves_ordered_inline_content_inside_insertions() {
        let xml = format!(
            r#"<w:p xmlns:w="{W_NS}" xmlns:r="{R_NS}"><w:ins w:id="1"><w:r><w:t>before </w:t></w:r><w:hyperlink r:id="rId1"><w:r><w:t>linked</w:t></w:r></w:hyperlink><w:fldSimple w:instr=" PAGE "><w:r><w:t>4</w:t></w:r></w:fldSimple></w:ins></w:p>"#
        );
        let mut reader = Reader::from_reader(xml.as_bytes());
        let mut buffer = Vec::new();
        assert!(matches!(
            reader.read_event_into(&mut buffer),
            Ok(Event::Start(_))
        ));
        let paragraph = CT_P::from_xml(&mut reader).expect("paragraph parses");
        let paragraph = ParagraphRef { inner: &paragraph };
        let paragraph_content = paragraph.content().collect::<Vec<_>>();
        let [ParagraphContentRef::Insertion(insertion)] = paragraph_content.as_slice() else {
            panic!("insertion is exposed");
        };
        let content = insertion.content().collect::<Vec<_>>();
        assert!(matches!(&content[0], ParagraphContentRef::Run(run) if run.text() == "before "));
        assert!(
            matches!(&content[1], ParagraphContentRef::Hyperlink(link) if link.relationship_id() == Some("rId1") && link.text() == "linked")
        );
        assert!(
            matches!(&content[2], ParagraphContentRef::SimpleField(field) if field.cached_text() == "4")
        );
    }

    #[test]
    fn reader_omits_safe_proofing_markers() {
        let xml = format!(
            r#"<w:p xmlns:w="{W_NS}"><w:proofErr w:type="spellStart"/><w:r><w:t>word</w:t></w:r><w:proofErr w:type="spellEnd"/></w:p>"#
        );
        let mut reader = Reader::from_reader(xml.as_bytes());
        let mut buffer = Vec::new();
        assert!(matches!(
            reader.read_event_into(&mut buffer),
            Ok(Event::Start(_))
        ));
        let paragraph = CT_P::from_xml(&mut reader).expect("paragraph parses");
        let paragraph = ParagraphRef { inner: &paragraph };

        let items = paragraph.items().collect::<Vec<_>>();
        assert!(matches!(items.as_slice(), [ParagraphItemRef::Run(run)] if run.text() == "word"));
    }

    #[test]
    fn reader_parses_tooltip_from_document_namespace_scope() {
        let xml = format!(
            r#"<w:document xmlns:w="{W_NS}" xmlns:r="{R_NS}"><w:body><w:p><w:hyperlink w:tooltip="Open &amp; inspect" r:id="rId1"><w:r><w:t>linked</w:t></w:r></w:hyperlink></w:p></w:body></w:document>"#
        );
        let document = rdocx_oxml::CT_Document::from_xml(xml.as_bytes()).expect("document parses");
        let rdocx_oxml::BodyContent::Paragraph(paragraph) = &document.body.content[0] else {
            panic!("paragraph is retained");
        };
        let paragraph = ParagraphRef { inner: paragraph };
        let content = paragraph.content().collect::<Vec<_>>();
        let [ParagraphContentRef::Hyperlink(hyperlink)] = content.as_slice() else {
            panic!("hyperlink is exposed");
        };
        assert_eq!(hyperlink.relationship_id(), Some("rId1"));
        assert_eq!(hyperlink.tooltip(), Some("Open & inspect"));
        assert!(!hyperlink.has_unmodeled_semantic_attributes());
    }
}

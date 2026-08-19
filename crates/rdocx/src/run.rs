//! Run — a contiguous stretch of text with uniform formatting.

use rdocx_oxml::drawing::CT_Drawing;
use rdocx_oxml::properties::{CT_RPr, CT_Shd};
use rdocx_oxml::shared::ST_Underline;
use rdocx_oxml::text::{BreakType, CT_R, CT_Text, FieldType, RunContent};
use rdocx_oxml::units::{HalfPoint, Twips};

use crate::Length;

/// A break embedded in a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakKind {
    /// A line break within the current paragraph.
    Line,
    /// A page break.
    Page,
    /// A column break.
    Column,
}

/// The instruction of a simple Word field embedded in a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind<'a> {
    /// The current page number.
    Page,
    /// The document's total page count.
    NumPages,
    /// Any other field instruction.
    Other(&'a str),
}

/// Semantic kind of drawing exposed by the reader facade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingKind {
    /// A DrawingML picture with an image relationship.
    Image,
    /// An anchored DrawingML shape.
    Shape,
    /// Any other drawing construct.
    Other,
}

/// How an image relationship obtains its content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingRelationshipKind {
    /// The drawing refers to an image part within the package.
    Embedded,
    /// The drawing refers to an external image relationship.
    Linked,
}

/// An immutable drawing embedded in a run.
#[derive(Debug, Clone, Copy)]
pub struct DrawingRef<'a> {
    inner: &'a CT_Drawing,
}

impl<'a> DrawingRef<'a> {
    /// Semantic content kind.
    pub fn kind(&self) -> DrawingKind {
        if self.relationship_id().is_some() {
            DrawingKind::Image
        } else if self
            .inner
            .anchor
            .as_ref()
            .is_some_and(|anchor| anchor.shape.is_some())
        {
            DrawingKind::Shape
        } else {
            DrawingKind::Other
        }
    }

    /// Whether this drawing is inline with the surrounding text.
    pub fn is_inline(&self) -> bool {
        self.inner.inline.is_some()
    }

    /// Whether this drawing is floating or anchored.
    pub fn is_anchor(&self) -> bool {
        self.inner.anchor.is_some()
    }

    /// The relationship ID for the drawing's embedded or linked image, when
    /// present.
    pub fn relationship_id(&self) -> Option<&str> {
        self.inner
            .inline
            .as_ref()
            .and_then(|inline| {
                (!inline.embed_id.is_empty())
                    .then_some(inline.embed_id.as_str())
                    .or(inline.link_id.as_deref())
            })
            .or_else(|| {
                self.inner.anchor.as_ref().and_then(|anchor| {
                    (!anchor.embed_id.is_empty())
                        .then_some(anchor.embed_id.as_str())
                        .or(anchor.link_id.as_deref())
                })
            })
    }

    /// Whether the image relationship is embedded in the package or linked.
    pub fn relationship_kind(&self) -> Option<DrawingRelationshipKind> {
        self.inner
            .inline
            .as_ref()
            .map(|inline| inline.embed_id.is_empty() && inline.link_id.is_some())
            .or_else(|| {
                self.inner
                    .anchor
                    .as_ref()
                    .map(|anchor| anchor.embed_id.is_empty() && anchor.link_id.is_some())
            })
            .filter(|_| self.relationship_id().is_some())
            .map(|linked| {
                if linked {
                    DrawingRelationshipKind::Linked
                } else {
                    DrawingRelationshipKind::Embedded
                }
            })
    }

    /// The drawing description, commonly used as image alt text.
    pub fn description(&self) -> Option<&str> {
        self.inner
            .inline
            .as_ref()
            .and_then(|inline| inline.description.as_deref())
            .or_else(|| {
                self.inner
                    .anchor
                    .as_ref()
                    .and_then(|anchor| anchor.description.as_deref())
            })
    }

    /// The drawing name from its non-visual properties.
    pub fn name(&self) -> Option<&str> {
        self.inner
            .inline
            .as_ref()
            .and_then(|inline| inline.name.as_deref())
            .or_else(|| {
                self.inner
                    .anchor
                    .as_ref()
                    .and_then(|anchor| anchor.name.as_deref())
            })
    }

    /// The drawing width.
    pub fn width(&self) -> Option<Length> {
        self.inner
            .inline
            .as_ref()
            .map(|inline| Length::emu(inline.extent_cx.0))
            .or_else(|| {
                self.inner
                    .anchor
                    .as_ref()
                    .map(|anchor| Length::emu(anchor.extent_cx.0))
            })
    }

    /// The drawing height.
    pub fn height(&self) -> Option<Length> {
        self.inner
            .inline
            .as_ref()
            .map(|inline| Length::emu(inline.extent_cy.0))
            .or_else(|| {
                self.inner
                    .anchor
                    .as_ref()
                    .map(|anchor| Length::emu(anchor.extent_cy.0))
            })
    }
}

/// One direct child of a run, in source order.
#[derive(Debug, Clone, Copy)]
pub enum RunItemRef<'a> {
    /// Literal text.
    Text(&'a str),
    /// Text in a deleted revision.
    DeletedText(&'a str),
    /// A tab character.
    Tab,
    /// A line, page, or column break.
    Break(BreakKind),
    /// An inline or anchored drawing.
    Drawing(DrawingRef<'a>),
    /// A simple Word field with its cached display text.
    Field {
        /// The parsed field instruction.
        kind: FieldKind<'a>,
        /// The field's cached display text.
        display: &'a str,
    },
    /// A footnote reference ID.
    FootnoteReference(i32),
    /// An endnote reference ID.
    EndnoteReference(i32),
    /// A comment reference ID.
    CommentReference(i32),
    /// A preserved run child that rdocx does not model.
    UnsupportedXml(&'a [u8]),
}

/// One run child exposed through the compatibility reader facade.
pub enum RunContentRef<'a> {
    /// Literal text.
    Text(&'a str),
    /// A tab character.
    Tab,
    /// A line, page, or column break.
    Break(BreakKind),
    /// An inline or anchored drawing.
    Drawing(DrawingRef<'a>),
    /// A footnote reference ID.
    FootnoteReference(i32),
    /// An endnote reference ID.
    EndnoteReference(i32),
    /// Content that the compatibility facade does not model.
    UnsupportedXml(crate::UnsupportedXmlRef<'a>),
}

/// Underline style for runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderlineStyle {
    None,
    Single,
    Double,
    Thick,
    Dotted,
    Dash,
    Wave,
    Words,
}

impl UnderlineStyle {
    fn to_code(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Single => 1,
            Self::Words => 2,
            Self::Double => 3,
            Self::Dotted => 4,
            Self::Thick => 6,
            Self::Dash => 7,
            Self::Wave => 11,
        }
    }
}

fn underline_from_code(code: i32) -> Option<ST_Underline> {
    match code {
        0 => Some(ST_Underline::None),
        1 => Some(ST_Underline::Single),
        2 => Some(ST_Underline::Words),
        3 => Some(ST_Underline::Double),
        4 => Some(ST_Underline::Dotted),
        6 => Some(ST_Underline::Thick),
        7 => Some(ST_Underline::Dash),
        9 => Some(ST_Underline::DotDash),
        10 => Some(ST_Underline::DotDotDash),
        11 => Some(ST_Underline::Wave),
        _ => None,
    }
}

fn underline_to_code(value: ST_Underline) -> i32 {
    match value {
        ST_Underline::None => 0,
        ST_Underline::Single => 1,
        ST_Underline::Words => 2,
        ST_Underline::Double => 3,
        ST_Underline::Dotted => 4,
        ST_Underline::Thick => 6,
        ST_Underline::Dash => 7,
        ST_Underline::DotDash => 9,
        ST_Underline::DotDotDash => 10,
        ST_Underline::Wave => 11,
    }
}

/// A run of text within a paragraph.
///
/// All text in a run shares the same formatting (font, size, bold, etc.).
pub struct Run<'a> {
    pub(crate) inner: &'a mut CT_R,
}

impl<'a> Run<'a> {
    /// Get the text content of this run.
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Set the text content, replacing all existing content.
    pub fn set_text(&mut self, text: &str) {
        self.inner
            .replace_content(vec![RunContent::Text(CT_Text::new(text))]);
    }

    /// Add text to this run.
    pub fn add_text(&mut self, text: &str) {
        self.inner
            .append_content(RunContent::Text(CT_Text::new(text)));
    }

    /// Set bold formatting.
    pub fn bold(mut self, val: bool) -> Self {
        self.set_bold(val);
        self
    }

    /// Set bold formatting in place.
    pub fn set_bold(&mut self, val: bool) {
        self.set_bold_value(Some(val));
    }

    /// Set or clear direct bold formatting in place.
    pub fn set_bold_value(&mut self, val: Option<bool>) {
        if val.is_none() && self.inner.properties.is_none() {
            return;
        }
        let rpr = self.ensure_rpr();
        rpr.bold = val;
        rpr.bold_cs = val;
    }

    /// Set italic formatting.
    pub fn italic(mut self, val: bool) -> Self {
        self.set_italic(val);
        self
    }

    /// Set italic formatting in place.
    pub fn set_italic(&mut self, val: bool) {
        self.set_italic_value(Some(val));
    }

    /// Set or clear direct italic formatting in place.
    pub fn set_italic_value(&mut self, val: Option<bool>) {
        if val.is_none() && self.inner.properties.is_none() {
            return;
        }
        let rpr = self.ensure_rpr();
        rpr.italic = val;
        rpr.italic_cs = val;
    }

    /// Set underline formatting (simple on/off).
    pub fn underline(mut self, val: bool) -> Self {
        self.set_underline(val);
        self
    }

    /// Set underline formatting in place.
    pub fn set_underline(&mut self, val: bool) {
        self.set_underline_style_value(Some(if val {
            UnderlineStyle::Single
        } else {
            UnderlineStyle::None
        }));
    }

    /// Set underline with a specific style.
    pub fn underline_style(mut self, style: UnderlineStyle) -> Self {
        self.set_underline_style(style);
        self
    }

    /// Set an underline style in place.
    pub fn set_underline_style(&mut self, style: UnderlineStyle) {
        self.set_underline_style_value(Some(style));
    }

    /// Set or clear direct underline formatting in place.
    pub fn set_underline_style_value(&mut self, style: Option<UnderlineStyle>) {
        let applied = self.set_underline_code_value(style.map(UnderlineStyle::to_code));
        debug_assert!(applied);
    }

    /// Set or clear a direct underline code used by language bindings.
    ///
    /// Returns false without mutation when `code` is not in the bounded public
    /// Python underline inventory.
    pub fn set_underline_code_value(&mut self, code: Option<i32>) -> bool {
        let underline = match code {
            Some(code) => match underline_from_code(code) {
                Some(underline) => Some(underline),
                None => return false,
            },
            None => None,
        };
        if underline.is_none() && self.inner.properties.is_none() {
            return true;
        }
        self.ensure_rpr().underline = underline;
        true
    }

    /// Set font size in points.
    pub fn size(mut self, pt: f64) -> Self {
        self.set_size(pt);
        self
    }

    /// Set font size in place.
    pub fn set_size(&mut self, pt: f64) {
        self.set_size_value(Some(pt));
    }

    /// Set or clear the direct font size in points.
    pub fn set_size_value(&mut self, pt: Option<f64>) {
        if pt.is_none() && self.inner.properties.is_none() {
            return;
        }
        let hp = pt.map(HalfPoint::from_pt);
        let rpr = self.ensure_rpr();
        rpr.sz = hp;
        rpr.sz_cs = hp;
    }

    /// Set the font name.
    pub fn font(mut self, name: &str) -> Self {
        self.set_font(name);
        self
    }

    /// Set the font name in place.
    pub fn set_font(&mut self, name: &str) {
        self.set_font_value(Some(name));
    }

    /// Set or clear the direct font name.
    pub fn set_font_value(&mut self, name: Option<&str>) {
        if name.is_none() && self.inner.properties.is_none() {
            return;
        }
        let rpr = self.ensure_rpr();
        rpr.font_ascii = name.map(str::to_owned);
        rpr.font_hansi = name.map(str::to_owned);
        rpr.font_east_asia = name.map(str::to_owned);
        rpr.font_cs = name.map(str::to_owned);
    }

    /// Set text color as a hex string (e.g., "FF0000" for red).
    pub fn color(mut self, hex: &str) -> Self {
        self.set_color(hex);
        self
    }

    /// Set text color in place.
    pub fn set_color(&mut self, hex: &str) {
        self.set_color_value(Some(hex));
    }

    /// Set or clear the direct text color.
    pub fn set_color_value(&mut self, hex: Option<&str>) {
        if hex.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_rpr().color = hex.map(str::to_owned);
    }

    /// Set highlight color as a hex fill value.
    pub fn highlight(mut self, color: &str) -> Self {
        self.set_highlight(color);
        self
    }

    /// Set highlight color in place.
    pub fn set_highlight(&mut self, color: &str) {
        self.ensure_rpr().shading = Some(CT_Shd {
            val: "clear".to_string(),
            color: Some("auto".to_string()),
            fill: Some(color.to_string()),
        });
    }

    /// Set strikethrough formatting.
    pub fn strike(mut self, val: bool) -> Self {
        self.set_strike(val);
        self
    }

    /// Set strikethrough formatting in place.
    pub fn set_strike(&mut self, val: bool) {
        self.set_strike_value(Some(val));
    }

    /// Set or clear direct strikethrough formatting in place.
    pub fn set_strike_value(&mut self, val: Option<bool>) {
        if val.is_none() && self.inner.properties.is_none() {
            return;
        }
        self.ensure_rpr().strike = val;
    }

    /// Set double strikethrough.
    pub fn double_strike(mut self, val: bool) -> Self {
        self.set_double_strike(val);
        self
    }

    /// Set double strikethrough in place.
    pub fn set_double_strike(&mut self, val: bool) {
        self.ensure_rpr().dstrike = Some(val);
    }

    /// Set all caps.
    pub fn all_caps(mut self, val: bool) -> Self {
        self.set_all_caps(val);
        self
    }

    /// Set all caps in place.
    pub fn set_all_caps(&mut self, val: bool) {
        self.ensure_rpr().caps = Some(val);
    }

    /// Set small caps.
    pub fn small_caps(mut self, val: bool) -> Self {
        self.set_small_caps(val);
        self
    }

    /// Set small caps in place.
    pub fn set_small_caps(&mut self, val: bool) {
        self.ensure_rpr().small_caps = Some(val);
    }

    /// Set superscript.
    pub fn superscript(mut self) -> Self {
        self.set_superscript();
        self
    }

    /// Set superscript in place.
    pub fn set_superscript(&mut self) {
        self.ensure_rpr().vert_align = Some("superscript".to_string());
    }

    /// Set subscript.
    pub fn subscript(mut self) -> Self {
        self.set_subscript();
        self
    }

    /// Set subscript in place.
    pub fn set_subscript(&mut self) {
        self.ensure_rpr().vert_align = Some("subscript".to_string());
    }

    /// Set character spacing (positive = expanded, negative = condensed).
    pub fn character_spacing(mut self, spacing: Length) -> Self {
        self.set_character_spacing(spacing);
        self
    }

    /// Set character spacing in place.
    pub fn set_character_spacing(&mut self, spacing: Length) {
        self.ensure_rpr().spacing = Some(spacing.as_twips());
    }

    /// Set character width scale in percent (100 = normal).
    pub fn width_scale(mut self, percent: u32) -> Self {
        self.set_width_scale(percent);
        self
    }

    /// Set character width scale in place.
    pub fn set_width_scale(&mut self, percent: u32) {
        self.ensure_rpr().width_scale = Some(percent);
    }

    /// Set text position (positive = raised, negative = lowered) in half-points.
    pub fn position(mut self, half_points: i32) -> Self {
        self.set_position(half_points);
        self
    }

    /// Set text position in place.
    pub fn set_position(&mut self, half_points: i32) {
        self.ensure_rpr().position = Some(half_points);
    }

    /// Set hidden/vanish text.
    pub fn hidden(mut self, val: bool) -> Self {
        self.set_hidden(val);
        self
    }

    /// Set hidden/vanish text in place.
    pub fn set_hidden(&mut self, val: bool) {
        self.ensure_rpr().vanish = Some(val);
    }

    /// Set the character style by ID.
    pub fn style(mut self, style_id: &str) -> Self {
        self.set_style(style_id);
        self
    }

    /// Set the character style by ID in place.
    pub fn set_style(&mut self, style_id: &str) {
        self.ensure_rpr().style_id = Some(style_id.to_string());
    }

    fn ensure_rpr(&mut self) -> &mut CT_RPr {
        self.inner.ensure_properties()
    }
}

/// An immutable reference to a run.
pub struct RunRef<'a> {
    pub(crate) inner: &'a CT_R,
}

impl<'a> RunRef<'a> {
    /// Get the text content of this run.
    pub fn text(&self) -> String {
        self.inner.text()
    }

    /// Iterate over run content through the compatibility facade.
    pub fn content(&self) -> impl Iterator<Item = RunContentRef<'a>> {
        let property_boundary = usize::from(self.inner.properties.is_some());
        let mut content = Vec::with_capacity(self.inner.content.len() + self.inner.extra_xml.len());
        for index in 0..=self.inner.content.len() {
            let boundary = property_boundary + index;
            content.extend(
                self.inner
                    .extra_xml_positions
                    .iter()
                    .zip(&self.inner.extra_xml)
                    .enumerate()
                    .filter(|(_, (position, _))| **position == boundary)
                    .filter(|(raw_index, _)| {
                        !self
                            .inner
                            .modeled_field_marker_raw_indices
                            .contains(raw_index)
                            && !self
                                .inner
                                .omitted_alt_drawing_raw_indices
                                .contains(raw_index)
                    })
                    .map(|(_, (_, raw))| {
                        RunContentRef::UnsupportedXml(crate::UnsupportedXmlRef::new(raw))
                    }),
            );
            if let Some(item) = self.inner.content.get(index) {
                content.push(run_content_from_modeled(item));
            }
        }
        content.into_iter()
    }

    /// Iterate over direct run items in source order.
    ///
    /// This retains deleted text, comments, controls, drawings, fields, note
    /// references, and preserved unmodelled XML at their original boundaries.
    pub fn items(&self) -> impl Iterator<Item = RunItemRef<'_>> {
        let property_boundary = usize::from(self.inner.properties.is_some());
        let mut items = Vec::with_capacity(self.inner.content.len() + self.inner.extra_xml.len());
        for index in 0..=self.inner.content.len() {
            let boundary = property_boundary + index;
            items.extend(
                self.inner
                    .extra_xml_positions
                    .iter()
                    .zip(&self.inner.extra_xml)
                    .enumerate()
                    .filter(|(_, (position, _))| **position == boundary)
                    .filter(|(raw_index, _)| {
                        !self
                            .inner
                            .modeled_field_marker_raw_indices
                            .contains(raw_index)
                            && !self
                                .inner
                                .omitted_alt_drawing_raw_indices
                                .contains(raw_index)
                    })
                    .map(|(_, (_, raw))| RunItemRef::UnsupportedXml(raw.as_slice())),
            );
            if let Some(content) = self.inner.content.get(index) {
                items.push(match content {
                    RunContent::Text(text) => RunItemRef::Text(&text.text),
                    RunContent::DeletedText(text) => RunItemRef::DeletedText(&text.text),
                    RunContent::Tab => RunItemRef::Tab,
                    RunContent::Break(kind) => RunItemRef::Break(match kind {
                        BreakType::Line => BreakKind::Line,
                        BreakType::Page => BreakKind::Page,
                        BreakType::Column => BreakKind::Column,
                    }),
                    RunContent::Drawing(drawing) => {
                        RunItemRef::Drawing(DrawingRef { inner: drawing })
                    }
                    RunContent::Field {
                        field_type,
                        display,
                    } => RunItemRef::Field {
                        kind: match field_type {
                            FieldType::Page => FieldKind::Page,
                            FieldType::NumPages => FieldKind::NumPages,
                            FieldType::Ref { instruction, .. }
                            | FieldType::PageRef { instruction, .. } => {
                                FieldKind::Other(instruction)
                            }
                            FieldType::Other(instruction) => FieldKind::Other(instruction),
                        },
                        display,
                    },
                    RunContent::FootnoteRef { id } => RunItemRef::FootnoteReference(*id),
                    RunContent::EndnoteRef { id } => RunItemRef::EndnoteReference(*id),
                    RunContent::CommentReference { id, .. } => RunItemRef::CommentReference(*id),
                });
            }
        }
        items.into_iter()
    }

    /// The footnote id referenced by this run, if it holds a
    /// `<w:footnoteReference/>`.
    pub fn footnote_id(&self) -> Option<i32> {
        use rdocx_oxml::text::RunContent;
        self.inner.content.iter().find_map(|c| match c {
            RunContent::FootnoteRef { id } => Some(*id),
            _ => None,
        })
    }

    /// Check if bold.
    pub fn is_bold(&self) -> bool {
        self.bold_value().unwrap_or(false)
    }

    /// Get direct bold formatting without collapsing inheritance.
    pub fn bold_value(&self) -> Option<bool> {
        self.inner.properties.as_ref().and_then(|rpr| rpr.bold)
    }

    /// Check if italic.
    pub fn is_italic(&self) -> bool {
        self.italic_value().unwrap_or(false)
    }

    /// Get direct italic formatting without collapsing inheritance.
    pub fn italic_value(&self) -> Option<bool> {
        self.inner.properties.as_ref().and_then(|rpr| rpr.italic)
    }

    /// Check if strikethrough.
    pub fn is_strike(&self) -> bool {
        self.strike_value().unwrap_or(false)
    }

    /// Get direct strikethrough formatting without collapsing inheritance.
    pub fn strike_value(&self) -> Option<bool> {
        self.inner.properties.as_ref().and_then(|rpr| rpr.strike)
    }

    /// Check if underlined (any underline style other than none).
    pub fn is_underline(&self) -> bool {
        self.underline_code_value().is_some_and(|code| code != 0)
    }

    /// Get the direct underline code used by language bindings.
    pub fn underline_code_value(&self) -> Option<i32> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.underline)
            .map(underline_to_code)
    }

    /// Get font size in points, if set.
    pub fn size(&self) -> Option<f64> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.sz)
            .map(|hp| hp.to_pt())
    }

    /// Get font name, if set.
    pub fn font_name(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.font_ascii.as_deref())
    }

    /// Get text color, if set.
    pub fn color(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.color.as_deref())
    }

    /// Get character spacing in twips, if set.
    pub fn character_spacing(&self) -> Option<Twips> {
        self.inner.properties.as_ref().and_then(|rpr| rpr.spacing)
    }

    /// Get the highlight color, if set: either the `w:highlight` keyword
    /// (e.g. "yellow") or the shading fill value the `highlight()` builder
    /// writes — OOXML has two mechanisms for highlighted text.
    pub fn highlight(&self) -> Option<String> {
        let rpr = self.inner.properties.as_ref()?;
        if let Some(h) = rpr.highlight {
            return Some(h.to_str().to_string());
        }
        rpr.shading.as_ref().and_then(|sh| sh.fill.clone())
    }

    /// If this run contains an inline image, return (rel_id, alt text).
    pub fn inline_image(&self) -> Option<(&str, Option<&str>)> {
        use rdocx_oxml::text::RunContent;
        for c in &self.inner.content {
            if let RunContent::Drawing(d) = c
                && let Some(inline) = &d.inline
            {
                return Some((inline.embed_id.as_str(), inline.description.as_deref()));
            }
        }
        None
    }

    /// Get raised/lowered text position in half-points, if set.
    /// (LibreOffice encodes super/subscript this way on HTML import.)
    pub fn position(&self) -> Option<i32> {
        self.inner.properties.as_ref().and_then(|rpr| rpr.position)
    }

    /// Get vertical alignment (superscript/subscript), if set.
    pub fn vert_align(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.vert_align.as_deref())
    }

    /// Get the character style ID, if set.
    pub fn style_id(&self) -> Option<&str> {
        self.inner
            .properties
            .as_ref()
            .and_then(|rpr| rpr.style_id.as_deref())
    }
}

fn run_content_from_modeled<'a>(item: &'a RunContent) -> RunContentRef<'a> {
    match item {
        RunContent::Text(text) | RunContent::DeletedText(text) => RunContentRef::Text(&text.text),
        RunContent::Tab => RunContentRef::Tab,
        RunContent::Break(kind) => RunContentRef::Break(match kind {
            BreakType::Line => BreakKind::Line,
            BreakType::Page => BreakKind::Page,
            BreakType::Column => BreakKind::Column,
        }),
        RunContent::Drawing(drawing) => RunContentRef::Drawing(DrawingRef { inner: drawing }),
        RunContent::Field { display, .. } => RunContentRef::Text(display),
        RunContent::FootnoteRef { id } => RunContentRef::FootnoteReference(*id),
        RunContent::EndnoteRef { id } => RunContentRef::EndnoteReference(*id),
        RunContent::CommentReference { .. } => {
            RunContentRef::UnsupportedXml(crate::UnsupportedXmlRef::modeled_with_child_content(
                crate::unsupported_xml::WORD_NAMESPACE,
                "commentReference",
                false,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdocx_oxml::text::{CT_R, FieldType};

    #[test]
    fn run_items_preserve_typed_and_raw_child_order() {
        let xml = br#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:x="urn:foreign"><w:rPr/><w:t>first</w:t><x:before-comment/><w:commentReference w:id="4"/><x:before-tab/><w:tab/><w:delText>removed</w:delText><w:footnoteReference w:id="7"/><w:endnoteReference w:id="8"/><w:br w:type="page"/></w:r>"#;
        let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        let run = match reader.read_event_into(&mut buffer).unwrap() {
            quick_xml::events::Event::Start(_) => CT_R::from_xml(&mut reader).unwrap(),
            event => panic!("expected run start, got {event:?}"),
        };
        let run = RunRef { inner: &run };

        let items = run
            .items()
            .map(|item| match item {
                RunItemRef::Text(text) => format!("text:{text}"),
                RunItemRef::DeletedText(text) => format!("deleted:{text}"),
                RunItemRef::Tab => "tab".to_owned(),
                RunItemRef::Break(BreakKind::Page) => "break:page".to_owned(),
                RunItemRef::FootnoteReference(id) => format!("footnote:{id}"),
                RunItemRef::EndnoteReference(id) => format!("endnote:{id}"),
                RunItemRef::CommentReference(id) => format!("comment:{id}"),
                RunItemRef::UnsupportedXml(raw) => {
                    format!("raw:{}", std::str::from_utf8(raw).unwrap())
                }
                item => panic!("unexpected item: {item:?}"),
            })
            .collect::<Vec<_>>();

        assert_eq!(
            items,
            [
                "text:first",
                "raw:<x:before-comment/>",
                "comment:4",
                "raw:<x:before-tab/>",
                "tab",
                "deleted:removed",
                "footnote:7",
                "endnote:8",
                "break:page",
            ]
        );
    }

    #[test]
    fn run_items_expose_simple_fields() {
        let run = CT_R {
            properties: None,
            content: vec![RunContent::Field {
                field_type: FieldType::Ref {
                    bookmark: "target".to_owned(),
                    instruction: " REF target \\h ".to_owned(),
                },
                display: "Target text".to_owned(),
            }],
            simple_field: None,
            extra_xml: Vec::new(),
            extra_xml_positions: Vec::new(),
            field_markers: Vec::new(),
            field_marker_raw_indices: Vec::new(),
            modeled_field_marker_raw_indices: Vec::new(),
            alt_drawings: Vec::new(),
            omitted_alt_drawing_raw_indices: Vec::new(),
        };
        let run = RunRef { inner: &run };

        let items = run.items().collect::<Vec<_>>();
        assert!(matches!(
            items.as_slice(),
            [RunItemRef::Field {
                kind: FieldKind::Other(" REF target \\h "),
                display: "Target text",
            }]
        ));
    }

    #[test]
    fn reader_models_unlocked_empty_complex_field_markers() {
        let xml = br#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:fldChar w:fldCharType="begin" w:fldLock="0"/></w:r><w:r><w:instrText xml:space="preserve"> HYPERLINK &quot;mailto:reader@example.test&quot; </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate" w:fldLock="0"/></w:r><w:r><w:t>Reader link</w:t></w:r><w:r><w:fldChar w:fldCharType="end" w:fldLock="0"/></w:r></w:p>"#;
        let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        let paragraph = match reader.read_event_into(&mut buffer).unwrap() {
            quick_xml::events::Event::Start(_) => {
                rdocx_oxml::text::CT_P::from_xml(&mut reader).unwrap()
            }
            event => panic!("expected paragraph start, got {event:?}"),
        };
        let marker_runs = [0, 1, 2, 4];

        for index in marker_runs {
            let run = RunRef {
                inner: &paragraph.runs[index],
            };
            assert!(run.items().next().is_none());
            assert!(run.content().next().is_none());
        }
    }

    #[test]
    fn reader_models_legacy_form_text_fields() {
        let xml = br#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:r><w:fldChar w:fldCharType="begin"><w:ffData><w:name w:val="resident"/><w:enabled/><w:textInput/></w:ffData></w:fldChar></w:r><w:r><w:instrText xml:space="preserve"> FORMTEXT </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>Visible resident</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
        let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        let paragraph = match reader.read_event_into(&mut buffer).unwrap() {
            quick_xml::events::Event::Start(_) => {
                rdocx_oxml::text::CT_P::from_xml(&mut reader).unwrap()
            }
            event => panic!("expected paragraph start, got {event:?}"),
        };

        for index in [0, 1, 2, 4] {
            let run = RunRef {
                inner: &paragraph.runs[index],
            };
            assert!(run.items().next().is_none());
            assert!(run.content().next().is_none());
        }
        assert_eq!(
            RunRef {
                inner: &paragraph.runs[3]
            }
            .text(),
            "Visible resident"
        );
    }

    #[test]
    fn reader_retains_complex_field_markers_with_unknown_attributes() {
        let xml = br#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:fldChar w:fldCharType="begin" w:fldLock="0" w:unknown="value"/></w:r>"#;
        let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        let run = match reader.read_event_into(&mut buffer).unwrap() {
            quick_xml::events::Event::Start(_) => CT_R::from_xml(&mut reader).unwrap(),
            event => panic!("expected run start, got {event:?}"),
        };
        let run = RunRef { inner: &run };

        let items = run.items().collect::<Vec<_>>();
        let [RunItemRef::UnsupportedXml(raw)] = items.as_slice() else {
            panic!("unmodeled marker attribute is retained");
        };
        assert!(std::str::from_utf8(raw).unwrap().contains("w:unknown"));
    }

    #[test]
    fn reader_omits_textless_alternate_content_shapes() {
        let xml = br#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:v="urn:schemas-microsoft-com:vml"><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wp:anchor><wp:extent cx="914400" cy="457200"/><a:graphic><a:graphicData><wps:wsp><wps:spPr><a:prstGeom prst="line"/></wps:spPr></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></mc:Choice><mc:Fallback><w:pict><v:shape id="fallback"/></w:pict></mc:Fallback></mc:AlternateContent></w:r></w:p>"#;
        let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        let paragraph = match reader.read_event_into(&mut buffer).unwrap() {
            quick_xml::events::Event::Start(_) => {
                rdocx_oxml::text::CT_P::from_xml(&mut reader).unwrap()
            }
            event => panic!("expected paragraph start, got {event:?}"),
        };
        let run = RunRef {
            inner: &paragraph.runs[0],
        };

        assert!(run.items().next().is_none());
        assert!(run.content().next().is_none());
    }

    #[test]
    fn run_content_exposes_nested_modeled_leaves_as_raw_facts() {
        let xml = br#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:tab><w:t>must not disappear</w:t></w:tab></w:r>"#;
        let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        let run = match reader.read_event_into(&mut buffer).unwrap() {
            quick_xml::events::Event::Start(_) => CT_R::from_xml(&mut reader).unwrap(),
            event => panic!("expected run start, got {event:?}"),
        };
        let run = RunRef { inner: &run };
        let content = run.content().collect::<Vec<_>>();
        let [RunContentRef::UnsupportedXml(raw)] = content.as_slice() else {
            panic!("nested tab is retained as raw XML");
        };
        assert_eq!(raw.local_name(), "tab");
        assert!(raw.has_child_content());
    }

    #[test]
    fn drawing_facts_distinguish_linked_images() {
        let xml = br#"<w:r xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:drawing><wp:inline><wp:extent cx="10" cy="20"/><a:graphic><a:blip r:link="rId7"/></a:graphic></wp:inline></w:drawing></w:r>"#;
        let mut reader = quick_xml::Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        let run = match reader.read_event_into(&mut buffer).unwrap() {
            quick_xml::events::Event::Start(_) => CT_R::from_xml(&mut reader).unwrap(),
            event => panic!("expected run start, got {event:?}"),
        };
        let run = RunRef { inner: &run };

        let items = run.items().collect::<Vec<_>>();
        let [RunItemRef::Drawing(drawing)] = items.as_slice() else {
            panic!("expected one drawing item");
        };
        assert_eq!(drawing.kind(), DrawingKind::Image);
        assert_eq!(drawing.relationship_id(), Some("rId7"));
        assert_eq!(
            drawing.relationship_kind(),
            Some(DrawingRelationshipKind::Linked)
        );
    }
}

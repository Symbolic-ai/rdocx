//! Drawing elements for inline and anchor images: `CT_Drawing`, `CT_Inline`, `CT_Anchor`.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::error::Result;
use crate::namespace::{
    MC_NS, R_NS, matches_local_name, matches_namespace_attribute, matches_namespace_element,
    matches_namespace_name, matches_word_name,
};
use crate::raw_xml::{NamespaceContext, capture_element};
use crate::units::Emu;

/// Namespaces used in drawing markup.
pub mod drawing_ns {
    pub const WP: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
    pub const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
    pub const PIC: &str = "http://schemas.openxmlformats.org/drawingml/2006/picture";
    pub const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
}

/// Horizontal relative-from for anchor positioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_RelativeFromH {
    Page,
    Margin,
    Column,
    Character,
    LeftMargin,
    RightMargin,
    InsideMargin,
    OutsideMargin,
}

impl ST_RelativeFromH {
    pub fn from_str(s: &str) -> Self {
        match s {
            "page" => Self::Page,
            "margin" => Self::Margin,
            "column" => Self::Column,
            "character" => Self::Character,
            "leftMargin" => Self::LeftMargin,
            "rightMargin" => Self::RightMargin,
            "insideMargin" => Self::InsideMargin,
            "outsideMargin" => Self::OutsideMargin,
            _ => Self::Page,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Margin => "margin",
            Self::Column => "column",
            Self::Character => "character",
            Self::LeftMargin => "leftMargin",
            Self::RightMargin => "rightMargin",
            Self::InsideMargin => "insideMargin",
            Self::OutsideMargin => "outsideMargin",
        }
    }
}

/// Vertical relative-from for anchor positioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ST_RelativeFromV {
    Page,
    Margin,
    Paragraph,
    Line,
    TopMargin,
    BottomMargin,
    InsideMargin,
    OutsideMargin,
}

impl ST_RelativeFromV {
    pub fn from_str(s: &str) -> Self {
        match s {
            "page" => Self::Page,
            "margin" => Self::Margin,
            "paragraph" => Self::Paragraph,
            "line" => Self::Line,
            "topMargin" => Self::TopMargin,
            "bottomMargin" => Self::BottomMargin,
            "insideMargin" => Self::InsideMargin,
            "outsideMargin" => Self::OutsideMargin,
            _ => Self::Page,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Margin => "margin",
            Self::Paragraph => "paragraph",
            Self::Line => "line",
            Self::TopMargin => "topMargin",
            Self::BottomMargin => "bottomMargin",
            Self::InsideMargin => "insideMargin",
            Self::OutsideMargin => "outsideMargin",
        }
    }
}

/// Wrapping type for anchored drawings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapType {
    None,
}

/// `CT_Anchor` — An anchored (floating) drawing element.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Anchor {
    /// Whether the drawing is behind document text.
    pub behind_doc: bool,
    /// Horizontal position offset in EMUs.
    pub pos_h_offset: Emu,
    /// Horizontal relative-from.
    pub pos_h_relative_from: ST_RelativeFromH,
    /// Vertical position offset in EMUs.
    pub pos_v_offset: Emu,
    /// Vertical relative-from.
    pub pos_v_relative_from: ST_RelativeFromV,
    /// Width in EMUs.
    pub extent_cx: Emu,
    /// Height in EMUs.
    pub extent_cy: Emu,
    /// Wrapping type.
    pub wrap: WrapType,
    /// Relationship ID referencing the image part.
    pub embed_id: String,
    /// Relationship ID referencing an externally linked image.
    pub link_id: Option<String>,
    /// Z-order relative height.
    pub relative_height: u32,
    /// Optional description/alt text.
    pub description: Option<String>,
    /// Optional name.
    pub name: Option<String>,
    /// Raw XML bytes for the entire wp:anchor element (used for round-trip preservation).
    /// When present, to_xml uses this instead of structured serialization.
    pub raw_xml: Option<Vec<u8>>,
    /// Shape content, when the anchor holds a `wps:wsp` rather than a picture.
    pub shape: Option<CT_Shape>,
}

/// A `wps:wsp` shape: preset geometry, an optional fill, and optional text.
///
/// Word writes these inside `mc:AlternateContent`, with a VML fallback beside
/// them. Only what we can actually draw is modelled here.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CT_Shape {
    /// Preset geometry name from `a:prstGeom@prst`, e.g. "rect" or "line".
    pub preset: Option<String>,
    /// Solid fill colour as RRGGBB, from the shape's own fill rather than its
    /// outline. `None` covers both `a:noFill` and a fill we cannot resolve,
    /// such as a theme colour.
    pub solid_fill: Option<String>,
    /// Paragraphs of the shape's text box, from `wps:txbx/w:txbxContent`.
    pub text: Vec<crate::text::CT_P>,
}

/// Parse the DrawingML held inside a captured `mc:AlternateContent` block.
///
/// Word writes a shape as a compatibility block: the modern DrawingML sits in
/// `mc:Choice` and a VML fallback in `mc:Fallback`. We read the Choice and
/// ignore the Fallback.
///
/// The caller keeps the raw bytes for write back, so whatever comes out of
/// here must not be serialised again or the element ends up duplicated.
pub fn parse_alternate_content(raw: &[u8]) -> Option<CT_Drawing> {
    parse_alternate_content_with_context(raw, &NamespaceContext::default())
}

pub(crate) fn parse_alternate_content_with_context(
    raw: &[u8],
    context: &NamespaceContext,
) -> Option<CT_Drawing> {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_choice = false;
    let mut contexts = vec![context.clone()];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let parent = contexts.last().expect("namespace root");
                let child = parent.with_element(e);
                if matches_namespace_element(e, parent, b"mc", MC_NS, b"Choice") {
                    in_choice = true;
                } else if in_choice && crate::namespace::matches_word_element(e, parent, b"drawing")
                {
                    return CT_Drawing::from_xml_with_context(&mut reader, &child).ok();
                }
                contexts.push(child);
            }
            Ok(Event::End(ref e)) => {
                let current = contexts.last().expect("namespace root");
                if matches_namespace_name(e.name().as_ref(), current, b"mc", MC_NS, b"Choice") {
                    in_choice = false;
                }
                if contexts.len() > 1 {
                    contexts.pop();
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// Pull the preset geometry and solid fill out of a captured `wps:spPr`.
///
/// The fill has to be told apart from the outline colour. Both are written as
/// `a:srgbClr`, and the outline sits inside `a:ln`, so anything at or below an
/// `a:ln` is skipped.
fn parse_shape_props(raw: &[u8]) -> (Option<String>, Option<String>) {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut preset = None;
    let mut fill = None;
    let mut ln_depth = 0usize;
    let mut in_solid_fill = false;

    // Only a Start can open a scope. A self-closing a:ln or a:solidFill has no
    // children, so it must not change the depth or the fill flag.
    loop {
        let event = reader.read_event_into(&mut buf);
        match event {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                let local = name.as_ref();
                if matches_local_name(local, b"ln") {
                    ln_depth += 1;
                } else if matches_local_name(local, b"solidFill") {
                    in_solid_fill = true;
                } else if matches_local_name(local, b"prstGeom") {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"prst" {
                            preset = std::str::from_utf8(&attr.value).ok().map(str::to_string);
                        }
                    }
                } else if matches_local_name(local, b"srgbClr")
                    && in_solid_fill
                    && ln_depth == 0
                    && fill.is_none()
                {
                    // srgbClr can carry children such as a:alpha, so it turns
                    // up as a Start as well as an Empty.
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"val" {
                            fill = std::str::from_utf8(&attr.value).ok().map(str::to_string);
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let local = name.as_ref();
                if matches_local_name(local, b"prstGeom") {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"prst" {
                            preset = std::str::from_utf8(&attr.value).ok().map(str::to_string);
                        }
                    }
                } else if matches_local_name(local, b"srgbClr")
                    && in_solid_fill
                    && ln_depth == 0
                    && fill.is_none()
                {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"val" {
                            fill = std::str::from_utf8(&attr.value).ok().map(str::to_string);
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let local = name.as_ref();
                if matches_local_name(local, b"ln") {
                    ln_depth = ln_depth.saturating_sub(1);
                } else if matches_local_name(local, b"solidFill") {
                    in_solid_fill = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    (preset, fill)
}

impl CT_Anchor {
    pub fn relationship_id(&self) -> Option<&str> {
        (!self.embed_id.is_empty())
            .then_some(self.embed_id.as_str())
            .or(self.link_id.as_deref())
    }

    pub fn is_linked(&self) -> bool {
        self.embed_id.is_empty() && self.link_id.is_some()
    }

    /// Create an anchor for a full-page background image.
    pub fn background(embed_id: &str, page_width_emu: i64, page_height_emu: i64) -> Self {
        CT_Anchor {
            behind_doc: true,
            pos_h_offset: Emu(0),
            pos_h_relative_from: ST_RelativeFromH::Page,
            pos_v_offset: Emu(0),
            pos_v_relative_from: ST_RelativeFromV::Page,
            extent_cx: Emu(page_width_emu),
            extent_cy: Emu(page_height_emu),
            wrap: WrapType::None,
            embed_id: embed_id.to_string(),
            link_id: None,
            relative_height: 0,
            description: Some("Background".to_string()),
            name: Some("Background".to_string()),
            raw_xml: None,
            shape: None,
        }
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<Self> {
        let context = NamespaceContext::default().with_element(start);
        Self::from_xml_with_context(reader, start, &context)
    }

    fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut behind_doc = false;
        let mut relative_height = 0u32;

        // Parse attributes from the <wp:anchor> start tag
        for attr in start.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            let val = attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)?;
            if key == b"behindDoc" {
                behind_doc = val == "1" || val == "true";
            } else if key == b"relativeHeight" {
                relative_height = val.parse().unwrap_or(0);
            }
        }

        let mut pos_h_offset = Emu(0);
        let mut pos_h_relative_from = ST_RelativeFromH::Page;
        let mut pos_v_offset = Emu(0);
        let mut pos_v_relative_from = ST_RelativeFromV::Page;
        let mut extent_cx = Emu(0);
        let mut extent_cy = Emu(0);
        let mut embed_id = String::new();
        let mut link_id = None;
        let mut shape: Option<CT_Shape> = None;
        let mut description = None;
        let mut name = None;
        let mut buf = Vec::new();
        let mut contexts = vec![context.clone()];

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    let ename = e.name();
                    let parent = contexts.last().expect("namespace root");
                    if matches_wp_element(e, parent, b"extent") {
                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = attr.key.as_ref();
                            let val = attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)?;
                            if key == b"cx" {
                                extent_cx = Emu(val.parse()?);
                            } else if key == b"cy" {
                                extent_cy = Emu(val.parse()?);
                            }
                        }
                    } else if matches_wp_element(e, parent, b"docPr") {
                        parse_drawing_description(e, &mut description, &mut name)?;
                    } else if matches_a_element(e, parent, b"blip") {
                        let element_context = parent.with_element(e);
                        parse_blip_relationships(e, &element_context, &mut embed_id, &mut link_id)?;
                    } else if matches_local_name(ename.as_ref(), b"simplePos") {
                        // Ignore simplePos
                    } else if matches_local_name(ename.as_ref(), b"wrapNone") {
                        // Already default
                    }
                }
                Ok(Event::Start(ref e)) => {
                    let ename = e.name();
                    let parent = contexts.last().expect("namespace root");
                    let child = parent.with_element(e);
                    if matches_wp_element(e, parent, b"positionH") {
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"relativeFrom" {
                                pos_h_relative_from = ST_RelativeFromH::from_str(
                                    attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)?
                                        .as_ref(),
                                );
                            }
                        }
                        // Read child <wp:posOffset>
                        let mut inner_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut inner_buf) {
                                Ok(Event::Start(ref ie))
                                    if matches_local_name(ie.name().as_ref(), b"posOffset") =>
                                {
                                    let text = crate::xml_text::decode_escaped(
                                        &reader.read_text(ie.name())?,
                                    )?;
                                    pos_h_offset = Emu(text.trim().parse().unwrap_or(0));
                                }
                                Ok(Event::End(ref ie))
                                    if matches_local_name(ie.name().as_ref(), b"positionH") =>
                                {
                                    break;
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => return Err(e.into()),
                                _ => {}
                            }
                            inner_buf.clear();
                        }
                    } else if matches_wp_element(e, parent, b"positionV") {
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"relativeFrom" {
                                pos_v_relative_from = ST_RelativeFromV::from_str(
                                    attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)?
                                        .as_ref(),
                                );
                            }
                        }
                        let mut inner_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut inner_buf) {
                                Ok(Event::Start(ref ie))
                                    if matches_local_name(ie.name().as_ref(), b"posOffset") =>
                                {
                                    let text = crate::xml_text::decode_escaped(
                                        &reader.read_text(ie.name())?,
                                    )?;
                                    pos_v_offset = Emu(text.trim().parse().unwrap_or(0));
                                }
                                Ok(Event::End(ref ie))
                                    if matches_local_name(ie.name().as_ref(), b"positionV") =>
                                {
                                    break;
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => return Err(e.into()),
                                _ => {}
                            }
                            inner_buf.clear();
                        }
                    } else if matches_local_name(ename.as_ref(), b"spPr") {
                        // Capture the shape properties and read geometry and
                        // fill out of them separately, so the fill colour is
                        // not confused with the outline colour.
                        let raw = capture_element(reader, e)?;
                        let (preset, solid_fill) = parse_shape_props(&raw);
                        let s = shape.get_or_insert_with(CT_Shape::default);
                        s.preset = preset;
                        s.solid_fill = solid_fill;
                    } else if matches_local_name(ename.as_ref(), b"txbxContent") {
                        // A shape's text box holds ordinary w:p paragraphs.
                        let mut inner_buf = Vec::new();
                        let mut paragraphs = Vec::new();
                        loop {
                            match reader.read_event_into(&mut inner_buf) {
                                Ok(Event::Start(ref ie))
                                    if matches_local_name(ie.name().as_ref(), b"p") =>
                                {
                                    paragraphs.push(crate::text::CT_P::from_xml(reader)?);
                                }
                                Ok(Event::End(ref ie))
                                    if matches_local_name(ie.name().as_ref(), b"txbxContent") =>
                                {
                                    break;
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => return Err(e.into()),
                                _ => {}
                            }
                            inner_buf.clear();
                        }
                        shape.get_or_insert_with(CT_Shape::default).text = paragraphs;
                    } else if matches_a_element(e, parent, b"blip") {
                        parse_blip_relationships(e, &child, &mut embed_id, &mut link_id)?;
                        reader.read_to_end_into(ename, &mut Vec::new())?;
                    } else if matches_wp_element(e, parent, b"docPr") {
                        parse_drawing_description(e, &mut description, &mut name)?;
                        reader.read_to_end_into(ename, &mut Vec::new())?;
                    } else {
                        // Continue into nested elements (graphic, graphicData, pic, etc.)
                        contexts.push(child);
                    }
                }
                Ok(Event::End(ref e)) => {
                    if matches_wp_name(e.name().as_ref(), context, b"anchor") {
                        break;
                    }
                    if contexts.len() > 1 {
                        contexts.pop();
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Anchor {
            behind_doc,
            pos_h_offset,
            pos_h_relative_from,
            pos_v_offset,
            pos_v_relative_from,
            extent_cx,
            extent_cy,
            wrap: WrapType::None,
            embed_id,
            link_id,
            relative_height,
            description,
            name,
            raw_xml: None, // Will be set by CT_Drawing::from_xml
            shape,
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        // If we have raw XML from parsing, use it for perfect round-trip
        if let Some(ref raw) = self.raw_xml {
            writer.get_mut().write_all(raw)?;
            return Ok(());
        }

        let mut buf = itoa::Buffer::new();
        let mut anchor = BytesStart::new("wp:anchor");
        anchor.push_attribute(("behindDoc", if self.behind_doc { "1" } else { "0" }));
        anchor.push_attribute(("simplePos", "0"));
        anchor.push_attribute(("relativeHeight", buf.format(self.relative_height)));
        anchor.push_attribute(("locked", "0"));
        anchor.push_attribute(("layoutInCell", "1"));
        anchor.push_attribute(("allowOverlap", "1"));
        writer.write_event(Event::Start(anchor))?;

        // wp:simplePos
        let mut sp = BytesStart::new("wp:simplePos");
        sp.push_attribute(("x", "0"));
        sp.push_attribute(("y", "0"));
        writer.write_event(Event::Empty(sp))?;

        // wp:positionH
        let mut pos_h = BytesStart::new("wp:positionH");
        pos_h.push_attribute(("relativeFrom", self.pos_h_relative_from.to_str()));
        writer.write_event(Event::Start(pos_h))?;
        writer.write_event(Event::Start(BytesStart::new("wp:posOffset")))?;
        writer.write_event(Event::Text(BytesText::new(
            &self.pos_h_offset.0.to_string(),
        )))?;
        writer.write_event(Event::End(BytesEnd::new("wp:posOffset")))?;
        writer.write_event(Event::End(BytesEnd::new("wp:positionH")))?;

        // wp:positionV
        let mut pos_v = BytesStart::new("wp:positionV");
        pos_v.push_attribute(("relativeFrom", self.pos_v_relative_from.to_str()));
        writer.write_event(Event::Start(pos_v))?;
        writer.write_event(Event::Start(BytesStart::new("wp:posOffset")))?;
        writer.write_event(Event::Text(BytesText::new(
            &self.pos_v_offset.0.to_string(),
        )))?;
        writer.write_event(Event::End(BytesEnd::new("wp:posOffset")))?;
        writer.write_event(Event::End(BytesEnd::new("wp:positionV")))?;

        // wp:extent
        let mut extent = BytesStart::new("wp:extent");
        extent.push_attribute(("cx", buf.format(self.extent_cx.0)));
        extent.push_attribute(("cy", buf.format(self.extent_cy.0)));
        writer.write_event(Event::Empty(extent))?;

        // wp:wrapNone
        writer.write_event(Event::Empty(BytesStart::new("wp:wrapNone")))?;

        // wp:docPr
        let mut doc_pr = BytesStart::new("wp:docPr");
        doc_pr.push_attribute(("id", "1"));
        doc_pr.push_attribute(("name", self.name.as_deref().unwrap_or("Picture")));
        if let Some(ref desc) = self.description {
            doc_pr.push_attribute(("descr", desc.as_str()));
        }
        writer.write_event(Event::Empty(doc_pr))?;

        // a:graphic (same pic:pic structure as inline)
        write_graphic_element(
            writer,
            &self.embed_id,
            self.link_id.as_deref(),
            self.extent_cx,
            self.extent_cy,
            self.name.as_deref(),
        )?;

        writer.write_event(Event::End(BytesEnd::new("wp:anchor")))?;
        Ok(())
    }
}

/// `CT_Inline` — An inline drawing (image) element.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Inline {
    /// Width in EMUs
    pub extent_cx: Emu,
    /// Height in EMUs
    pub extent_cy: Emu,
    /// Relationship ID referencing the image part
    pub embed_id: String,
    /// Relationship ID referencing an externally linked image.
    pub link_id: Option<String>,
    /// Optional description/alt text
    pub description: Option<String>,
    /// Optional name
    pub name: Option<String>,
    /// Raw XML bytes for the entire wp:inline element (used for round-trip preservation).
    /// When present, to_xml uses this instead of structured serialization.
    pub raw_xml: Option<Vec<u8>>,
}

impl CT_Inline {
    pub fn relationship_id(&self) -> Option<&str> {
        (!self.embed_id.is_empty())
            .then_some(self.embed_id.as_str())
            .or(self.link_id.as_deref())
    }

    pub fn is_linked(&self) -> bool {
        self.embed_id.is_empty() && self.link_id.is_some()
    }

    pub fn new(embed_id: &str, width_emu: i64, height_emu: i64) -> Self {
        CT_Inline {
            extent_cx: Emu(width_emu),
            extent_cy: Emu(height_emu),
            embed_id: embed_id.to_string(),
            link_id: None,
            description: None,
            name: None,
            raw_xml: None,
        }
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut cx = Emu(0);
        let mut cy = Emu(0);
        let mut embed_id = String::new();
        let mut link_id = None;
        let mut description = None;
        let mut name = None;
        let mut buf = Vec::new();
        let mut contexts = vec![context.clone()];

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    let parent = contexts.last().expect("namespace root");
                    if matches_wp_element(e, parent, b"extent") {
                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = attr.key.as_ref();
                            let val = attr.normalized_value(quick_xml::XmlVersion::Implicit1_0)?;
                            if key == b"cx" {
                                cx = Emu(val.parse()?);
                            } else if key == b"cy" {
                                cy = Emu(val.parse()?);
                            }
                        }
                    } else if matches_wp_element(e, parent, b"docPr") {
                        parse_drawing_description(e, &mut description, &mut name)?;
                    } else if matches_a_element(e, parent, b"blip") {
                        let element_context = parent.with_element(e);
                        parse_blip_relationships(e, &element_context, &mut embed_id, &mut link_id)?;
                    }
                }
                Ok(Event::Start(ref e)) => {
                    let ename = e.name();
                    let parent = contexts.last().expect("namespace root");
                    let child = parent.with_element(e);
                    if matches_a_element(e, parent, b"blip") {
                        parse_blip_relationships(e, &child, &mut embed_id, &mut link_id)?;
                        reader.read_to_end_into(ename, &mut Vec::new())?;
                    } else if matches_wp_element(e, parent, b"docPr") {
                        parse_drawing_description(e, &mut description, &mut name)?;
                        reader.read_to_end_into(ename, &mut Vec::new())?;
                    } else {
                        contexts.push(child);
                    }
                }
                Ok(Event::End(ref e)) => {
                    if matches_wp_name(e.name().as_ref(), context, b"inline") {
                        break;
                    }
                    if contexts.len() > 1 {
                        contexts.pop();
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Inline {
            extent_cx: cx,
            extent_cy: cy,
            embed_id,
            link_id,
            description,
            name,
            raw_xml: None, // Will be set by CT_Drawing::from_xml
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        // If we have raw XML from parsing, use it for perfect round-trip
        if let Some(ref raw) = self.raw_xml {
            writer.get_mut().write_all(raw)?;
            return Ok(());
        }

        // wp:inline
        let mut buf = itoa::Buffer::new();
        let mut inline = BytesStart::new("wp:inline");
        inline.push_attribute(("distT", "0"));
        inline.push_attribute(("distB", "0"));
        inline.push_attribute(("distL", "0"));
        inline.push_attribute(("distR", "0"));
        writer.write_event(Event::Start(inline))?;

        // wp:extent
        let mut extent = BytesStart::new("wp:extent");
        extent.push_attribute(("cx", buf.format(self.extent_cx.0)));
        extent.push_attribute(("cy", buf.format(self.extent_cy.0)));
        writer.write_event(Event::Empty(extent))?;

        // wp:docPr
        let mut doc_pr = BytesStart::new("wp:docPr");
        doc_pr.push_attribute(("id", "1"));
        doc_pr.push_attribute(("name", self.name.as_deref().unwrap_or("Picture")));
        if let Some(ref desc) = self.description {
            doc_pr.push_attribute(("descr", desc.as_str()));
        }
        writer.write_event(Event::Empty(doc_pr))?;

        // a:graphic
        write_graphic_element(
            writer,
            &self.embed_id,
            self.link_id.as_deref(),
            self.extent_cx,
            self.extent_cy,
            self.name.as_deref(),
        )?;

        writer.write_event(Event::End(BytesEnd::new("wp:inline")))?;

        Ok(())
    }
}

fn parse_blip_relationships(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    embed_id: &mut String,
    link_id: &mut Option<String>,
) -> Result<()> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        let value = attribute
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)?
            .into_owned();
        if matches_namespace_attribute(attribute.key.as_ref(), context, b"r", R_NS, b"embed") {
            *embed_id = value;
        } else if matches_namespace_attribute(attribute.key.as_ref(), context, b"r", R_NS, b"link")
        {
            *link_id = Some(value);
        }
    }
    Ok(())
}

fn parse_drawing_description(
    element: &BytesStart<'_>,
    description: &mut Option<String>,
    name: &mut Option<String>,
) -> Result<()> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        let value = attribute
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)?
            .into_owned();
        if attribute.key.as_ref() == b"descr" {
            *description = Some(value);
        } else if attribute.key.as_ref() == b"name" {
            *name = Some(value);
        }
    }
    Ok(())
}

fn matches_wp_element(element: &BytesStart<'_>, context: &NamespaceContext, local: &[u8]) -> bool {
    matches_namespace_element(element, context, b"wp", drawing_ns::WP, local)
}

fn matches_wp_name(name: &[u8], context: &NamespaceContext, local: &[u8]) -> bool {
    matches_namespace_name(name, context, b"wp", drawing_ns::WP, local)
}

fn matches_a_element(element: &BytesStart<'_>, context: &NamespaceContext, local: &[u8]) -> bool {
    matches_namespace_element(element, context, b"a", drawing_ns::A, local)
}

/// Write the `a:graphic` > `a:graphicData` > `pic:pic` structure (shared by inline and anchor).
fn write_graphic_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    embed_id: &str,
    link_id: Option<&str>,
    cx: Emu,
    cy: Emu,
    name: Option<&str>,
) -> Result<()> {
    let mut buf = itoa::Buffer::new();
    let mut graphic = BytesStart::new("a:graphic");
    graphic.push_attribute(("xmlns:a", drawing_ns::A));
    writer.write_event(Event::Start(graphic))?;

    let mut gd = BytesStart::new("a:graphicData");
    gd.push_attribute(("uri", drawing_ns::PIC));
    writer.write_event(Event::Start(gd))?;

    let mut pic = BytesStart::new("pic:pic");
    pic.push_attribute(("xmlns:pic", drawing_ns::PIC));
    writer.write_event(Event::Start(pic))?;

    // pic:nvPicPr
    writer.write_event(Event::Start(BytesStart::new("pic:nvPicPr")))?;
    let mut cnvpr = BytesStart::new("pic:cNvPr");
    cnvpr.push_attribute(("id", "0"));
    cnvpr.push_attribute(("name", name.unwrap_or("Picture")));
    writer.write_event(Event::Empty(cnvpr))?;
    writer.write_event(Event::Empty(BytesStart::new("pic:cNvPicPr")))?;
    writer.write_event(Event::End(BytesEnd::new("pic:nvPicPr")))?;

    // pic:blipFill
    writer.write_event(Event::Start(BytesStart::new("pic:blipFill")))?;
    let mut blip = BytesStart::new("a:blip");
    if !embed_id.is_empty() {
        blip.push_attribute(("r:embed", embed_id));
    } else if let Some(link_id) = link_id {
        blip.push_attribute(("r:link", link_id));
    }
    writer.write_event(Event::Empty(blip))?;
    writer.write_event(Event::Start(BytesStart::new("a:stretch")))?;
    writer.write_event(Event::Empty(BytesStart::new("a:fillRect")))?;
    writer.write_event(Event::End(BytesEnd::new("a:stretch")))?;
    writer.write_event(Event::End(BytesEnd::new("pic:blipFill")))?;

    // pic:spPr
    writer.write_event(Event::Start(BytesStart::new("pic:spPr")))?;
    writer.write_event(Event::Start(BytesStart::new("a:xfrm")))?;
    let mut off = BytesStart::new("a:off");
    off.push_attribute(("x", "0"));
    off.push_attribute(("y", "0"));
    writer.write_event(Event::Empty(off))?;
    let mut ext = BytesStart::new("a:ext");
    ext.push_attribute(("cx", buf.format(cx.0)));
    ext.push_attribute(("cy", buf.format(cy.0)));
    writer.write_event(Event::Empty(ext))?;
    writer.write_event(Event::End(BytesEnd::new("a:xfrm")))?;
    let mut prst = BytesStart::new("a:prstGeom");
    prst.push_attribute(("prst", "rect"));
    writer.write_event(Event::Start(prst))?;
    writer.write_event(Event::Empty(BytesStart::new("a:avLst")))?;
    writer.write_event(Event::End(BytesEnd::new("a:prstGeom")))?;
    writer.write_event(Event::End(BytesEnd::new("pic:spPr")))?;

    writer.write_event(Event::End(BytesEnd::new("pic:pic")))?;
    writer.write_event(Event::End(BytesEnd::new("a:graphicData")))?;
    writer.write_event(Event::End(BytesEnd::new("a:graphic")))?;

    Ok(())
}

/// `CT_Drawing` — A drawing element that wraps inline or anchor images.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Drawing {
    pub inline: Option<CT_Inline>,
    pub anchor: Option<CT_Anchor>,
}

impl CT_Drawing {
    pub fn inline(inline: CT_Inline) -> Self {
        CT_Drawing {
            inline: Some(inline),
            anchor: None,
        }
    }

    pub fn anchor(anchor: CT_Anchor) -> Self {
        CT_Drawing {
            inline: None,
            anchor: Some(anchor),
        }
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub(crate) fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut inline = None;
        let mut anchor = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_wp_element(e, context, b"inline") {
                        // Capture full raw XML, then re-parse for structured fields
                        let raw = capture_element(reader, e)?;
                        let mut re_reader = Reader::from_reader(raw.as_slice());
                        re_reader.config_mut().trim_text(true);
                        // Skip to the <wp:inline> start
                        let mut rbuf = Vec::new();
                        loop {
                            match re_reader.read_event_into(&mut rbuf) {
                                Ok(Event::Start(ref ie))
                                    if matches_wp_element(ie, context, b"inline") =>
                                {
                                    let inline_context = context.with_element(ie);
                                    let mut inl = CT_Inline::from_xml_with_context(
                                        &mut re_reader,
                                        &inline_context,
                                    )?;
                                    inl.raw_xml = Some(raw);
                                    inline = Some(inl);
                                    break;
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => return Err(e.into()),
                                _ => {}
                            }
                            rbuf.clear();
                        }
                    } else if matches_wp_element(e, context, b"anchor") {
                        // Capture full raw XML, then re-parse for structured fields
                        let raw = capture_element(reader, e)?;
                        let mut re_reader = Reader::from_reader(raw.as_slice());
                        re_reader.config_mut().trim_text(true);
                        let mut rbuf = Vec::new();
                        loop {
                            match re_reader.read_event_into(&mut rbuf) {
                                Ok(Event::Start(ref ie))
                                    if matches_wp_element(ie, context, b"anchor") =>
                                {
                                    let anchor_context = context.with_element(ie);
                                    let mut anc = CT_Anchor::from_xml_with_context(
                                        &mut re_reader,
                                        ie,
                                        &anchor_context,
                                    )?;
                                    anc.raw_xml = Some(raw);
                                    anchor = Some(anc);
                                    break;
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => return Err(e.into()),
                                _ => {}
                            }
                            rbuf.clear();
                        }
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e))
                    if matches_word_name(e.name().as_ref(), context, b"drawing") =>
                {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Drawing { inline, anchor })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let drawing = BytesStart::new("w:drawing");
        writer.write_event(Event::Start(drawing))?;

        if let Some(ref inl) = self.inline {
            inl.to_xml(writer)?;
        }
        if let Some(ref anc) = self.anchor {
            anc.to_xml(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:drawing")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_drawing(xml: &str) -> CT_Drawing {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let context = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"drawing") => {
                    break NamespaceContext::default().with_element(e);
                }
                _ => {}
            }
            buf.clear();
        };
        CT_Drawing::from_xml_with_context(&mut reader, &context).unwrap()
    }

    #[test]
    fn round_trip_inline_drawing() {
        let inline = CT_Inline {
            extent_cx: Emu(914400), // 1 inch
            extent_cy: Emu(457200), // 0.5 inch
            embed_id: "rId5".to_string(),
            link_id: None,
            description: Some("A test image".to_string()),
            name: Some("TestPic".to_string()),
            raw_xml: None,
        };

        let drawing = CT_Drawing::inline(inline);

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        drawing.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let parsed = parse_drawing(&xml);
        let inl = parsed.inline.unwrap();
        assert_eq!(inl.extent_cx, Emu(914400));
        assert_eq!(inl.extent_cy, Emu(457200));
        assert_eq!(inl.embed_id, "rId5");
    }

    #[test]
    fn parses_linked_inline_image_relationship() {
        let drawing = parse_drawing(concat!(
            r#"<w:drawing xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" "#,
            r#"xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" "#,
            r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
            r#"<wp:inline><wp:extent cx="10" cy="20"/><a:graphic><a:blip r:link="rId7"/>"#,
            r#"</a:graphic></wp:inline></w:drawing>"#,
        ));
        let inline = drawing.inline.unwrap();
        assert!(inline.embed_id.is_empty());
        assert_eq!(inline.link_id.as_deref(), Some("rId7"));
        assert_eq!(inline.relationship_id(), Some("rId7"));
        assert!(inline.is_linked());
    }

    #[test]
    fn drawing_facts_use_resolved_namespaces_and_decoded_attributes() {
        let drawing = parse_drawing(concat!(
            r#"<w:drawing xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" "#,
            r#"xmlns:x="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" "#,
            r#"xmlns:y="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
            r#"xmlns:q="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
            r#"xmlns:foo="urn:foreign">"#,
            r#"<foo:inline><foo:docPr descr="wrong"/><foo:blip q:embed="wrong"/></foo:inline>"#,
            r#"<x:inline><x:extent cx="10" cy="20"/><foo:docPr descr="wrong"/>"#,
            r#"<x:docPr descr="A &amp; B" name="Picture &quot;One&quot;"/>"#,
            r#"<y:graphic><foo:blip q:embed="wrong"/><y:blip q:embed="rId5"/></y:graphic>"#,
            r#"</x:inline></w:drawing>"#,
        ));

        let inline = drawing.inline.unwrap();
        assert_eq!(inline.embed_id, "rId5");
        assert_eq!(inline.description.as_deref(), Some("A & B"));
        assert_eq!(inline.name.as_deref(), Some("Picture \"One\""));
    }

    #[test]
    fn ct_anchor_background_constructor() {
        let anchor = CT_Anchor::background("rId1", 7772400, 10058400);
        assert!(anchor.behind_doc);
        assert_eq!(anchor.pos_h_offset, Emu(0));
        assert_eq!(anchor.pos_v_offset, Emu(0));
        assert_eq!(anchor.pos_h_relative_from, ST_RelativeFromH::Page);
        assert_eq!(anchor.pos_v_relative_from, ST_RelativeFromV::Page);
        assert_eq!(anchor.extent_cx, Emu(7772400));
        assert_eq!(anchor.extent_cy, Emu(10058400));
        assert_eq!(anchor.embed_id, "rId1");
        assert_eq!(anchor.relative_height, 0);
    }

    #[test]
    fn ct_anchor_round_trip_xml() {
        let anchor = CT_Anchor::background("rId3", 7772400, 10058400);

        let drawing = CT_Drawing::anchor(anchor);
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        drawing.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let parsed = parse_drawing(&xml);
        assert!(parsed.anchor.is_some());
        let anc = parsed.anchor.unwrap();
        assert!(anc.behind_doc);
        assert_eq!(anc.pos_h_offset, Emu(0));
        assert_eq!(anc.pos_v_offset, Emu(0));
        assert_eq!(anc.pos_h_relative_from, ST_RelativeFromH::Page);
        assert_eq!(anc.pos_v_relative_from, ST_RelativeFromV::Page);
        assert_eq!(anc.extent_cx, Emu(7772400));
        assert_eq!(anc.extent_cy, Emu(10058400));
        assert_eq!(anc.embed_id, "rId3");
    }

    #[test]
    fn ct_drawing_with_anchor_and_inline() {
        // A drawing can have either inline or anchor (not both in practice, but test both paths)
        let inline = CT_Inline::new("rId1", 914400, 457200);
        let d1 = CT_Drawing::inline(inline);
        assert!(d1.inline.is_some());
        assert!(d1.anchor.is_none());

        let anchor = CT_Anchor::background("rId2", 7772400, 10058400);
        let d2 = CT_Drawing::anchor(anchor);
        assert!(d2.inline.is_none());
        assert!(d2.anchor.is_some());
    }
}

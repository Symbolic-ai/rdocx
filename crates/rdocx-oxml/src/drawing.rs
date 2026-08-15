//! Drawing elements for inline and anchor images: `CT_Drawing`, `CT_Inline`, `CT_Anchor`.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use oxml_core::xml::{
    StrictXmlCursor, StrictXmlDocument, StrictXmlElement, StrictXmlNode, StrictXmlParsed,
    parse_reader_element, parse_reader_started_element,
};

use crate::error::{OxmlError, Result};
#[cfg(test)]
use crate::namespace::matches_local_name;
use crate::namespace::{MC_NS, R_NS, W_NS};
use crate::raw_xml::NamespaceContext;
use crate::units::Emu;

/// Namespaces used in drawing markup.
pub mod drawing_ns {
    pub const WP: &str = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
    pub const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
    pub const PIC: &str = "http://schemas.openxmlformats.org/drawingml/2006/picture";
    pub const R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    pub const WPS: &str = "http://schemas.microsoft.com/office/word/2010/wordprocessingShape";
}

fn child_elements(element: &StrictXmlElement) -> impl Iterator<Item = &StrictXmlElement> {
    element
        .children()
        .iter()
        .filter_map(StrictXmlNode::as_element)
}

fn find_descendant<'a>(
    element: &'a StrictXmlElement,
    namespace: Option<&str>,
    local: &str,
) -> Option<&'a StrictXmlElement> {
    child_elements(element).find_map(|child| {
        child
            .is_named(namespace, local)
            .then_some(child)
            .or_else(|| find_descendant(child, namespace, local))
    })
}

fn direct_child<'a>(
    element: &'a StrictXmlElement,
    namespace: Option<&str>,
    local: &str,
) -> Option<&'a StrictXmlElement> {
    child_elements(element).find(|child| child.is_named(namespace, local))
}

fn take_element(
    cursor: &mut StrictXmlCursor,
    index: usize,
    description: &str,
) -> Result<StrictXmlElement> {
    cursor
        .take_child(index)
        .and_then(StrictXmlNode::into_element)
        .ok_or_else(|| OxmlError::MissingElement(description.to_string()))
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
    let root = StrictXmlDocument::parse(raw).ok()?.into_root();
    parse_alternate_content_element(&root).ok().flatten()
}

pub(crate) fn parse_alternate_content_element(
    element: &StrictXmlElement,
) -> Result<Option<CT_Drawing>> {
    if !element.is_named(Some(MC_NS), "AlternateContent") {
        return Ok(None);
    }
    let Some(choice) = direct_child(element, Some(MC_NS), "Choice") else {
        return Ok(None);
    };
    let Some(drawing) = find_descendant(choice, Some(W_NS), "drawing") else {
        return Ok(None);
    };
    Ok(Some(CT_Drawing::from_strict_xml(drawing.clone())?.value))
}

/// Pull the preset geometry and solid fill out of a captured `wps:spPr`.
///
/// The fill has to be told apart from the outline colour. Both are written as
/// `a:srgbClr`, and the outline sits inside `a:ln`, so anything at or below an
/// `a:ln` is skipped.

fn strict_extent(element: &StrictXmlElement) -> Result<(Emu, Emu)> {
    let Some(extent) = direct_child(element, Some(drawing_ns::WP), "extent") else {
        return Ok((Emu(0), Emu(0)));
    };
    Ok((
        Emu(extent.attribute(None, "cx").unwrap_or("0").parse()?),
        Emu(extent.attribute(None, "cy").unwrap_or("0").parse()?),
    ))
}

fn strict_drawing_identity(
    element: &StrictXmlElement,
) -> (String, Option<String>, Option<String>, Option<String>) {
    let blip = find_descendant(element, Some(drawing_ns::A), "blip");
    let embed_id = blip
        .and_then(|element| element.attribute(Some(R_NS), "embed"))
        .unwrap_or_default()
        .to_string();
    let link_id = blip
        .and_then(|element| element.attribute(Some(R_NS), "link"))
        .map(str::to_string);
    let doc_properties = direct_child(element, Some(drawing_ns::WP), "docPr");
    let description = doc_properties
        .and_then(|element| element.attribute(None, "descr"))
        .map(str::to_string);
    let name = doc_properties
        .and_then(|element| element.attribute(None, "name"))
        .map(str::to_string);
    (embed_id, link_id, description, name)
}

fn strict_position(element: &StrictXmlElement, local: &str) -> Result<Option<(String, Emu)>> {
    let Some(position) = direct_child(element, Some(drawing_ns::WP), local) else {
        return Ok(None);
    };
    let relative_from = position
        .attribute(None, "relativeFrom")
        .unwrap_or("page")
        .to_string();
    let offset = direct_child(position, Some(drawing_ns::WP), "posOffset")
        .map(StrictXmlElement::text_content)
        .unwrap_or_default();
    Ok(Some((
        relative_from,
        Emu(offset.trim().parse().unwrap_or(0)),
    )))
}

fn strict_shape(element: &StrictXmlElement) -> Result<Option<CT_Shape>> {
    let Some(shape_root) = find_descendant(element, Some(drawing_ns::WPS), "wsp") else {
        return Ok(None);
    };
    let shape_properties = find_descendant(shape_root, Some(drawing_ns::WPS), "spPr");
    let preset = shape_properties
        .and_then(|properties| find_descendant(properties, Some(drawing_ns::A), "prstGeom"))
        .and_then(|geometry| geometry.attribute(None, "prst"))
        .map(str::to_string);
    let solid_fill = shape_properties.and_then(find_shape_fill);
    let mut text = Vec::new();
    if let Some(text_box) = find_descendant(shape_root, Some(W_NS), "txbxContent") {
        for paragraph in child_elements(text_box).filter(|child| child.is_named(Some(W_NS), "p")) {
            text.push(crate::text::CT_P::from_strict_xml(paragraph.clone())?);
        }
    }
    Ok(Some(CT_Shape {
        preset,
        solid_fill,
        text,
    }))
}

fn find_shape_fill(element: &StrictXmlElement) -> Option<String> {
    for child in child_elements(element) {
        if child.is_named(Some(drawing_ns::A), "ln") {
            continue;
        }
        if child.is_named(Some(drawing_ns::A), "solidFill")
            && let Some(color) = find_descendant(child, Some(drawing_ns::A), "srgbClr")
                .and_then(|color| color.attribute(None, "val"))
        {
            return Some(color.to_string());
        }
        if let Some(color) = find_shape_fill(child) {
            return Some(color);
        }
    }
    None
}

impl CT_Anchor {
    fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let raw_xml = Some(element.raw_xml().bytes().to_vec());
        let behind_doc = matches!(element.attribute(None, "behindDoc"), Some("1" | "true"));
        let relative_height = element
            .attribute(None, "relativeHeight")
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let (extent_cx, extent_cy) = strict_extent(&element)?;
        let (embed_id, link_id, description, name) = strict_drawing_identity(&element);
        let (pos_h_relative_from, pos_h_offset) = strict_position(&element, "positionH")?
            .map(|(relative_from, offset)| (ST_RelativeFromH::from_str(&relative_from), offset))
            .unwrap_or((ST_RelativeFromH::Page, Emu(0)));
        let (pos_v_relative_from, pos_v_offset) = strict_position(&element, "positionV")?
            .map(|(relative_from, offset)| (ST_RelativeFromV::from_str(&relative_from), offset))
            .unwrap_or((ST_RelativeFromV::Page, Emu(0)));
        let shape = strict_shape(&element)?;
        Ok(Self {
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
            raw_xml,
            shape,
        })
    }

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
        let element =
            parse_reader_started_element(reader, context, Some(drawing_ns::WP), "anchor", start)?;
        Self::from_strict_xml(element)
    }
    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        // If we have raw XML from parsing, use it for perfect round-trip
        if let Some(ref raw) = self.raw_xml {
            writer.get_mut().write_all(raw)?;
            return Ok(());
        }

        let mut buf = itoa::Buffer::new();
        let mut anchor = BytesStart::new("wp:anchor");
        anchor.push_attribute(("xmlns:wp", drawing_ns::WP));
        anchor.push_attribute(("xmlns:r", drawing_ns::R));
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
    fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let raw_xml = Some(element.raw_xml().bytes().to_vec());
        let (extent_cx, extent_cy) = strict_extent(&element)?;
        let (embed_id, link_id, description, name) = strict_drawing_identity(&element);
        Ok(Self {
            extent_cx,
            extent_cy,
            embed_id,
            link_id,
            description,
            name,
            raw_xml,
        })
    }

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
        let element = parse_reader_element(reader, context, Some(drawing_ns::WP), "inline", [])?;
        Self::from_strict_xml(element)
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
        inline.push_attribute(("xmlns:wp", drawing_ns::WP));
        inline.push_attribute(("xmlns:r", drawing_ns::R));
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

    pub(crate) fn from_strict_xml(element: StrictXmlElement) -> Result<StrictXmlParsed<Self>> {
        element.parse(|cursor| {
            let mut drawing = Self {
                inline: None,
                anchor: None,
            };
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                if child.is_named(Some(drawing_ns::WP), "inline") && drawing.inline.is_none() {
                    let child = take_element(cursor, index, "wp:inline")?;
                    drawing.inline = Some(CT_Inline::from_strict_xml(child)?);
                } else if child.is_named(Some(drawing_ns::WP), "anchor") && drawing.anchor.is_none()
                {
                    let child = take_element(cursor, index, "wp:anchor")?;
                    drawing.anchor = Some(CT_Anchor::from_strict_xml(child)?);
                }
            }
            Ok(drawing)
        })
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub(crate) fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let element = parse_reader_element(reader, context, Some(W_NS), "drawing", [])?;
        Ok(Self::from_strict_xml(element)?.value)
    }
    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut drawing = BytesStart::new("w:drawing");
        drawing.push_attribute(("xmlns:w", W_NS));
        drawing.push_attribute(("xmlns:wp", drawing_ns::WP));
        drawing.push_attribute(("xmlns:r", drawing_ns::R));
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

//! Header and footer elements: `CT_HdrFtr`.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::name::{Namespace, ResolveResult};
use quick_xml::reader::NsReader;
use quick_xml::{Reader, Writer, XmlVersion};

use crate::error::{OxmlError, Result};
use crate::namespace::{W_NS, matches_local_name};
use crate::numbering::word_prefixes_at;
use crate::properties::is_word_element;
use crate::raw_xml::{capture_element, capture_empty_element};
use crate::text::CT_P;

const VML_NS: &str = "urn:schemas-microsoft-com:vml";
const OFFICE_NS: &str = "urn:schemas-microsoft-com:office:office";
const RELATIONSHIPS_NS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// A conservative layout projection of one VML watermark shape.
#[derive(Debug, Clone, PartialEq)]
pub enum VmlWatermark {
    Text {
        text: String,
        width_pt: f64,
        height_pt: f64,
        rotation_degrees: f64,
        color: String,
        font_family: Option<String>,
        opacity: f64,
    },
    Image {
        relationship_id: String,
        width_pt: f64,
        height_pt: f64,
        rotation_degrees: f64,
        opacity: f64,
    },
}

impl VmlWatermark {
    /// Project a supported `w:pict` subtree without taking ownership of its XML.
    #[doc(hidden)]
    pub fn from_pict_xml(xml: &[u8]) -> Option<Self> {
        parse_vml_watermark(xml)
    }

    /// Write the canonical VML subtree used by the native authoring facade.
    #[doc(hidden)]
    pub fn to_pict_xml(&self) -> Vec<u8> {
        let mut writer = Writer::new(Vec::new());
        let mut pict = BytesStart::new("w:pict");
        pict.push_attribute(("xmlns:w", W_NS));
        pict.push_attribute(("xmlns:v", VML_NS));
        pict.push_attribute(("xmlns:o", OFFICE_NS));
        pict.push_attribute(("xmlns:r", RELATIONSHIPS_NS));
        writer
            .write_event(Event::Start(pict))
            .expect("writing VML to memory cannot fail");

        let (width, height, rotation, opacity) = match self {
            Self::Text {
                width_pt,
                height_pt,
                rotation_degrees,
                opacity,
                ..
            }
            | Self::Image {
                width_pt,
                height_pt,
                rotation_degrees,
                opacity,
                ..
            } => (*width_pt, *height_pt, *rotation_degrees, *opacity),
        };
        let style = format!(
            "position:absolute;width:{}pt;height:{}pt;rotation:{};z-index:-251654144;mso-position-horizontal:center;mso-position-horizontal-relative:margin;mso-position-vertical:center;mso-position-vertical-relative:margin",
            compact_number(width),
            compact_number(height),
            compact_number(rotation)
        );
        let (shape_type_id, shape_type_number, text_path) = match self {
            Self::Text { .. } => ("_x0000_t136", "136", true),
            Self::Image { .. } => ("_x0000_t75", "75", false),
        };
        let mut shape_type = BytesStart::new("v:shapetype");
        shape_type.push_attribute(("id", shape_type_id));
        shape_type.push_attribute(("coordsize", "21600,21600"));
        shape_type.push_attribute(("o:spt", shape_type_number));
        shape_type.push_attribute(("path", "m0,0l21600,0,21600,21600,0,21600xe"));
        if !text_path {
            shape_type.push_attribute(("filled", "f"));
            shape_type.push_attribute(("stroked", "f"));
        }
        writer
            .write_event(Event::Start(shape_type))
            .expect("writing VML to memory cannot fail");
        let mut shape_path = BytesStart::new("v:path");
        if text_path {
            shape_path.push_attribute(("textpathok", "t"));
        } else {
            shape_path.push_attribute(("o:extrusionok", "f"));
            shape_path.push_attribute(("o:connecttype", "rect"));
        }
        writer
            .write_event(Event::Empty(shape_path))
            .expect("writing VML to memory cannot fail");
        if text_path {
            let mut template_text_path = BytesStart::new("v:textpath");
            template_text_path.push_attribute(("on", "t"));
            template_text_path.push_attribute(("fitshape", "t"));
            writer
                .write_event(Event::Empty(template_text_path))
                .expect("writing VML to memory cannot fail");
        }
        writer
            .write_event(Event::End(BytesEnd::new("v:shapetype")))
            .expect("writing VML to memory cannot fail");

        let mut shape = BytesStart::new("v:shape");
        shape.push_attribute(("id", "rdocx-watermark"));
        shape.push_attribute(("o:spid", "_x0000_s1025"));
        shape.push_attribute((
            "type",
            match self {
                Self::Text { .. } => "#_x0000_t136",
                Self::Image { .. } => "#_x0000_t75",
            },
        ));
        shape.push_attribute(("style", style.as_str()));
        shape.push_attribute(("stroked", "f"));
        if let Self::Text { color, .. } = self {
            shape.push_attribute(("fillcolor", color.as_str()));
        }
        writer
            .write_event(Event::Start(shape))
            .expect("writing VML to memory cannot fail");

        let mut fill = BytesStart::new("v:fill");
        let opacity = compact_number(opacity);
        fill.push_attribute(("opacity", opacity.as_str()));
        writer
            .write_event(Event::Empty(fill))
            .expect("writing VML to memory cannot fail");

        match self {
            Self::Text {
                text, font_family, ..
            } => {
                let family = font_family.as_deref().unwrap_or("Calibri");
                let text_style = format!("font-family:\"{family}\";font-size:1pt");
                let mut textpath = BytesStart::new("v:textpath");
                textpath.push_attribute(("on", "t"));
                textpath.push_attribute(("fitshape", "t"));
                textpath.push_attribute(("style", text_style.as_str()));
                textpath.push_attribute(("string", text.as_str()));
                writer
                    .write_event(Event::Empty(textpath))
                    .expect("writing VML to memory cannot fail");
            }
            Self::Image {
                relationship_id, ..
            } => {
                let mut image = BytesStart::new("v:imagedata");
                image.push_attribute(("r:id", relationship_id.as_str()));
                image.push_attribute(("o:title", ""));
                writer
                    .write_event(Event::Empty(image))
                    .expect("writing VML to memory cannot fail");
            }
        }

        writer
            .write_event(Event::End(BytesEnd::new("v:shape")))
            .expect("writing VML to memory cannot fail");
        writer
            .write_event(Event::End(BytesEnd::new("w:pict")))
            .expect("writing VML to memory cannot fail");
        writer.into_inner()
    }
}

/// `CT_HdrFtr` — Content of a header or footer part.
///
/// Contains paragraphs (and potentially tables, same as a document body).
#[derive(Debug, Clone, PartialEq)]
pub struct CT_HdrFtr {
    pub paragraphs: Vec<CT_P>,
    /// Supported VML watermark shapes projected from the original part bytes.
    watermarks: Vec<VmlWatermark>,
    /// Extra namespace declarations captured from the root element.
    pub extra_namespaces: Vec<(String, String)>,
    /// Unknown child elements captured as raw XML.
    pub extra_xml: Vec<Vec<u8>>,
}

#[allow(non_snake_case)]
impl CT_HdrFtr {
    pub fn new() -> Self {
        CT_HdrFtr {
            paragraphs: Vec::new(),
            watermarks: Vec::new(),
            extra_namespaces: Vec::new(),
            extra_xml: Vec::new(),
        }
    }

    /// Get the combined text of all paragraphs.
    pub fn text(&self) -> String {
        self.paragraphs
            .iter()
            .map(|p| p.text())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Return supported watermark projections in document order.
    pub fn watermarks(&self) -> &[VmlWatermark] {
        &self.watermarks
    }

    /// Parse from XML bytes (the content of header*.xml or footer*.xml).
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let watermarks = parse_vml_watermarks(xml);
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);

        let mut paragraphs = Vec::new();
        let mut extra_namespaces = Vec::new();
        let mut extra_xml = Vec::new();
        let mut buf = Vec::new();
        let mut word_prefixes = Vec::new();

        let known_ns: &[&[u8]] = &[b"xmlns:w", b"xmlns:r", b"xmlns"];

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, &word_prefixes)?;
                    if is_word_element(name.as_ref(), b"p", &prefixes) {
                        paragraphs.push(CT_P::from_xml_with_prefixes(&mut reader, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"hdr", &prefixes)
                        || is_word_element(name.as_ref(), b"ftr", &prefixes)
                    {
                        // Capture extra namespace declarations from root element
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if (key.starts_with(b"xmlns:") || key == b"xmlns")
                                && !known_ns.contains(&key)
                            {
                                let key_str = std::str::from_utf8(key).unwrap_or("").to_string();
                                let val_str =
                                    std::str::from_utf8(&attr.value).unwrap_or("").to_string();
                                extra_namespaces.push((key_str, val_str));
                            }
                        }
                        word_prefixes = prefixes;
                    } else {
                        // Capture unknown elements as raw XML
                        extra_xml.push(capture_element(&mut reader, e)?);
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    if !matches_local_name(name.as_ref(), b"hdr")
                        && !matches_local_name(name.as_ref(), b"ftr")
                    {
                        extra_xml.push(capture_empty_element(e)?);
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_HdrFtr {
            paragraphs,
            watermarks,
            extra_namespaces,
            extra_xml,
        })
    }

    /// Serialize to XML bytes as a header.
    pub fn to_xml_header(&self) -> Result<Vec<u8>> {
        self.to_xml_root("w:hdr")
    }

    /// Serialize to XML bytes as a footer.
    pub fn to_xml_footer(&self) -> Result<Vec<u8>> {
        self.to_xml_root("w:ftr")
    }

    fn to_xml_root(&self, root_tag: &str) -> Result<Vec<u8>> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut start = BytesStart::new(root_tag);
        start.push_attribute(("xmlns:w", W_NS));
        start.push_attribute((
            "xmlns:r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ));

        // Always emit xmlns:wp for drawing elements
        let wp_ns = "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing";
        let mut has_wp = false;
        for (key, _) in &self.extra_namespaces {
            if key == "xmlns:wp" {
                has_wp = true;
                break;
            }
        }
        if !has_wp {
            start.push_attribute(("xmlns:wp", wp_ns));
        }

        // Replay captured extra namespaces
        for (key, val) in &self.extra_namespaces {
            start.push_attribute((key.as_str(), val.as_str()));
        }

        writer.write_event(Event::Start(start))?;

        for p in &self.paragraphs {
            p.to_xml(&mut writer)?;
        }

        // Write captured unknown elements
        for raw in &self.extra_xml {
            writer.get_mut().extend_from_slice(raw);
        }

        writer.write_event(Event::End(BytesEnd::new(root_tag)))?;

        Ok(writer.into_inner())
    }
}

/// Replace the exact API-owned VML shape without reconstructing its header.
#[doc(hidden)]
pub fn replace_authored_watermark(xml: &[u8], watermark: &VmlWatermark) -> Result<Vec<u8>> {
    let replacement_pict = watermark.to_pict_xml();
    let replacement_range = api_owned_shape_ranges(&replacement_pict)?
        .into_iter()
        .next()
        .ok_or_else(|| OxmlError::MissingElement("generated watermark shape".to_owned()))?;
    let mut replacement_shape = replacement_pict[replacement_range].to_vec();
    let shape_start_end = replacement_shape
        .iter()
        .position(|byte| *byte == b'>')
        .ok_or_else(|| OxmlError::MissingElement("generated watermark shape start".to_owned()))?;
    replacement_shape.splice(
        shape_start_end..shape_start_end,
        format!(" xmlns:v=\"{VML_NS}\" xmlns:o=\"{OFFICE_NS}\" xmlns:r=\"{RELATIONSHIPS_NS}\"")
            .into_bytes(),
    );

    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut depth = 0usize;
    let mut pict_depth = None;
    let mut owned_start = None;
    let mut owned_ranges = Vec::new();
    let mut header_end = None;
    let mut empty_header = None;

    loop {
        let event_start = reader.buffer_position() as usize;
        let (namespace, event) = reader.read_resolved_event_into(&mut buffer)?;
        let is_word = namespace_is(&namespace, W_NS);
        let is_vml = namespace_is(&namespace, VML_NS);
        let mut completed_owner = None;
        let mut empty_owner = false;
        let mut completed_header = false;
        let mut empty_header_name = None;
        match event {
            Event::Start(ref element) => {
                depth += 1;
                let name = element.name();
                let local = local_name(name.as_ref());
                if is_word && local == b"pict" {
                    pict_depth = Some(depth);
                } else if pict_depth.is_some()
                    && is_vml
                    && local == b"shape"
                    && raw_unqualified_attribute(element, b"id").as_deref()
                        == Some("rdocx-watermark")
                {
                    owned_start = Some((depth, event_start));
                }
            }
            Event::Empty(ref element) => {
                let name = element.name();
                let local = local_name(name.as_ref());
                if depth == 0 && is_word && local == b"hdr" {
                    empty_header_name = Some(name.as_ref().to_vec());
                } else if pict_depth.is_some()
                    && is_vml
                    && local == b"shape"
                    && raw_unqualified_attribute(element, b"id").as_deref()
                        == Some("rdocx-watermark")
                {
                    empty_owner = true;
                }
            }
            Event::End(ref element) => {
                let name = element.name();
                let local = local_name(name.as_ref());
                if owned_start.is_some_and(|(owner_depth, _)| owner_depth == depth)
                    && is_vml
                    && local == b"shape"
                    && let Some((_, start)) = owned_start.take()
                {
                    completed_owner = Some(start);
                }
                if pict_depth == Some(depth) && is_word && local == b"pict" {
                    pict_depth = None;
                }
                if depth == 1 && is_word && local == b"hdr" {
                    completed_header = true;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
        drop(event);
        drop(namespace);
        let event_end = reader.buffer_position() as usize;
        if let Some(start) = completed_owner {
            owned_ranges.push(start..event_end);
        }
        if empty_owner {
            owned_ranges.push(event_start..event_end);
        }
        if completed_header {
            header_end = Some(event_start);
        }
        if let Some(root_name) = empty_header_name {
            empty_header = Some((event_start..event_end, root_name));
        }
        buffer.clear();
    }

    if !owned_ranges.is_empty() {
        let shape_type_id = match watermark {
            VmlWatermark::Text { .. } => "_x0000_t136",
            VmlWatermark::Image { .. } => "_x0000_t75",
        };
        if !contains_vml_shapetype(xml, shape_type_id)? {
            let attribute = format!(" type=\"#{shape_type_id}\"");
            if let Some(position) = replacement_shape
                .windows(attribute.len())
                .position(|window| window == attribute.as_bytes())
            {
                replacement_shape.drain(position..position + attribute.len());
            }
        }
        let mut output = Vec::with_capacity(xml.len() + replacement_shape.len());
        let mut copied = 0usize;
        for (index, range) in owned_ranges.into_iter().enumerate() {
            output.extend_from_slice(&xml[copied..range.start]);
            if index == 0 {
                output.extend_from_slice(&replacement_shape);
            }
            copied = range.end;
        }
        output.extend_from_slice(&xml[copied..]);
        return Ok(output);
    }

    let mut paragraph = format!("<w:p xmlns:w=\"{W_NS}\"><w:r>").into_bytes();
    paragraph.extend_from_slice(&replacement_pict);
    paragraph.extend_from_slice(b"</w:r></w:p>");
    if let Some(position) = header_end {
        let mut output = Vec::with_capacity(xml.len() + paragraph.len());
        output.extend_from_slice(&xml[..position]);
        output.extend_from_slice(&paragraph);
        output.extend_from_slice(&xml[position..]);
        return Ok(output);
    }
    if let Some((range, root_name)) = empty_header {
        let empty = &xml[range.clone()];
        let close = empty
            .windows(2)
            .rposition(|window| window == b"/>")
            .ok_or_else(|| OxmlError::InvalidValue("empty header root".to_owned()))?;
        let mut output = Vec::with_capacity(xml.len() + paragraph.len() + root_name.len() + 2);
        output.extend_from_slice(&xml[..range.start]);
        output.extend_from_slice(&empty[..close]);
        output.push(b'>');
        output.extend_from_slice(&paragraph);
        output.extend_from_slice(b"</");
        output.extend_from_slice(&root_name);
        output.push(b'>');
        output.extend_from_slice(&xml[range.end..]);
        return Ok(output);
    }
    Err(OxmlError::MissingElement("header root".to_owned()))
}

fn api_owned_shape_ranges(xml: &[u8]) -> Result<Vec<std::ops::Range<usize>>> {
    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut depth = 0usize;
    let mut pict_depth = None;
    let mut owned_start = None;
    let mut ranges = Vec::new();
    loop {
        let event_start = reader.buffer_position() as usize;
        let (namespace, event) = reader.read_resolved_event_into(&mut buffer)?;
        let is_word = namespace_is(&namespace, W_NS);
        let is_vml = namespace_is(&namespace, VML_NS);
        let mut completed_owner = None;
        let mut empty_owner = false;
        match event {
            Event::Start(ref element) => {
                depth += 1;
                let name = element.name();
                let local = local_name(name.as_ref());
                if is_word && local == b"pict" {
                    pict_depth = Some(depth);
                } else if pict_depth.is_some()
                    && is_vml
                    && local == b"shape"
                    && raw_unqualified_attribute(element, b"id").as_deref()
                        == Some("rdocx-watermark")
                {
                    owned_start = Some((depth, event_start));
                }
            }
            Event::Empty(ref element)
                if pict_depth.is_some()
                    && is_vml
                    && local_name(element.name().as_ref()) == b"shape"
                    && raw_unqualified_attribute(element, b"id").as_deref()
                        == Some("rdocx-watermark") =>
            {
                empty_owner = true;
            }
            Event::End(ref element) => {
                let name = element.name();
                let local = local_name(name.as_ref());
                if owned_start.is_some_and(|(owner_depth, _)| owner_depth == depth)
                    && is_vml
                    && local == b"shape"
                    && let Some((_, start)) = owned_start.take()
                {
                    completed_owner = Some(start);
                }
                if pict_depth == Some(depth) && is_word && local == b"pict" {
                    pict_depth = None;
                }
                depth = depth.saturating_sub(1);
            }
            Event::Eof => break,
            _ => {}
        }
        drop(event);
        drop(namespace);
        let event_end = reader.buffer_position() as usize;
        if let Some(start) = completed_owner {
            ranges.push(start..event_end);
        }
        if empty_owner {
            ranges.push(event_start..event_end);
        }
        buffer.clear();
    }
    Ok(ranges)
}

fn contains_vml_shapetype(xml: &[u8], expected_id: &str) -> Result<bool> {
    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    loop {
        let (namespace, event) = reader.read_resolved_event_into(&mut buffer)?;
        match event {
            Event::Start(ref element) | Event::Empty(ref element)
                if namespace_is(&namespace, VML_NS)
                    && local_name(element.name().as_ref()) == b"shapetype"
                    && raw_unqualified_attribute(element, b"id").as_deref()
                        == Some(expected_id) =>
            {
                return Ok(true);
            }
            Event::Eof => return Ok(false),
            _ => {}
        }
        buffer.clear();
    }
}

impl Default for CT_HdrFtr {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_vml_watermark(xml: &[u8]) -> Option<VmlWatermark> {
    parse_vml_watermarks(xml).into_iter().next()
}

#[derive(Default)]
struct PendingWatermark {
    width: Option<f64>,
    height: Option<f64>,
    rotation: f64,
    color: Option<String>,
    opacity: f64,
    text: Option<String>,
    font_family: Option<String>,
    relationship_id: Option<String>,
}

impl PendingWatermark {
    fn finish(self) -> Option<VmlWatermark> {
        let width_pt = self.width?;
        let height_pt = self.height?;
        if !width_pt.is_finite()
            || !height_pt.is_finite()
            || !self.rotation.is_finite()
            || !self.opacity.is_finite()
            || width_pt <= 0.0
            || height_pt <= 0.0
        {
            return None;
        }
        let opacity = self.opacity.clamp(0.0, 1.0);
        match (self.text, self.relationship_id) {
            (Some(text), None) => Some(VmlWatermark::Text {
                text,
                width_pt,
                height_pt,
                rotation_degrees: self.rotation,
                color: self.color.unwrap_or_else(|| "D9D9D9".to_owned()),
                font_family: self.font_family,
                opacity,
            }),
            (None, Some(relationship_id)) => Some(VmlWatermark::Image {
                relationship_id,
                width_pt,
                height_pt,
                rotation_degrees: self.rotation,
                opacity,
            }),
            _ => None,
        }
    }
}

fn parse_vml_watermarks(xml: &[u8]) -> Vec<VmlWatermark> {
    let mut reader = NsReader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut in_pict = false;
    let mut pending = None;
    let mut watermarks = Vec::new();

    loop {
        let Ok((namespace, event)) = reader.read_resolved_event_into(&mut buffer) else {
            return Vec::new();
        };
        match event {
            Event::Start(ref element) | Event::Empty(ref element) => {
                let name = element.name();
                let local = local_name(name.as_ref());
                if namespace_is(&namespace, W_NS) && local == b"pict" {
                    in_pict = true;
                } else if in_pict && namespace_is(&namespace, VML_NS) && local == b"shape" {
                    let Some(style) = unqualified_attribute(&reader, element, b"style") else {
                        buffer.clear();
                        continue;
                    };
                    pending = Some(PendingWatermark {
                        width: style_number(&style, "width", "pt"),
                        height: style_number(&style, "height", "pt"),
                        rotation: style_number(&style, "rotation", "").unwrap_or(0.0),
                        color: unqualified_attribute(&reader, element, b"fillcolor")
                            .map(|value| value.trim_start_matches('#').to_owned()),
                        opacity: style_number(&style, "opacity", "").unwrap_or(1.0),
                        ..PendingWatermark::default()
                    });
                } else if let Some(shape) = pending.as_mut()
                    && namespace_is(&namespace, VML_NS)
                    && local == b"fill"
                {
                    shape.opacity = unqualified_attribute(&reader, element, b"opacity")
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(shape.opacity);
                } else if let Some(shape) = pending.as_mut()
                    && namespace_is(&namespace, VML_NS)
                    && local == b"textpath"
                {
                    shape.text = unqualified_attribute(&reader, element, b"string");
                    shape.font_family = unqualified_attribute(&reader, element, b"style")
                        .and_then(|style| style_value(&style, "font-family"))
                        .map(|family| family.trim_matches(['\'', '"']).to_owned());
                } else if let Some(shape) = pending.as_mut()
                    && namespace_is(&namespace, VML_NS)
                    && local == b"imagedata"
                {
                    shape.relationship_id =
                        namespaced_attribute(&reader, element, RELATIONSHIPS_NS, b"id");
                }
            }
            Event::End(ref element)
                if namespace_is(&namespace, VML_NS)
                    && local_name(element.name().as_ref()) == b"shape" =>
            {
                if let Some(watermark) = pending.take().and_then(PendingWatermark::finish) {
                    watermarks.push(watermark);
                }
            }
            Event::End(ref element)
                if namespace_is(&namespace, W_NS)
                    && local_name(element.name().as_ref()) == b"pict" =>
            {
                in_pict = false;
                pending = None;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    watermarks
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn namespace_is(namespace: &ResolveResult<'_>, expected: &str) -> bool {
    matches!(namespace, ResolveResult::Bound(Namespace(uri)) if *uri == expected.as_bytes())
}

fn unqualified_attribute(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
    expected: &[u8],
) -> Option<String> {
    element.attributes().flatten().find_map(|attribute| {
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        if matches!(namespace, ResolveResult::Unbound) && local.as_ref() == expected {
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                .ok()
                .map(|value| value.into_owned())
        } else {
            None
        }
    })
}

fn raw_unqualified_attribute(element: &BytesStart<'_>, expected: &[u8]) -> Option<String> {
    element.attributes().flatten().find_map(|attribute| {
        (attribute.key.as_ref() == expected)
            .then(|| {
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                    .ok()
                    .map(|value| value.into_owned())
            })
            .flatten()
    })
}

fn namespaced_attribute(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
    expected_namespace: &str,
    expected_local: &[u8],
) -> Option<String> {
    element.attributes().flatten().find_map(|attribute| {
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        if namespace_is(&namespace, expected_namespace) && local.as_ref() == expected_local {
            attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())
                .ok()
                .map(|value| value.into_owned())
        } else {
            None
        }
    })
}

fn style_value(style: &str, expected: &str) -> Option<String> {
    style.split(';').find_map(|declaration| {
        let (name, value) = declaration.split_once(':')?;
        name.trim()
            .eq_ignore_ascii_case(expected)
            .then(|| value.trim().to_owned())
    })
}

fn style_number(style: &str, name: &str, suffix: &str) -> Option<f64> {
    style_value(style, name)?
        .strip_suffix(suffix)?
        .trim()
        .parse()
        .ok()
}

fn compact_number(value: f64) -> String {
    let mut value = format!("{value:.6}");
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    value
}

/// Header/footer reference type in section properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HdrFtrType {
    /// Default header/footer
    Default,
    /// First page header/footer
    First,
    /// Even page header/footer
    Even,
}

impl HdrFtrType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "first" => Self::First,
            "even" => Self::Even,
            _ => Self::Default,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::First => "first",
            Self::Even => "even",
        }
    }
}

/// A header or footer reference (stored in section properties).
#[derive(Debug, Clone, PartialEq)]
pub struct HdrFtrRef {
    /// The type (default, first, even)
    pub hdr_ftr_type: HdrFtrType,
    /// Relationship ID
    pub rel_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_header() {
        let mut hdr = CT_HdrFtr::new();
        let mut p = CT_P::new();
        p.add_run("Page Header");
        hdr.paragraphs.push(p);

        let xml = hdr.to_xml_header().unwrap();
        let parsed = CT_HdrFtr::from_xml(&xml).unwrap();
        assert_eq!(parsed.paragraphs.len(), 1);
        assert_eq!(parsed.text(), "Page Header");
    }

    #[test]
    fn round_trip_footer() {
        let mut ftr = CT_HdrFtr::new();
        let mut p = CT_P::new();
        p.add_run("Page Footer");
        ftr.paragraphs.push(p);

        let xml = ftr.to_xml_footer().unwrap();
        let parsed = CT_HdrFtr::from_xml(&xml).unwrap();
        assert_eq!(parsed.text(), "Page Footer");
    }

    #[test]
    fn empty_header() {
        let hdr = CT_HdrFtr::new();
        let xml = hdr.to_xml_header().unwrap();
        let parsed = CT_HdrFtr::from_xml(&xml).unwrap();
        assert_eq!(parsed.paragraphs.len(), 0);
    }

    #[test]
    fn aliased_header_paragraph_properties_keep_root_scope() {
        let xml = format!(
            r#"<q:hdr xmlns:q="{W_NS}" xmlns:ext="urn:producer"><ext:p><ext:pPr><ext:jc ext:val="right"/></ext:pPr></ext:p><q:p><q:pPr><ext:jc ext:val="right"/><q:jc q:val="center"/></q:pPr><q:r><q:t>Header</q:t></q:r></q:p></q:hdr>"#
        );
        let parsed = CT_HdrFtr::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(parsed.paragraphs.len(), 1);
        assert_eq!(parsed.text(), "Header");
        assert_eq!(
            parsed.paragraphs[0].properties.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn default_namespace_header_properties_keep_root_scope() {
        let xml = format!(
            r#"<hdr xmlns="{W_NS}" xmlns:w="{W_NS}" xmlns:ext="urn:producer"><ext:p><ext:pPr><ext:jc ext:val="right"/></ext:pPr></ext:p><p><pPr><ext:jc ext:val="right"/><jc w:val="center"/></pPr><r><t>Header</t></r></p></hdr>"#
        );
        let parsed = CT_HdrFtr::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(parsed.paragraphs.len(), 1);
        assert_eq!(parsed.text(), "Header");
        assert_eq!(
            parsed.paragraphs[0].properties.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn word_vml_watermarks_parse_and_preserve_source_bytes() {
        let text_pict = r##"<q:pict><x:shape style="width:468pt;height:117pt;rotation:315" fillcolor="#D9D9D9"><x:textpath string="DRAFT" style="font-family:&quot;Calibri&quot;"/></x:shape></q:pict>"##;
        let image_pict = r#"<q:pict><x:shape style="width:72pt;height:36pt;rotation:0"><x:fill opacity=".25"/><x:imagedata rel:id="rId7"/></x:shape></q:pict>"#;
        let ordinary_pict = r#"<q:pict><x:shape id="ordinary"><x:path/></x:shape></q:pict>"#;
        let xml = format!(
            r#"<q:hdr xmlns:q="{W_NS}" xmlns:x="{VML_NS}" xmlns:rel="{RELATIONSHIPS_NS}"><q:p><q:r>{text_pict}{image_pict}{ordinary_pict}</q:r></q:p></q:hdr>"#
        );
        let parsed = CT_HdrFtr::from_xml(xml.as_bytes()).unwrap();
        assert!(matches!(
            &parsed.watermarks()[0],
            VmlWatermark::Text {
                text,
                width_pt: 468.0,
                height_pt: 117.0,
                rotation_degrees: 315.0,
                color,
                font_family: Some(font_family),
                opacity: 1.0,
            } if text == "DRAFT" && color == "D9D9D9" && font_family == "Calibri"
        ));
        assert!(matches!(
            &parsed.watermarks()[1],
            VmlWatermark::Image {
                relationship_id,
                width_pt: 72.0,
                height_pt: 36.0,
                rotation_degrees: 0.0,
                opacity: 0.25,
            } if relationship_id == "rId7"
        ));
        let serialized = parsed.to_xml_header().unwrap();
        for source in [text_pict, image_pict, ordinary_pict] {
            assert!(
                serialized
                    .windows(source.len())
                    .any(|window| window == source.as_bytes()),
                "{}",
                String::from_utf8_lossy(&serialized)
            );
        }
    }

    #[test]
    fn generated_watermarks_write_fixed_prefixes_and_vml_child_order() {
        let watermark = VmlWatermark::Text {
            text: "DRAFT".to_owned(),
            width_pt: 468.0,
            height_pt: 117.0,
            rotation_degrees: 315.0,
            color: "D9D9D9".to_owned(),
            font_family: Some("Calibri".to_owned()),
            opacity: 0.5,
        };
        let xml = watermark.to_pict_xml();
        let fill = xml
            .windows(b"<v:fill".len())
            .position(|w| w == b"<v:fill")
            .unwrap();
        let textpath = xml
            .windows(b"<v:textpath".len())
            .rposition(|w| w == b"<v:textpath")
            .unwrap();
        assert!(fill < textpath);
        let shape_type = xml
            .windows(b"<v:shapetype".len())
            .position(|window| window == b"<v:shapetype")
            .unwrap();
        let shape = xml
            .windows(b"<v:shape ".len())
            .position(|window| window == b"<v:shape ")
            .unwrap();
        assert!(shape_type < shape);
        assert!(
            xml.windows(b"id=\"_x0000_t136\"".len())
                .any(|window| { window == b"id=\"_x0000_t136\"" })
        );
        assert!(
            xml.windows(b"type=\"#_x0000_t136\"".len())
                .any(|window| { window == b"type=\"#_x0000_t136\"" })
        );
        assert!(xml.starts_with(
            br#"<w:pict xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#
        ));
        assert_eq!(VmlWatermark::from_pict_xml(&xml), Some(watermark.clone()));

        let image = VmlWatermark::Image {
            relationship_id: "rId3".to_owned(),
            width_pt: 72.0,
            height_pt: 36.0,
            rotation_degrees: 0.0,
            opacity: 0.5,
        };
        let image_xml = image.to_pict_xml();
        let fill = image_xml
            .windows(b"<v:fill".len())
            .position(|w| w == b"<v:fill")
            .unwrap();
        let image_data = image_xml
            .windows(b"<v:imagedata".len())
            .position(|w| w == b"<v:imagedata")
            .unwrap();
        assert!(fill < image_data);
        assert!(
            image_xml
                .windows(b"id=\"_x0000_t75\"".len())
                .any(|window| window == b"id=\"_x0000_t75\"")
        );
        assert!(
            image_xml
                .windows(b"type=\"#_x0000_t75\"".len())
                .any(|window| window == b"type=\"#_x0000_t75\"")
        );
        assert_eq!(VmlWatermark::from_pict_xml(&image_xml), Some(image));

        let legacy = format!(
            r##"<w:hdr xmlns:w="{W_NS}" xmlns:v="{VML_NS}"><w:p><w:r><w:pict><v:shape id="rdocx-watermark" type="#_x0000_t136" style="width:468pt;height:117pt"><v:textpath string="OLD"/></v:shape></w:pict></w:r></w:p></w:hdr>"##
        );
        let replaced = replace_authored_watermark(legacy.as_bytes(), &watermark).unwrap();
        assert!(
            !replaced
                .windows(b"type=\"#_x0000_t136\"".len())
                .any(|window| window == b"type=\"#_x0000_t136\"")
        );
        assert_eq!(
            CT_HdrFtr::from_xml(&replaced).unwrap().watermarks(),
            &[watermark]
        );
    }

    #[test]
    fn non_watermark_w_pict_remains_opaque() {
        let xml = format!(
            r#"<w:hdr xmlns:w="{W_NS}" xmlns:v="urn:schemas-microsoft-com:vml"><w:p><w:r><w:pict><v:shape id="ordinary"><v:path/></v:shape></w:pict></w:r></w:p></w:hdr>"#
        );
        let parsed = CT_HdrFtr::from_xml(xml.as_bytes()).unwrap();
        assert!(parsed.watermarks().is_empty());
        let serialized = parsed.to_xml_header().unwrap();
        assert!(
            serialized
                .windows(b"id=\"ordinary\"".len())
                .any(|w| w == b"id=\"ordinary\"")
        );
    }

    #[test]
    fn authored_watermark_patch_preserves_complete_header_bytes() {
        let closing = b"</q:hdr>";
        let source = format!(
            r#"<?xml version="1.0"?><q:hdr xmlns:q="{W_NS}" xmlns:mc="urn:mc" xmlns:x="urn:producer" mc:Ignorable="x" x:root="kept"><q:tbl><q:tr><q:tc><q:p/></q:tc></q:tr></q:tbl><q:sdt><q:sdtPr/><q:sdtContent><q:p/></q:sdtContent></q:sdt><q:p><q:r xmlns:v="{VML_NS}"><q:pict><v:shape id="producer"><v:path/></v:shape></q:pict></q:r></q:p>{}</q:hdr>"#,
            "tail"
        );
        let watermark = VmlWatermark::Text {
            text: "DRAFT".to_owned(),
            width_pt: 468.0,
            height_pt: 117.0,
            rotation_degrees: 315.0,
            color: "D9D9D9".to_owned(),
            font_family: Some("Calibri".to_owned()),
            opacity: 0.5,
        };
        let updated = replace_authored_watermark(source.as_bytes(), &watermark).unwrap();
        let prefix = &source.as_bytes()[..source.len() - closing.len()];
        assert_eq!(&updated[..prefix.len()], prefix);
        assert_eq!(&updated[updated.len() - closing.len()..], closing);
        assert_eq!(
            CT_HdrFtr::from_xml(&updated).unwrap().watermarks(),
            &[watermark]
        );
    }

    #[test]
    fn authored_watermark_ownership_requires_the_vml_shape_id_attribute() {
        let unrelated = br#"<x:data>id=&quot;rdocx-watermark&quot;</x:data>"#;
        let source = format!(
            r#"<w:hdr xmlns:w="{W_NS}" xmlns:x="urn:producer"><w:p><w:r>{}</w:r></w:p></w:hdr>"#,
            String::from_utf8_lossy(unrelated)
        );
        let watermark = VmlWatermark::Text {
            text: "FINAL".to_owned(),
            width_pt: 468.0,
            height_pt: 117.0,
            rotation_degrees: 315.0,
            color: "D9D9D9".to_owned(),
            font_family: Some("Calibri".to_owned()),
            opacity: 0.5,
        };
        let updated = replace_authored_watermark(source.as_bytes(), &watermark).unwrap();
        assert!(
            updated
                .windows(unrelated.len())
                .any(|window| window == unrelated)
        );
        let replaced = replace_authored_watermark(&updated, &watermark).unwrap();
        assert!(
            replaced
                .windows(unrelated.len())
                .any(|window| window == unrelated)
        );
        assert_eq!(
            CT_HdrFtr::from_xml(&replaced).unwrap().watermarks(),
            &[watermark]
        );
    }

    #[test]
    fn foreign_same_local_end_tags_do_not_terminate_vml_projection() {
        let xml = format!(
            r#"<w:hdr xmlns:w="{W_NS}" xmlns:v="{VML_NS}" xmlns:x="urn:producer"><w:p><w:r><w:pict><v:shape style="width:468pt;height:117pt"><x:shape><x:pict/></x:shape><x:pict></x:pict><v:textpath string="DRAFT"/></v:shape></w:pict></w:r></w:p></w:hdr>"#
        );
        let parsed = CT_HdrFtr::from_xml(xml.as_bytes()).unwrap();
        assert!(matches!(
            parsed.watermarks(),
            [VmlWatermark::Text { text, .. }] if text == "DRAFT"
        ));
    }
}

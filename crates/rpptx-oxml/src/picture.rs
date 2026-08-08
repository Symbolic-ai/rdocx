use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::fill::BlipFill;
use oxml_drawing::namespace::A_NS;
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::shape_props::CT_ShapeProperties;
use oxml_drawing::xfrm::CT_Transform2D;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::namespace::{
    FIXED_SHAPE_TREE_PREFIXES, MC_NS, NamespaceBindings, P_NS, R_NS, non_visual_drawing_id,
    root_attributes, self_contained_attributes, set_non_visual_drawing_name,
};
use crate::placeholder::{ApplicationProperties, CT_Placeholder, parse_application_properties};

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;

/// A typed `p:pic` shape-tree child.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_Picture {
    pub placeholder: Option<CT_Placeholder>,
    pub blip_fill: Option<BlipFill>,
    pub shape_properties: CT_ShapeProperties,
    raw: Box<PictureRaw>,
}

#[derive(Clone, Debug, PartialEq)]
struct PictureRaw {
    raw_attributes: RawAttributes,
    non_visual_attributes: RawAttributes,
    non_visual_children: OrderedRawChildren,
    non_visual_drawing_properties: Vec<u8>,
    non_visual_picture_properties: Vec<u8>,
    application_properties_attributes: RawAttributes,
    application_properties_raw_children: OrderedRawChildren,
    raw_children: OrderedRawChildren,
}

struct ParsedNonVisualPicture {
    placeholder: Option<CT_Placeholder>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    non_visual_drawing_properties: Vec<u8>,
    non_visual_picture_properties: Vec<u8>,
    application_properties_attributes: RawAttributes,
    application_properties_raw_children: OrderedRawChildren,
}

impl CT_Picture {
    /// Creates a relationship-backed picture with canonical non-visual shells.
    pub fn new(
        id: u32,
        name: &str,
        relationship_id: &str,
        transform: CT_Transform2D,
    ) -> Result<Self> {
        let name = quick_xml::escape::escape(name);
        let relationship_id = quick_xml::escape::escape(relationship_id);
        let transform_xml = transform.to_xml().map_err(drawing_error)?;
        let transform_xml = std::str::from_utf8(&transform_xml)?;
        Self::from_xml(
            format!(
                r#"<p:pic xmlns:p="{P_NS}" xmlns:a="{A_NS}" xmlns:r="{R_NS}"><p:nvPicPr><p:cNvPr id="{id}" name="{name}"/><p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed="{relationship_id}"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr>{transform_xml}</p:spPr></p:pic>"#
            )
            .as_bytes(),
        )
    }

    pub(crate) fn non_visual_id(&self) -> Option<u32> {
        non_visual_drawing_id(&self.raw.non_visual_drawing_properties)
    }

    /// Changes the producer-facing non-visual picture name.
    pub fn set_name(&mut self, name: &str) -> Result<()> {
        set_non_visual_drawing_name(&mut self.raw.non_visual_drawing_properties, name)
    }

    /// Parses a complete `p:pic` with any prefix bound to PresentationML.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_fragment(xml, &[])
    }

    pub(crate) fn from_fragment(xml: &[u8], inherited: &[(String, String)]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let namespaces =
                        NamespaceBindings::from_entries(inherited).with_start(&start)?;
                    if local_name(start.name().as_ref()) != b"pic"
                        || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                    {
                        return Err(unexpected(&start));
                    }
                    namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
                    return Self::from_reader(&mut reader, &start, &namespaces, inherited);
                }
                Event::Empty(start) => {
                    return Err(OxmlError::MissingElement(format!(
                        "{} requires p:nvPicPr, a blip-fill choice, and p:spPr",
                        String::from_utf8_lossy(start.name().as_ref())
                    )));
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:pic".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_reader(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        namespaces: &NamespaceBindings,
        inherited: &[(String, String)],
    ) -> Result<Self> {
        let mut non_visual = None;
        let mut blip_fill = None;
        let mut shape_properties = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut state = 0usize;
        let mut has_alternate_blip = false;
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) => {
                    let child_namespaces = namespaces.with_start(&child)?;
                    let name = local_name(child.name().as_ref()).to_vec();
                    let uri = child_namespaces.element_uri(child.name().as_ref());
                    let raw = capture_element(reader, &child)?;
                    capture_picture_child(
                        &name,
                        uri,
                        raw,
                        &child_namespaces,
                        &mut non_visual,
                        &mut blip_fill,
                        &mut shape_properties,
                        &mut raw_children,
                        &mut state,
                        &mut has_alternate_blip,
                    )?;
                }
                Event::Empty(child) => {
                    let child_namespaces = namespaces.with_start(&child)?;
                    let name = local_name(child.name().as_ref()).to_vec();
                    let uri = child_namespaces.element_uri(child.name().as_ref());
                    let raw = capture_empty_element(&child)?;
                    capture_picture_child(
                        &name,
                        uri,
                        raw,
                        &child_namespaces,
                        &mut non_visual,
                        &mut blip_fill,
                        &mut shape_properties,
                        &mut raw_children,
                        &mut state,
                        &mut has_alternate_blip,
                    )?;
                }
                Event::End(end) if local_name(end.name().as_ref()) == b"pic" => break,
                Event::Eof => return Err(OxmlError::MissingElement("closing p:pic".to_owned())),
                _ => {}
            }
            buffer.clear();
        }

        if blip_fill.is_none() && !has_alternate_blip {
            return Err(OxmlError::MissingElement(
                "p:pic blip-fill choice".to_owned(),
            ));
        }
        let non_visual = required(non_visual, "p:nvPicPr")?;
        Ok(Self {
            placeholder: non_visual.placeholder,
            blip_fill,
            shape_properties: required(shape_properties, "p:spPr")?,
            raw: Box::new(PictureRaw {
                raw_attributes: self_contained_attributes(
                    start,
                    FIXED_SHAPE_TREE_PREFIXES,
                    inherited,
                )?,
                non_visual_attributes: non_visual.raw_attributes,
                non_visual_children: non_visual.raw_children,
                non_visual_drawing_properties: non_visual.non_visual_drawing_properties,
                non_visual_picture_properties: non_visual.non_visual_picture_properties,
                application_properties_attributes: non_visual.application_properties_attributes,
                application_properties_raw_children: non_visual.application_properties_raw_children,
                raw_children,
            }),
        })
    }

    /// Serialises a self-contained picture with fixed modelled prefixes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml_internal(&mut writer, true)?;
        Ok(writer.into_inner())
    }

    pub(crate) fn write_xml_internal<W: Write>(
        &self,
        writer: &mut Writer<W>,
        declare_namespaces: bool,
    ) -> Result<()> {
        let mut start = BytesStart::new("p:pic");
        if declare_namespaces {
            start.push_attribute(("xmlns:p", P_NS));
            start.push_attribute(("xmlns:a", A_NS));
            start.push_attribute(("xmlns:r", R_NS));
            start.push_attribute(("xmlns:mc", MC_NS));
        }
        push_attributes(&mut start, &self.raw.raw_attributes);
        writer.write_event(Event::Start(start))?;

        emit_raw(writer, self.raw.raw_children.at(0))?;
        self.write_non_visual_properties(writer)?;
        emit_raw(writer, self.raw.raw_children.at(1))?;
        if let Some(blip_fill) = &self.blip_fill {
            blip_fill
                .write_xml_as(writer, "p:blipFill")
                .map_err(drawing_error)?;
        }
        emit_raw(writer, self.raw.raw_children.at(2))?;
        self.shape_properties
            .write_xml_as(writer, "p:spPr")
            .map_err(drawing_error)?;
        for boundary in 3..=5 {
            emit_raw(writer, self.raw.raw_children.at(boundary))?;
        }
        writer.write_event(Event::End(BytesEnd::new("p:pic")))?;
        Ok(())
    }

    fn write_non_visual_properties<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("p:nvPicPr");
        push_attributes(&mut start, &self.raw.non_visual_attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw.non_visual_children.at(0))?;
        writer
            .get_mut()
            .write_all(&self.raw.non_visual_drawing_properties)?;
        emit_raw(writer, self.raw.non_visual_children.at(1))?;
        writer
            .get_mut()
            .write_all(&self.raw.non_visual_picture_properties)?;
        emit_raw(writer, self.raw.non_visual_children.at(2))?;

        let mut application = BytesStart::new("p:nvPr");
        push_attributes(
            &mut application,
            &self.raw.application_properties_attributes,
        );
        if self.placeholder.is_none() && self.raw.application_properties_raw_children.is_empty() {
            writer.write_event(Event::Empty(application))?;
        } else {
            writer.write_event(Event::Start(application))?;
            emit_raw(writer, self.raw.application_properties_raw_children.at(0))?;
            if let Some(placeholder) = &self.placeholder {
                placeholder.write_xml(writer, false)?;
            }
            emit_raw(writer, self.raw.application_properties_raw_children.at(1))?;
            writer.write_event(Event::End(BytesEnd::new("p:nvPr")))?;
        }

        emit_raw(writer, self.raw.non_visual_children.at(3))?;
        writer.write_event(Event::End(BytesEnd::new("p:nvPicPr")))?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_picture_child(
    name: &[u8],
    uri: Option<&str>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    non_visual: &mut Option<ParsedNonVisualPicture>,
    blip_fill: &mut Option<BlipFill>,
    shape_properties: &mut Option<CT_ShapeProperties>,
    raw_children: &mut OrderedRawChildren,
    state: &mut usize,
    has_alternate_blip: &mut bool,
) -> Result<()> {
    if uri == Some(P_NS) && matches!(name, b"nvPicPr" | b"blipFill" | b"spPr") {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
    }
    match (uri, name) {
        (Some(P_NS), b"nvPicPr") if *state == 0 && non_visual.is_none() => {
            *non_visual = Some(parse_non_visual_picture(&raw, namespaces)?);
            *state = 1;
        }
        (Some(P_NS), b"blipFill") if *state == 1 && blip_fill.is_none() && !*has_alternate_blip => {
            validate_blip_fill(&raw, namespaces)?;
            *blip_fill = Some(BlipFill::from_xml(&raw).map_err(drawing_error)?);
            *state = 2;
        }
        (Some(MC_NS), b"AlternateContent")
            if *state == 1 && blip_fill.is_none() && !*has_alternate_blip =>
        {
            namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
            if !alternate_contains_blip_fill(&raw, namespaces)? {
                return Err(OxmlError::InvalidValue(
                    "p:pic alternate content does not contain a p:blipFill choice".to_owned(),
                ));
            }
            raw_children.push(1, raw);
            *has_alternate_blip = true;
        }
        (Some(P_NS), b"spPr") if matches!(*state, 1 | 2) && shape_properties.is_none() => {
            if blip_fill.is_none() && !*has_alternate_blip {
                return Err(OxmlError::InvalidValue(
                    "p:spPr must follow the p:pic blip-fill choice".to_owned(),
                ));
            }
            *shape_properties = Some(CT_ShapeProperties::from_xml(&raw).map_err(drawing_error)?);
            *state = 3;
        }
        (Some(P_NS), b"style") if *state == 3 => {
            raw_children.push(3, raw);
            *state = 4;
        }
        (Some(P_NS), b"extLst") if matches!(*state, 3 | 4) => {
            raw_children.push(4, raw);
            *state = 5;
        }
        (Some(P_NS), b"nvPicPr" | b"blipFill" | b"spPr" | b"style" | b"extLst") => {
            return Err(OxmlError::InvalidValue(
                "p:pic children are outside schema order or duplicated".to_owned(),
            ));
        }
        (Some(MC_NS), b"AlternateContent") => {
            return Err(OxmlError::InvalidValue(
                "p:pic blip-fill choice is duplicated or outside schema order".to_owned(),
            ));
        }
        _ => raw_children.push(*state, raw),
    }
    Ok(())
}

fn alternate_contains_blip_fill(xml: &[u8], inherited: &NamespaceBindings) -> Result<bool> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = inherited.with_start(&start)?;
                loop {
                    buffer.clear();
                    match reader.read_event_into(&mut buffer)? {
                        Event::Start(branch) => {
                            let branch_namespaces = namespaces.with_start(&branch)?;
                            let is_branch = branch_namespaces.element_uri(branch.name().as_ref())
                                == Some(MC_NS)
                                && matches!(
                                    local_name(branch.name().as_ref()),
                                    b"Choice" | b"Fallback"
                                );
                            if is_branch
                                && branch_contains_blip_fill(
                                    &mut reader,
                                    &branch,
                                    &branch_namespaces,
                                )?
                            {
                                return Ok(true);
                            }
                            if !is_branch {
                                capture_element(&mut reader, &branch)?;
                            }
                        }
                        Event::Empty(_) => {}
                        Event::End(end)
                            if local_name(end.name().as_ref()) == b"AlternateContent" =>
                        {
                            return Ok(false);
                        }
                        Event::Eof => {
                            return Err(OxmlError::MissingElement(
                                "closing mc:AlternateContent".to_owned(),
                            ));
                        }
                        _ => {}
                    }
                }
            }
            Event::Empty(_) => return Ok(false),
            Event::Eof => {
                return Err(OxmlError::MissingElement("mc:AlternateContent".to_owned()));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn branch_contains_blip_fill(
    reader: &mut Reader<&[u8]>,
    branch: &BytesStart<'_>,
    namespaces: &NamespaceBindings,
) -> Result<bool> {
    let end_name = local_name(branch.name().as_ref()).to_vec();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(child) => {
                let child_namespaces = namespaces.with_start(&child)?;
                if child_namespaces.element_uri(child.name().as_ref()) == Some(P_NS)
                    && local_name(child.name().as_ref()) == b"blipFill"
                {
                    return Ok(true);
                }
                capture_element(reader, &child)?;
            }
            Event::Empty(child) => {
                let child_namespaces = namespaces.with_start(&child)?;
                if child_namespaces.element_uri(child.name().as_ref()) == Some(P_NS)
                    && local_name(child.name().as_ref()) == b"blipFill"
                {
                    return Ok(true);
                }
            }
            Event::End(end) if local_name(end.name().as_ref()) == end_name => return Ok(false),
            Event::Eof => {
                return Err(OxmlError::MissingElement(format!(
                    "closing mc:{}",
                    String::from_utf8_lossy(&end_name)
                )));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_non_visual_picture(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<ParsedNonVisualPicture> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = inherited.with_start(&start)?;
                let raw_attributes = root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?;
                let mut drawing = None;
                let mut picture = None;
                let mut application = None;
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                loop {
                    buffer.clear();
                    match reader.read_event_into(&mut buffer)? {
                        Event::Start(child) => {
                            let child_namespaces = namespaces.with_start(&child)?;
                            let name = local_name(child.name().as_ref()).to_vec();
                            let uri = child_namespaces.element_uri(child.name().as_ref());
                            let raw = capture_element(&mut reader, &child)?;
                            capture_non_visual_child(
                                &name,
                                uri,
                                raw,
                                &child_namespaces,
                                &mut drawing,
                                &mut picture,
                                &mut application,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        Event::Empty(child) => {
                            let child_namespaces = namespaces.with_start(&child)?;
                            let name = local_name(child.name().as_ref()).to_vec();
                            let uri = child_namespaces.element_uri(child.name().as_ref());
                            let raw = capture_empty_element(&child)?;
                            capture_non_visual_child(
                                &name,
                                uri,
                                raw,
                                &child_namespaces,
                                &mut drawing,
                                &mut picture,
                                &mut application,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        Event::End(end) if local_name(end.name().as_ref()) == b"nvPicPr" => break,
                        Event::Eof => {
                            return Err(OxmlError::MissingElement("closing p:nvPicPr".to_owned()));
                        }
                        _ => {}
                    }
                }
                let application = required(application, "p:nvPr")?;
                return Ok(ParsedNonVisualPicture {
                    placeholder: application.placeholder,
                    raw_attributes,
                    raw_children,
                    non_visual_drawing_properties: required(drawing, "p:cNvPr")?,
                    non_visual_picture_properties: required(picture, "p:cNvPicPr")?,
                    application_properties_attributes: application.raw_attributes,
                    application_properties_raw_children: application.raw_children,
                });
            }
            Event::Empty(_) => {
                return Err(OxmlError::MissingElement(
                    "p:nvPicPr requires p:cNvPr, p:cNvPicPr, and p:nvPr".to_owned(),
                ));
            }
            Event::Eof => return Err(OxmlError::MissingElement("p:nvPicPr".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_non_visual_child(
    name: &[u8],
    uri: Option<&str>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    drawing: &mut Option<Vec<u8>>,
    picture: &mut Option<Vec<u8>>,
    application: &mut Option<ApplicationProperties>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if uri == Some(P_NS) && matches!(name, b"cNvPr" | b"cNvPicPr" | b"nvPr") {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
    }
    match (uri, name) {
        (Some(P_NS), b"cNvPr") if *boundary == 0 && drawing.is_none() => {
            *drawing = Some(raw);
            *boundary = 1;
        }
        (Some(P_NS), b"cNvPicPr") if *boundary == 1 && picture.is_none() => {
            *picture = Some(raw);
            *boundary = 2;
        }
        (Some(P_NS), b"nvPr") if *boundary == 2 && application.is_none() => {
            *application = Some(parse_application_properties(&raw, namespaces)?);
            *boundary = 3;
        }
        (Some(P_NS), b"cNvPr" | b"cNvPicPr" | b"nvPr") => {
            return Err(OxmlError::InvalidValue(
                "p:nvPicPr children must be p:cNvPr, p:cNvPicPr, and p:nvPr in order".to_owned(),
            ));
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

fn validate_blip_fill(xml: &[u8], inherited: &NamespaceBindings) -> Result<()> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = inherited.with_start(&start)?;
                if namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                    || local_name(start.name().as_ref()) != b"blipFill"
                {
                    return Err(unexpected(&start));
                }
                loop {
                    buffer.clear();
                    match reader.read_event_into(&mut buffer)? {
                        Event::Start(child) => {
                            let child_namespaces = namespaces.with_start(&child)?;
                            validate_blip_fill_child(&child, &child_namespaces)?;
                            capture_element(&mut reader, &child)?;
                        }
                        Event::Empty(child) => {
                            let child_namespaces = namespaces.with_start(&child)?;
                            validate_blip_fill_child(&child, &child_namespaces)?;
                        }
                        Event::End(end) if local_name(end.name().as_ref()) == b"blipFill" => {
                            return Ok(());
                        }
                        Event::Eof => {
                            return Err(OxmlError::MissingElement("closing p:blipFill".to_owned()));
                        }
                        _ => {}
                    }
                }
            }
            Event::Empty(start) => {
                let namespaces = inherited.with_start(&start)?;
                if namespaces.element_uri(start.name().as_ref()) == Some(P_NS)
                    && local_name(start.name().as_ref()) == b"blipFill"
                {
                    return Ok(());
                }
                return Err(unexpected(&start));
            }
            Event::Eof => return Err(OxmlError::MissingElement("p:blipFill".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn validate_blip_fill_child(child: &BytesStart<'_>, namespaces: &NamespaceBindings) -> Result<()> {
    let qualified_name = child.name();
    let name = local_name(qualified_name.as_ref());
    if matches!(name, b"blip" | b"srcRect" | b"stretch" | b"tile")
        && namespaces.element_uri(child.name().as_ref()) != Some(A_NS)
    {
        return Err(OxmlError::InvalidValue(format!(
            "p:blipFill {} must use the DrawingML namespace",
            String::from_utf8_lossy(name)
        )));
    }
    if name == b"blip" && namespaces.element_uri(child.name().as_ref()) == Some(A_NS) {
        for attribute in child.attributes() {
            let attribute = attribute?;
            let attribute_name = attribute.key.as_ref();
            if matches!(local_name(attribute_name), b"embed" | b"link")
                && namespaces.attribute_uri(attribute_name) != Some(R_NS)
            {
                return Err(OxmlError::InvalidValue(format!(
                    "a:blip {} must use the Office relationships namespace",
                    String::from_utf8_lossy(attribute_name)
                )));
            }
        }
    }
    Ok(())
}

fn drawing_error(error: impl std::fmt::Display) -> OxmlError {
    OxmlError::InvalidValue(error.to_string())
}

fn required<T>(value: Option<T>, name: &str) -> Result<T> {
    value.ok_or_else(|| OxmlError::MissingElement(name.to_owned()))
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn unexpected(start: &BytesStart<'_>) -> OxmlError {
    OxmlError::InvalidValue(format!(
        "unexpected PresentationML picture element: {}",
        String::from_utf8_lossy(start.name().as_ref())
    ))
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &RawAttributes) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn emit_raw<'a, W: Write>(
    writer: &mut Writer<W>,
    children: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    for child in children {
        writer.get_mut().write_all(child)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use oxml_core::units::Emu;
    use oxml_drawing::xfrm::{CT_Point2D, CT_PositiveSize2D, CT_Transform2D};

    use super::CT_Picture;

    #[test]
    fn picture_constructor_round_trips_in_schema_order() {
        let mut transform = CT_Transform2D::default();
        transform.offset = Some(CT_Point2D {
            x: Emu(10),
            y: Emu(20),
        });
        transform.extent = Some(CT_PositiveSize2D {
            cx: Emu(30),
            cy: Emu(40),
        });
        let picture = CT_Picture::new(7, "Picture & 7", "rId9", transform).unwrap();

        let xml = picture.to_xml().unwrap();
        let text = String::from_utf8(xml.clone()).unwrap();
        assert!(text.contains("<p:cNvPr id=\"7\" name=\"Picture &amp; 7\"/>"));
        assert!(text.contains("r:embed=\"rId9\""));
        assert!(text.contains("<a:off x=\"10\" y=\"20\"/>"));
        assert!(text.contains("<a:ext cx=\"30\" cy=\"40\"/>"));
        let non_visual = text.find("<p:nvPicPr").unwrap();
        let blip_fill = text.find("<p:blipFill").unwrap();
        let shape_properties = text.find("<p:spPr").unwrap();
        assert!(non_visual < blip_fill);
        assert!(blip_fill < shape_properties);

        let reparsed = CT_Picture::from_xml(&xml).unwrap();
        assert_eq!(reparsed.non_visual_id(), Some(7));
        assert_eq!(
            reparsed.blip_fill.unwrap().blip.unwrap().embed.as_deref(),
            Some("rId9")
        );
        assert_eq!(
            reparsed.shape_properties.transform,
            picture.shape_properties.transform
        );
    }
}

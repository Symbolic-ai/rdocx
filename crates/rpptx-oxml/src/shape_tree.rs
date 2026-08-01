use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::namespace::A_NS;
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::xfrm::CT_Transform2D;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::graphic_frame::CT_GraphicFrame;
use crate::namespace::{
    FIXED_SHAPE_TREE_PREFIXES, MC_NS, NamespaceBindings, P_NS, R_NS, root_attributes,
    self_contained_attributes,
};
use crate::picture::CT_Picture;
use crate::placeholder::CT_Placeholder;

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;

/// One shape-tree child in document and z-order.
// The public PresentationML model intentionally stores CT_Picture directly.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeTreeChild {
    Shape(CT_Shape),
    Picture(CT_Picture),
    GraphicFrame(Box<CT_GraphicFrame>),
    GroupShape(Box<CT_GroupShape>),
    Connector(Vec<u8>),
    AlternateContent(Vec<u8>),
}

impl ShapeTreeChild {
    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        match self {
            Self::Shape(shape) => shape.write_xml_internal(writer, false)?,
            Self::Picture(picture) => picture.write_xml_internal(writer, false)?,
            Self::GraphicFrame(frame) => frame.write_xml_internal(writer, false)?,
            Self::Connector(xml) | Self::AlternateContent(xml) => {
                writer.get_mut().write_all(xml)?
            }
            Self::GroupShape(group) => group.write_xml_internal(writer, false)?,
        }
        Ok(())
    }
}

/// A partial typed `p:sp` model that owns its placeholder identity.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_Shape {
    pub placeholder: Option<CT_Placeholder>,
    raw: Box<ShapeRaw>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ShapeRaw {
    raw_attributes: RawAttributes,
    non_visual_attributes: RawAttributes,
    non_visual_children: OrderedRawChildren,
    non_visual_drawing_properties: Vec<u8>,
    non_visual_shape_properties: Vec<u8>,
    application_properties_attributes: RawAttributes,
    application_properties_raw_children: OrderedRawChildren,
    shape_properties: Vec<u8>,
    raw_children: OrderedRawChildren,
}

/// The recursive group-shape form used inside a shape tree.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_GroupShape {
    pub children: Vec<ShapeTreeChild>,
    non_visual_group_properties: NonVisualGroupProperties,
    group_properties: GroupProperties,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

/// The ordered `p:spTree` root shared by slides, layouts, and masters.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_ShapeTree {
    pub children: Vec<ShapeTreeChild>,
    non_visual_group_properties: NonVisualGroupProperties,
    group_properties: GroupProperties,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GroupProperties {
    transform: Option<CT_Transform2D>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NonVisualGroupProperties {
    raw_attributes: RawAttributes,
    raw_children: Vec<Vec<u8>>,
}

#[derive(Clone, Copy)]
enum GroupKind {
    ShapeTree,
    GroupShape,
}

impl GroupKind {
    const fn local_name(self) -> &'static [u8] {
        match self {
            Self::ShapeTree => b"spTree",
            Self::GroupShape => b"grpSp",
        }
    }

    const fn tag(self) -> &'static str {
        match self {
            Self::ShapeTree => "p:spTree",
            Self::GroupShape => "p:grpSp",
        }
    }
}

struct ParsedGroup {
    non_visual_group_properties: NonVisualGroupProperties,
    group_properties: GroupProperties,
    children: Vec<ShapeTreeChild>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

impl CT_Shape {
    /// Parses a complete `p:sp` with any prefix bound to PresentationML.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_fragment(xml, &[])
    }

    fn from_fragment(xml: &[u8], inherited: &[(String, String)]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let namespaces =
                        NamespaceBindings::from_entries(inherited).with_start(&start)?;
                    if local_name(start.name().as_ref()) != b"sp"
                        || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                    {
                        return Err(unexpected(&start));
                    }
                    namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
                    return Self::from_reader(&mut reader, &start, &namespaces, inherited);
                }
                Event::Empty(start) => {
                    return Err(OxmlError::MissingElement(format!(
                        "{} requires p:nvSpPr and p:spPr",
                        String::from_utf8_lossy(start.name().as_ref())
                    )));
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:sp".to_owned())),
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
        let mut shape_properties = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0usize;
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) => {
                    let child_namespaces = namespaces.with_start(&child)?;
                    let name = local_name(child.name().as_ref()).to_vec();
                    let uri = child_namespaces.element_uri(child.name().as_ref());
                    let raw = capture_element(reader, &child)?;
                    capture_shape_child(
                        &name,
                        uri,
                        raw,
                        &child_namespaces,
                        &mut non_visual,
                        &mut shape_properties,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::Empty(child) => {
                    let child_namespaces = namespaces.with_start(&child)?;
                    let name = local_name(child.name().as_ref()).to_vec();
                    let uri = child_namespaces.element_uri(child.name().as_ref());
                    let raw = capture_empty_element(&child)?;
                    capture_shape_child(
                        &name,
                        uri,
                        raw,
                        &child_namespaces,
                        &mut non_visual,
                        &mut shape_properties,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::End(end) if local_name(end.name().as_ref()) == b"sp" => break,
                Event::Eof => return Err(OxmlError::MissingElement("closing p:sp".to_owned())),
                _ => {}
            }
            buffer.clear();
        }

        let non_visual = required(non_visual, "p:nvSpPr")?;
        Ok(Self {
            placeholder: non_visual.placeholder,
            raw: Box::new(ShapeRaw {
                raw_attributes: self_contained_attributes(
                    start,
                    FIXED_SHAPE_TREE_PREFIXES,
                    inherited,
                )?,
                non_visual_attributes: non_visual.raw_attributes,
                non_visual_children: non_visual.raw_children,
                non_visual_drawing_properties: non_visual.non_visual_drawing_properties,
                non_visual_shape_properties: non_visual.non_visual_shape_properties,
                application_properties_attributes: non_visual.application_properties_attributes,
                application_properties_raw_children: non_visual.application_properties_raw_children,
                shape_properties: required(shape_properties, "p:spPr")?,
                raw_children,
            }),
        })
    }

    /// Serialises a self-contained shape with fixed modelled prefixes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml_internal(&mut writer, true)?;
        Ok(writer.into_inner())
    }

    fn write_xml_internal<W: Write>(
        &self,
        writer: &mut Writer<W>,
        declare_namespaces: bool,
    ) -> Result<()> {
        let mut start = BytesStart::new("p:sp");
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
        writer.get_mut().write_all(&self.raw.shape_properties)?;
        emit_raw(writer, self.raw.raw_children.at(2))?;
        writer.write_event(Event::End(BytesEnd::new("p:sp")))?;
        Ok(())
    }

    fn write_non_visual_properties<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("p:nvSpPr");
        push_attributes(&mut start, &self.raw.non_visual_attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw.non_visual_children.at(0))?;
        writer
            .get_mut()
            .write_all(&self.raw.non_visual_drawing_properties)?;
        emit_raw(writer, self.raw.non_visual_children.at(1))?;
        writer
            .get_mut()
            .write_all(&self.raw.non_visual_shape_properties)?;
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
        writer.write_event(Event::End(BytesEnd::new("p:nvSpPr")))?;
        Ok(())
    }
}

struct ParsedNonVisualShape {
    placeholder: Option<CT_Placeholder>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    non_visual_drawing_properties: Vec<u8>,
    non_visual_shape_properties: Vec<u8>,
    application_properties_attributes: RawAttributes,
    application_properties_raw_children: OrderedRawChildren,
}

#[allow(clippy::too_many_arguments)]
fn capture_shape_child(
    name: &[u8],
    uri: Option<&str>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    non_visual: &mut Option<ParsedNonVisualShape>,
    shape_properties: &mut Option<Vec<u8>>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if uri == Some(P_NS) && matches!(name, b"nvSpPr" | b"spPr") {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
    }
    match (uri, name) {
        (Some(P_NS), b"nvSpPr") => {
            if *boundary != 0 || non_visual.is_some() {
                return Err(OxmlError::InvalidValue(
                    "p:nvSpPr must be the first modelled p:sp child".to_owned(),
                ));
            }
            *non_visual = Some(parse_non_visual_shape(&raw, namespaces)?);
            *boundary = 1;
        }
        (Some(P_NS), b"spPr") => {
            if *boundary != 1 || shape_properties.is_some() {
                return Err(OxmlError::InvalidValue(
                    "p:spPr must immediately follow p:nvSpPr".to_owned(),
                ));
            }
            *shape_properties = Some(raw);
            *boundary = 2;
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

fn parse_non_visual_shape(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<ParsedNonVisualShape> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = inherited.with_start(&start)?;
                let raw_attributes = root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?;
                let mut drawing = None;
                let mut shape = None;
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
                            capture_non_visual_shape_child(
                                &name,
                                uri,
                                raw,
                                &child_namespaces,
                                &mut drawing,
                                &mut shape,
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
                            capture_non_visual_shape_child(
                                &name,
                                uri,
                                raw,
                                &child_namespaces,
                                &mut drawing,
                                &mut shape,
                                &mut application,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        Event::End(end) if local_name(end.name().as_ref()) == b"nvSpPr" => break,
                        Event::Eof => {
                            return Err(OxmlError::MissingElement("closing p:nvSpPr".to_owned()));
                        }
                        _ => {}
                    }
                }
                let (placeholder, app_attributes, app_raw) = required(application, "p:nvPr")?;
                return Ok(ParsedNonVisualShape {
                    placeholder,
                    raw_attributes,
                    raw_children,
                    non_visual_drawing_properties: required(drawing, "p:cNvPr")?,
                    non_visual_shape_properties: required(shape, "p:cNvSpPr")?,
                    application_properties_attributes: app_attributes,
                    application_properties_raw_children: app_raw,
                });
            }
            Event::Empty(_) => {
                return Err(OxmlError::MissingElement(
                    "p:nvSpPr requires p:cNvPr, p:cNvSpPr, and p:nvPr".to_owned(),
                ));
            }
            Event::Eof => return Err(OxmlError::MissingElement("p:nvSpPr".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

type ParsedApplicationProperties = (Option<CT_Placeholder>, RawAttributes, OrderedRawChildren);

#[allow(clippy::too_many_arguments)]
fn capture_non_visual_shape_child(
    name: &[u8],
    uri: Option<&str>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    drawing: &mut Option<Vec<u8>>,
    shape: &mut Option<Vec<u8>>,
    application: &mut Option<ParsedApplicationProperties>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if uri == Some(P_NS) && matches!(name, b"cNvPr" | b"cNvSpPr" | b"nvPr") {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
    }
    match (uri, name) {
        (Some(P_NS), b"cNvPr") if *boundary == 0 && drawing.is_none() => {
            *drawing = Some(raw);
            *boundary = 1;
        }
        (Some(P_NS), b"cNvSpPr") if *boundary == 1 && shape.is_none() => {
            *shape = Some(raw);
            *boundary = 2;
        }
        (Some(P_NS), b"nvPr") if *boundary == 2 && application.is_none() => {
            *application = Some(parse_application_properties(&raw, namespaces)?);
            *boundary = 3;
        }
        (Some(P_NS), b"cNvPr" | b"cNvSpPr" | b"nvPr") => {
            return Err(OxmlError::InvalidValue(
                "p:nvSpPr children must be p:cNvPr, p:cNvSpPr, and p:nvPr in order".to_owned(),
            ));
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

fn parse_application_properties(
    xml: &[u8],
    inherited: &NamespaceBindings,
) -> Result<ParsedApplicationProperties> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = inherited.with_start(&start)?;
                let raw_attributes = root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?;
                let mut placeholder = None;
                let mut raw_children = OrderedRawChildren::default();
                let mut boundary = 0usize;
                loop {
                    buffer.clear();
                    match reader.read_event_into(&mut buffer)? {
                        Event::Start(child) => {
                            let child_namespaces = namespaces.with_start(&child)?;
                            let is_placeholder =
                                child_namespaces.element_uri(child.name().as_ref()) == Some(P_NS)
                                    && local_name(child.name().as_ref()) == b"ph";
                            let raw = capture_element(&mut reader, &child)?;
                            capture_application_child(
                                is_placeholder,
                                raw,
                                &child_namespaces,
                                &mut placeholder,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        Event::Empty(child) => {
                            let child_namespaces = namespaces.with_start(&child)?;
                            let is_placeholder =
                                child_namespaces.element_uri(child.name().as_ref()) == Some(P_NS)
                                    && local_name(child.name().as_ref()) == b"ph";
                            let raw = capture_empty_element(&child)?;
                            capture_application_child(
                                is_placeholder,
                                raw,
                                &child_namespaces,
                                &mut placeholder,
                                &mut raw_children,
                                &mut boundary,
                            )?;
                        }
                        Event::End(end) if local_name(end.name().as_ref()) == b"nvPr" => break,
                        Event::Eof => {
                            return Err(OxmlError::MissingElement("closing p:nvPr".to_owned()));
                        }
                        _ => {}
                    }
                }
                return Ok((placeholder, raw_attributes, raw_children));
            }
            Event::Empty(start) => {
                return Ok((
                    None,
                    root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?,
                    OrderedRawChildren::default(),
                ));
            }
            Event::Eof => return Err(OxmlError::MissingElement("p:nvPr".to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn capture_application_child(
    is_placeholder: bool,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    placeholder: &mut Option<CT_Placeholder>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if is_placeholder {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
        if placeholder.is_some() {
            return Err(OxmlError::InvalidValue("duplicate p:ph".to_owned()));
        }
        *placeholder = Some(CT_Placeholder::from_fragment(&raw, &namespaces.entries())?);
        *boundary = 1;
    } else {
        raw_children.push(*boundary, raw);
    }
    Ok(())
}

impl CT_ShapeTree {
    /// Parses a complete `p:spTree` with any prefix bound to PresentationML.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        Self::from_fragment(xml, &[])
    }

    pub(crate) fn from_fragment(xml: &[u8], inherited: &[(String, String)]) -> Result<Self> {
        let parsed = parse_group(xml, inherited, GroupKind::ShapeTree)?;
        Ok(Self {
            non_visual_group_properties: parsed.non_visual_group_properties,
            children: parsed.children,
            group_properties: parsed.group_properties,
            raw_attributes: parsed.raw_attributes,
            raw_children: parsed.raw_children,
        })
    }

    /// Returns the typed DrawingML group transform when `p:grpSpPr` has one.
    pub fn group_transform(&self) -> Option<&CT_Transform2D> {
        self.group_properties.transform.as_ref()
    }

    /// Serialises a self-contained shape-tree fragment with fixed prefixes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    pub(crate) fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        write_group(
            writer,
            GroupKind::ShapeTree,
            &self.non_visual_group_properties,
            &self.group_properties,
            &self.children,
            &self.raw_attributes,
            &self.raw_children,
            true,
        )
    }
}

impl CT_GroupShape {
    /// Parses a complete recursive `p:grpSp` element.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let parsed = parse_group(xml, &[], GroupKind::GroupShape)?;
        Ok(Self::from_parsed(parsed))
    }

    fn from_fragment(xml: &[u8], inherited: &[(String, String)]) -> Result<Self> {
        Ok(Self::from_parsed(parse_group(
            xml,
            inherited,
            GroupKind::GroupShape,
        )?))
    }

    fn from_parsed(parsed: ParsedGroup) -> Self {
        Self {
            non_visual_group_properties: parsed.non_visual_group_properties,
            children: parsed.children,
            group_properties: parsed.group_properties,
            raw_attributes: parsed.raw_attributes,
            raw_children: parsed.raw_children,
        }
    }

    /// Returns the typed DrawingML group transform when `p:grpSpPr` has one.
    pub fn group_transform(&self) -> Option<&CT_Transform2D> {
        self.group_properties.transform.as_ref()
    }

    /// Serialises a self-contained group-shape fragment with fixed prefixes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml_internal(&mut writer, true)?;
        Ok(writer.into_inner())
    }

    fn write_xml_internal<W: Write>(
        &self,
        writer: &mut Writer<W>,
        declare_namespaces: bool,
    ) -> Result<()> {
        write_group(
            writer,
            GroupKind::GroupShape,
            &self.non_visual_group_properties,
            &self.group_properties,
            &self.children,
            &self.raw_attributes,
            &self.raw_children,
            declare_namespaces,
        )
    }
}

fn parse_group(xml: &[u8], inherited: &[(String, String)], kind: GroupKind) -> Result<ParsedGroup> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = NamespaceBindings::from_entries(inherited).with_start(&start)?;
                if local_name(start.name().as_ref()) != kind.local_name()
                    || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                {
                    return Err(unexpected(&start));
                }
                namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
                return parse_group_children(&mut reader, &start, &namespaces, kind);
            }
            Event::Empty(start) => {
                let namespaces = NamespaceBindings::from_entries(inherited).with_start(&start)?;
                if local_name(start.name().as_ref()) != kind.local_name()
                    || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                {
                    return Err(unexpected(&start));
                }
                return Err(OxmlError::MissingElement(format!(
                    "{} requires p:nvGrpSpPr and p:grpSpPr",
                    kind.tag()
                )));
            }
            Event::Eof => return Err(OxmlError::MissingElement(kind.tag().to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_group_children(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    namespaces: &NamespaceBindings,
    kind: GroupKind,
) -> Result<ParsedGroup> {
    let raw_attributes = root_attributes(start, FIXED_SHAPE_TREE_PREFIXES)?;
    let mut non_visual = None;
    let mut group_properties = None;
    let mut children = Vec::new();
    let mut raw_children = OrderedRawChildren::default();
    let mut state = 0usize;
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(child) => {
                let child_namespaces = namespaces.with_start(&child)?;
                let name = local_name(child.name().as_ref()).to_vec();
                let uri = child_namespaces.element_uri(child.name().as_ref());
                let raw = capture_element(reader, &child)?;
                capture_group_child(
                    &name,
                    uri,
                    raw,
                    &child_namespaces,
                    &mut non_visual,
                    &mut group_properties,
                    &mut children,
                    &mut raw_children,
                    &mut state,
                )?;
            }
            Event::Empty(child) => {
                let child_namespaces = namespaces.with_start(&child)?;
                let name = local_name(child.name().as_ref()).to_vec();
                let uri = child_namespaces.element_uri(child.name().as_ref());
                let raw = capture_empty_element(&child)?;
                capture_group_child(
                    &name,
                    uri,
                    raw,
                    &child_namespaces,
                    &mut non_visual,
                    &mut group_properties,
                    &mut children,
                    &mut raw_children,
                    &mut state,
                )?;
            }
            Event::End(end) if local_name(end.name().as_ref()) == kind.local_name() => break,
            Event::Eof => {
                return Err(OxmlError::MissingElement(format!("closing {}", kind.tag())));
            }
            _ => {}
        }
        buffer.clear();
    }
    Ok(ParsedGroup {
        non_visual_group_properties: required(non_visual, "p:nvGrpSpPr")?,
        group_properties: required(group_properties, "p:grpSpPr")?,
        children,
        raw_attributes,
        raw_children,
    })
}

#[allow(clippy::too_many_arguments)]
fn capture_group_child(
    name: &[u8],
    uri: Option<&str>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    non_visual: &mut Option<NonVisualGroupProperties>,
    group_properties: &mut Option<GroupProperties>,
    children: &mut Vec<ShapeTreeChild>,
    raw_children: &mut OrderedRawChildren,
    state: &mut usize,
) -> Result<()> {
    if uri == Some(P_NS) && matches!(name, b"nvGrpSpPr" | b"grpSpPr" | b"grpSp") {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
    }
    if *state == 0 {
        if uri == Some(P_NS) && name == b"nvGrpSpPr" {
            *non_visual = Some(NonVisualGroupProperties::from_fragment(&raw)?);
            *state = 1;
            return Ok(());
        }
        return Err(OxmlError::InvalidValue(
            "p:nvGrpSpPr must be the first shape-tree element".to_owned(),
        ));
    }
    if *state == 1 {
        if uri == Some(P_NS) && name == b"grpSpPr" {
            *group_properties = Some(GroupProperties::from_fragment(&raw, namespaces)?);
            *state = 2;
            return Ok(());
        }
        return Err(OxmlError::InvalidValue(
            "p:grpSpPr must immediately follow p:nvGrpSpPr".to_owned(),
        ));
    }

    if uri == Some(P_NS) && matches!(name, b"nvGrpSpPr" | b"grpSpPr") {
        return Err(OxmlError::InvalidValue(format!(
            "duplicate p:{} shape-tree element",
            String::from_utf8_lossy(name)
        )));
    }

    match (uri, name) {
        (Some(P_NS), b"sp") => children.push(ShapeTreeChild::Shape(CT_Shape::from_fragment(
            &raw,
            &namespaces.entries(),
        )?)),
        (Some(P_NS), b"pic") => children.push(ShapeTreeChild::Picture(CT_Picture::from_fragment(
            &raw,
            &namespaces.entries(),
        )?)),
        (Some(P_NS), b"graphicFrame") => {
            children.push(ShapeTreeChild::GraphicFrame(Box::new(
                CT_GraphicFrame::from_fragment(&raw, &namespaces.entries())?,
            )));
        }
        (Some(P_NS), b"grpSp") => children.push(ShapeTreeChild::GroupShape(Box::new(
            CT_GroupShape::from_fragment(&raw, &namespaces.entries())?,
        ))),
        (Some(P_NS), b"cxnSp") => children.push(ShapeTreeChild::Connector(raw)),
        (Some(MC_NS), b"AlternateContent") => {
            children.push(ShapeTreeChild::AlternateContent(raw));
        }
        _ => raw_children.push(children.len(), raw),
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn write_group<W: Write>(
    writer: &mut Writer<W>,
    kind: GroupKind,
    non_visual: &NonVisualGroupProperties,
    group_properties: &GroupProperties,
    children: &[ShapeTreeChild],
    attributes: &RawAttributes,
    raw_children: &OrderedRawChildren,
    declare_namespaces: bool,
) -> Result<()> {
    let mut start = BytesStart::new(kind.tag());
    if declare_namespaces {
        start.push_attribute(("xmlns:p", P_NS));
        start.push_attribute(("xmlns:a", A_NS));
        start.push_attribute(("xmlns:r", R_NS));
        start.push_attribute(("xmlns:mc", MC_NS));
    }
    push_attributes(&mut start, attributes);
    writer.write_event(Event::Start(start))?;
    non_visual.write_xml(writer)?;
    group_properties.write_xml(writer)?;
    for (index, child) in children.iter().enumerate() {
        emit_raw(writer, raw_children.at(index))?;
        child.write_xml(writer)?;
    }
    emit_raw(writer, raw_children.at(children.len()))?;
    writer.write_event(Event::End(BytesEnd::new(kind.tag())))?;
    Ok(())
}

impl NonVisualGroupProperties {
    fn from_fragment(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let raw_attributes = root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?;
                    let mut raw_children = Vec::new();
                    loop {
                        buffer.clear();
                        match reader.read_event_into(&mut buffer)? {
                            Event::Start(child) => {
                                raw_children.push(capture_element(&mut reader, &child)?);
                            }
                            Event::Empty(child) => {
                                raw_children.push(capture_empty_element(&child)?);
                            }
                            Event::End(end) if local_name(end.name().as_ref()) == b"nvGrpSpPr" => {
                                return Ok(Self {
                                    raw_attributes,
                                    raw_children,
                                });
                            }
                            Event::Eof => {
                                return Err(OxmlError::MissingElement(
                                    "closing p:nvGrpSpPr".to_owned(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                Event::Empty(start) => {
                    return Ok(Self {
                        raw_attributes: root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?,
                        raw_children: Vec::new(),
                    });
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:nvGrpSpPr".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("p:nvGrpSpPr");
        push_attributes(&mut start, &self.raw_attributes);
        if self.raw_children.is_empty() {
            writer.write_event(Event::Empty(start))?;
            return Ok(());
        }
        writer.write_event(Event::Start(start))?;
        for child in &self.raw_children {
            writer.get_mut().write_all(child)?;
        }
        writer.write_event(Event::End(BytesEnd::new("p:nvGrpSpPr")))?;
        Ok(())
    }
}

impl GroupProperties {
    fn from_fragment(xml: &[u8], inherited: &NamespaceBindings) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let namespaces = inherited.with_start(&start)?;
                    return Self::from_reader(&mut reader, &start, &namespaces);
                }
                Event::Empty(start) => {
                    return Ok(Self {
                        transform: None,
                        raw_attributes: root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?,
                        raw_children: OrderedRawChildren::default(),
                    });
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:grpSpPr".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_reader(
        reader: &mut Reader<&[u8]>,
        start: &BytesStart<'_>,
        namespaces: &NamespaceBindings,
    ) -> Result<Self> {
        let mut transform = None;
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0usize;
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(child) => {
                    let child_namespaces = namespaces.with_start(&child)?;
                    let is_transform = child_namespaces.element_uri(child.name().as_ref())
                        == Some(A_NS)
                        && local_name(child.name().as_ref()) == b"xfrm";
                    if is_transform {
                        child_namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
                    }
                    let raw = capture_element(reader, &child)?;
                    capture_group_property_child(
                        is_transform,
                        raw,
                        &mut transform,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::Empty(child) => {
                    let child_namespaces = namespaces.with_start(&child)?;
                    let is_transform = child_namespaces.element_uri(child.name().as_ref())
                        == Some(A_NS)
                        && local_name(child.name().as_ref()) == b"xfrm";
                    if is_transform {
                        child_namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
                    }
                    let raw = capture_empty_element(&child)?;
                    capture_group_property_child(
                        is_transform,
                        raw,
                        &mut transform,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::End(end) if local_name(end.name().as_ref()) == b"grpSpPr" => break,
                Event::Eof => {
                    return Err(OxmlError::MissingElement("closing p:grpSpPr".to_owned()));
                }
                _ => {}
            }
            buffer.clear();
        }
        Ok(Self {
            transform,
            raw_attributes: root_attributes(start, FIXED_SHAPE_TREE_PREFIXES)?,
            raw_children,
        })
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut start = BytesStart::new("p:grpSpPr");
        push_attributes(&mut start, &self.raw_attributes);
        if self.transform.is_none() && self.raw_children.is_empty() {
            writer.write_event(Event::Empty(start))?;
            return Ok(());
        }
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(transform) = &self.transform {
            transform
                .write_xml(writer)
                .map_err(|error| OxmlError::InvalidValue(error.to_string()))?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        writer.write_event(Event::End(BytesEnd::new("p:grpSpPr")))?;
        Ok(())
    }
}

fn capture_group_property_child(
    is_transform: bool,
    raw: Vec<u8>,
    transform: &mut Option<CT_Transform2D>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if is_transform {
        if transform.is_some() {
            return Err(OxmlError::InvalidValue(
                "duplicate DrawingML group transform".to_owned(),
            ));
        }
        if !raw_children.is_empty() {
            return Err(OxmlError::InvalidValue(
                "a:xfrm must precede other p:grpSpPr children".to_owned(),
            ));
        }
        *transform = Some(
            CT_Transform2D::from_xml(&raw)
                .map_err(|error| OxmlError::InvalidValue(error.to_string()))?,
        );
        *boundary = 1;
    } else {
        raw_children.push(*boundary, raw);
    }
    Ok(())
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

fn required<T>(value: Option<T>, name: &str) -> Result<T> {
    value.ok_or_else(|| OxmlError::MissingElement(name.to_owned()))
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn unexpected(element: &BytesStart<'_>) -> OxmlError {
    OxmlError::UnexpectedElement(String::from_utf8_lossy(element.name().as_ref()).into_owned())
}

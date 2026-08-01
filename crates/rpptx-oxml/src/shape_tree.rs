use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::namespace::A_NS;
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::xfrm::CT_Transform2D;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::namespace::{
    FIXED_SHAPE_TREE_PREFIXES, MC_NS, NamespaceBindings, P_NS, R_NS, root_attributes,
};

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;

/// One shape-tree child in document and z-order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShapeTreeChild {
    Shape(Vec<u8>),
    Picture(Vec<u8>),
    GraphicFrame(Vec<u8>),
    GroupShape(Box<CT_GroupShape>),
    Connector(Vec<u8>),
    AlternateContent(Vec<u8>),
}

impl ShapeTreeChild {
    fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        match self {
            Self::Shape(xml)
            | Self::Picture(xml)
            | Self::GraphicFrame(xml)
            | Self::Connector(xml)
            | Self::AlternateContent(xml) => writer.get_mut().write_all(xml)?,
            Self::GroupShape(group) => group.write_xml_internal(writer, false)?,
        }
        Ok(())
    }
}

/// The recursive group-shape form used inside a shape tree.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_GroupShape {
    pub children: Vec<ShapeTreeChild>,
    non_visual_group_properties: NonVisualGroupProperties,
    group_properties: GroupProperties,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

/// The ordered `p:spTree` root shared by slides, layouts, and masters.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
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
        (Some(P_NS), b"sp") => children.push(ShapeTreeChild::Shape(raw)),
        (Some(P_NS), b"pic") => children.push(ShapeTreeChild::Picture(raw)),
        (Some(P_NS), b"graphicFrame") => {
            children.push(ShapeTreeChild::GraphicFrame(raw));
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

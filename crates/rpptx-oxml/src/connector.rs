use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::namespace::A_NS;
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::shape_props::CT_ShapeProperties;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::namespace::{
    FIXED_SHAPE_TREE_PREFIXES, MC_NS, NamespaceBindings, P_NS, R_NS, all_attributes,
    root_attributes, self_contained_attributes,
};

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;

/// One optional connector endpoint in `p:cNvCxnSpPr`.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_Connection {
    pub id: u32,
    pub idx: u32,
    raw_attributes: RawAttributes,
    raw_content: Vec<u8>,
}

/// The typed endpoint and shape-property subset of one `p:cxnSp`.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_ConnectionShape {
    pub start_connection: Option<CT_Connection>,
    pub end_connection: Option<CT_Connection>,
    pub shape_properties: CT_ShapeProperties,
    raw: Box<ConnectionShapeRaw>,
}

#[derive(Clone, Debug, PartialEq)]
struct ConnectionShapeRaw {
    raw_attributes: RawAttributes,
    non_visual: NonVisualConnectionShape,
    style: Option<Vec<u8>>,
    extension_list: Option<Vec<u8>>,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NonVisualConnectionShape {
    raw_attributes: RawAttributes,
    drawing_properties: RawElement,
    connector_properties: NonVisualConnectorProperties,
    application_properties: RawElement,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RawElement {
    raw_attributes: RawAttributes,
    raw_content: Vec<u8>,
    was_empty: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NonVisualConnectorProperties {
    raw_attributes: RawAttributes,
    locks: Option<Vec<u8>>,
    extension_list: Option<Vec<u8>>,
    raw_children: OrderedRawChildren,
}

impl CT_ConnectionShape {
    pub(crate) fn non_visual_id(&self) -> Option<u32> {
        self.raw
            .non_visual
            .drawing_properties
            .raw_attributes
            .iter()
            .find(|(name, _)| name == "id")
            .and_then(|(_, value)| value.parse().ok())
    }

    /// Changes the producer-facing non-visual connector name.
    pub fn set_name(&mut self, name: &str) -> Result<()> {
        let attributes = &mut self.raw.non_visual.drawing_properties.raw_attributes;
        if let Some((_, value)) = attributes
            .iter_mut()
            .find(|(attribute, _)| attribute == "name")
        {
            *value = name.to_owned();
        } else {
            attributes.push(("name".to_owned(), name.to_owned()));
        }
        Ok(())
    }

    /// Parses a complete `p:cxnSp` with any prefix bound to PresentationML.
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
                    if local_name(start.name().as_ref()) != b"cxnSp"
                        || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                    {
                        return Err(unexpected(&start));
                    }
                    namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
                    return Self::from_reader(&mut reader, &start, &namespaces, inherited);
                }
                Event::Empty(start) => {
                    return Err(OxmlError::MissingElement(format!(
                        "{} requires p:nvCxnSpPr and p:spPr",
                        String::from_utf8_lossy(start.name().as_ref())
                    )));
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:cxnSp".to_owned())),
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
        let mut style = None;
        let mut extension_list = None;
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
                    capture_root_child(
                        &name,
                        uri,
                        raw,
                        &child_namespaces,
                        &mut non_visual,
                        &mut shape_properties,
                        &mut style,
                        &mut extension_list,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::Empty(child) => {
                    let child_namespaces = namespaces.with_start(&child)?;
                    let name = local_name(child.name().as_ref()).to_vec();
                    let uri = child_namespaces.element_uri(child.name().as_ref());
                    let raw = capture_empty_element(&child)?;
                    capture_root_child(
                        &name,
                        uri,
                        raw,
                        &child_namespaces,
                        &mut non_visual,
                        &mut shape_properties,
                        &mut style,
                        &mut extension_list,
                        &mut raw_children,
                        &mut boundary,
                    )?;
                }
                Event::End(end) if local_name(end.name().as_ref()) == b"cxnSp" => break,
                Event::Eof => {
                    return Err(OxmlError::MissingElement("closing p:cxnSp".to_owned()));
                }
                _ => {}
            }
            buffer.clear();
        }

        let (start_connection, end_connection, connector_properties) =
            required(non_visual, "p:nvCxnSpPr")?.into_parts();
        Ok(Self {
            start_connection,
            end_connection,
            shape_properties: required(shape_properties, "p:spPr")?,
            raw: Box::new(ConnectionShapeRaw {
                raw_attributes: self_contained_attributes(
                    start,
                    FIXED_SHAPE_TREE_PREFIXES,
                    inherited,
                )?,
                non_visual: connector_properties,
                style,
                extension_list,
                raw_children,
            }),
        })
    }

    /// Serialises a self-contained connector with fixed modelled prefixes.
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
        let mut start = BytesStart::new("p:cxnSp");
        if declare_namespaces {
            start.push_attribute(("xmlns:p", P_NS));
            start.push_attribute(("xmlns:a", A_NS));
            start.push_attribute(("xmlns:r", R_NS));
            start.push_attribute(("xmlns:mc", MC_NS));
        }
        push_attributes(&mut start, &self.raw.raw_attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw.raw_children.at(0))?;
        self.raw.non_visual.write_xml(
            writer,
            self.start_connection.as_ref(),
            self.end_connection.as_ref(),
        )?;
        emit_raw(writer, self.raw.raw_children.at(1))?;
        self.shape_properties
            .write_xml_as(writer, "p:spPr")
            .map_err(drawing_error)?;
        emit_raw(writer, self.raw.raw_children.at(2))?;
        if let Some(style) = &self.raw.style {
            writer.get_mut().write_all(style)?;
        }
        emit_raw(writer, self.raw.raw_children.at(3))?;
        if let Some(extension_list) = &self.raw.extension_list {
            writer.get_mut().write_all(extension_list)?;
        }
        emit_raw(writer, self.raw.raw_children.at(4))?;
        writer.write_event(Event::End(BytesEnd::new("p:cxnSp")))?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_root_child(
    name: &[u8],
    uri: Option<&str>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    non_visual: &mut Option<ParsedNonVisualConnectionShape>,
    shape_properties: &mut Option<CT_ShapeProperties>,
    style: &mut Option<Vec<u8>>,
    extension_list: &mut Option<Vec<u8>>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if uri == Some(P_NS) && matches!(name, b"nvCxnSpPr" | b"spPr" | b"style" | b"extLst") {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
    }
    match (uri, name) {
        (Some(P_NS), b"nvCxnSpPr") if *boundary == 0 && non_visual.is_none() => {
            *non_visual = Some(NonVisualConnectionShape::from_fragment(&raw, namespaces)?);
            *boundary = 1;
        }
        (Some(P_NS), b"spPr") if *boundary == 1 && shape_properties.is_none() => {
            *shape_properties = Some(CT_ShapeProperties::from_xml(&raw).map_err(drawing_error)?);
            *boundary = 2;
        }
        (Some(P_NS), b"style") if *boundary == 2 && style.is_none() => {
            *style = Some(raw);
            *boundary = 3;
        }
        (Some(P_NS), b"extLst") if matches!(*boundary, 2 | 3) && extension_list.is_none() => {
            *extension_list = Some(raw);
            *boundary = 4;
        }
        (Some(P_NS), b"nvCxnSpPr" | b"spPr" | b"style" | b"extLst") => {
            return Err(OxmlError::InvalidValue(
                "p:cxnSp children must be p:nvCxnSpPr, p:spPr, optional p:style, and optional p:extLst in order"
                    .to_owned(),
            ));
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

struct ParsedNonVisualConnectionShape {
    start_connection: Option<CT_Connection>,
    end_connection: Option<CT_Connection>,
    raw: NonVisualConnectionShape,
}

impl ParsedNonVisualConnectionShape {
    fn into_parts(
        self,
    ) -> (
        Option<CT_Connection>,
        Option<CT_Connection>,
        NonVisualConnectionShape,
    ) {
        (self.start_connection, self.end_connection, self.raw)
    }
}

impl NonVisualConnectionShape {
    fn from_fragment(
        xml: &[u8],
        inherited: &NamespaceBindings,
    ) -> Result<ParsedNonVisualConnectionShape> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let start = start.into_owned();
                    let namespaces = inherited.with_start(&start)?;
                    let mut drawing_properties = None;
                    let mut connector_properties = None;
                    let mut application_properties = None;
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
                                    &mut drawing_properties,
                                    &mut connector_properties,
                                    &mut application_properties,
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
                                    &mut drawing_properties,
                                    &mut connector_properties,
                                    &mut application_properties,
                                    &mut raw_children,
                                    &mut boundary,
                                )?;
                            }
                            Event::End(end) if local_name(end.name().as_ref()) == b"nvCxnSpPr" => {
                                let (start_connection, end_connection, connector_properties) =
                                    required(connector_properties, "p:cNvCxnSpPr")?.into_parts();
                                return Ok(ParsedNonVisualConnectionShape {
                                    start_connection,
                                    end_connection,
                                    raw: Self {
                                        raw_attributes: root_attributes(
                                            &start,
                                            FIXED_SHAPE_TREE_PREFIXES,
                                        )?,
                                        drawing_properties: required(
                                            drawing_properties,
                                            "p:cNvPr",
                                        )?,
                                        connector_properties,
                                        application_properties: required(
                                            application_properties,
                                            "p:nvPr",
                                        )?,
                                        raw_children,
                                    },
                                });
                            }
                            Event::Eof => {
                                return Err(OxmlError::MissingElement(
                                    "closing p:nvCxnSpPr".to_owned(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                Event::Empty(_) => {
                    return Err(OxmlError::MissingElement(
                        "p:nvCxnSpPr requires p:cNvPr, p:cNvCxnSpPr, and p:nvPr".to_owned(),
                    ));
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:nvCxnSpPr".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn write_xml<W: Write>(
        &self,
        writer: &mut Writer<W>,
        start_connection: Option<&CT_Connection>,
        end_connection: Option<&CT_Connection>,
    ) -> Result<()> {
        let mut start = BytesStart::new("p:nvCxnSpPr");
        push_attributes(&mut start, &self.raw_attributes);
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw_children.at(0))?;
        self.drawing_properties.write_xml(writer, "p:cNvPr")?;
        emit_raw(writer, self.raw_children.at(1))?;
        self.connector_properties
            .write_xml(writer, start_connection, end_connection)?;
        emit_raw(writer, self.raw_children.at(2))?;
        self.application_properties.write_xml(writer, "p:nvPr")?;
        emit_raw(writer, self.raw_children.at(3))?;
        writer.write_event(Event::End(BytesEnd::new("p:nvCxnSpPr")))?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_non_visual_child(
    name: &[u8],
    uri: Option<&str>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    drawing_properties: &mut Option<RawElement>,
    connector_properties: &mut Option<ParsedNonVisualConnectorProperties>,
    application_properties: &mut Option<RawElement>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if uri == Some(P_NS) && matches!(name, b"cNvPr" | b"cNvCxnSpPr" | b"nvPr") {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
    }
    match (uri, name) {
        (Some(P_NS), b"cNvPr") if *boundary == 0 && drawing_properties.is_none() => {
            *drawing_properties = Some(RawElement::from_fragment(&raw)?);
            *boundary = 1;
        }
        (Some(P_NS), b"cNvCxnSpPr") if *boundary == 1 && connector_properties.is_none() => {
            *connector_properties = Some(NonVisualConnectorProperties::from_fragment(
                &raw, namespaces,
            )?);
            *boundary = 2;
        }
        (Some(P_NS), b"nvPr") if *boundary == 2 && application_properties.is_none() => {
            *application_properties = Some(RawElement::from_fragment(&raw)?);
            *boundary = 3;
        }
        (Some(P_NS), b"cNvPr" | b"cNvCxnSpPr" | b"nvPr") => {
            return Err(OxmlError::InvalidValue(
                "p:nvCxnSpPr children must be p:cNvPr, p:cNvCxnSpPr, and p:nvPr in order"
                    .to_owned(),
            ));
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

struct ParsedNonVisualConnectorProperties {
    start_connection: Option<CT_Connection>,
    end_connection: Option<CT_Connection>,
    raw: NonVisualConnectorProperties,
}

impl ParsedNonVisualConnectorProperties {
    fn into_parts(
        self,
    ) -> (
        Option<CT_Connection>,
        Option<CT_Connection>,
        NonVisualConnectorProperties,
    ) {
        (self.start_connection, self.end_connection, self.raw)
    }
}

impl NonVisualConnectorProperties {
    fn from_fragment(
        xml: &[u8],
        inherited: &NamespaceBindings,
    ) -> Result<ParsedNonVisualConnectorProperties> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let start = start.into_owned();
                    let namespaces = inherited.with_start(&start)?;
                    let mut start_connection = None;
                    let mut end_connection = None;
                    let mut locks = None;
                    let mut extension_list = None;
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
                                capture_connector_property_child(
                                    &name,
                                    uri,
                                    raw,
                                    &child_namespaces,
                                    &mut locks,
                                    &mut start_connection,
                                    &mut end_connection,
                                    &mut extension_list,
                                    &mut raw_children,
                                    &mut boundary,
                                )?;
                            }
                            Event::Empty(child) => {
                                let child_namespaces = namespaces.with_start(&child)?;
                                let name = local_name(child.name().as_ref()).to_vec();
                                let uri = child_namespaces.element_uri(child.name().as_ref());
                                let raw = capture_empty_element(&child)?;
                                capture_connector_property_child(
                                    &name,
                                    uri,
                                    raw,
                                    &child_namespaces,
                                    &mut locks,
                                    &mut start_connection,
                                    &mut end_connection,
                                    &mut extension_list,
                                    &mut raw_children,
                                    &mut boundary,
                                )?;
                            }
                            Event::End(end) if local_name(end.name().as_ref()) == b"cNvCxnSpPr" => {
                                return Ok(ParsedNonVisualConnectorProperties {
                                    start_connection,
                                    end_connection,
                                    raw: Self {
                                        raw_attributes: root_attributes(
                                            &start,
                                            FIXED_SHAPE_TREE_PREFIXES,
                                        )?,
                                        locks,
                                        extension_list,
                                        raw_children,
                                    },
                                });
                            }
                            Event::Eof => {
                                return Err(OxmlError::MissingElement(
                                    "closing p:cNvCxnSpPr".to_owned(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                Event::Empty(start) => {
                    return Ok(ParsedNonVisualConnectorProperties {
                        start_connection: None,
                        end_connection: None,
                        raw: Self {
                            raw_attributes: root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?,
                            locks: None,
                            extension_list: None,
                            raw_children: OrderedRawChildren::default(),
                        },
                    });
                }
                Event::Eof => return Err(OxmlError::MissingElement("p:cNvCxnSpPr".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn write_xml<W: Write>(
        &self,
        writer: &mut Writer<W>,
        start_connection: Option<&CT_Connection>,
        end_connection: Option<&CT_Connection>,
    ) -> Result<()> {
        let mut start = BytesStart::new("p:cNvCxnSpPr");
        push_attributes(&mut start, &self.raw_attributes);
        if self.locks.is_none()
            && start_connection.is_none()
            && end_connection.is_none()
            && self.extension_list.is_none()
            && self.raw_children.is_empty()
        {
            writer.write_event(Event::Empty(start))?;
            return Ok(());
        }
        writer.write_event(Event::Start(start))?;
        emit_raw(writer, self.raw_children.at(0))?;
        if let Some(locks) = &self.locks {
            writer.get_mut().write_all(locks)?;
        }
        emit_raw(writer, self.raw_children.at(1))?;
        if let Some(connection) = start_connection {
            connection.write_xml(writer, "a:stCxn")?;
        }
        emit_raw(writer, self.raw_children.at(2))?;
        if let Some(connection) = end_connection {
            connection.write_xml(writer, "a:endCxn")?;
        }
        emit_raw(writer, self.raw_children.at(3))?;
        if let Some(extension_list) = &self.extension_list {
            writer.get_mut().write_all(extension_list)?;
        }
        emit_raw(writer, self.raw_children.at(4))?;
        writer.write_event(Event::End(BytesEnd::new("p:cNvCxnSpPr")))?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn capture_connector_property_child(
    name: &[u8],
    uri: Option<&str>,
    raw: Vec<u8>,
    namespaces: &NamespaceBindings,
    locks: &mut Option<Vec<u8>>,
    start_connection: &mut Option<CT_Connection>,
    end_connection: &mut Option<CT_Connection>,
    extension_list: &mut Option<Vec<u8>>,
    raw_children: &mut OrderedRawChildren,
    boundary: &mut usize,
) -> Result<()> {
    if uri == Some(A_NS) && matches!(name, b"cxnSpLocks" | b"stCxn" | b"endCxn" | b"extLst") {
        namespaces.reject_writer_conflicts(FIXED_SHAPE_TREE_PREFIXES)?;
    }
    match (uri, name) {
        (Some(A_NS), b"cxnSpLocks") if *boundary == 0 && locks.is_none() => {
            *locks = Some(raw);
            *boundary = 1;
        }
        (Some(A_NS), b"stCxn") if *boundary <= 1 && start_connection.is_none() => {
            *start_connection = Some(CT_Connection::from_fragment(&raw)?);
            *boundary = 2;
        }
        (Some(A_NS), b"endCxn") if *boundary <= 2 && end_connection.is_none() => {
            *end_connection = Some(CT_Connection::from_fragment(&raw)?);
            *boundary = 3;
        }
        (Some(A_NS), b"extLst") if *boundary <= 3 && extension_list.is_none() => {
            *extension_list = Some(raw);
            *boundary = 4;
        }
        (Some(A_NS), b"cxnSpLocks" | b"stCxn" | b"endCxn" | b"extLst") => {
            return Err(OxmlError::InvalidValue(
                "p:cNvCxnSpPr children must be a:cxnSpLocks, a:stCxn, a:endCxn, and a:extLst in order"
                    .to_owned(),
            ));
        }
        _ => raw_children.push(*boundary, raw),
    }
    Ok(())
}

impl CT_Connection {
    fn from_fragment(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let raw_attributes = endpoint_attributes(&start)?;
                    let id = required_endpoint_attribute(&start, "id")?;
                    let idx = required_endpoint_attribute(&start, "idx")?;
                    let content_start = reader.buffer_position() as usize;
                    let content_end = endpoint_content_end(&mut reader, &start)?;
                    return Ok(Self {
                        id,
                        idx,
                        raw_attributes,
                        raw_content: xml[content_start..content_end].to_vec(),
                    });
                }
                Event::Empty(start) => {
                    return Ok(Self {
                        id: required_endpoint_attribute(&start, "id")?,
                        idx: required_endpoint_attribute(&start, "idx")?,
                        raw_attributes: endpoint_attributes(&start)?,
                        raw_content: Vec::new(),
                    });
                }
                Event::Eof => {
                    return Err(OxmlError::MissingElement("connector endpoint".to_owned()));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>, name: &str) -> Result<()> {
        let mut start = BytesStart::new(name);
        let id = self.id.to_string();
        let idx = self.idx.to_string();
        start.push_attribute(("id", id.as_str()));
        start.push_attribute(("idx", idx.as_str()));
        push_attributes(&mut start, &self.raw_attributes);
        if self.raw_content.is_empty() {
            writer.write_event(Event::Empty(start))?;
        } else {
            writer.write_event(Event::Start(start))?;
            writer.get_mut().write_all(&self.raw_content)?;
            writer.write_event(Event::End(BytesEnd::new(name)))?;
        }
        Ok(())
    }
}

impl RawElement {
    fn from_fragment(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(start) => {
                    let raw_attributes = root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?;
                    let content_start = reader.buffer_position() as usize;
                    let content_end = endpoint_content_end(&mut reader, &start)?;
                    return Ok(Self {
                        raw_attributes,
                        raw_content: xml[content_start..content_end].to_vec(),
                        was_empty: false,
                    });
                }
                Event::Empty(start) => {
                    return Ok(Self {
                        raw_attributes: root_attributes(&start, FIXED_SHAPE_TREE_PREFIXES)?,
                        raw_content: Vec::new(),
                        was_empty: true,
                    });
                }
                Event::Eof => return Err(OxmlError::MissingElement("raw element".to_owned())),
                _ => {}
            }
            buffer.clear();
        }
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>, name: &str) -> Result<()> {
        let mut start = BytesStart::new(name);
        push_attributes(&mut start, &self.raw_attributes);
        if self.was_empty {
            writer.write_event(Event::Empty(start))?;
        } else {
            writer.write_event(Event::Start(start))?;
            writer.get_mut().write_all(&self.raw_content)?;
            writer.write_event(Event::End(BytesEnd::new(name)))?;
        }
        Ok(())
    }
}

fn endpoint_attributes(start: &BytesStart<'_>) -> Result<RawAttributes> {
    Ok(all_attributes(start)?
        .into_iter()
        .filter(|(name, _)| name != "id" && name != "idx")
        .collect())
}

fn required_endpoint_attribute(start: &BytesStart<'_>, expected: &str) -> Result<u32> {
    let value = all_attributes(start)?
        .into_iter()
        .find_map(|(name, value)| (name == expected).then_some(value))
        .ok_or_else(|| {
            OxmlError::InvalidValue(format!(
                "{} requires unqualified @{expected}",
                String::from_utf8_lossy(start.name().as_ref())
            ))
        })?;
    value.parse::<u32>().map_err(|_| {
        OxmlError::InvalidValue(format!(
            "{} has malformed @{expected}: {value}",
            String::from_utf8_lossy(start.name().as_ref())
        ))
    })
}

fn endpoint_content_end(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<usize> {
    let expected = local_name(start.name().as_ref()).to_vec();
    let mut depth = 0usize;
    let mut buffer = Vec::new();
    loop {
        let before = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Start(_) => depth += 1,
            Event::End(end) if depth == 0 && local_name(end.name().as_ref()) == expected => {
                return Ok(before);
            }
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "closing connector endpoint".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
}

fn drawing_error(error: impl std::fmt::Display) -> OxmlError {
    OxmlError::InvalidValue(error.to_string())
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

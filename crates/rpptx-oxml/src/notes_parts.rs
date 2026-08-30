use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_drawing::color::ColorMap;
use oxml_drawing::namespace::A_NS;
use oxml_drawing::order::OrderedRawChildren;
use oxml_drawing::text::CT_TextListStyle;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::namespace::{FIXED_MODEL_PREFIXES, NamespaceBindings, P_NS, R_NS, root_attributes};
use crate::placeholder::PhType;
use crate::shape_tree::ShapeTreeChild;
use crate::slide_parts::{
    CT_ColorMapOverride, CT_CommonSlideData, CT_HeaderFooter, ParsedColorMap, parse_color_map,
    write_color_map,
};

pub type Result<T> = std::result::Result<T, OxmlError>;
type RawAttributes = Vec<(String, String)>;

/// A notes-slide part with typed common slide data and speaker-note text.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_NotesSlide {
    pub common_slide_data: CT_CommonSlideData,
    pub color_map_override: Option<CT_ColorMapOverride>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
}

/// A notes-master part with typed common slide data, colour map, and notes style.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_NotesMaster {
    pub common_slide_data: CT_CommonSlideData,
    pub color_map: ColorMap,
    pub header_footer: Option<CT_HeaderFooter>,
    pub notes_style: Option<CT_TextListStyle>,
    raw_attributes: RawAttributes,
    color_map_attributes: RawAttributes,
    color_map_children: OrderedRawChildren,
    raw_children: OrderedRawChildren,
}

/// A handout-master part with typed common slide data and header/footer settings.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct CT_HandoutMaster {
    pub common_slide_data: CT_CommonSlideData,
    pub color_map: ColorMap,
    pub header_footer: Option<CT_HeaderFooter>,
    raw_attributes: RawAttributes,
    color_map_attributes: RawAttributes,
    color_map_children: OrderedRawChildren,
    raw_children: OrderedRawChildren,
}

#[derive(Clone, Copy)]
enum RootKind {
    NotesSlide,
    NotesMaster,
    HandoutMaster,
}

impl RootKind {
    const fn local_name(self) -> &'static [u8] {
        match self {
            Self::NotesSlide => b"notes",
            Self::NotesMaster => b"notesMaster",
            Self::HandoutMaster => b"handoutMaster",
        }
    }

    const fn tag(self) -> &'static str {
        match self {
            Self::NotesSlide => "p:notes",
            Self::NotesMaster => "p:notesMaster",
            Self::HandoutMaster => "p:handoutMaster",
        }
    }
}

#[derive(Default)]
struct ParsedRoot {
    common_slide_data: Option<CT_CommonSlideData>,
    color_map_override: Option<CT_ColorMapOverride>,
    color_map: Option<ParsedColorMap>,
    notes_style: Option<CT_TextListStyle>,
    raw_attributes: RawAttributes,
    raw_children: OrderedRawChildren,
    boundary: usize,
    header_footer: Option<CT_HeaderFooter>,
    extension_list_seen: bool,
}

impl CT_NotesSlide {
    /// Parses a complete notes-slide part with any PresentationML prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let parsed = parse_root(xml, RootKind::NotesSlide)?;
        Ok(Self {
            common_slide_data: required(parsed.common_slide_data, "p:cSld")?,
            color_map_override: parsed.color_map_override,
            raw_attributes: parsed.raw_attributes,
            raw_children: parsed.raw_children,
        })
    }

    /// Serialises a notes-slide part with fixed modelled prefixes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        write_root_start(&mut writer, RootKind::NotesSlide, &self.raw_attributes)?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        self.common_slide_data.write_xml(&mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(1))?;
        if let Some(color_map_override) = &self.color_map_override {
            color_map_override.write_xml(&mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(2))?;
        emit_raw(&mut writer, self.raw_children.at(3))?;
        writer.write_event(Event::End(BytesEnd::new(RootKind::NotesSlide.tag())))?;
        Ok(writer.into_inner())
    }

    /// Returns text from effective body placeholders in shape-tree order.
    pub fn notes_text(&self) -> String {
        let mut bodies = Vec::new();
        collect_body_text(&self.common_slide_data.shape_tree.children, &mut bodies);
        bodies.join("\n")
    }
}

impl CT_NotesMaster {
    /// Parses a complete notes-master part with any PresentationML prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let parsed = parse_root(xml, RootKind::NotesMaster)?;
        let color_map = required(parsed.color_map, "p:clrMap")?;
        Ok(Self {
            common_slide_data: required(parsed.common_slide_data, "p:cSld")?,
            color_map: color_map.value,
            header_footer: parsed.header_footer,
            notes_style: parsed.notes_style,
            raw_attributes: parsed.raw_attributes,
            color_map_attributes: color_map.raw_attributes,
            color_map_children: color_map.raw_children,
            raw_children: parsed.raw_children,
        })
    }

    /// Serialises a notes-master part with fixed modelled prefixes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        write_root_start(&mut writer, RootKind::NotesMaster, &self.raw_attributes)?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        self.common_slide_data.write_xml(&mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(1))?;
        write_color_map(
            &mut writer,
            "p:clrMap",
            &self.color_map,
            &self.color_map_attributes,
            &self.color_map_children,
        )?;
        emit_raw(&mut writer, self.raw_children.at(2))?;
        if let Some(header_footer) = &self.header_footer {
            header_footer.write_xml(&mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(3))?;
        if let Some(notes_style) = &self.notes_style {
            notes_style
                .write_xml_as(&mut writer, "p:notesStyle")
                .map_err(|error| OxmlError::InvalidValue(error.to_string()))?;
        }
        emit_raw(&mut writer, self.raw_children.at(4))?;
        emit_raw(&mut writer, self.raw_children.at(5))?;
        writer.write_event(Event::End(BytesEnd::new(RootKind::NotesMaster.tag())))?;
        Ok(writer.into_inner())
    }
}

impl CT_HandoutMaster {
    /// Parses a complete handout-master part with any PresentationML prefix.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let parsed = parse_root(xml, RootKind::HandoutMaster)?;
        let color_map = required(parsed.color_map, "p:clrMap")?;
        Ok(Self {
            common_slide_data: required(parsed.common_slide_data, "p:cSld")?,
            color_map: color_map.value,
            header_footer: parsed.header_footer,
            raw_attributes: parsed.raw_attributes,
            color_map_attributes: color_map.raw_attributes,
            color_map_children: color_map.raw_children,
            raw_children: parsed.raw_children,
        })
    }

    /// Serialises a handout-master part with fixed modelled prefixes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        write_root_start(&mut writer, RootKind::HandoutMaster, &self.raw_attributes)?;
        emit_raw(&mut writer, self.raw_children.at(0))?;
        self.common_slide_data.write_xml(&mut writer)?;
        emit_raw(&mut writer, self.raw_children.at(1))?;
        write_color_map(
            &mut writer,
            "p:clrMap",
            &self.color_map,
            &self.color_map_attributes,
            &self.color_map_children,
        )?;
        emit_raw(&mut writer, self.raw_children.at(2))?;
        if let Some(header_footer) = &self.header_footer {
            header_footer.write_xml(&mut writer)?;
        }
        emit_raw(&mut writer, self.raw_children.at(3))?;
        emit_raw(&mut writer, self.raw_children.at(4))?;
        writer.write_event(Event::End(BytesEnd::new(RootKind::HandoutMaster.tag())))?;
        Ok(writer.into_inner())
    }
}

fn parse_root(xml: &[u8], kind: RootKind) -> Result<ParsedRoot> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                let namespaces = NamespaceBindings::default().with_start(&start)?;
                if local_name(start.name().as_ref()) != kind.local_name()
                    || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                {
                    return Err(unexpected(&start));
                }
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                return parse_root_children(&mut reader, &start, &namespaces, kind);
            }
            Event::Empty(start) => {
                let namespaces = NamespaceBindings::default().with_start(&start)?;
                if local_name(start.name().as_ref()) != kind.local_name()
                    || namespaces.element_uri(start.name().as_ref()) != Some(P_NS)
                {
                    return Err(unexpected(&start));
                }
                return Err(OxmlError::MissingElement(format!(
                    "{} requires p:cSld",
                    kind.tag()
                )));
            }
            Event::Eof => return Err(OxmlError::MissingElement(kind.tag().to_owned())),
            _ => {}
        }
        buffer.clear();
    }
}

fn parse_root_children(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart<'_>,
    root_namespaces: &NamespaceBindings,
    kind: RootKind,
) -> Result<ParsedRoot> {
    let mut parsed = ParsedRoot {
        raw_attributes: root_attributes(start, FIXED_MODEL_PREFIXES)?,
        ..ParsedRoot::default()
    };
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(child) => {
                let namespaces = root_namespaces.with_start(&child)?;
                let name = local_name(child.name().as_ref()).to_vec();
                let is_p = namespaces.element_uri(child.name().as_ref()) == Some(P_NS);
                let raw = capture_element(reader, &child)?;
                parsed.capture_child(&name, is_p, &namespaces, raw, kind)?;
            }
            Event::Empty(child) => {
                let namespaces = root_namespaces.with_start(&child)?;
                let name = local_name(child.name().as_ref()).to_vec();
                let is_p = namespaces.element_uri(child.name().as_ref()) == Some(P_NS);
                let raw = capture_empty_element(&child)?;
                parsed.capture_child(&name, is_p, &namespaces, raw, kind)?;
            }
            Event::End(end) if local_name(end.name().as_ref()) == kind.local_name() => {
                return Ok(parsed);
            }
            event @ (Event::Text(_)
            | Event::CData(_)
            | Event::Comment(_)
            | Event::PI(_)
            | Event::DocType(_)) => parsed
                .raw_children
                .push(parsed.boundary, capture_event(event)?),
            Event::Eof => return Err(OxmlError::MissingElement(format!("closing {}", kind.tag()))),
            _ => {}
        }
        buffer.clear();
    }
}

impl ParsedRoot {
    fn capture_child(
        &mut self,
        name: &[u8],
        is_p: bool,
        namespaces: &NamespaceBindings,
        raw: Vec<u8>,
        kind: RootKind,
    ) -> Result<()> {
        if !is_p {
            self.raw_children.push(self.boundary, raw);
            return Ok(());
        }
        match (kind, name) {
            (_, b"cSld") => {
                if self.boundary != 0 || self.common_slide_data.is_some() {
                    return Err(out_of_order(kind, "p:cSld"));
                }
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                self.common_slide_data = Some(CT_CommonSlideData::from_fragment(&raw, namespaces)?);
                self.boundary = 1;
            }
            (RootKind::NotesSlide, b"clrMapOvr") => {
                if self.boundary != 1 || self.color_map_override.is_some() {
                    return Err(out_of_order(kind, "p:clrMapOvr"));
                }
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                self.color_map_override =
                    Some(CT_ColorMapOverride::from_fragment(&raw, namespaces)?);
                self.boundary = 2;
            }
            (RootKind::NotesMaster | RootKind::HandoutMaster, b"clrMap") => {
                if self.boundary != 1 || self.color_map.is_some() {
                    return Err(out_of_order(kind, "p:clrMap"));
                }
                self.color_map = Some(parse_color_map(&raw, namespaces)?);
                self.boundary = 2;
            }
            (RootKind::NotesMaster | RootKind::HandoutMaster, b"hf") => {
                if self.boundary != 2 || self.header_footer.is_some() {
                    return Err(out_of_order(kind, "p:hf"));
                }
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                self.header_footer = Some(CT_HeaderFooter::from_fragment(&raw, namespaces)?);
                self.boundary = 3;
            }
            (RootKind::NotesMaster, b"notesStyle") => {
                if !matches!(self.boundary, 2 | 3) || self.notes_style.is_some() {
                    return Err(out_of_order(kind, "p:notesStyle"));
                }
                namespaces.reject_writer_conflicts(FIXED_MODEL_PREFIXES)?;
                self.notes_style = Some(
                    CT_TextListStyle::from_xml(&raw)
                        .map_err(|error| OxmlError::InvalidValue(error.to_string()))?,
                );
                self.boundary = 4;
            }
            (_, b"extLst") => {
                let valid_boundary = match kind {
                    RootKind::NotesSlide => matches!(self.boundary, 1 | 2),
                    RootKind::NotesMaster => matches!(self.boundary, 2..=4),
                    RootKind::HandoutMaster => matches!(self.boundary, 2 | 3),
                };
                if !valid_boundary || self.extension_list_seen {
                    return Err(out_of_order(kind, "p:extLst"));
                }
                let at = match kind {
                    RootKind::NotesSlide => 2,
                    RootKind::NotesMaster => 4,
                    RootKind::HandoutMaster => 3,
                };
                self.raw_children.push(at, raw);
                self.extension_list_seen = true;
                self.boundary = at + 1;
            }
            _ => self.raw_children.push(self.boundary, raw),
        }
        Ok(())
    }
}

fn collect_body_text(children: &[ShapeTreeChild], bodies: &mut Vec<String>) {
    for child in children {
        match child {
            ShapeTreeChild::Shape(shape)
                if shape
                    .placeholder
                    .as_ref()
                    .is_some_and(|placeholder| placeholder.effective_type() == PhType::Body) =>
            {
                if let Some(text_body) = &shape.text_body {
                    let text = text_body.plain_text();
                    if !text.is_empty() {
                        bodies.push(text);
                    }
                }
            }
            ShapeTreeChild::GroupShape(group) => collect_body_text(&group.children, bodies),
            ShapeTreeChild::AlternateContent(alternate) => {
                if let Some(fallback) = alternate.selected_fallback() {
                    collect_body_text(fallback, bodies);
                }
            }
            _ => {}
        }
    }
}

fn write_root_start<W: Write>(
    writer: &mut Writer<W>,
    kind: RootKind,
    attributes: &RawAttributes,
) -> Result<()> {
    let mut root = BytesStart::new(kind.tag());
    root.push_attribute(("xmlns:p", P_NS));
    root.push_attribute(("xmlns:a", A_NS));
    root.push_attribute(("xmlns:r", R_NS));
    push_attributes(&mut root, attributes);
    writer.write_event(Event::Start(root))?;
    Ok(())
}

fn push_attributes(start: &mut BytesStart<'_>, attributes: &RawAttributes) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn capture_event(event: Event<'_>) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    writer.write_event(event.into_owned())?;
    Ok(writer.into_inner())
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

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn required<T>(value: Option<T>, name: &str) -> Result<T> {
    value.ok_or_else(|| OxmlError::MissingElement(name.to_owned()))
}

fn out_of_order(kind: RootKind, child: &str) -> OxmlError {
    OxmlError::InvalidValue(format!(
        "{child} is duplicated or out of sequence in {}",
        kind.tag()
    ))
}

fn unexpected(element: &BytesStart<'_>) -> OxmlError {
    OxmlError::UnexpectedElement(String::from_utf8_lossy(element.name().as_ref()).into_owned())
}

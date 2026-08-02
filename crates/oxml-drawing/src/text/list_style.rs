use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::{local_name, matches_local_name};
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::namespace::reject_conflicting_a_prefix;
use crate::order::OrderedRawChildren;

use super::body::{Result, TextError, missing_end};
use super::paragraph::CT_TextParagraphProperties;

/// DrawingML list defaults for each of the nine paragraph levels.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CT_TextListStyle {
    pub level1: Option<CT_TextParagraphProperties>,
    pub level2: Option<CT_TextParagraphProperties>,
    pub level3: Option<CT_TextParagraphProperties>,
    pub level4: Option<CT_TextParagraphProperties>,
    pub level5: Option<CT_TextParagraphProperties>,
    pub level6: Option<CT_TextParagraphProperties>,
    pub level7: Option<CT_TextParagraphProperties>,
    pub level8: Option<CT_TextParagraphProperties>,
    pub level9: Option<CT_TextParagraphProperties>,
    raw_attributes: Vec<(String, String)>,
    raw_children: OrderedRawChildren,
}

impl CT_TextListStyle {
    /// Parses a complete list-style element with any prefix, including the
    /// PresentationML wrappers that share this DrawingML content model.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if is_list_style_root(element.name().as_ref()) => {
                    reject_conflicting_a_prefix(&element)?;
                    return Self::from_element(&mut reader, &element);
                }
                Event::Empty(element) if is_list_style_root(element.name().as_ref()) => {
                    reject_conflicting_a_prefix(&element)?;
                    return Self::from_start(&element);
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(unexpected(&element));
                }
                Event::Eof => {
                    return Err(TextError::Xml(OxmlError::MissingElement(
                        "DrawingML list style".to_owned(),
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_start(start: &BytesStart<'_>) -> Result<Self> {
        Ok(Self {
            raw_attributes: capture_raw_attributes(start)?,
            ..Self::default()
        })
    }

    fn from_element(reader: &mut Reader<&[u8]>, start: &BytesStart<'_>) -> Result<Self> {
        let mut style = Self::from_start(start)?;
        let root_name = local_name(start.name().as_ref()).to_vec();
        let mut boundary = 0;
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) => {
                    let name = local_name(element.name().as_ref()).to_vec();
                    if list_level(&name)?.is_some() {
                        reject_conflicting_a_prefix(&element)?;
                    }
                    let raw = capture_element(reader, &element)?;
                    style.capture_child(&name, raw, &mut boundary)?;
                }
                Event::Empty(element) => {
                    let name = local_name(element.name().as_ref()).to_vec();
                    if list_level(&name)?.is_some() {
                        reject_conflicting_a_prefix(&element)?;
                    }
                    let raw = capture_empty_element(&element)?;
                    style.capture_child(&name, raw, &mut boundary)?;
                }
                Event::End(element)
                    if local_name(element.name().as_ref()) == root_name.as_slice() =>
                {
                    return Ok(style);
                }
                Event::Eof => {
                    return Err(missing_end(
                        std::str::from_utf8(&root_name).unwrap_or("list style"),
                    ));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn capture_child(&mut self, name: &[u8], raw: Vec<u8>, boundary: &mut usize) -> Result<()> {
        if let Some(level) = list_level(name)? {
            if self.level(level).is_some() {
                return Err(TextError::DuplicateElement(
                    String::from_utf8_lossy(name).into_owned(),
                ));
            }
            self.set_level(level, CT_TextParagraphProperties::from_xml(&raw)?);
            *boundary = (*boundary).max(level);
            return Ok(());
        }

        let schema_boundary = match name {
            b"defPPr" => 0,
            b"extLst" => 9,
            _ => *boundary,
        };
        self.raw_children
            .push((*boundary).max(schema_boundary), raw);
        *boundary = (*boundary).max(schema_boundary);
        Ok(())
    }

    /// Writes the list style with fixed prefixes and ascending level order.
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.write_xml_as(writer, "a:lstStyle")
    }

    /// Writes the list style using a caller-selected OOXML wrapper tag.
    pub fn write_xml_as<W: Write>(&self, writer: &mut Writer<W>, tag: &str) -> Result<()> {
        let mut start = BytesStart::new(tag);
        push_raw_attributes(&mut start, &self.raw_attributes);
        let has_levels = (1..=9).any(|level| self.level(level).is_some());
        if !has_levels && self.raw_children.is_empty() {
            return write_empty(writer, start);
        }

        write_start(writer, start)?;
        emit_raw(writer, self.raw_children.at(0))?;
        for level in 1..=9 {
            if let Some(properties) = self.level(level) {
                let tag = level_tag(level)
                    .ok_or_else(|| TextError::UnexpectedElement(format!("list level {level}")))?;
                properties.write_xml(writer, tag)?;
            }
            emit_raw(writer, self.raw_children.at(level))?;
        }
        write_end(writer, tag)
    }

    /// Serialises a complete list-style fragment.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    /// Returns one level by its one-based DrawingML number.
    pub fn level(&self, level: usize) -> Option<&CT_TextParagraphProperties> {
        match level {
            1 => self.level1.as_ref(),
            2 => self.level2.as_ref(),
            3 => self.level3.as_ref(),
            4 => self.level4.as_ref(),
            5 => self.level5.as_ref(),
            6 => self.level6.as_ref(),
            7 => self.level7.as_ref(),
            8 => self.level8.as_ref(),
            9 => self.level9.as_ref(),
            _ => None,
        }
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }

    fn set_level(&mut self, level: usize, properties: CT_TextParagraphProperties) {
        let slot = match level {
            1 => &mut self.level1,
            2 => &mut self.level2,
            3 => &mut self.level3,
            4 => &mut self.level4,
            5 => &mut self.level5,
            6 => &mut self.level6,
            7 => &mut self.level7,
            8 => &mut self.level8,
            9 => &mut self.level9,
            _ => return,
        };
        *slot = Some(properties);
    }
}

fn is_list_style_root(name: &[u8]) -> bool {
    matches_local_name(name, b"lstStyle")
        || matches_local_name(name, b"defaultTextStyle")
        || matches_local_name(name, b"titleStyle")
        || matches_local_name(name, b"bodyStyle")
        || matches_local_name(name, b"otherStyle")
        || matches_local_name(name, b"notesStyle")
}

fn list_level(name: &[u8]) -> Result<Option<usize>> {
    let Some(number) = name
        .strip_prefix(b"lvl")
        .and_then(|name| name.strip_suffix(b"pPr"))
    else {
        return Ok(None);
    };
    if !number.is_empty() && number.iter().all(u8::is_ascii_digit) {
        if number.len() == 1 && (b'1'..=b'9').contains(&number[0]) {
            return Ok(Some(usize::from(number[0] - b'0')));
        }
        return Err(TextError::UnexpectedElement(
            String::from_utf8_lossy(name).into_owned(),
        ));
    }
    Ok(None)
}

fn level_tag(level: usize) -> Option<&'static str> {
    match level {
        1 => Some("a:lvl1pPr"),
        2 => Some("a:lvl2pPr"),
        3 => Some("a:lvl3pPr"),
        4 => Some("a:lvl4pPr"),
        5 => Some("a:lvl5pPr"),
        6 => Some("a:lvl6pPr"),
        7 => Some("a:lvl7pPr"),
        8 => Some("a:lvl8pPr"),
        9 => Some("a:lvl9pPr"),
        _ => None,
    }
}

fn capture_raw_attributes(start: &BytesStart<'_>) -> Result<Vec<(String, String)>> {
    let mut raw = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(OxmlError::from)?;
        let name = std::str::from_utf8(attribute.key.as_ref())
            .map_err(OxmlError::from)?
            .to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())
            .map_err(OxmlError::from)?
            .into_owned();
        raw.push((name, value));
    }
    Ok(raw)
}

fn push_raw_attributes(start: &mut BytesStart<'_>, attributes: &[(String, String)]) {
    for (name, value) in attributes {
        start.push_attribute((name.as_str(), value.as_str()));
    }
}

fn emit_raw<'a, W: Write>(
    writer: &mut Writer<W>,
    children: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    for child in children {
        writer.get_mut().write_all(child).map_err(OxmlError::from)?;
    }
    Ok(())
}

fn write_start<W: Write>(writer: &mut Writer<W>, start: BytesStart<'_>) -> Result<()> {
    writer
        .write_event(Event::Start(start))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_empty<W: Write>(writer: &mut Writer<W>, start: BytesStart<'_>) -> Result<()> {
    writer
        .write_event(Event::Empty(start))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn write_end<W: Write>(writer: &mut Writer<W>, tag: &str) -> Result<()> {
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(OxmlError::from)?;
    Ok(())
}

fn unexpected(element: &BytesStart<'_>) -> TextError {
    TextError::UnexpectedElement(String::from_utf8_lossy(element.name().as_ref()).into_owned())
}

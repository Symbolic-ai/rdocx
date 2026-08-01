use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::matches_local_name;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::order::OrderedRawChildren;

pub mod body;
pub mod paragraph;

pub use body::{
    CT_TextBodyProperties, Coordinate32Value, NormalAutofit, TextAnchor, TextAutofit, TextError,
    TextVertical, TextWrap,
};
pub use paragraph::{
    CT_RegularTextRun, CT_TextCharacterProperties, CT_TextField, CT_TextLineBreak,
    CT_TextParagraph, CT_TextParagraphProperties, TextAlignment, TextFont, TextHyperlink,
    TextPointValue, TextRun, TextSpace, TextSpacing, TextStrike, TextUnderline, TextValue,
};

use body::{Result, missing_end};

/// The `a:txBody` shell with typed body properties and opaque later text stages.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CT_TextBody {
    pub body_properties: CT_TextBodyProperties,
    list_style: Option<OpaqueTextElement>,
    paragraphs: Vec<CT_TextParagraph>,
    raw_children: OrderedRawChildren,
}

impl CT_TextBody {
    /// Parses a complete `a:txBody` while retaining later text stages.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"txBody") => {
                    return Self::from_element(&mut reader);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"txBody") => {
                    return Err(TextError::MissingBodyProperties);
                }
                Event::Start(element) | Event::Empty(element) => {
                    return Err(TextError::UnexpectedElement(element_name(&element)));
                }
                Event::Eof => {
                    return Err(TextError::Xml(OxmlError::MissingElement(
                        "DrawingML text body".to_owned(),
                    )));
                }
                _ => {}
            }
            buffer.clear();
        }
    }

    fn from_element(reader: &mut Reader<&[u8]>) -> Result<Self> {
        let mut body_properties = None;
        let mut list_style = None;
        let mut paragraphs = Vec::new();
        let mut raw_children = OrderedRawChildren::default();
        let mut boundary = 0;
        let mut buffer = Vec::new();

        loop {
            match reader
                .read_event_into(&mut buffer)
                .map_err(OxmlError::from)?
            {
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"bodyPr") => {
                    if body_properties.is_some() {
                        return Err(TextError::DuplicateElement("bodyPr".to_owned()));
                    }
                    body_properties = Some(CT_TextBodyProperties::from_element(reader, &element)?);
                    boundary = boundary.max(1);
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"bodyPr") => {
                    if body_properties.is_some() {
                        return Err(TextError::DuplicateElement("bodyPr".to_owned()));
                    }
                    body_properties = Some(CT_TextBodyProperties::from_start(&element)?);
                    boundary = boundary.max(1);
                }
                Event::Start(element)
                    if matches_local_name(element.name().as_ref(), b"lstStyle") =>
                {
                    if list_style.is_some() {
                        return Err(TextError::DuplicateElement("lstStyle".to_owned()));
                    }
                    let raw = capture_element(reader, &element)?;
                    list_style = Some(OpaqueTextElement::from_xml(&raw, b"lstStyle")?);
                    boundary = boundary.max(2);
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"lstStyle") =>
                {
                    if list_style.is_some() {
                        return Err(TextError::DuplicateElement("lstStyle".to_owned()));
                    }
                    let raw = capture_empty_element(&element)?;
                    list_style = Some(OpaqueTextElement::from_xml(&raw, b"lstStyle")?);
                    boundary = boundary.max(2);
                }
                Event::Start(element) if matches_local_name(element.name().as_ref(), b"p") => {
                    let raw = capture_element(reader, &element)?;
                    paragraphs.push(CT_TextParagraph::from_xml(&raw)?);
                    boundary = boundary.max(2 + paragraphs.len());
                }
                Event::Empty(element) if matches_local_name(element.name().as_ref(), b"p") => {
                    let raw = capture_empty_element(&element)?;
                    paragraphs.push(CT_TextParagraph::from_xml(&raw)?);
                    boundary = boundary.max(2 + paragraphs.len());
                }
                Event::Start(element) => {
                    raw_children.push(boundary, capture_element(reader, &element)?)
                }
                Event::Empty(element) => {
                    raw_children.push(boundary, capture_empty_element(&element)?)
                }
                Event::End(element) if matches_local_name(element.name().as_ref(), b"txBody") => {
                    break;
                }
                Event::Eof => return Err(missing_end("txBody")),
                _ => {}
            }
            buffer.clear();
        }

        let body_properties = body_properties.ok_or(TextError::MissingBodyProperties)?;
        if paragraphs.is_empty() {
            return Err(TextError::MissingParagraph);
        }
        Ok(Self {
            body_properties,
            list_style,
            paragraphs,
            raw_children,
        })
    }

    /// Writes the shell with canonical outer prefixes and schema order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        self.write_xml(&mut writer)?;
        Ok(writer.into_inner())
    }

    /// Writes the shell into an existing XML writer.
    pub fn write_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        if self.paragraphs.is_empty() {
            return Err(TextError::MissingParagraph);
        }
        writer
            .write_event(Event::Start(BytesStart::new("a:txBody")))
            .map_err(OxmlError::from)?;
        emit_raw(writer, self.raw_children.at(0))?;
        self.body_properties.write_xml(writer)?;
        emit_raw(writer, self.raw_children.at(1))?;
        if let Some(list_style) = &self.list_style {
            list_style.write_xml(writer, "a:lstStyle")?;
        }
        emit_raw(writer, self.raw_children.at(2))?;
        for (index, paragraph) in self.paragraphs.iter().enumerate() {
            paragraph.write_xml(writer)?;
            emit_raw(writer, self.raw_children.at(3 + index))?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("a:txBody")))
            .map_err(OxmlError::from)?;
        Ok(())
    }

    pub fn has_list_style(&self) -> bool {
        self.list_style.is_some()
    }

    pub fn paragraph_count(&self) -> usize {
        self.paragraphs.len()
    }

    pub fn paragraphs(&self) -> &[CT_TextParagraph] {
        &self.paragraphs
    }

    pub fn raw_children(&self) -> &OrderedRawChildren {
        &self.raw_children
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OpaqueTextElement {
    empty: bool,
    inner_xml: Vec<u8>,
}

impl OpaqueTextElement {
    fn from_xml(xml: &[u8], expected: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut buffer = Vec::new();
        match reader
            .read_event_into(&mut buffer)
            .map_err(OxmlError::from)?
        {
            Event::Empty(element) if matches_local_name(element.name().as_ref(), expected) => {
                Ok(Self {
                    empty: true,
                    inner_xml: Vec::new(),
                })
            }
            Event::Start(element) if matches_local_name(element.name().as_ref(), expected) => {
                let inner_start = reader.buffer_position() as usize;
                let mut depth = 0usize;
                loop {
                    let event_start = reader.buffer_position() as usize;
                    buffer.clear();
                    match reader
                        .read_event_into(&mut buffer)
                        .map_err(OxmlError::from)?
                    {
                        Event::Start(_) => depth += 1,
                        Event::End(element)
                            if depth == 0
                                && matches_local_name(element.name().as_ref(), expected) =>
                        {
                            return Ok(Self {
                                empty: false,
                                inner_xml: xml[inner_start..event_start].to_vec(),
                            });
                        }
                        Event::End(_) => depth = depth.saturating_sub(1),
                        Event::Eof => {
                            return Err(missing_end(&String::from_utf8_lossy(expected)));
                        }
                        _ => {}
                    }
                }
            }
            Event::Start(element) | Event::Empty(element) => {
                Err(TextError::UnexpectedElement(element_name(&element)))
            }
            _ => Err(TextError::UnexpectedElement(
                String::from_utf8_lossy(expected).into_owned(),
            )),
        }
    }

    fn write_xml<W: Write>(&self, writer: &mut Writer<W>, tag: &str) -> Result<()> {
        if self.empty {
            writer
                .write_event(Event::Empty(BytesStart::new(tag)))
                .map_err(OxmlError::from)?;
            return Ok(());
        }
        writer
            .write_event(Event::Start(BytesStart::new(tag)))
            .map_err(OxmlError::from)?;
        writer
            .get_mut()
            .write_all(&self.inner_xml)
            .map_err(OxmlError::from)?;
        writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(OxmlError::from)?;
        Ok(())
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

fn element_name(element: &BytesStart<'_>) -> String {
    String::from_utf8_lossy(element.name().as_ref()).into_owned()
}

#[cfg(test)]
mod tests {
    use super::CT_TextBody;

    #[test]
    fn text_body_reads_any_prefix_and_writes_the_fixed_a_prefix() {
        let xml = br#"<q:txBody><x:before/><q:bodyPr anchor="ctr"><q:noAutofit/></q:bodyPr><x:afterBody/><q:lstStyle><x:listChild/></q:lstStyle><x:beforeParagraph/><q:p><x:run>kept</x:run></q:p><x:afterParagraph/></q:txBody>"#;
        let body = CT_TextBody::from_xml(xml).unwrap();
        assert!(body.has_list_style());
        assert_eq!(body.paragraph_count(), 1);
        assert_eq!(body.to_xml().unwrap(), br#"<a:txBody><x:before/><a:bodyPr anchor="ctr"><a:noAutofit/></a:bodyPr><x:afterBody/><a:lstStyle><x:listChild/></a:lstStyle><x:beforeParagraph/><a:p><x:run>kept</x:run></a:p><x:afterParagraph/></a:txBody>"#);
    }
}

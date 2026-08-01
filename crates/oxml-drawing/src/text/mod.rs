use std::io::Write;

use oxml_core::OxmlError;
use oxml_core::raw_xml::{capture_element, capture_empty_element};
use oxml_core::xml::matches_local_name;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::order::OrderedRawChildren;

pub mod body;
pub mod bullet;
pub mod list_style;
pub mod paragraph;

pub use body::{
    CT_TextBodyProperties, Coordinate32Value, NormalAutofit, TextAnchor, TextAutofit, TextError,
    TextVertical, TextWrap,
};
pub use bullet::{
    TextAutoNumber, TextAutoNumberScheme, TextBullet, TextBulletCharacter, TextBulletChoice,
    TextBulletColor, TextBulletSize, TextBulletSizeValue, TextNoBullet,
};
pub use list_style::CT_TextListStyle;
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
    list_style: Option<CT_TextListStyle>,
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
                    list_style = Some(CT_TextListStyle::from_xml(&raw)?);
                    boundary = boundary.max(2);
                }
                Event::Empty(element)
                    if matches_local_name(element.name().as_ref(), b"lstStyle") =>
                {
                    if list_style.is_some() {
                        return Err(TextError::DuplicateElement("lstStyle".to_owned()));
                    }
                    let raw = capture_empty_element(&element)?;
                    list_style = Some(CT_TextListStyle::from_xml(&raw)?);
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
            list_style.write_xml(writer)?;
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

    pub fn list_style(&self) -> Option<&CT_TextListStyle> {
        self.list_style.as_ref()
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
    use std::panic;

    use super::{CT_TextBody, CT_TextListStyle};

    #[test]
    fn text_body_reads_any_prefix_and_writes_the_fixed_a_prefix() {
        let xml = br#"<q:txBody><x:before/><q:bodyPr anchor="ctr"><q:noAutofit/></q:bodyPr><x:afterBody/><q:lstStyle><x:listChild/></q:lstStyle><x:beforeParagraph/><q:p><x:run>kept</x:run></q:p><x:afterParagraph/></q:txBody>"#;
        let body = CT_TextBody::from_xml(xml).unwrap();
        assert!(body.has_list_style());
        assert_eq!(body.paragraph_count(), 1);
        assert_eq!(body.to_xml().unwrap(), br#"<a:txBody><x:before/><a:bodyPr anchor="ctr"><a:noAutofit/></a:bodyPr><x:afterBody/><a:lstStyle><x:listChild/></a:lstStyle><x:beforeParagraph/><a:p><x:run>kept</x:run></a:p><x:afterParagraph/></a:txBody>"#);
    }

    #[test]
    fn schema_valid_text_body_using_all_nine_list_levels_round_trips_structurally() {
        let xml = br#"<q:txBody><q:bodyPr/><q:lstStyle><q:lvl1pPr lvl="0"><q:buChar char="*"/></q:lvl1pPr><q:lvl2pPr marL="100"><q:buAutoNum type="arabicPeriod" startAt="2"/></q:lvl2pPr><q:lvl3pPr><q:buNone/></q:lvl3pPr><q:lvl4pPr><q:defRPr sz="1200"/></q:lvl4pPr><q:lvl5pPr algn="ctr"/><x:extension x:id="5"><x:child>one &amp; two</x:child></x:extension><q:lvl6pPr marR="200"/><q:lvl7pPr><q:spcBef><q:spcPts val="600"/></q:spcBef></q:lvl7pPr><q:lvl8pPr><q:buSzPct val="125000"/><q:buFont typeface="Wingdings"/><q:buChar char="o"/></q:lvl8pPr><q:lvl9pPr indent="-100"/></q:lstStyle><q:p><q:pPr lvl="1"/><q:r><q:t xml:space="preserve"> item </q:t></q:r></q:p></q:txBody>"#;
        let expected = br#"<a:txBody><a:bodyPr/><a:lstStyle><a:lvl1pPr lvl="0"><a:buChar char="*"/></a:lvl1pPr><a:lvl2pPr marL="100"><a:buAutoNum type="arabicPeriod" startAt="2"/></a:lvl2pPr><a:lvl3pPr><a:buNone/></a:lvl3pPr><a:lvl4pPr><a:defRPr sz="1200"/></a:lvl4pPr><a:lvl5pPr algn="ctr"/><x:extension x:id="5"><x:child>one &amp; two</x:child></x:extension><a:lvl6pPr marR="200"/><a:lvl7pPr><a:spcBef><a:spcPts val="600"/></a:spcBef></a:lvl7pPr><a:lvl8pPr><a:buSzPct val="125000"/><a:buFont typeface="Wingdings"/><a:buChar char="o"/></a:lvl8pPr><a:lvl9pPr indent="-100"/></a:lstStyle><a:p><a:pPr lvl="1"/><a:r><a:t xml:space="preserve"> item </a:t></a:r></a:p></a:txBody>"#;

        let body = CT_TextBody::from_xml(xml).unwrap();
        let written = body.to_xml().unwrap();
        assert_eq!(written, expected);
        assert_eq!(CT_TextBody::from_xml(&written).unwrap(), body);
    }

    #[test]
    fn list_style_levels_write_in_ascending_schema_order() {
        let body = CT_TextBody::from_xml(
            br#"<q:txBody><q:bodyPr/><q:lstStyle><q:lvl9pPr indent="-9"/><q:lvl5pPr indent="-5"/><q:lvl1pPr indent="-1"/></q:lstStyle><q:p/></q:txBody>"#,
        )
        .unwrap();
        assert_eq!(
            body.to_xml().unwrap(),
            br#"<a:txBody><a:bodyPr/><a:lstStyle><a:lvl1pPr indent="-1"/><a:lvl5pPr indent="-5"/><a:lvl9pPr indent="-9"/></a:lstStyle><a:p/></a:txBody>"#
        );
    }

    #[test]
    fn unknown_list_style_children_round_trip_byte_for_byte() {
        let body = CT_TextBody::from_xml(
            br#"<q:txBody><q:bodyPr/><q:lstStyle><x:before x:id="1"/><q:lvl1pPr/><x:between><x:nested>one &amp; two</x:nested><!--note--></x:between><q:lvl2pPr/><x:after x:id="9"/></q:lstStyle><q:p/></q:txBody>"#,
        )
        .unwrap();
        assert_eq!(
            body.to_xml().unwrap(),
            br#"<a:txBody><a:bodyPr/><a:lstStyle><x:before x:id="1"/><a:lvl1pPr/><x:between><x:nested>one &amp; two</x:nested><!--note--></x:between><a:lvl2pPr/><x:after x:id="9"/></a:lstStyle><a:p/></a:txBody>"#
        );
    }

    #[test]
    fn list_style_rejects_nested_fixed_prefix_rebinding() {
        let xml = br#"<p:defaultTextStyle xmlns:p="urn:presentation" xmlns:d="http://schemas.openxmlformats.org/drawingml/2006/main"><d:lvl1pPr xmlns:a="urn:producer"><a:raw/></d:lvl1pPr></p:defaultTextStyle>"#;
        assert!(CT_TextListStyle::from_xml(xml).is_err());
    }

    #[test]
    fn invalid_list_levels_return_errors_without_panicking() {
        let cases: &[&[u8]] = &[
            br#"<q:txBody><q:bodyPr/><q:lstStyle><q:lvl0pPr/></q:lstStyle><q:p/></q:txBody>"#,
            br#"<q:txBody><q:bodyPr/><q:lstStyle><q:lvl10pPr/></q:lstStyle><q:p/></q:txBody>"#,
            br#"<q:txBody><q:bodyPr/><q:lstStyle><q:lvl01pPr/></q:lstStyle><q:p/></q:txBody>"#,
            br#"<q:txBody><q:bodyPr/><q:lstStyle><q:lvl1pPr/><q:lvl1pPr/></q:lstStyle><q:p/></q:txBody>"#,
            br#"<q:txBody><q:bodyPr/><q:lstStyle><q:lvl4pPr lvl="9"/></q:lstStyle><q:p/></q:txBody>"#,
            br#"<q:txBody><q:bodyPr/><q:lstStyle><q:lvl7pPr><q:buChar/></q:lvl7pPr></q:lstStyle><q:p/></q:txBody>"#,
        ];

        for xml in cases {
            let result = panic::catch_unwind(|| CT_TextBody::from_xml(xml));
            assert!(result.is_ok(), "list-style parser panicked");
            assert!(result.unwrap().is_err(), "invalid list level parsed");
        }
    }
}

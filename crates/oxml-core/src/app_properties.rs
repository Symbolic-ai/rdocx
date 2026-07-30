//! Application-specific properties from `docProps/app.xml`.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::collections::HashSet;
use std::io::Write;

use crate::error::{OxmlError, Result};
use crate::raw_xml::{capture_element, capture_empty_element};
use crate::xml::{extra_namespace_declarations, local_name};
use crate::xml_text::read_element_text;

const EXTENDED_PROPERTIES_NS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties";
const VARIANT_TYPES_NS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum KnownProperty {
    Template,
    Manager,
    Company,
    Pages,
    Words,
    Characters,
    PresentationFormat,
    Lines,
    Paragraphs,
    Slides,
    Notes,
    TotalTime,
    HiddenSlides,
    MultimediaClips,
    ScaleCrop,
    LinksUpToDate,
    CharactersWithSpaces,
    SharedDocument,
    HyperlinksChanged,
    Application,
    ApplicationVersion,
}

const CANONICAL_ORDER: &[KnownProperty] = &[
    KnownProperty::Template,
    KnownProperty::Manager,
    KnownProperty::Company,
    KnownProperty::Pages,
    KnownProperty::Words,
    KnownProperty::Characters,
    KnownProperty::PresentationFormat,
    KnownProperty::Lines,
    KnownProperty::Paragraphs,
    KnownProperty::Slides,
    KnownProperty::Notes,
    KnownProperty::TotalTime,
    KnownProperty::HiddenSlides,
    KnownProperty::MultimediaClips,
    KnownProperty::ScaleCrop,
    KnownProperty::LinksUpToDate,
    KnownProperty::CharactersWithSpaces,
    KnownProperty::SharedDocument,
    KnownProperty::HyperlinksChanged,
    KnownProperty::Application,
    KnownProperty::ApplicationVersion,
];

impl KnownProperty {
    fn from_name(name: &[u8]) -> Option<Self> {
        match name {
            b"Template" => Some(Self::Template),
            b"Manager" => Some(Self::Manager),
            b"Company" => Some(Self::Company),
            b"Pages" => Some(Self::Pages),
            b"Words" => Some(Self::Words),
            b"Characters" => Some(Self::Characters),
            b"PresentationFormat" => Some(Self::PresentationFormat),
            b"Lines" => Some(Self::Lines),
            b"Paragraphs" => Some(Self::Paragraphs),
            b"Slides" => Some(Self::Slides),
            b"Notes" => Some(Self::Notes),
            b"TotalTime" => Some(Self::TotalTime),
            b"HiddenSlides" => Some(Self::HiddenSlides),
            b"MMClips" => Some(Self::MultimediaClips),
            b"ScaleCrop" => Some(Self::ScaleCrop),
            b"LinksUpToDate" => Some(Self::LinksUpToDate),
            b"CharactersWithSpaces" => Some(Self::CharactersWithSpaces),
            b"SharedDoc" => Some(Self::SharedDocument),
            b"HyperlinksChanged" => Some(Self::HyperlinksChanged),
            b"Application" => Some(Self::Application),
            b"AppVersion" => Some(Self::ApplicationVersion),
            _ => None,
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Self::Template => "Template",
            Self::Manager => "Manager",
            Self::Company => "Company",
            Self::Pages => "Pages",
            Self::Words => "Words",
            Self::Characters => "Characters",
            Self::PresentationFormat => "PresentationFormat",
            Self::Lines => "Lines",
            Self::Paragraphs => "Paragraphs",
            Self::Slides => "Slides",
            Self::Notes => "Notes",
            Self::TotalTime => "TotalTime",
            Self::HiddenSlides => "HiddenSlides",
            Self::MultimediaClips => "MMClips",
            Self::ScaleCrop => "ScaleCrop",
            Self::LinksUpToDate => "LinksUpToDate",
            Self::CharactersWithSpaces => "CharactersWithSpaces",
            Self::SharedDocument => "SharedDoc",
            Self::HyperlinksChanged => "HyperlinksChanged",
            Self::Application => "Application",
            Self::ApplicationVersion => "AppVersion",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ChildOrder {
    Known(KnownProperty),
    Raw(usize),
}

/// Shared application properties from a Word or PowerPoint package.
#[derive(Debug, Clone, Default)]
pub struct AppProperties {
    pub template: Option<String>,
    pub manager: Option<String>,
    pub company: Option<String>,
    pub pages: Option<i32>,
    pub words: Option<i32>,
    pub characters: Option<i32>,
    pub presentation_format: Option<String>,
    pub lines: Option<i32>,
    pub paragraphs: Option<i32>,
    pub slides: Option<i32>,
    pub notes: Option<i32>,
    pub total_time: Option<i32>,
    pub hidden_slides: Option<i32>,
    pub multimedia_clips: Option<i32>,
    pub scale_crop: Option<bool>,
    pub links_up_to_date: Option<bool>,
    pub characters_with_spaces: Option<i32>,
    pub shared_document: Option<bool>,
    pub hyperlinks_changed: Option<bool>,
    pub application: Option<String>,
    pub application_version: Option<String>,
    child_order: Vec<ChildOrder>,
    extra_xml: Vec<Vec<u8>>,
    extra_namespaces: Vec<(String, String)>,
}

impl PartialEq for AppProperties {
    fn eq(&self, other: &Self) -> bool {
        self.template == other.template
            && self.manager == other.manager
            && self.company == other.company
            && self.pages == other.pages
            && self.words == other.words
            && self.characters == other.characters
            && self.presentation_format == other.presentation_format
            && self.lines == other.lines
            && self.paragraphs == other.paragraphs
            && self.slides == other.slides
            && self.notes == other.notes
            && self.total_time == other.total_time
            && self.hidden_slides == other.hidden_slides
            && self.multimedia_clips == other.multimedia_clips
            && self.scale_crop == other.scale_crop
            && self.links_up_to_date == other.links_up_to_date
            && self.characters_with_spaces == other.characters_with_spaces
            && self.shared_document == other.shared_document
            && self.hyperlinks_changed == other.hyperlinks_changed
            && self.application == other.application
            && self.application_version == other.application_version
            && self.extra_xml == other.extra_xml
            && self.extra_namespaces == other.extra_namespaces
    }
}

impl AppProperties {
    /// Parse a `docProps/app.xml` part.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        let mut properties = Self::default();
        let mut seen = HashSet::new();
        let mut root_open = false;
        let mut root_closed = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) => {
                    let qualified_name = element.name();
                    let name = local_name(qualified_name.as_ref());
                    if name == b"Properties" {
                        if root_open || root_closed {
                            return Err(OxmlError::UnexpectedElement("Properties".to_owned()));
                        }
                        root_open = true;
                        properties
                            .extra_namespaces
                            .extend(extra_namespace_declarations(element)?);
                    } else if !root_open {
                        return Err(OxmlError::UnexpectedElement(
                            String::from_utf8_lossy(name).into_owned(),
                        ));
                    } else if let Some(property) = KnownProperty::from_name(name) {
                        if !seen.insert(property) {
                            return Err(OxmlError::InvalidValue(format!(
                                "duplicate application property {}",
                                property.tag()
                            )));
                        }
                        let text = read_element_text(&mut reader, element.name());
                        properties.set_text(property, text)?;
                        properties.child_order.push(ChildOrder::Known(property));
                    } else {
                        let raw = capture_element(&mut reader, element)?;
                        let index = properties.extra_xml.len();
                        properties.extra_xml.push(raw);
                        properties.child_order.push(ChildOrder::Raw(index));
                    }
                }
                Ok(Event::Empty(ref element)) => {
                    let qualified_name = element.name();
                    let name = local_name(qualified_name.as_ref());
                    if name == b"Properties" {
                        if root_open || root_closed {
                            return Err(OxmlError::UnexpectedElement("Properties".to_owned()));
                        }
                        root_closed = true;
                    } else if !root_open {
                        return Err(OxmlError::UnexpectedElement(
                            String::from_utf8_lossy(name).into_owned(),
                        ));
                    } else if let Some(property) = KnownProperty::from_name(name) {
                        if !seen.insert(property) {
                            return Err(OxmlError::InvalidValue(format!(
                                "duplicate application property {}",
                                property.tag()
                            )));
                        }
                        properties.set_text(property, String::new())?;
                        properties.child_order.push(ChildOrder::Known(property));
                    } else {
                        let index = properties.extra_xml.len();
                        properties.extra_xml.push(capture_empty_element(element)?);
                        properties.child_order.push(ChildOrder::Raw(index));
                    }
                }
                Ok(Event::End(ref element))
                    if local_name(element.name().as_ref()) == b"Properties" =>
                {
                    if !root_open {
                        return Err(OxmlError::UnexpectedElement("Properties".to_owned()));
                    }
                    root_open = false;
                    root_closed = true;
                }
                Ok(Event::Eof) => break,
                Err(error) => return Err(error.into()),
                _ => {}
            }
            buf.clear();
        }

        if root_closed {
            Ok(properties)
        } else {
            Err(OxmlError::MissingElement("Properties root".to_owned()))
        }
    }

    /// Serialize a `docProps/app.xml` part.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new(Vec::new());
        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut root = BytesStart::new("Properties");
        root.push_attribute(("xmlns", EXTENDED_PROPERTIES_NS));
        root.push_attribute(("xmlns:vt", VARIANT_TYPES_NS));
        for (name, value) in &self.extra_namespaces {
            root.push_attribute((name.as_str(), value.as_str()));
        }
        writer.write_event(Event::Start(root))?;

        let mut written = HashSet::new();
        for child in &self.child_order {
            match child {
                ChildOrder::Known(property) if written.insert(*property) => {
                    self.write_property(&mut writer, *property)?;
                }
                ChildOrder::Known(_) => {}
                ChildOrder::Raw(index) => {
                    if let Some(raw) = self.extra_xml.get(*index) {
                        writer.get_mut().write_all(raw)?;
                    }
                }
            }
        }
        for property in CANONICAL_ORDER {
            if written.insert(*property) {
                self.write_property(&mut writer, *property)?;
            }
        }

        writer.write_event(Event::End(BytesEnd::new("Properties")))?;
        Ok(writer.into_inner())
    }

    fn set_text(&mut self, property: KnownProperty, text: String) -> Result<()> {
        match property {
            KnownProperty::Template => self.template = Some(text),
            KnownProperty::Manager => self.manager = Some(text),
            KnownProperty::Company => self.company = Some(text),
            KnownProperty::PresentationFormat => self.presentation_format = Some(text),
            KnownProperty::Application => self.application = Some(text),
            KnownProperty::ApplicationVersion => self.application_version = Some(text),
            KnownProperty::Pages => self.pages = Some(parse_i32(property, &text)?),
            KnownProperty::Words => self.words = Some(parse_i32(property, &text)?),
            KnownProperty::Characters => self.characters = Some(parse_i32(property, &text)?),
            KnownProperty::Lines => self.lines = Some(parse_i32(property, &text)?),
            KnownProperty::Paragraphs => self.paragraphs = Some(parse_i32(property, &text)?),
            KnownProperty::Slides => self.slides = Some(parse_i32(property, &text)?),
            KnownProperty::Notes => self.notes = Some(parse_i32(property, &text)?),
            KnownProperty::TotalTime => self.total_time = Some(parse_i32(property, &text)?),
            KnownProperty::HiddenSlides => self.hidden_slides = Some(parse_i32(property, &text)?),
            KnownProperty::MultimediaClips => {
                self.multimedia_clips = Some(parse_i32(property, &text)?)
            }
            KnownProperty::CharactersWithSpaces => {
                self.characters_with_spaces = Some(parse_i32(property, &text)?)
            }
            KnownProperty::ScaleCrop => self.scale_crop = Some(parse_bool(property, &text)?),
            KnownProperty::LinksUpToDate => {
                self.links_up_to_date = Some(parse_bool(property, &text)?)
            }
            KnownProperty::SharedDocument => {
                self.shared_document = Some(parse_bool(property, &text)?)
            }
            KnownProperty::HyperlinksChanged => {
                self.hyperlinks_changed = Some(parse_bool(property, &text)?)
            }
        }
        Ok(())
    }

    fn write_property(&self, writer: &mut Writer<Vec<u8>>, property: KnownProperty) -> Result<()> {
        match property {
            KnownProperty::Template => write_text(writer, property.tag(), self.template.as_deref()),
            KnownProperty::Manager => write_text(writer, property.tag(), self.manager.as_deref()),
            KnownProperty::Company => write_text(writer, property.tag(), self.company.as_deref()),
            KnownProperty::PresentationFormat => {
                write_text(writer, property.tag(), self.presentation_format.as_deref())
            }
            KnownProperty::Application => {
                write_text(writer, property.tag(), self.application.as_deref())
            }
            KnownProperty::ApplicationVersion => {
                write_text(writer, property.tag(), self.application_version.as_deref())
            }
            KnownProperty::Pages => write_i32(writer, property.tag(), self.pages),
            KnownProperty::Words => write_i32(writer, property.tag(), self.words),
            KnownProperty::Characters => write_i32(writer, property.tag(), self.characters),
            KnownProperty::Lines => write_i32(writer, property.tag(), self.lines),
            KnownProperty::Paragraphs => write_i32(writer, property.tag(), self.paragraphs),
            KnownProperty::Slides => write_i32(writer, property.tag(), self.slides),
            KnownProperty::Notes => write_i32(writer, property.tag(), self.notes),
            KnownProperty::TotalTime => write_i32(writer, property.tag(), self.total_time),
            KnownProperty::HiddenSlides => write_i32(writer, property.tag(), self.hidden_slides),
            KnownProperty::MultimediaClips => {
                write_i32(writer, property.tag(), self.multimedia_clips)
            }
            KnownProperty::CharactersWithSpaces => {
                write_i32(writer, property.tag(), self.characters_with_spaces)
            }
            KnownProperty::ScaleCrop => write_bool(writer, property.tag(), self.scale_crop),
            KnownProperty::LinksUpToDate => {
                write_bool(writer, property.tag(), self.links_up_to_date)
            }
            KnownProperty::SharedDocument => {
                write_bool(writer, property.tag(), self.shared_document)
            }
            KnownProperty::HyperlinksChanged => {
                write_bool(writer, property.tag(), self.hyperlinks_changed)
            }
        }
    }
}

fn parse_i32(property: KnownProperty, text: &str) -> Result<i32> {
    text.trim().parse().map_err(|_| {
        OxmlError::InvalidValue(format!(
            "{} must be an integer, got {text:?}",
            property.tag()
        ))
    })
}

fn parse_bool(property: KnownProperty, text: &str) -> Result<bool> {
    match text.trim() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(OxmlError::InvalidValue(format!(
            "{} must be a Boolean, got {text:?}",
            property.tag()
        ))),
    }
}

fn write_text(writer: &mut Writer<Vec<u8>>, tag: &str, value: Option<&str>) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

fn write_i32(writer: &mut Writer<Vec<u8>>, tag: &str, value: Option<i32>) -> Result<()> {
    write_text(
        writer,
        tag,
        value.map(|number| number.to_string()).as_deref(),
    )
}

fn write_bool(writer: &mut Writer<Vec<u8>>, tag: &str, value: Option<bool>) -> Result<()> {
    write_text(writer, tag, value.map(|flag| flag.to_string()).as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_app_properties_round_trip_without_presentation_fields() {
        let xml = br#"<?xml version="1.0"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Template>Normal.dotm</Template><Pages>3</Pages><Words>240</Words>
  <Characters>1200</Characters><Lines>20</Lines><Paragraphs>8</Paragraphs>
  <CharactersWithSpaces>1439</CharactersWithSpaces><Application>Word</Application>
</Properties>"#;
        let properties = AppProperties::from_xml(xml).unwrap();
        assert_eq!(properties.pages, Some(3));
        assert_eq!(properties.words, Some(240));
        assert_eq!(properties.characters_with_spaces, Some(1439));
        assert_eq!(properties.presentation_format, None);
        assert_eq!(properties.slides, None);
        assert_eq!(properties.notes, None);
        assert_eq!(properties.hidden_slides, None);
        assert_eq!(properties.multimedia_clips, None);

        let output = properties.to_xml().unwrap();
        let output_text = std::str::from_utf8(&output).unwrap();
        for absent in [
            "PresentationFormat",
            "Slides",
            "Notes",
            "HiddenSlides",
            "MMClips",
        ] {
            assert!(!output_text.contains(&format!("<{absent}>")));
        }
        assert_eq!(AppProperties::from_xml(&output).unwrap(), properties);
    }

    #[test]
    fn powerpoint_app_properties_round_trip_without_word_fields() {
        let xml = br#"<ep:Properties xmlns:ep="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <ep:PresentationFormat>On-screen Show (16:9)</ep:PresentationFormat>
  <ep:Slides>12</ep:Slides><ep:Notes>2</ep:Notes><ep:HiddenSlides>1</ep:HiddenSlides>
  <ep:MMClips>3</ep:MMClips><ep:ScaleCrop>1</ep:ScaleCrop>
</ep:Properties>"#;
        let properties = AppProperties::from_xml(xml).unwrap();
        assert_eq!(
            properties.presentation_format.as_deref(),
            Some("On-screen Show (16:9)")
        );
        assert_eq!(properties.slides, Some(12));
        assert_eq!(properties.scale_crop, Some(true));
        assert_eq!(properties.pages, None);
        assert_eq!(properties.words, None);
        assert_eq!(properties.characters, None);
        assert_eq!(properties.lines, None);
        assert_eq!(properties.paragraphs, None);
        assert_eq!(properties.characters_with_spaces, None);

        let output = properties.to_xml().unwrap();
        let output_text = std::str::from_utf8(&output).unwrap();
        for absent in ["Pages", "Words", "Characters", "Lines", "Paragraphs"] {
            assert!(!output_text.contains(&format!("<{absent}>")));
        }
        assert_eq!(AppProperties::from_xml(&output).unwrap(), properties);
    }

    #[test]
    fn unknown_app_property_subtree_is_preserved_verbatim() {
        let xml = br#"<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:x="urn:test"><Application>rdocx</Application><x:Future x:id="7"><x:value>one &amp; two</x:value></x:Future><AppVersion>1.0</AppVersion></Properties>"#;
        let properties = AppProperties::from_xml(xml).unwrap();
        let output = properties.to_xml().unwrap();
        let raw = br#"<x:Future x:id="7"><x:value>one &amp; two</x:value></x:Future>"#;
        assert!(output.windows(raw.len()).any(|window| window == raw));
        let text = std::str::from_utf8(&output).unwrap();
        assert!(text.find("<Application>").unwrap() < text.find("<x:Future").unwrap());
        assert!(text.find("<x:Future").unwrap() < text.find("<AppVersion>").unwrap());
    }

    #[test]
    fn newly_constructed_properties_round_trip_as_equal() {
        let properties = AppProperties {
            application: Some("rdocx".to_owned()),
            pages: Some(2),
            scale_crop: Some(false),
            ..Default::default()
        };

        let output = properties.to_xml().unwrap();
        assert_eq!(AppProperties::from_xml(&output).unwrap(), properties);
    }

    #[test]
    fn malformed_app_property_roots_are_rejected() {
        assert!(AppProperties::from_xml(b"").is_err());
        assert!(AppProperties::from_xml(b"<Wrong><Pages>1</Pages></Wrong>").is_err());
        assert!(AppProperties::from_xml(b"<Properties><Pages>1</Pages>").is_err());
        assert!(AppProperties::from_xml(b"<Properties/><Properties/>").is_err());
    }
}

//! Parsing and writing of `[Content_Types].xml`.

use std::collections::HashMap;

use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};

use oxml_core::xml::{StrictXmlCursor, StrictXmlDocument, StrictXmlElement, StrictXmlNode};

use crate::error::{OpcError, Result};

pub const RELATIONSHIPS: &str = "application/vnd.openxmlformats-package.relationships+xml";
pub const XML: &str = "application/xml";

const CONTENT_TYPES_NS: &str = "http://schemas.openxmlformats.org/package/2006/content-types";

pub const CORE_PROPERTIES: &str = "application/vnd.openxmlformats-package.core-properties+xml";
pub const EXTENDED_PROPERTIES: &str =
    "application/vnd.openxmlformats-officedocument.extended-properties+xml";
pub const CUSTOM_PROPERTIES: &str =
    "application/vnd.openxmlformats-officedocument.custom-properties+xml";
pub const THEME: &str = "application/vnd.openxmlformats-officedocument.theme+xml";
pub const CHART: &str = "application/vnd.openxmlformats-officedocument.drawingml.chart+xml";

pub const PRESENTATION: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml";
pub const SLIDESHOW: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideshow.main+xml";
pub const SLIDE: &str = "application/vnd.openxmlformats-officedocument.presentationml.slide+xml";
pub const SLIDE_LAYOUT: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml";
pub const SLIDE_MASTER: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml";
pub const NOTES_SLIDE: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml";
pub const NOTES_MASTER: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml";
pub const PRES_PROPS: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.presProps+xml";
pub const VIEW_PROPS: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.viewProps+xml";
pub const TABLE_STYLES: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.tableStyles+xml";
pub const HANDOUT_MASTER: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.handoutMaster+xml";

pub const WORKBOOK: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml";
pub const EMBEDDED_WORKBOOK: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
pub const WORKSHEET: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml";
pub const SHARED_STRINGS: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml";
pub const STYLES: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml";

/// A single content type entry, either a Default by extension or an Override by part name.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Default {
        extension: String,
        content_type: String,
    },
    Override {
        part_name: String,
        content_type: String,
    },
}

/// Parsed `[Content_Types].xml`.
#[derive(Debug, Clone)]
pub struct ContentTypes {
    pub defaults: HashMap<String, String>,
    pub overrides: HashMap<String, String>,
}

impl ContentTypes {
    /// Parse from XML bytes.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let root = StrictXmlDocument::parse(xml)
            .map_err(|_| OpcError::InvalidContentTypes)?
            .into_root();
        if !root.is_named(Some(CONTENT_TYPES_NS), "Types") {
            return Err(OpcError::InvalidContentTypes);
        }
        let parsed = root
            .parse(|cursor| {
                let mut content_types = Self {
                    defaults: HashMap::new(),
                    overrides: HashMap::new(),
                };
                for index in 0..cursor.child_slots() {
                    let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                        continue;
                    };
                    let kind = if child.is_named(Some(CONTENT_TYPES_NS), "Default") {
                        Some("Default")
                    } else if child.is_named(Some(CONTENT_TYPES_NS), "Override") {
                        Some("Override")
                    } else {
                        None
                    };
                    let Some(kind) = kind else {
                        continue;
                    };
                    let child = cursor
                        .take_child(index)
                        .and_then(StrictXmlNode::into_element)
                        .ok_or_else(invalid_content_type_value)?;
                    let (key, value) = parse_content_type(child, kind)?;
                    if kind == "Default" {
                        content_types.defaults.insert(key, value);
                    } else {
                        content_types.overrides.insert(key, value);
                    }
                }
                Ok(content_types)
            })
            .map_err(|_| OpcError::InvalidContentTypes)?;
        if !parsed.leftovers.is_empty() {
            return Err(OpcError::InvalidContentTypes);
        }
        Ok(parsed.value)
    }

    /// Serialize to XML bytes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut types_start = BytesStart::new("Types");
        types_start.push_attribute((
            "xmlns",
            "http://schemas.openxmlformats.org/package/2006/content-types",
        ));
        writer.write_event(Event::Start(types_start))?;

        // Write defaults sorted for deterministic output
        let mut sorted_defaults: Vec<_> = self.defaults.iter().collect();
        sorted_defaults.sort_by_key(|(k, _)| (*k).clone());
        for (ext, ct) in sorted_defaults {
            let mut elem = BytesStart::new("Default");
            elem.push_attribute(("Extension", ext.as_str()));
            elem.push_attribute(("ContentType", ct.as_str()));
            writer.write_event(Event::Empty(elem))?;
        }

        // Write overrides sorted for deterministic output
        let mut sorted_overrides: Vec<_> = self.overrides.iter().collect();
        sorted_overrides.sort_by_key(|(k, _)| (*k).clone());
        for (pn, ct) in sorted_overrides {
            let mut elem = BytesStart::new("Override");
            elem.push_attribute(("PartName", pn.as_str()));
            elem.push_attribute(("ContentType", ct.as_str()));
            writer.write_event(Event::Empty(elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("Types")))?;

        Ok(writer.into_inner())
    }

    /// Look up the content type for a given part name.
    pub fn content_type_for(&self, part_name: &str) -> Option<&str> {
        // Check overrides first
        if let Some(ct) = self.overrides.get(part_name) {
            return Some(ct.as_str());
        }
        // Fall back to defaults by extension
        if let Some(dot_pos) = part_name.rfind('.') {
            let ext = &part_name[dot_pos + 1..];
            if let Some(ct) = self.defaults.get(ext) {
                return Some(ct.as_str());
            }
        }
        None
    }

    /// Add a default content type for an extension (e.g., "png" -> "image/png").
    pub fn add_default(&mut self, extension: &str, content_type: &str) {
        self.defaults
            .entry(extension.to_string())
            .or_insert_with(|| content_type.to_string());
    }

    /// Add an override content type for a specific part name.
    pub fn add_override(&mut self, part_name: &str, content_type: &str) {
        self.overrides
            .insert(part_name.to_string(), content_type.to_string());
    }

    /// Create the minimal content types shared by every OPC package.
    pub fn minimal() -> Self {
        let mut defaults = HashMap::new();
        defaults.insert("rels".to_string(), RELATIONSHIPS.to_string());
        defaults.insert("xml".to_string(), XML.to_string());

        ContentTypes {
            defaults,
            overrides: HashMap::new(),
        }
    }
}

fn parse_content_type(
    element: StrictXmlElement,
    kind: &str,
) -> oxml_core::Result<(String, String)> {
    let parsed = element.parse(|cursor| parse_content_type_attributes(cursor, kind))?;
    if !parsed.leftovers.is_empty() {
        return Err(invalid_content_type_value());
    }
    Ok(parsed.value)
}

fn parse_content_type_attributes(
    cursor: &mut StrictXmlCursor,
    kind: &str,
) -> oxml_core::Result<(String, String)> {
    let key_name = if kind == "Default" {
        "Extension"
    } else {
        "PartName"
    };
    let key = cursor
        .take_attribute(None, key_name)
        .ok_or_else(invalid_content_type_value)?;
    let value = cursor
        .take_attribute(None, "ContentType")
        .ok_or_else(invalid_content_type_value)?;
    Ok((key, value))
}

fn invalid_content_type_value() -> oxml_core::OxmlError {
    oxml_core::OxmlError::InvalidValue("invalid content type XML".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn docx_content_types() -> ContentTypes {
        let mut content_types = ContentTypes::minimal();
        content_types.add_override(
            "/word/document.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        );
        content_types.add_override(
            "/word/styles.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
        );
        content_types
    }

    #[test]
    fn minimal_content_types_contain_only_universal_defaults() {
        let content_types = ContentTypes::minimal();

        assert_eq!(content_types.defaults.len(), 2);
        assert_eq!(
            content_types.defaults.get("rels").map(String::as_str),
            Some("application/vnd.openxmlformats-package.relationships+xml")
        );
        assert_eq!(
            content_types.defaults.get("xml").map(String::as_str),
            Some("application/xml")
        );
        assert!(content_types.overrides.is_empty());
    }

    #[test]
    fn round_trip_content_types() {
        let ct = docx_content_types();
        let xml = ct.to_xml().unwrap();
        let parsed = ContentTypes::from_xml(&xml).unwrap();
        assert_eq!(parsed.defaults.len(), ct.defaults.len());
        assert_eq!(parsed.overrides.len(), ct.overrides.len());
        assert_eq!(
            parsed.content_type_for("/word/document.xml"),
            Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
            )
        );
    }

    #[test]
    fn spelling_and_prefix_do_not_change_content_type_semantics() {
        let variants = [
            br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#.as_slice(),
            br#"<c:Types xmlns:c="http://schemas.openxmlformats.org/package/2006/content-types"><c:Default Extension="xml" ContentType="application/xml"></c:Default></c:Types>"#.as_slice(),
        ];
        let parsed = variants.map(ContentTypes::from_xml).map(Result::unwrap);
        assert_eq!(parsed[0].defaults, parsed[1].defaults);
        assert_eq!(parsed[0].overrides, parsed[1].overrides);
    }

    #[test]
    fn foreign_or_unconsumed_content_type_semantics_are_rejected() {
        for xml in [
            br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types" xmlns:x="urn:foreign"><x:Default Extension="xml" ContentType="application/xml"/></Types>"#.as_slice(),
            br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml" Extra="value"/></Types>"#.as_slice(),
            br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"><Override PartName="/word/document.xml" ContentType="application/xml"/></Default></Types>"#.as_slice(),
        ] {
            assert!(ContentTypes::from_xml(xml).is_err(), "{xml:?}");
        }
    }

    #[test]
    fn lookup_by_extension() {
        let ct = docx_content_types();
        assert_eq!(
            ct.content_type_for("/word/_rels/document.xml.rels"),
            Some("application/vnd.openxmlformats-package.relationships+xml")
        );
    }
}

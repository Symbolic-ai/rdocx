//! Style elements: `CT_Styles`, `CT_Style`, `CT_DocDefaults`.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use oxml_core::xml::{
    StrictXmlCompleteness, StrictXmlCursor, StrictXmlDocument, StrictXmlElement, StrictXmlNode,
    parse_reader_element, parse_reader_started_element,
};

use crate::error::{OxmlError, Result};
use crate::namespace::W_NS;
use crate::properties::{CT_PPr, CT_RPr};
use crate::raw_xml::NamespaceContext;

fn take_element(
    cursor: &mut StrictXmlCursor,
    index: usize,
    description: &str,
) -> Result<StrictXmlElement> {
    cursor
        .take_child(index)
        .and_then(StrictXmlNode::into_element)
        .ok_or_else(|| OxmlError::MissingElement(description.to_string()))
}

/// The type of a style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleType {
    Paragraph,
    Character,
    Table,
    Numbering,
}

impl StyleType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "paragraph" => Ok(StyleType::Paragraph),
            "character" => Ok(StyleType::Character),
            "table" => Ok(StyleType::Table),
            "numbering" => Ok(StyleType::Numbering),
            _ => Err(OxmlError::InvalidValue(format!("invalid style type: {s}"))),
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            StyleType::Paragraph => "paragraph",
            StyleType::Character => "character",
            StyleType::Table => "table",
            StyleType::Numbering => "numbering",
        }
    }
}

/// `CT_Style` — A single style definition.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_Style {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub style_id: String,
    pub style_type: StyleType,
    pub name: Option<String>,
    pub based_on: Option<String>,
    pub next_style: Option<String>,
    pub is_default: bool,
    pub ppr: Option<CT_PPr>,
    pub rpr: Option<CT_RPr>,
}

#[allow(non_snake_case)]
impl CT_Style {
    fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let style_id = cursor
                .take_attribute(Some(W_NS), "styleId")
                .unwrap_or_default();
            let style_type = cursor
                .take_attribute(Some(W_NS), "type")
                .map(|value| StyleType::from_str(&value))
                .transpose()?
                .unwrap_or(StyleType::Paragraph);
            let is_default = match cursor.attribute(Some(W_NS), "default") {
                Some("1" | "true") => {
                    cursor.take_attribute(Some(W_NS), "default");
                    true
                }
                Some("0" | "false") => {
                    cursor.take_attribute(Some(W_NS), "default");
                    false
                }
                _ => false,
            };
            let mut style = Self {
                completeness: StrictXmlCompleteness::default(),
                style_id,
                style_type,
                name: None,
                based_on: None,
                next_style: None,
                is_default,
                ppr: None,
                rpr: None,
            };
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                let local = ["name", "basedOn", "next", "pPr", "rPr"]
                    .into_iter()
                    .find(|local| child.is_named(Some(W_NS), local));
                let Some(local) = local else {
                    continue;
                };
                let child = take_element(cursor, index, local)?;
                let completeness = match local {
                    "pPr" => {
                        let properties = CT_PPr::from_strict_xml(child)?;
                        let completeness = properties.completeness.clone();
                        style.ppr = Some(properties);
                        completeness
                    }
                    "rPr" => {
                        let properties = CT_RPr::from_strict_xml(child)?;
                        let completeness = properties.completeness.clone();
                        style.rpr = Some(properties);
                        completeness
                    }
                    _ => {
                        let parsed_value =
                            child.parse(|cursor| Ok(cursor.take_attribute(Some(W_NS), "val")))?;
                        let (value, leftovers) = parsed_value.into_parts();
                        match local {
                            "name" => style.name = value,
                            "basedOn" => style.based_on = value,
                            "next" => style.next_style = value,
                            _ => unreachable!(),
                        }
                        StrictXmlCompleteness::from_leftovers(leftovers)
                    }
                };
                descendants.push(completeness);
            }
            Ok(style)
        })?;
        let (mut style, leftovers) = parsed.into_parts();
        style.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(style)
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>, attrs: &BytesStart) -> Result<Self> {
        let context = NamespaceContext::default().with_element(attrs);
        let element = parse_reader_started_element(reader, &context, Some(W_NS), "style", attrs)?;
        Self::from_strict_xml(element)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut e = BytesStart::new("w:style");
        e.push_attribute(("w:type", self.style_type.to_str()));
        e.push_attribute(("w:styleId", self.style_id.as_str()));
        if self.is_default {
            e.push_attribute(("w:default", "1"));
        }
        writer.write_event(Event::Start(e))?;

        if let Some(ref name) = self.name {
            let mut ne = BytesStart::new("w:name");
            ne.push_attribute(("w:val", name.as_str()));
            writer.write_event(Event::Empty(ne))?;
        }

        if let Some(ref based_on) = self.based_on {
            let mut be = BytesStart::new("w:basedOn");
            be.push_attribute(("w:val", based_on.as_str()));
            writer.write_event(Event::Empty(be))?;
        }

        if let Some(ref next) = self.next_style {
            let mut ne = BytesStart::new("w:next");
            ne.push_attribute(("w:val", next.as_str()));
            writer.write_event(Event::Empty(ne))?;
        }

        if let Some(ref ppr) = self.ppr {
            ppr.to_xml(writer)?;
        }
        if let Some(ref rpr) = self.rpr {
            rpr.to_xml(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:style")))?;
        Ok(())
    }
}

impl Default for CT_Style {
    fn default() -> Self {
        Self {
            completeness: StrictXmlCompleteness::default(),
            style_id: String::new(),
            style_type: StyleType::Paragraph,
            name: None,
            based_on: None,
            next_style: None,
            is_default: false,
            ppr: None,
            rpr: None,
        }
    }
}

/// `CT_DocDefaults` — Document-level default properties.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_DocDefaults {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub rpr: Option<CT_RPr>,
    pub ppr: Option<CT_PPr>,
}

#[allow(non_snake_case)]
impl CT_DocDefaults {
    fn from_strict_xml(element: StrictXmlElement) -> Result<Self> {
        let mut descendants = Vec::new();
        let parsed = element.parse(|cursor| {
            let mut defaults = Self::default();
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                let local = if child.is_named(Some(W_NS), "rPrDefault") {
                    Some("rPrDefault")
                } else if child.is_named(Some(W_NS), "pPrDefault") {
                    Some("pPrDefault")
                } else {
                    None
                };
                let Some(local) = local else {
                    continue;
                };
                let wrapper = take_element(cursor, index, local)?;
                let mut wrapper_descendants = Vec::new();
                let parsed_wrapper = wrapper.parse(|cursor| {
                    for index in 0..cursor.child_slots() {
                        let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                            continue;
                        };
                        let matches = (local == "rPrDefault" && child.is_named(Some(W_NS), "rPr"))
                            || (local == "pPrDefault" && child.is_named(Some(W_NS), "pPr"));
                        if !matches {
                            continue;
                        }
                        let child = take_element(cursor, index, "default properties")?;
                        if local == "rPrDefault" {
                            let properties = CT_RPr::from_strict_xml(child)?;
                            wrapper_descendants.push(properties.completeness.clone());
                            defaults.rpr = Some(properties);
                        } else {
                            let properties = CT_PPr::from_strict_xml(child)?;
                            wrapper_descendants.push(properties.completeness.clone());
                            defaults.ppr = Some(properties);
                        }
                        break;
                    }
                    Ok(())
                })?;
                descendants.push(StrictXmlCompleteness::new(
                    parsed_wrapper.leftovers,
                    wrapper_descendants,
                ));
            }
            Ok(defaults)
        })?;
        let (mut defaults, leftovers) = parsed.into_parts();
        defaults.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(defaults)
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        let context = NamespaceContext::default();
        let element = parse_reader_element(reader, &context, Some(W_NS), "docDefaults", [])?;
        Self::from_strict_xml(element)
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:docDefaults")))?;

        if let Some(ref rpr) = self.rpr {
            writer.write_event(Event::Start(BytesStart::new("w:rPrDefault")))?;
            rpr.to_xml(writer)?;
            writer.write_event(Event::End(BytesEnd::new("w:rPrDefault")))?;
        }

        if let Some(ref ppr) = self.ppr {
            writer.write_event(Event::Start(BytesStart::new("w:pPrDefault")))?;
            ppr.to_xml(writer)?;
            writer.write_event(Event::End(BytesEnd::new("w:pPrDefault")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:docDefaults")))?;
        Ok(())
    }
}

/// `CT_Styles` — The styles part (word/styles.xml).
#[derive(Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_Styles {
    #[doc(hidden)]
    pub completeness: StrictXmlCompleteness,
    pub doc_defaults: Option<CT_DocDefaults>,
    pub styles: Vec<CT_Style>,
}

#[allow(non_snake_case)]
impl CT_Styles {
    pub fn new() -> Self {
        CT_Styles {
            completeness: StrictXmlCompleteness::default(),
            doc_defaults: None,
            styles: Vec::new(),
        }
    }

    /// Parse from XML bytes (the content of word/styles.xml).
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let root = StrictXmlDocument::parse(xml)?.into_root();
        if !root.is_named(Some(W_NS), "styles") {
            return Err(OxmlError::MissingElement("w:styles".to_string()));
        }
        let mut descendants = Vec::new();
        let parsed = root.parse(|cursor| {
            let mut styles = Self::new();
            for index in 0..cursor.child_slots() {
                let Some(StrictXmlNode::Element(child)) = cursor.child(index) else {
                    continue;
                };
                if child.is_named(Some(W_NS), "docDefaults") && styles.doc_defaults.is_none() {
                    let child = take_element(cursor, index, "docDefaults")?;
                    let defaults = CT_DocDefaults::from_strict_xml(child)?;
                    descendants.push(defaults.completeness.clone());
                    styles.doc_defaults = Some(defaults);
                } else if child.is_named(Some(W_NS), "style") {
                    let child = take_element(cursor, index, "style")?;
                    let style = CT_Style::from_strict_xml(child)?;
                    descendants.push(style.completeness.clone());
                    styles.styles.push(style);
                }
            }
            Ok(styles)
        })?;
        let (mut styles, leftovers) = parsed.into_parts();
        styles.completeness = StrictXmlCompleteness::new(leftovers, descendants);
        Ok(styles)
    }

    /// Serialize to XML bytes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut styles_start = BytesStart::new("w:styles");
        styles_start.push_attribute(("xmlns:w", W_NS));
        styles_start.push_attribute((
            "xmlns:r",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        ));
        writer.write_event(Event::Start(styles_start))?;

        if let Some(ref defaults) = self.doc_defaults {
            defaults.to_xml(&mut writer)?;
        }

        for style in &self.styles {
            style.to_xml(&mut writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:styles")))?;

        Ok(writer.into_inner())
    }

    /// Find a style by its ID.
    pub fn get_by_id(&self, style_id: &str) -> Option<&CT_Style> {
        self.styles.iter().find(|s| s.style_id == style_id)
    }

    /// Find the default style for a given type.
    pub fn get_default(&self, style_type: StyleType) -> Option<&CT_Style> {
        self.styles
            .iter()
            .find(|s| s.style_type == style_type && s.is_default)
    }

    /// Create a minimal default styles part for a new document.
    pub fn new_default() -> Self {
        use crate::units::HalfPoint;

        let normal = CT_Style {
            style_id: "Normal".to_string(),
            style_type: StyleType::Paragraph,
            name: Some("Normal".to_string()),
            based_on: None,
            next_style: None,
            is_default: true,
            ppr: None,
            rpr: None,
            ..Default::default()
        };

        let heading1 = CT_Style {
            style_id: "Heading1".to_string(),
            style_type: StyleType::Paragraph,
            name: Some("heading 1".to_string()),
            based_on: Some("Normal".to_string()),
            next_style: Some("Normal".to_string()),
            is_default: false,
            ppr: Some(CT_PPr {
                keep_next: Some(true),
                keep_lines: Some(true),
                space_before: Some(crate::units::Twips(240)),
                space_after: Some(crate::units::Twips(0)),
                ..Default::default()
            }),
            rpr: Some(CT_RPr {
                sz: Some(HalfPoint(32)),
                sz_cs: Some(HalfPoint(32)),
                bold: Some(true),
                bold_cs: Some(true),
                color: Some("2F5496".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let doc_defaults = CT_DocDefaults {
            rpr: Some(CT_RPr {
                font_ascii: Some("Calibri".to_string()),
                font_hansi: Some("Calibri".to_string()),
                font_east_asia: Some("Calibri".to_string()),
                font_cs: Some("Times New Roman".to_string()),
                sz: Some(HalfPoint(22)),
                sz_cs: Some(HalfPoint(22)),
                ..Default::default()
            }),
            ppr: Some(CT_PPr {
                space_after: Some(crate::units::Twips(160)),
                line_spacing: Some(crate::units::Twips(259)),
                line_rule: Some("auto".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        CT_Styles {
            doc_defaults: Some(doc_defaults),
            styles: vec![normal, heading1],
            ..Default::default()
        }
    }
}

impl Default for CT_Styles {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the `w:val` attribute from an element.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_styles() {
        let styles = CT_Styles::new_default();
        let xml = styles.to_xml().unwrap();
        let parsed = CT_Styles::from_xml(&xml).unwrap();

        assert_eq!(parsed.styles.len(), 2);
        assert!(parsed.doc_defaults.is_some());

        let normal = parsed.get_by_id("Normal").unwrap();
        assert_eq!(normal.name, Some("Normal".to_string()));
        assert!(normal.is_default);

        let h1 = parsed.get_by_id("Heading1").unwrap();
        assert_eq!(h1.based_on, Some("Normal".to_string()));
    }

    #[test]
    fn find_default_style() {
        let styles = CT_Styles::new_default();
        let default_para = styles.get_default(StyleType::Paragraph).unwrap();
        assert_eq!(default_para.style_id, "Normal");
    }

    #[test]
    fn styles_require_word_namespaces_and_qualified_attributes() {
        let xml = format!(
            r#"<z:styles xmlns:z="{W_NS}" xmlns:x="urn:foreign">
              <x:style z:styleId="Foreign"><x:name z:val="Foreign"/></x:style>
              <z:style styleId="Unqualified" type="paragraph"><z:name val="Ignored"/></z:style>
              <z:style z:styleId="Accepted" z:type="paragraph"><z:name z:val="Accepted"/></z:style>
            </z:styles>"#
        );

        let styles = CT_Styles::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(styles.styles.len(), 2);
        assert!(styles.get_by_id("Foreign").is_none());
        assert!(styles.get_by_id("Unqualified").is_none());
        assert_eq!(
            styles.get_by_id("Accepted").unwrap().name.as_deref(),
            Some("Accepted")
        );
    }

    #[test]
    fn truncated_styles_are_rejected() {
        let xml = format!(r#"<w:styles xmlns:w="{W_NS}"><w:style w:styleId="Normal">"#);
        assert!(CT_Styles::from_xml(xml.as_bytes()).is_err());
    }
}

//! Style elements: `CT_Styles`, `CT_Style`, `CT_DocDefaults`.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::{OxmlError, Result};
use crate::namespace::{W_NS, matches_word_attribute, matches_word_element, matches_word_name};
use crate::properties::{CT_PPr, CT_RPr};
use crate::raw_xml::NamespaceContext;

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
    pub fn from_xml(reader: &mut Reader<&[u8]>, attrs: &BytesStart) -> Result<Self> {
        let context = NamespaceContext::default().with_element(attrs);
        Self::from_xml_with_context(reader, attrs, &context)
    }

    fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        attrs: &BytesStart,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut style_id = String::new();
        let mut style_type = StyleType::Paragraph;
        let mut is_default = false;

        for attr in attrs.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            if matches_word_attribute(key, context, b"styleId") {
                style_id = std::str::from_utf8(&attr.value)?.to_string();
            } else if matches_word_attribute(key, context, b"type") {
                style_type = StyleType::from_str(std::str::from_utf8(&attr.value)?)?;
            } else if matches_word_attribute(key, context, b"default") {
                is_default = std::str::from_utf8(&attr.value)? == "1"
                    || std::str::from_utf8(&attr.value)? == "true";
            }
        }

        let mut name = None;
        let mut based_on = None;
        let mut next_style = None;
        let mut ppr = None;
        let mut rpr = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    let element_context = context.with_element(e);
                    if matches_word_element(e, context, b"name") {
                        name = get_val_attr(e, &element_context)?;
                    } else if matches_word_element(e, context, b"basedOn") {
                        based_on = get_val_attr(e, &element_context)?;
                    } else if matches_word_element(e, context, b"next") {
                        next_style = get_val_attr(e, &element_context)?;
                    }
                }
                Ok(Event::Start(ref e)) => {
                    let ename = e.name();
                    let child_context = context.with_element(e);
                    if matches_word_element(e, context, b"pPr") {
                        ppr = Some(CT_PPr::from_xml_with_context(reader, &child_context)?);
                    } else if matches_word_element(e, context, b"rPr") {
                        rpr = Some(CT_RPr::from_xml_with_context(reader, &child_context)?);
                    } else if matches_word_element(e, context, b"name") {
                        name = get_val_attr(e, &child_context)?;
                        reader.read_to_end_into(ename, &mut Vec::new())?;
                    } else if matches_word_element(e, context, b"basedOn") {
                        based_on = get_val_attr(e, &child_context)?;
                        reader.read_to_end_into(ename, &mut Vec::new())?;
                    } else if matches_word_element(e, context, b"next") {
                        next_style = get_val_attr(e, &child_context)?;
                        reader.read_to_end_into(ename, &mut Vec::new())?;
                    } else {
                        reader.read_to_end_into(ename, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e))
                    if matches_word_name(e.name().as_ref(), context, b"style") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:style".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Style {
            style_id,
            style_type,
            name,
            based_on,
            next_style,
            is_default,
            ppr,
            rpr,
        })
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

/// `CT_DocDefaults` — Document-level default properties.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_DocDefaults {
    pub rpr: Option<CT_RPr>,
    pub ppr: Option<CT_PPr>,
}

#[allow(non_snake_case)]
impl CT_DocDefaults {
    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut defaults = CT_DocDefaults::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let child_context = context.with_element(e);
                    if matches_word_element(e, context, b"rPrDefault") {
                        // Read into rPrDefault, expecting rPr child
                        defaults.rpr =
                            Self::parse_pr_default(reader, &child_context, b"rPrDefault")?;
                    } else if matches_word_element(e, context, b"pPrDefault") {
                        defaults.ppr = Self::parse_ppr_default(reader, &child_context)?;
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e))
                    if matches_word_name(e.name().as_ref(), context, b"docDefaults") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement(
                        "closing w:docDefaults".to_string(),
                    ));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(defaults)
    }

    fn parse_pr_default(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
        end_tag: &[u8],
    ) -> Result<Option<CT_RPr>> {
        let mut rpr = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_word_element(e, context, b"rPr") {
                        let child_context = context.with_element(e);
                        rpr = Some(CT_RPr::from_xml_with_context(reader, &child_context)?);
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e)) if matches_word_name(e.name().as_ref(), context, end_tag) => {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement(format!(
                        "closing w:{}",
                        String::from_utf8_lossy(end_tag)
                    )));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(rpr)
    }

    fn parse_ppr_default(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Option<CT_PPr>> {
        let mut ppr = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_word_element(e, context, b"pPr") {
                        let child_context = context.with_element(e);
                        ppr = Some(CT_PPr::from_xml_with_context(reader, &child_context)?);
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e))
                    if matches_word_name(e.name().as_ref(), context, b"pPrDefault") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement(
                        "closing w:pPrDefault".to_string(),
                    ));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(ppr)
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
    pub doc_defaults: Option<CT_DocDefaults>,
    pub styles: Vec<CT_Style>,
}

#[allow(non_snake_case)]
impl CT_Styles {
    pub fn new() -> Self {
        CT_Styles {
            doc_defaults: None,
            styles: Vec::new(),
        }
    }

    /// Parse from XML bytes (the content of word/styles.xml).
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);

        let mut doc_defaults = None;
        let mut styles = Vec::new();
        let mut root_context = None;
        let mut root_closed = false;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if root_context.is_none() && !root_closed {
                        let document_context = NamespaceContext::default();
                        if !matches_word_element(e, &document_context, b"styles") {
                            return Err(OxmlError::UnexpectedElement(
                                String::from_utf8_lossy(name.as_ref()).into_owned(),
                            ));
                        }
                        root_context = Some(document_context.with_element(e));
                    } else if let Some(context) = root_context.as_ref() {
                        if matches_word_element(e, context, b"docDefaults") {
                            let child_context = context.with_element(e);
                            doc_defaults = Some(CT_DocDefaults::from_xml_with_context(
                                &mut reader,
                                &child_context,
                            )?);
                        } else if matches_word_element(e, context, b"style") {
                            let child_context = context.with_element(e);
                            styles.push(CT_Style::from_xml_with_context(
                                &mut reader,
                                e,
                                &child_context,
                            )?);
                        } else {
                            reader.read_to_end_into(name, &mut Vec::new())?;
                        }
                    } else {
                        return Err(OxmlError::UnexpectedElement(
                            String::from_utf8_lossy(name.as_ref()).into_owned(),
                        ));
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let document_context = NamespaceContext::default();
                    if root_context.is_none()
                        && !root_closed
                        && matches_word_element(e, &document_context, b"styles")
                    {
                        root_closed = true;
                    } else if root_context.is_none() || root_closed {
                        return Err(OxmlError::UnexpectedElement(
                            String::from_utf8_lossy(e.name().as_ref()).into_owned(),
                        ));
                    }
                }
                Ok(Event::End(ref e)) => {
                    let Some(context) = root_context.as_ref() else {
                        return Err(OxmlError::UnexpectedElement(
                            String::from_utf8_lossy(e.name().as_ref()).into_owned(),
                        ));
                    };
                    if matches_word_name(e.name().as_ref(), context, b"styles") {
                        root_context = None;
                        root_closed = true;
                    }
                }
                Ok(Event::Eof) if root_closed => break,
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:styles".to_string()));
                }
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_Styles {
            doc_defaults,
            styles,
        })
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
        };

        CT_Styles {
            doc_defaults: Some(doc_defaults),
            styles: vec![normal, heading1],
        }
    }
}

impl Default for CT_Styles {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the `w:val` attribute from an element.
fn get_val_attr(e: &BytesStart, context: &NamespaceContext) -> Result<Option<String>> {
    for attr in e.attributes() {
        let attr = attr?;
        if matches_word_attribute(attr.key.as_ref(), context, b"val") {
            return Ok(Some(std::str::from_utf8(&attr.value)?.to_string()));
        }
    }
    Ok(None)
}

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

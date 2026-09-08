//! Style elements: `CT_Styles`, `CT_Style`, `CT_DocDefaults`.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::error::{OxmlError, Result};
use crate::namespace::{W_NS, matches_local_name};
use crate::numbering::{parse_scoped_ppr, word_prefixes_at};
use crate::properties::{CT_PPr, CT_RPr, is_word_attribute, is_word_element};
use crate::raw_xml::{capture_element, capture_empty_element};
use crate::table::{CT_TblPr, CT_TcPr};

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
    pub linked_style: Option<String>,
    pub auto_redefine: Option<bool>,
    pub hidden: Option<bool>,
    pub ui_priority: Option<u32>,
    pub semi_hidden: Option<bool>,
    pub unhide_when_used: Option<bool>,
    pub quick_format: Option<bool>,
    pub locked: Option<bool>,
    pub is_default: bool,
    pub ppr: Option<CT_PPr>,
    pub rpr: Option<CT_RPr>,
    /// Typed projection of the style's base table properties.
    pub table_properties: Option<CT_TblPr>,
    #[doc(hidden)]
    pub table_properties_original: Option<CT_TblPr>,
    /// Preserved self-contained bytes for the base table properties.
    pub table_properties_xml: Option<Vec<u8>>,
    /// Preserved conditional table-style regions and typed projections.
    pub conditional_table_styles: Vec<CT_TblStylePr>,
    /// Unmodeled root attributes and namespace declarations.
    #[doc(hidden)]
    pub extra_attributes: Vec<(String, String)>,
    /// Preserved explicit forms of modeled scalar children.
    #[doc(hidden)]
    pub modeled_xml: Vec<(u8, Vec<u8>)>,
    /// Preserved style children keyed to their schema-order rank.
    pub extra_xml: Vec<(u8, Vec<u8>)>,
}

/// One conditional table-style region.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_TblStylePr {
    pub region: String,
    pub paragraph_properties: Option<CT_PPr>,
    pub table_properties: Option<CT_TblPr>,
    pub cell_properties: Option<CT_TcPr>,
    /// Unmodeled region attributes and namespace declarations.
    #[doc(hidden)]
    pub extra_attributes: Vec<(String, String)>,
    pub raw_xml: Vec<u8>,
}

#[allow(non_snake_case)]
impl CT_Style {
    pub fn from_xml(reader: &mut Reader<&[u8]>, attrs: &BytesStart) -> Result<Self> {
        let prefixes = word_prefixes_at(attrs, &["w".to_string()])?;
        let namespace_bindings = namespace_bindings_at(attrs, &[])?;
        Self::from_xml_with_prefixes(reader, attrs, &prefixes, &namespace_bindings)
    }

    fn from_xml_with_prefixes(
        reader: &mut Reader<&[u8]>,
        attrs: &BytesStart,
        word_prefixes: &[String],
        namespace_bindings: &[(String, String)],
    ) -> Result<Self> {
        reject_conflicting_style_prefixes(attrs)?;
        reject_conflicting_style_bindings(namespace_bindings)?;
        let mut style_id = String::new();
        let mut style_type = StyleType::Paragraph;
        let mut is_default = false;

        for attr in attrs.attributes() {
            let attr = attr?;
            let key = attr.key.as_ref();
            if is_word_attribute(key, b"styleId", word_prefixes) {
                style_id = std::str::from_utf8(&attr.value)?.to_string();
            } else if is_word_attribute(key, b"type", word_prefixes) {
                style_type = StyleType::from_str(std::str::from_utf8(&attr.value)?)?;
            } else if is_word_attribute(key, b"default", word_prefixes) {
                is_default = std::str::from_utf8(&attr.value)? == "1"
                    || std::str::from_utf8(&attr.value)? == "true";
            }
        }

        let mut name = None;
        let mut based_on = None;
        let mut next_style = None;
        let mut linked_style = None;
        let mut auto_redefine = None;
        let mut hidden = None;
        let mut ui_priority = None;
        let mut semi_hidden = None;
        let mut unhide_when_used = None;
        let mut quick_format = None;
        let mut locked = None;
        let mut ppr = None;
        let mut rpr = None;
        let mut table_properties = None;
        let mut table_properties_xml = None;
        let mut conditional_table_styles = Vec::new();
        let mut modeled_xml = Vec::new();
        let mut extra_xml = Vec::new();
        let extra_attributes = raw_style_attributes(
            attrs,
            &[b"styleId", b"type", b"default"],
            word_prefixes,
            namespace_bindings,
        )?;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    let ename = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    let bindings = namespace_bindings_at(e, namespace_bindings)?;
                    if let Some(rank) = style_scalar_rank(ename.as_ref(), &prefixes) {
                        let string_value = matches!(rank, 0 | 2 | 3 | 4)
                            .then(|| get_val_attr(e, &prefixes))
                            .transpose()?
                            .flatten();
                        let toggle_value = matches!(rank, 5 | 6 | 8 | 9 | 10 | 11)
                            .then(|| parse_style_toggle(e, &prefixes))
                            .transpose()?
                            .flatten();
                        let priority_value = (rank == 7)
                            .then(|| parse_style_priority(e, &prefixes))
                            .transpose()?
                            .flatten();
                        let raw = capture_empty_element(e)?;
                        if string_value.is_some()
                            || toggle_value.is_some()
                            || priority_value.is_some()
                        {
                            apply_style_scalar(
                                rank,
                                string_value,
                                toggle_value,
                                priority_value,
                                &mut name,
                                &mut based_on,
                                &mut next_style,
                                &mut linked_style,
                                &mut auto_redefine,
                                &mut hidden,
                                &mut ui_priority,
                                &mut semi_hidden,
                                &mut unhide_when_used,
                                &mut quick_format,
                                &mut locked,
                            );
                            if style_scalar_empty_requires_snapshot(e, rank)? {
                                modeled_xml
                                    .push((rank, make_style_raw_self_contained(&raw, &bindings)?));
                            }
                        } else {
                            extra_xml.push((rank, make_style_raw_self_contained(&raw, &bindings)?));
                        }
                    } else {
                        extra_xml.push((
                            style_child_rank(ename.as_ref(), &prefixes),
                            make_style_raw_self_contained(&capture_empty_element(e)?, &bindings)?,
                        ));
                    }
                }
                Ok(Event::Start(ref e)) => {
                    let ename = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    let bindings = namespace_bindings_at(e, namespace_bindings)?;
                    let scalar_rank = style_scalar_rank(ename.as_ref(), &prefixes);
                    if let Some(rank) = scalar_rank {
                        let string_value = matches!(rank, 0 | 2 | 3 | 4)
                            .then(|| get_val_attr(e, &prefixes))
                            .transpose()?
                            .flatten();
                        let toggle_value = matches!(rank, 5 | 6 | 8 | 9 | 10 | 11)
                            .then(|| parse_style_toggle(e, &prefixes))
                            .transpose()?
                            .flatten();
                        let priority_value = (rank == 7)
                            .then(|| parse_style_priority(e, &prefixes))
                            .transpose()?
                            .flatten();
                        let raw = capture_element(reader, e)?;
                        if explicit_style_element_has_only_trivia(&raw)?
                            && (string_value.is_some()
                                || toggle_value.is_some()
                                || priority_value.is_some())
                        {
                            apply_style_scalar(
                                rank,
                                string_value,
                                toggle_value,
                                priority_value,
                                &mut name,
                                &mut based_on,
                                &mut next_style,
                                &mut linked_style,
                                &mut auto_redefine,
                                &mut hidden,
                                &mut ui_priority,
                                &mut semi_hidden,
                                &mut unhide_when_used,
                                &mut quick_format,
                                &mut locked,
                            );
                            modeled_xml
                                .push((rank, make_style_raw_self_contained(&raw, &bindings)?));
                        } else {
                            extra_xml.push((rank, make_style_raw_self_contained(&raw, &bindings)?));
                        }
                    } else if is_word_element(ename.as_ref(), b"pPr", &prefixes) {
                        let raw = capture_element(reader, e)?;
                        ppr = Some(parse_scoped_ppr(&raw, word_prefixes)?);
                    } else if is_word_element(ename.as_ref(), b"rPr", &prefixes) {
                        rpr = Some(CT_RPr::from_xml(reader)?);
                    } else if is_word_element(ename.as_ref(), b"tblPr", &prefixes) {
                        let raw = capture_element(reader, e)?;
                        let preserved = make_style_raw_self_contained(&raw, &bindings)?;
                        table_properties =
                            Some(parse_style_table_properties(&preserved, &prefixes)?);
                        table_properties_xml = Some(preserved);
                    } else if is_word_element(ename.as_ref(), b"tblStylePr", &prefixes) {
                        reject_conflicting_style_prefixes(e)?;
                        reject_conflicting_style_bindings(&bindings)?;
                        let region = get_word_attr(e, b"type", &prefixes)?.unwrap_or_default();
                        let raw =
                            make_style_raw_self_contained(&capture_element(reader, e)?, &bindings)?;
                        let (paragraph_properties, conditional_table_properties, cell_properties) =
                            parse_conditional_style_properties(&raw, &prefixes)?;
                        conditional_table_styles.push(CT_TblStylePr {
                            region,
                            paragraph_properties,
                            table_properties: conditional_table_properties,
                            cell_properties,
                            extra_attributes: raw_style_attributes(
                                e,
                                &[b"type"],
                                &prefixes,
                                &bindings,
                            )?,
                            raw_xml: raw,
                        });
                    } else {
                        extra_xml.push((
                            style_child_rank(ename.as_ref(), &prefixes),
                            make_style_raw_self_contained(&capture_element(reader, e)?, &bindings)?,
                        ));
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"style") => {
                    break;
                }
                Ok(Event::Eof) => break,
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
            linked_style,
            auto_redefine,
            hidden,
            ui_priority,
            semi_hidden,
            unhide_when_used,
            quick_format,
            locked,
            is_default,
            ppr,
            rpr,
            table_properties: table_properties.clone(),
            table_properties_original: table_properties.clone(),
            table_properties_xml,
            conditional_table_styles,
            extra_attributes,
            modeled_xml,
            extra_xml,
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut e = BytesStart::new("w:style");
        e.push_attribute(("w:type", self.style_type.to_str()));
        e.push_attribute(("w:styleId", self.style_id.as_str()));
        if self.is_default {
            e.push_attribute(("w:default", "1"));
        }
        push_style_attributes(&mut e, &self.extra_attributes);
        writer.write_event(Event::Start(e))?;

        let mut extras = self.extra_xml.iter().collect::<Vec<_>>();
        extras.sort_by_key(|(rank, _)| *rank);
        let mut extra_index = 0;

        write_style_extras(writer, &extras, &mut extra_index, 0)?;
        write_style_string(writer, "w:name", 0, self.name.as_deref(), &self.modeled_xml)?;

        write_style_extras(writer, &extras, &mut extra_index, 2)?;
        write_style_string(
            writer,
            "w:basedOn",
            2,
            self.based_on.as_deref(),
            &self.modeled_xml,
        )?;

        write_style_extras(writer, &extras, &mut extra_index, 3)?;
        write_style_string(
            writer,
            "w:next",
            3,
            self.next_style.as_deref(),
            &self.modeled_xml,
        )?;

        write_style_extras(writer, &extras, &mut extra_index, 4)?;
        write_style_string(
            writer,
            "w:link",
            4,
            self.linked_style.as_deref(),
            &self.modeled_xml,
        )?;
        write_style_extras(writer, &extras, &mut extra_index, 5)?;
        write_style_toggle(
            writer,
            "w:autoRedefine",
            5,
            self.auto_redefine,
            &self.modeled_xml,
        )?;
        write_style_extras(writer, &extras, &mut extra_index, 6)?;
        write_style_toggle(writer, "w:hidden", 6, self.hidden, &self.modeled_xml)?;
        write_style_extras(writer, &extras, &mut extra_index, 7)?;
        write_style_priority(writer, self.ui_priority, &self.modeled_xml)?;
        write_style_extras(writer, &extras, &mut extra_index, 8)?;
        write_style_toggle(
            writer,
            "w:semiHidden",
            8,
            self.semi_hidden,
            &self.modeled_xml,
        )?;
        write_style_extras(writer, &extras, &mut extra_index, 9)?;
        write_style_toggle(
            writer,
            "w:unhideWhenUsed",
            9,
            self.unhide_when_used,
            &self.modeled_xml,
        )?;
        write_style_extras(writer, &extras, &mut extra_index, 10)?;
        write_style_toggle(
            writer,
            "w:qFormat",
            10,
            self.quick_format,
            &self.modeled_xml,
        )?;
        write_style_extras(writer, &extras, &mut extra_index, 11)?;
        write_style_toggle(writer, "w:locked", 11, self.locked, &self.modeled_xml)?;
        write_style_extras(writer, &extras, &mut extra_index, 20)?;
        if let Some(ref ppr) = self.ppr {
            ppr.to_xml(writer)?;
        }
        write_style_extras(writer, &extras, &mut extra_index, 21)?;
        if let Some(ref rpr) = self.rpr {
            rpr.to_xml(writer)?;
        }
        write_style_extras(writer, &extras, &mut extra_index, 22)?;
        if let Some(ref properties) = self.table_properties {
            let preserved_matches = self.table_properties_original.as_ref() == Some(properties);
            if preserved_matches {
                writer
                    .get_mut()
                    .write_all(self.table_properties_xml.as_deref().unwrap_or_default())?;
            } else {
                properties.to_xml(writer)?;
            }
        }
        write_style_extras(writer, &extras, &mut extra_index, 25)?;
        for conditional in &self.conditional_table_styles {
            writer
                .get_mut()
                .write_all(&serialize_conditional_table_style(conditional)?)?;
        }
        write_style_extras(writer, &extras, &mut extra_index, u8::MAX)?;

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
        Self::from_xml_with_prefixes(reader, &["w".to_string()])
    }

    fn from_xml_with_prefixes(
        reader: &mut Reader<&[u8]>,
        word_prefixes: &[String],
    ) -> Result<Self> {
        let mut defaults = CT_DocDefaults::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if matches_local_name(name.as_ref(), b"rPrDefault") {
                        // Read into rPrDefault, expecting rPr child
                        defaults.rpr = Self::parse_pr_default(reader, b"rPrDefault")?;
                    } else if matches_local_name(name.as_ref(), b"pPrDefault") {
                        defaults.ppr = Self::parse_ppr_default(reader, &prefixes)?;
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"docDefaults") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(defaults)
    }

    fn parse_pr_default(reader: &mut Reader<&[u8]>, end_tag: &[u8]) -> Result<Option<CT_RPr>> {
        let mut rpr = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    if matches_local_name(name.as_ref(), b"rPr") {
                        rpr = Some(CT_RPr::from_xml(reader)?);
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), end_tag) => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(rpr)
    }

    fn parse_ppr_default(
        reader: &mut Reader<&[u8]>,
        word_prefixes: &[String],
    ) -> Result<Option<CT_PPr>> {
        let mut ppr = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if is_word_element(name.as_ref(), b"pPr", &prefixes) {
                        let raw = capture_element(reader, e)?;
                        ppr = Some(parse_scoped_ppr(&raw, word_prefixes)?);
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"pPrDefault") => {
                    break;
                }
                Ok(Event::Eof) => break,
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
        let mut buf = Vec::new();
        let mut word_prefixes = Vec::new();
        let mut namespace_bindings = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, &word_prefixes)?;
                    if is_word_element(name.as_ref(), b"docDefaults", &prefixes) {
                        doc_defaults = Some(CT_DocDefaults::from_xml_with_prefixes(
                            &mut reader,
                            &prefixes,
                        )?);
                    } else if is_word_element(name.as_ref(), b"style", &prefixes) {
                        let bindings = namespace_bindings_at(e, &namespace_bindings)?;
                        styles.push(CT_Style::from_xml_with_prefixes(
                            &mut reader,
                            e,
                            &prefixes,
                            &bindings,
                        )?);
                    } else if is_word_element(name.as_ref(), b"styles", &prefixes) {
                        // Root element, continue
                        word_prefixes = prefixes;
                        namespace_bindings = namespace_bindings_at(e, &namespace_bindings)?;
                    } else {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                    }
                }
                Ok(Event::Eof) => break,
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
            linked_style: None,
            auto_redefine: None,
            hidden: None,
            ui_priority: None,
            semi_hidden: None,
            unhide_when_used: None,
            quick_format: None,
            locked: None,
            is_default: true,
            ppr: None,
            rpr: None,
            table_properties: None,
            table_properties_original: None,
            table_properties_xml: None,
            conditional_table_styles: Vec::new(),
            extra_attributes: Vec::new(),
            modeled_xml: Vec::new(),
            extra_xml: Vec::new(),
        };

        let heading1 = CT_Style {
            style_id: "Heading1".to_string(),
            style_type: StyleType::Paragraph,
            name: Some("heading 1".to_string()),
            based_on: Some("Normal".to_string()),
            next_style: Some("Normal".to_string()),
            linked_style: None,
            auto_redefine: None,
            hidden: None,
            ui_priority: None,
            semi_hidden: None,
            unhide_when_used: None,
            quick_format: None,
            locked: None,
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
            table_properties: None,
            table_properties_original: None,
            table_properties_xml: None,
            conditional_table_styles: Vec::new(),
            extra_attributes: Vec::new(),
            modeled_xml: Vec::new(),
            extra_xml: Vec::new(),
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

fn style_child_rank(name: &[u8], word_prefixes: &[String]) -> u8 {
    let local = name.rsplit(|byte| *byte == b':').next().unwrap_or(name);
    if !is_word_element(name, local, word_prefixes) {
        return 26;
    }
    match local {
        b"name" => 0,
        b"aliases" => 1,
        b"basedOn" => 2,
        b"next" => 3,
        b"link" => 4,
        b"autoRedefine" => 5,
        b"hidden" => 6,
        b"uiPriority" => 7,
        b"semiHidden" => 8,
        b"unhideWhenUsed" => 9,
        b"qFormat" => 10,
        b"locked" => 11,
        b"personal" => 12,
        b"personalCompose" => 13,
        b"personalReply" => 14,
        b"rsid" => 19,
        b"pPr" => 20,
        b"rPr" => 21,
        b"tblPr" => 22,
        b"trPr" => 23,
        b"tcPr" => 24,
        b"tblStylePr" => 25,
        _ => 26,
    }
}

fn style_scalar_rank(name: &[u8], word_prefixes: &[String]) -> Option<u8> {
    let rank = style_child_rank(name, word_prefixes);
    matches!(rank, 0 | 2..=11).then_some(rank)
}

fn style_scalar_empty_requires_snapshot(element: &BytesStart<'_>, rank: u8) -> Result<bool> {
    let canonical_name: &[u8] = match rank {
        0 => b"w:name",
        2 => b"w:basedOn",
        3 => b"w:next",
        4 => b"w:link",
        5 => b"w:autoRedefine",
        6 => b"w:hidden",
        7 => b"w:uiPriority",
        8 => b"w:semiHidden",
        9 => b"w:unhideWhenUsed",
        10 => b"w:qFormat",
        11 => b"w:locked",
        _ => unreachable!("style scalar rank"),
    };
    if element.name().as_ref() != canonical_name {
        return Ok(true);
    }

    for attribute in element.attributes() {
        if attribute?.key.as_ref() != b"w:val" {
            return Ok(true);
        }
    }
    Ok(false)
}

#[allow(clippy::too_many_arguments)]
fn apply_style_scalar(
    rank: u8,
    string_value: Option<String>,
    toggle_value: Option<bool>,
    priority_value: Option<u32>,
    name: &mut Option<String>,
    based_on: &mut Option<String>,
    next_style: &mut Option<String>,
    linked_style: &mut Option<String>,
    auto_redefine: &mut Option<bool>,
    hidden: &mut Option<bool>,
    ui_priority: &mut Option<u32>,
    semi_hidden: &mut Option<bool>,
    unhide_when_used: &mut Option<bool>,
    quick_format: &mut Option<bool>,
    locked: &mut Option<bool>,
) {
    match rank {
        0 => *name = string_value,
        2 => *based_on = string_value,
        3 => *next_style = string_value,
        4 => *linked_style = string_value,
        5 => *auto_redefine = toggle_value,
        6 => *hidden = toggle_value,
        7 => *ui_priority = priority_value,
        8 => *semi_hidden = toggle_value,
        9 => *unhide_when_used = toggle_value,
        10 => *quick_format = toggle_value,
        11 => *locked = toggle_value,
        _ => unreachable!("style scalar rank"),
    }
}

fn write_style_extras<W: std::io::Write>(
    writer: &mut Writer<W>,
    extras: &[&(u8, Vec<u8>)],
    index: &mut usize,
    before_rank: u8,
) -> Result<()> {
    while let Some((rank, raw)) = extras.get(*index).copied() {
        if *rank >= before_rank {
            break;
        }
        writer.get_mut().write_all(raw)?;
        *index += 1;
    }
    Ok(())
}

fn namespace_bindings_at(
    element: &BytesStart,
    inherited: &[(String, String)],
) -> Result<Vec<(String, String)>> {
    let mut bindings = inherited.to_vec();
    for attribute in element.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        let prefix = if key == b"xmlns" {
            Some(String::new())
        } else {
            key.strip_prefix(b"xmlns:")
                .map(|prefix| String::from_utf8_lossy(prefix).into_owned())
        };
        let Some(prefix) = prefix else {
            continue;
        };
        let uri = std::str::from_utf8(&attribute.value)?.to_owned();
        if let Some(binding) = bindings
            .iter_mut()
            .find(|(candidate, _)| candidate == &prefix)
        {
            binding.1 = uri;
        } else {
            bindings.push((prefix, uri));
        }
    }
    Ok(bindings)
}

fn reject_conflicting_style_prefixes(element: &BytesStart<'_>) -> Result<()> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        let expected = match attribute.key.as_ref() {
            b"xmlns:w" => Some(W_NS),
            b"xmlns:r" => {
                Some("http://schemas.openxmlformats.org/officeDocument/2006/relationships")
            }
            _ => None,
        };
        let Some(expected) = expected else {
            continue;
        };
        let value = std::str::from_utf8(&attribute.value)?;
        if value != expected {
            return Err(OxmlError::InvalidValue(format!(
                "{} conflicts with the fixed style writer namespace",
                String::from_utf8_lossy(attribute.key.as_ref())
            )));
        }
    }
    Ok(())
}

fn raw_style_attributes(
    element: &BytesStart<'_>,
    modeled: &[&[u8]],
    word_prefixes: &[String],
    namespace_bindings: &[(String, String)],
) -> Result<Vec<(String, String)>> {
    let mut attributes = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
            .into_owned();
        let canonical_namespace = (key == b"xmlns:w" && value == W_NS)
            || (key == b"xmlns:r"
                && value == "http://schemas.openxmlformats.org/officeDocument/2006/relationships");
        if !canonical_namespace
            && !modeled
                .iter()
                .any(|local| is_word_attribute(key, local, word_prefixes))
        {
            attributes.push((std::str::from_utf8(key)?.to_owned(), value));
        }
    }
    if !attributes.is_empty() {
        for (prefix, namespace) in namespace_bindings {
            if prefix.is_empty()
                || (prefix == "w" && namespace == W_NS)
                || (prefix == "r"
                    && namespace
                        == "http://schemas.openxmlformats.org/officeDocument/2006/relationships")
            {
                continue;
            }
            let declaration = format!("xmlns:{prefix}");
            if !attributes.iter().any(|(name, _)| name == &declaration) {
                attributes.push((declaration, namespace.clone()));
            }
        }
    }
    Ok(attributes)
}

fn reject_conflicting_style_bindings(bindings: &[(String, String)]) -> Result<()> {
    for (prefix, namespace) in bindings {
        let expected = match prefix.as_str() {
            "w" => Some(W_NS),
            "r" => Some("http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
            _ => None,
        };
        if expected.is_some_and(|expected| namespace != expected) {
            return Err(OxmlError::InvalidValue(format!(
                "xmlns:{prefix} conflicts with the fixed style writer namespace"
            )));
        }
    }
    Ok(())
}

fn push_style_attributes(element: &mut BytesStart<'_>, attributes: &[(String, String)]) {
    for (name, value) in attributes {
        element.push_attribute((name.as_str(), value.as_str()));
    }
}

fn make_style_raw_self_contained(
    raw: &[u8],
    namespace_bindings: &[(String, String)],
) -> Result<Vec<u8>> {
    let external_bindings = namespace_bindings
        .iter()
        .filter(|(prefix, namespace)| {
            !((prefix == "w" && namespace == W_NS)
                || (prefix == "r"
                    && namespace
                        == "http://schemas.openxmlformats.org/officeDocument/2006/relationships"))
        })
        .cloned()
        .collect::<Vec<_>>();
    crate::text::raw_with_external_bindings(raw, &external_bindings)
}

fn parse_style_table_properties(raw: &[u8], word_prefixes: &[String]) -> Result<CT_TblPr> {
    let mut reader = Reader::from_reader(raw);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref start)
                if is_word_element(start.name().as_ref(), b"tblPr", word_prefixes) =>
            {
                let prefixes = word_prefixes_at(start, word_prefixes)?;
                return CT_TblPr::from_xml_with_prefixes(&mut reader, &prefixes);
            }
            Event::Eof => return Ok(CT_TblPr::default()),
            _ => {}
        }
        buf.clear();
    }
}

fn explicit_style_element_has_only_trivia(raw: &[u8]) -> Result<bool> {
    let mut reader = Reader::from_reader(raw);
    let mut inside = false;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(_) if !inside => inside = true,
            Event::End(_) if inside => return Ok(true),
            Event::Text(text) if inside && text.as_ref().iter().all(u8::is_ascii_whitespace) => {}
            Event::Comment(_) | Event::PI(_) if inside => {}
            Event::Eof => return Ok(false),
            _ if inside => return Ok(false),
            _ => {}
        }
        buf.clear();
    }
}

fn parse_conditional_style_properties(
    raw: &[u8],
    word_prefixes: &[String],
) -> Result<(Option<CT_PPr>, Option<CT_TblPr>, Option<CT_TcPr>)> {
    let mut reader = Reader::from_reader(raw);
    let mut paragraph_properties = None;
    let mut table_properties = None;
    let mut cell_properties = None;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref start) => {
                let prefixes = word_prefixes_at(start, word_prefixes)?;
                if is_word_element(start.name().as_ref(), b"pPr", &prefixes) {
                    let captured = capture_element(&mut reader, start)?;
                    paragraph_properties = Some(parse_scoped_ppr(&captured, &prefixes)?);
                } else if is_word_element(start.name().as_ref(), b"tblPr", &prefixes) {
                    table_properties =
                        Some(CT_TblPr::from_xml_with_prefixes(&mut reader, &prefixes)?);
                } else if is_word_element(start.name().as_ref(), b"tcPr", &prefixes) {
                    cell_properties =
                        Some(CT_TcPr::from_xml_with_prefixes(&mut reader, &prefixes)?);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok((paragraph_properties, table_properties, cell_properties))
}

fn serialize_conditional_table_style(conditional: &CT_TblStylePr) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(conditional.raw_xml.as_slice());
    let mut prefixes = vec!["w".to_owned()];
    let mut original_region = None;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref start) => {
                prefixes = word_prefixes_at(start, &prefixes)?;
                original_region = get_word_attr(start, b"type", &prefixes)?;
                break;
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    let original = parse_conditional_style_properties(&conditional.raw_xml, &prefixes)?;
    if original_region.as_deref() == Some(conditional.region.as_str())
        && original.0 == conditional.paragraph_properties
        && original.1 == conditional.table_properties
        && original.2 == conditional.cell_properties
    {
        return Ok(conditional.raw_xml.clone());
    }

    let mut extras = preserved_conditional_style_children(&conditional.raw_xml)?;
    extras.sort_by_key(|(rank, _)| *rank);
    let extra_refs = extras.iter().collect::<Vec<_>>();
    let mut extra_index = 0;
    let mut writer = Writer::new(Vec::new());
    let mut start = BytesStart::new("w:tblStylePr");
    start.push_attribute(("w:type", conditional.region.as_str()));
    push_style_attributes(&mut start, &conditional.extra_attributes);
    writer.write_event(Event::Start(start))?;
    write_style_extras(&mut writer, &extra_refs, &mut extra_index, 0)?;
    if let Some(properties) = &conditional.paragraph_properties {
        properties.to_xml(&mut writer)?;
    }
    write_style_extras(&mut writer, &extra_refs, &mut extra_index, 2)?;
    if let Some(properties) = &conditional.table_properties {
        properties.to_xml(&mut writer)?;
    }
    write_style_extras(&mut writer, &extra_refs, &mut extra_index, 4)?;
    if let Some(properties) = &conditional.cell_properties {
        properties.to_xml(&mut writer)?;
    }
    write_style_extras(&mut writer, &extra_refs, &mut extra_index, u8::MAX)?;
    writer.write_event(Event::End(BytesEnd::new("w:tblStylePr")))?;
    Ok(writer.into_inner())
}

fn preserved_conditional_style_children(raw: &[u8]) -> Result<Vec<(u8, Vec<u8>)>> {
    let mut reader = Reader::from_reader(raw);
    let mut inside = false;
    let mut word_prefixes = vec!["w".to_owned()];
    let mut extras = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref start) if !inside => {
                word_prefixes = word_prefixes_at(start, &word_prefixes)?;
                inside = true;
            }
            Event::Start(ref start) => {
                let prefixes = word_prefixes_at(start, &word_prefixes)?;
                let local = start.local_name();
                if is_word_element(start.name().as_ref(), local.as_ref(), &prefixes)
                    && matches!(local.as_ref(), b"pPr" | b"tblPr" | b"tcPr")
                {
                    reader.read_to_end_into(start.name(), &mut Vec::new())?;
                } else {
                    let rank = match local.as_ref() {
                        b"rPr" => 1,
                        b"trPr" => 3,
                        _ => 5,
                    };
                    extras.push((rank, capture_element(&mut reader, start)?));
                }
            }
            Event::Empty(ref empty) => {
                let prefixes = word_prefixes_at(empty, &word_prefixes)?;
                let local = empty.local_name();
                if !(is_word_element(empty.name().as_ref(), local.as_ref(), &prefixes)
                    && matches!(local.as_ref(), b"pPr" | b"tblPr" | b"tcPr"))
                {
                    let rank = match local.as_ref() {
                        b"rPr" => 1,
                        b"trPr" => 3,
                        _ => 5,
                    };
                    extras.push((rank, capture_empty_element(empty)?));
                }
            }
            Event::End(_) | Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(extras)
}

/// Extract the `w:val` attribute from an element.
fn get_val_attr(e: &BytesStart, word_prefixes: &[String]) -> Result<Option<String>> {
    get_word_attr(e, b"val", word_prefixes)
}

fn get_word_attr(e: &BytesStart, local: &[u8], word_prefixes: &[String]) -> Result<Option<String>> {
    for attr in e.attributes() {
        let attr = attr?;
        if is_word_attribute(attr.key.as_ref(), local, word_prefixes) {
            return Ok(Some(std::str::from_utf8(&attr.value)?.to_string()));
        }
    }
    Ok(None)
}

fn parse_style_toggle(e: &BytesStart, word_prefixes: &[String]) -> Result<Option<bool>> {
    let value = get_word_attr(e, b"val", word_prefixes)?;
    match value.as_deref() {
        None | Some("1" | "true" | "on") => Ok(Some(true)),
        Some("0" | "false" | "off") => Ok(Some(false)),
        Some(value) => Err(OxmlError::InvalidValue(format!(
            "invalid style toggle value: {value}"
        ))),
    }
}

fn parse_style_priority(e: &BytesStart, word_prefixes: &[String]) -> Result<Option<u32>> {
    get_word_attr(e, b"val", word_prefixes)?
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| OxmlError::InvalidValue(format!("invalid style UI priority: {value}")))
        })
        .transpose()
}

fn preserved_style_attribute(raw: &[u8], local: &[u8]) -> Result<Option<String>> {
    let mut reader = Reader::from_reader(raw);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref start) | Event::Empty(ref start) => {
                let prefixes = word_prefixes_at(start, &["w".to_owned()])?;
                return get_word_attr(start, local, &prefixes);
            }
            Event::Eof => return Ok(None),
            _ => {}
        }
        buf.clear();
    }
}

fn preserved_modeled_xml(modeled_xml: &[(u8, Vec<u8>)], rank: u8) -> Option<&[u8]> {
    modeled_xml
        .iter()
        .find(|(candidate, _)| *candidate == rank)
        .map(|(_, raw)| raw.as_slice())
}

fn rewritten_style_scalar_start(
    element: &BytesStart<'_>,
    value: &str,
) -> Result<BytesStart<'static>> {
    let prefixes = word_prefixes_at(element, &["w".to_owned()])?;
    let element_name = std::str::from_utf8(element.name().as_ref())?.to_owned();
    let mut rewritten = BytesStart::new(element_name.clone());
    let mut wrote_value = false;
    for attribute in element.attributes() {
        let attribute = attribute?;
        let key = std::str::from_utf8(attribute.key.as_ref())?.to_owned();
        if is_word_attribute(attribute.key.as_ref(), b"val", &prefixes) {
            if !wrote_value {
                rewritten.push_attribute((key.as_str(), value));
                wrote_value = true;
            }
        } else {
            let attribute_value = attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                .into_owned();
            rewritten.push_attribute((key.as_str(), attribute_value.as_str()));
        }
    }
    if !wrote_value {
        let element_prefix = element_name
            .split_once(':')
            .map(|(prefix, _)| prefix.to_owned());
        let existing_prefix = element_prefix.or_else(|| {
            prefixes
                .iter()
                .find(|prefix| !prefix.is_empty() && !prefix.starts_with('\0'))
                .cloned()
        });
        let prefix = existing_prefix.unwrap_or_else(|| {
            let mut candidate = "rdocxStyle".to_owned();
            let mut suffix = 1usize;
            while element
                .attributes()
                .filter_map(std::result::Result::ok)
                .any(|attribute| attribute.key.as_ref() == format!("xmlns:{candidate}").as_bytes())
            {
                candidate = format!("rdocxStyle{suffix}");
                suffix += 1;
            }
            let declaration = format!("xmlns:{candidate}");
            rewritten.push_attribute((declaration.as_str(), W_NS));
            candidate
        });
        let key = format!("{prefix}:val");
        rewritten.push_attribute((key.as_str(), value));
    }
    Ok(rewritten.into_owned())
}

fn write_rewritten_style_scalar<W: std::io::Write>(
    writer: &mut Writer<W>,
    raw: &[u8],
    value: &str,
) -> Result<()> {
    let mut reader = Reader::from_reader(raw);
    let mut root_pending = true;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(element) if root_pending => {
                root_pending = false;
                writer.write_event(Event::Start(rewritten_style_scalar_start(&element, value)?))?;
            }
            Event::Empty(element) if root_pending => {
                writer.write_event(Event::Empty(rewritten_style_scalar_start(&element, value)?))?;
                break;
            }
            Event::Eof => break,
            event => writer.write_event(event.into_owned())?,
        }
        buf.clear();
    }
    Ok(())
}

fn write_style_string<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    rank: u8,
    value: Option<&str>,
    modeled_xml: &[(u8, Vec<u8>)],
) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if let Some(raw) = preserved_modeled_xml(modeled_xml, rank)
        && preserved_style_attribute(raw, b"val")?.as_deref() == Some(value)
    {
        writer.get_mut().write_all(raw)?;
        return Ok(());
    }
    if let Some(raw) = preserved_modeled_xml(modeled_xml, rank) {
        return write_rewritten_style_scalar(writer, raw, value);
    }
    let mut element = BytesStart::new(name);
    element.push_attribute(("w:val", value));
    writer.write_event(Event::Empty(element))?;
    Ok(())
}

fn write_style_toggle<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    rank: u8,
    value: Option<bool>,
    modeled_xml: &[(u8, Vec<u8>)],
) -> Result<()> {
    if let Some(value) = value {
        if let Some(raw) = preserved_modeled_xml(modeled_xml, rank) {
            let prefixes = vec!["w".to_owned()];
            let mut reader = Reader::from_reader(raw);
            let mut buf = Vec::new();
            let preserved = loop {
                match reader.read_event_into(&mut buf)? {
                    Event::Start(ref start) | Event::Empty(ref start) => {
                        let prefixes = word_prefixes_at(start, &prefixes)?;
                        break parse_style_toggle(start, &prefixes)?;
                    }
                    Event::Eof => break None,
                    _ => {}
                }
                buf.clear();
            };
            if preserved == Some(value) {
                writer.get_mut().write_all(raw)?;
                return Ok(());
            }
            return write_rewritten_style_scalar(writer, raw, if value { "1" } else { "0" });
        }
        let mut element = BytesStart::new(name);
        if !value {
            element.push_attribute(("w:val", "0"));
        }
        writer.write_event(Event::Empty(element))?;
    }
    Ok(())
}

fn write_style_priority<W: std::io::Write>(
    writer: &mut Writer<W>,
    priority: Option<u32>,
    modeled_xml: &[(u8, Vec<u8>)],
) -> Result<()> {
    let Some(priority) = priority else {
        return Ok(());
    };
    if let Some(raw) = preserved_modeled_xml(modeled_xml, 7)
        && preserved_style_attribute(raw, b"val")?
            .as_deref()
            .and_then(|value| value.parse::<u32>().ok())
            == Some(priority)
    {
        writer.get_mut().write_all(raw)?;
        return Ok(());
    }
    let value = priority.to_string();
    if let Some(raw) = preserved_modeled_xml(modeled_xml, 7) {
        return write_rewritten_style_scalar(writer, raw, &value);
    }
    let mut element = BytesStart::new("w:uiPriority");
    element.push_attribute(("w:val", value.as_str()));
    writer.write_event(Event::Empty(element))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_styles() {
        let styles = CT_Styles::new_default();
        let xml = styles.to_xml().unwrap();
        let parsed = CT_Styles::from_xml(&xml).unwrap();

        assert_eq!(parsed.to_xml().unwrap(), xml);

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
    fn linked_style_and_ui_flags_accept_aliases_and_serialize_in_schema_order() {
        let xml = format!(
            r#"<q:styles xmlns:q="{W_NS}" xmlns:x="urn:producer"><q:style q:type="paragraph" q:styleId="Corpus" x:root="&amp;"><q:name q:val="Corpus"> <!--name-trivia--> </q:name><q:next q:val="Normal"><?next kept?></q:next><q:link q:val="CorpusChar"> <!--link-trivia--> </q:link><q:autoRedefine q:val="0" x:keep="yes"/><q:hidden> </q:hidden><q:uiPriority q:val="17"></q:uiPriority><q:semiHidden q:val="false"></q:semiHidden><q:unhideWhenUsed></q:unhideWhenUsed><q:qFormat q:val="1"></q:qFormat><q:locked q:val="true"></q:locked><q:pPr><q:spacing q:after="0"/></q:pPr></q:style></q:styles>"#
        );
        let mut styles = CT_Styles::from_xml(xml.as_bytes()).unwrap();
        let style = styles.get_by_id("Corpus").unwrap();
        assert_eq!(style.next_style.as_deref(), Some("Normal"));
        assert_eq!(style.linked_style.as_deref(), Some("CorpusChar"));
        assert_eq!(style.auto_redefine, Some(false));
        assert_eq!(style.hidden, Some(true));
        assert_eq!(style.ui_priority, Some(17));
        assert_eq!(style.semi_hidden, Some(false));
        assert_eq!(style.unhide_when_used, Some(true));
        assert_eq!(style.quick_format, Some(true));
        assert_eq!(style.locked, Some(true));

        let serialized = String::from_utf8(styles.to_xml().unwrap()).unwrap();
        let positions = [
            "<q:next",
            "<q:link",
            "<q:autoRedefine",
            "<q:hidden",
            "<q:uiPriority",
            "<q:semiHidden",
            "<q:unhideWhenUsed",
            "<q:qFormat",
            "<q:locked",
            "<w:pPr",
        ]
        .map(|token| serialized.find(token).unwrap());
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(serialized.matches("name-trivia").count(), 1);
        assert_eq!(serialized.matches("next kept").count(), 1);
        assert_eq!(serialized.matches("link-trivia").count(), 1);
        assert_eq!(serialized.matches(r#"x:root="&amp;""#).count(), 1);
        assert_eq!(serialized.matches(r#"x:keep="yes""#).count(), 1);
        let reparsed = CT_Styles::from_xml(serialized.as_bytes()).unwrap();
        assert_eq!(reparsed.styles[0], styles.styles[0]);

        styles.styles[0].next_style = Some("Other".to_owned());
        styles.styles[0].auto_redefine = Some(true);
        styles.styles[0].ui_priority = Some(23);
        let changed = String::from_utf8(styles.to_xml().unwrap()).unwrap();
        assert!(changed.contains("<q:next"));
        assert!(changed.contains(r#"q:val="Other""#));
        assert_eq!(changed.matches("next kept").count(), 1);
        assert!(changed.contains("<q:autoRedefine"));
        assert!(changed.contains(r#"q:val="1""#));
        assert_eq!(changed.matches(r#"x:keep="yes""#).count(), 1);
        assert!(changed.contains("<q:uiPriority"));
        assert!(changed.contains(r#"q:val="23""#));
        let reparsed = CT_Styles::from_xml(changed.as_bytes()).unwrap();
        assert_eq!(reparsed.styles[0].next_style.as_deref(), Some("Other"));
        assert_eq!(reparsed.styles[0].auto_redefine, Some(true));
        assert_eq!(reparsed.styles[0].ui_priority, Some(23));
    }

    #[test]
    fn style_parser_rejects_prefixes_that_conflict_with_fixed_writer_bindings() {
        let xml = format!(
            r#"<q:styles xmlns:q="{W_NS}" xmlns:w="urn:not-word"><q:style q:type="paragraph" q:styleId="Corpus"></q:style></q:styles>"#
        );
        let error = CT_Styles::from_xml(xml.as_bytes()).unwrap_err();
        assert!(error.to_string().contains("xmlns:w"));
    }

    #[test]
    fn changed_table_style_properties_keep_one_unknown_child_in_schema_order() {
        let xml = format!(
            r#"<q:styles xmlns:q="{W_NS}" xmlns:x="urn:producer"><q:style q:type="table" q:styleId="Corpus"><q:tblPr><q:tblStyleRowBandSize q:val="2"/><x:producer x:exact="kept"/><q:shd q:val="clear" q:fill="EEEEEE"/></q:tblPr></q:style></q:styles>"#
        );
        let mut styles = CT_Styles::from_xml(xml.as_bytes()).unwrap();
        styles.styles[0]
            .table_properties
            .as_mut()
            .unwrap()
            .shading
            .as_mut()
            .unwrap()
            .fill = Some("D9EAF7".to_owned());

        let serialized = String::from_utf8(styles.to_xml().unwrap()).unwrap();
        assert_eq!(serialized.matches("tblStyleRowBandSize").count(), 1);
        assert_eq!(serialized.matches("x:producer").count(), 1);
        assert!(
            serialized.find("tblStyleRowBandSize").unwrap() < serialized.find("w:shd").unwrap()
        );
        assert_eq!(
            CT_Styles::from_xml(serialized.as_bytes()).unwrap().styles[0]
                .table_properties
                .as_ref()
                .and_then(|properties| properties.shading.as_ref())
                .and_then(|shading| shading.fill.as_deref()),
            Some("D9EAF7")
        );
    }

    #[test]
    fn aliased_style_paragraph_properties_use_ancestor_namespace_scope() {
        let xml = format!(
            r#"<q:styles xmlns:q="{W_NS}" xmlns:ext="urn:producer"><q:style q:type="paragraph" q:styleId="Alias"><q:pPr><ext:jc ext:val="right"/><q:jc q:val="center"/></q:pPr></q:style></q:styles>"#
        );
        let parsed = CT_Styles::from_xml(xml.as_bytes()).unwrap();
        let ppr = parsed.styles[0].ppr.as_ref().unwrap();
        assert_eq!(ppr.jc, Some(crate::shared::ST_Jc::Center));
    }

    #[test]
    fn direct_style_parser_uses_supplied_start_ancestor_scope() {
        let xml = format!(
            r#"<outer xmlns:ext="urn:producer"><q:style xmlns:q="{W_NS}" q:type="paragraph" q:styleId="Direct"><ext:pPr><ext:jc ext:val="right"/></ext:pPr><q:pPr><ext:jc ext:val="right"/><q:jc q:val="center"/></q:pPr></q:style></outer>"#
        );
        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();
        let parsed = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) if element.local_name().as_ref() == b"style" => {
                    break CT_Style::from_xml(&mut reader, element).unwrap();
                }
                Ok(Event::Eof) => panic!("missing style"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        };
        assert_eq!(
            parsed.ppr.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn direct_style_parser_does_not_promote_foreign_start_prefix() {
        let xml = r#"<outer><ext:style xmlns:ext="urn:producer" ext:type="paragraph" ext:styleId="Foreign"><ext:pPr><ext:jc ext:val="right"/></ext:pPr></ext:style></outer>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let parsed = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) if element.local_name().as_ref() == b"style" => {
                    break CT_Style::from_xml(&mut reader, element).unwrap();
                }
                Ok(Event::Eof) => panic!("missing style"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        };
        assert!(parsed.ppr.is_none());
    }

    #[test]
    fn direct_style_parser_accepts_default_word_namespace() {
        let xml = format!(
            r#"<outer xmlns:ext="urn:producer"><style xmlns="{W_NS}" xmlns:w="{W_NS}" w:type="paragraph" w:styleId="Direct"><ext:pPr><ext:jc ext:val="right"/></ext:pPr><pPr><ext:jc ext:val="right"/><jc w:val="center"/></pPr></style></outer>"#
        );
        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();
        let parsed = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) if element.local_name().as_ref() == b"style" => {
                    break CT_Style::from_xml(&mut reader, element).unwrap();
                }
                Ok(Event::Eof) => panic!("missing style"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        };
        assert_eq!(
            parsed.ppr.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn table_style_properties_are_namespace_aware_schema_ordered_and_preserved() {
        let xml = format!(
            r#"<q:styles xmlns:q="{W_NS}" xmlns:bad="urn:not-word" xmlns:x="urn:region"><q:style q:type="table" q:styleId="Dense"><bad:tblPr><bad:tblBorders><bad:top bad:val="double"/></bad:tblBorders></bad:tblPr><q:pPr><q:spacing q:after="40"/></q:pPr><q:rPr><q:b/></q:rPr><q:tblPr><q:tblBorders><q:top q:val="single" q:sz="8" q:color="112233"/></q:tblBorders><ext:keep xmlns:ext="urn:producer" ext:value="byte-identical"/></q:tblPr><q:tblStylePr q:type="firstRow" x:region="&amp;"><q:pPr><q:spacing q:after="0"/></q:pPr><ext:conditional xmlns:ext="urn:producer" ext:value="preserved"/><q:tcPr><q:shd q:val="clear" q:fill="AABBCC"/></q:tcPr></q:tblStylePr></q:style></q:styles>"#
        );
        let mut styles = CT_Styles::from_xml(xml.as_bytes()).unwrap();
        let style = styles.get_by_id("Dense").unwrap();
        assert_eq!(
            style
                .table_properties
                .as_ref()
                .and_then(|properties| properties.borders.as_ref())
                .and_then(|borders| borders.top.as_ref())
                .and_then(|border| border.sz),
            Some(8)
        );
        assert_eq!(style.conditional_table_styles.len(), 1);
        assert_eq!(style.conditional_table_styles[0].region, "firstRow");

        let serialized = String::from_utf8(styles.to_xml().unwrap()).unwrap();
        assert_eq!(serialized.matches("<q:tblPr").count(), 1);
        assert!(
            serialized
                .contains(r#"<ext:keep xmlns:ext="urn:producer" ext:value="byte-identical"/>"#)
        );
        let ppr = serialized.find("<w:pPr").unwrap();
        let rpr = serialized.find("<w:rPr").unwrap();
        let table_properties = serialized.find("<q:tblPr").unwrap();
        let conditional = serialized.find("<q:tblStylePr").unwrap();
        assert!(ppr < rpr && rpr < table_properties && table_properties < conditional);
        assert!(serialized.contains(r#"<bad:tblPr xmlns:bad="urn:not-word">"#));
        CT_Styles::from_xml(serialized.as_bytes()).expect("preserved prefixes remain bound");

        styles.styles[0]
            .table_properties
            .as_mut()
            .unwrap()
            .borders
            .as_mut()
            .unwrap()
            .top
            .as_mut()
            .unwrap()
            .color = Some("445566".to_owned());
        let changed = String::from_utf8(styles.to_xml().unwrap()).unwrap();
        assert_eq!(changed.matches("<w:tblPr>").count(), 1);
        assert_eq!(changed.matches("445566").count(), 1);
        assert!(
            changed.contains(r#"<ext:keep xmlns:ext="urn:producer" ext:value="byte-identical"/>"#)
        );
        styles.styles[0].conditional_table_styles[0]
            .cell_properties
            .as_mut()
            .unwrap()
            .shading
            .as_mut()
            .unwrap()
            .fill = Some("DDEEFF".to_owned());
        styles.styles[0].conditional_table_styles[0].region = "lastRow".to_owned();
        let changed = String::from_utf8(styles.to_xml().unwrap()).unwrap();
        assert_eq!(changed.matches("DDEEFF").count(), 1);
        assert!(!changed.contains("AABBCC"));
        assert!(changed.contains(r#"<w:tblStylePr w:type="lastRow" x:region="&amp;""#));
        assert_eq!(changed.matches(r#"x:region="&amp;""#).count(), 1);
        assert!(!changed.contains(r#"tblStylePr q:type="firstRow""#));
        assert!(
            changed
                .contains(r#"<ext:conditional xmlns:ext="urn:producer" ext:value="preserved"/>"#)
        );
        let reparsed = CT_Styles::from_xml(changed.as_bytes())
            .expect("typed conditional projection remains valid XML");
        assert_eq!(
            reparsed.styles[0].conditional_table_styles[0].region,
            "lastRow"
        );
        assert_eq!(
            reparsed.styles[0].conditional_table_styles[0]
                .cell_properties
                .as_ref()
                .and_then(|properties| properties.shading.as_ref())
                .and_then(|shading| shading.fill.as_deref()),
            Some("DDEEFF")
        );
    }
}

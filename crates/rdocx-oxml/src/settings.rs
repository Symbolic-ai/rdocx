//! Word document settings and document-protection metadata.

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::error::{OxmlError, Result};
use crate::namespace::W_NS;
use crate::properties::{is_word_attribute, is_word_element, word_prefixes_at};

/// The editing operation permitted by `w:documentProtection`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionMode {
    ReadOnly,
    Comments,
    TrackedChanges,
    Forms,
}

impl ProtectionMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "readOnly" => Some(Self::ReadOnly),
            "comments" => Some(Self::Comments),
            "trackedChanges" => Some(Self::TrackedChanges),
            "forms" => Some(Self::Forms),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "readOnly",
            Self::Comments => "comments",
            Self::TrackedChanges => "trackedChanges",
            Self::Forms => "forms",
        }
    }
}

/// The cryptographic provider category recorded by Word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptProviderType {
    RsaAes,
    RsaFull,
    Custom,
}

impl CryptProviderType {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "rsaAES" => Some(Self::RsaAes),
            "rsaFull" => Some(Self::RsaFull),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::RsaAes => "rsaAES",
            Self::RsaFull => "rsaFull",
            Self::Custom => "custom",
        }
    }
}

/// The algorithm class recorded by Word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptAlgorithmClass {
    Hash,
    Custom,
}

impl CryptAlgorithmClass {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "hash" => Some(Self::Hash),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Hash => "hash",
            Self::Custom => "custom",
        }
    }
}

/// The algorithm type recorded by Word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptAlgorithmType {
    Any,
    Custom,
}

impl CryptAlgorithmType {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "typeAny" => Some(Self::Any),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Any => "typeAny",
            Self::Custom => "custom",
        }
    }
}

/// A read-only projection of one valid `w:documentProtection` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentProtection {
    pub mode: ProtectionMode,
    pub enforcement: Option<bool>,
    pub formatting: Option<bool>,
    pub provider_type: Option<CryptProviderType>,
    pub algorithm_class: Option<CryptAlgorithmClass>,
    pub algorithm_type: Option<CryptAlgorithmType>,
    pub algorithm_sid: Option<u32>,
    pub spin_count: Option<u32>,
    pub hash: Option<String>,
    pub salt: Option<String>,
}

/// One valid document variable from `w:settings/w:docVars`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentVariable {
    pub name: String,
    pub value: String,
}

/// The typed contents of a Word settings part.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CT_Settings {
    document_protection: Option<DocumentProtection>,
    document_variables: Vec<DocumentVariable>,
    /// Parsed parts keep their complete producer bytes as the serialization
    /// source. This retains root attributes, child order, whitespace, and all
    /// unmodelled content without interpreting it.
    source_xml: Option<Vec<u8>>,
}

impl CT_Settings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a complete Word settings part.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(false);
        let mut root_prefixes = Vec::new();
        let mut protection = None;
        let mut protection_count = 0usize;
        let mut document_variables = Vec::new();
        let mut doc_vars_depth = None;
        let mut doc_vars_prefixes = Vec::new();
        let mut saw_root = false;
        let mut depth = 0usize;
        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer)? {
                Event::Start(element) => {
                    let inherited = if doc_vars_depth.is_some() {
                        &doc_vars_prefixes
                    } else {
                        &root_prefixes
                    };
                    let prefixes = word_prefixes_at(&element, inherited)?;
                    if !saw_root {
                        if !is_word_element(element.name().as_ref(), b"settings", &prefixes) {
                            return Err(OxmlError::MissingElement("settings root".to_owned()));
                        }
                        root_prefixes = prefixes;
                        saw_root = true;
                        depth = 1;
                    } else {
                        if depth == 1
                            && is_word_element(
                                element.name().as_ref(),
                                b"documentProtection",
                                &prefixes,
                            )
                        {
                            protection_count += 1;
                            protection = parse_document_protection(&element, &prefixes);
                        } else if depth == 1
                            && is_word_element(element.name().as_ref(), b"docVars", &prefixes)
                        {
                            doc_vars_depth = Some(depth + 1);
                            doc_vars_prefixes = prefixes.clone();
                        } else if doc_vars_depth == Some(depth)
                            && is_word_element(element.name().as_ref(), b"docVar", &prefixes)
                            && let Some(variable) = parse_document_variable(&element, &prefixes)
                        {
                            document_variables.push(variable);
                        }
                        depth += 1;
                    }
                }
                Event::Empty(element) => {
                    let inherited = if doc_vars_depth.is_some() {
                        &doc_vars_prefixes
                    } else {
                        &root_prefixes
                    };
                    let prefixes = word_prefixes_at(&element, inherited)?;
                    if !saw_root {
                        if !is_word_element(element.name().as_ref(), b"settings", &prefixes) {
                            return Err(OxmlError::MissingElement("settings root".to_owned()));
                        }
                        saw_root = true;
                    } else if depth == 1
                        && is_word_element(
                            element.name().as_ref(),
                            b"documentProtection",
                            &prefixes,
                        )
                    {
                        protection_count += 1;
                        protection = parse_document_protection(&element, &prefixes);
                    } else if doc_vars_depth == Some(depth)
                        && is_word_element(element.name().as_ref(), b"docVar", &prefixes)
                        && let Some(variable) = parse_document_variable(&element, &prefixes)
                    {
                        document_variables.push(variable);
                    }
                }
                Event::End(_) if depth > 0 => {
                    if doc_vars_depth == Some(depth) {
                        doc_vars_depth = None;
                        doc_vars_prefixes.clear();
                    }
                    depth -= 1;
                }
                Event::Eof => break,
                _ => {}
            }
            buffer.clear();
        }

        if !saw_root {
            return Err(OxmlError::MissingElement("settings root".to_owned()));
        }
        if protection_count != 1 {
            protection = None;
        }
        Ok(Self {
            document_protection: protection,
            document_variables,
            source_xml: Some(xml.to_vec()),
        })
    }

    /// Return valid document-protection metadata, when the part records it.
    pub fn document_protection(&self) -> Option<&DocumentProtection> {
        self.document_protection.as_ref()
    }

    /// Return every valid document variable in package order.
    pub fn document_variables(&self) -> &[DocumentVariable] {
        &self.document_variables
    }

    /// Serialize settings with fixed Word prefixes and schema child order.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        if let Some(source) = &self.source_xml {
            return Ok(source.clone());
        }

        let mut writer = Writer::new(Vec::new());
        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;
        let mut root = BytesStart::new("w:settings");
        root.push_attribute(("xmlns:w", W_NS));
        writer.write_event(Event::Start(root))?;
        if let Some(protection) = &self.document_protection {
            write_document_protection(&mut writer, protection)?;
        }
        writer.write_event(Event::End(BytesEnd::new("w:settings")))?;
        Ok(writer.into_inner())
    }
}

fn parse_document_variable(
    element: &BytesStart<'_>,
    prefixes: &[String],
) -> Option<DocumentVariable> {
    Some(DocumentVariable {
        name: word_attribute(element, b"name", prefixes).ok().flatten()?,
        value: word_attribute(element, b"val", prefixes).ok().flatten()?,
    })
}

fn parse_document_protection(
    element: &BytesStart<'_>,
    prefixes: &[String],
) -> Option<DocumentProtection> {
    let value = |name| word_attribute(element, name, prefixes).ok().flatten();
    let mode = ProtectionMode::parse(&value(b"edit")?)?;
    let enforcement = match value(b"enforcement") {
        Some(value) => Some(parse_on_off(&value)?),
        None => None,
    };
    let formatting = match value(b"formatting") {
        Some(value) => Some(parse_on_off(&value)?),
        None => None,
    };
    let provider_type = match value(b"cryptProviderType") {
        Some(value) => Some(CryptProviderType::parse(&value)?),
        None => None,
    };
    let algorithm_class = match value(b"cryptAlgorithmClass") {
        Some(value) => Some(CryptAlgorithmClass::parse(&value)?),
        None => None,
    };
    let algorithm_type = match value(b"cryptAlgorithmType") {
        Some(value) => Some(CryptAlgorithmType::parse(&value)?),
        None => None,
    };
    let algorithm_sid = match value(b"cryptAlgorithmSid") {
        Some(value) => Some(value.parse::<u32>().ok()?),
        None => None,
    };
    let spin_count = match value(b"cryptSpinCount") {
        Some(value) => Some(value.parse::<u32>().ok()?),
        None => None,
    };
    Some(DocumentProtection {
        mode,
        enforcement,
        formatting,
        provider_type,
        algorithm_class,
        algorithm_type,
        algorithm_sid,
        spin_count,
        hash: value(b"hash"),
        salt: value(b"salt"),
    })
}

fn word_attribute(
    element: &BytesStart<'_>,
    local: &[u8],
    prefixes: &[String],
) -> Result<Option<String>> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        if is_word_attribute(attribute.key.as_ref(), local, prefixes) {
            return Ok(Some(
                attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
}

fn parse_on_off(value: &str) -> Option<bool> {
    match value {
        "1" | "true" | "on" => Some(true),
        "0" | "false" | "off" => Some(false),
        _ => None,
    }
}

fn write_document_protection(
    writer: &mut Writer<Vec<u8>>,
    protection: &DocumentProtection,
) -> Result<()> {
    let mut element = BytesStart::new("w:documentProtection");
    element.push_attribute(("w:edit", protection.mode.as_str()));
    if let Some(formatting) = protection.formatting {
        element.push_attribute(("w:formatting", if formatting { "1" } else { "0" }));
    }
    if let Some(enforcement) = protection.enforcement {
        element.push_attribute(("w:enforcement", if enforcement { "1" } else { "0" }));
    }
    if let Some(provider_type) = protection.provider_type {
        element.push_attribute(("w:cryptProviderType", provider_type.as_str()));
    }
    if let Some(algorithm_class) = protection.algorithm_class {
        element.push_attribute(("w:cryptAlgorithmClass", algorithm_class.as_str()));
    }
    if let Some(algorithm_type) = protection.algorithm_type {
        element.push_attribute(("w:cryptAlgorithmType", algorithm_type.as_str()));
    }
    let algorithm_sid = protection.algorithm_sid.map(|value| value.to_string());
    if let Some(value) = &algorithm_sid {
        element.push_attribute(("w:cryptAlgorithmSid", value.as_str()));
    }
    let spin_count = protection.spin_count.map(|value| value.to_string());
    if let Some(value) = &spin_count {
        element.push_attribute(("w:cryptSpinCount", value.as_str()));
    }
    if let Some(value) = &protection.hash {
        element.push_attribute(("w:hash", value.as_str()));
    }
    if let Some(value) = &protection.salt {
        element.push_attribute(("w:salt", value.as_str()));
    }
    writer.write_event(Event::Empty(element))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(mode: &str, enforcement: &str, formatting: &str) -> Vec<u8> {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><x:settings xmlns:x="{W_NS}" xmlns:p="urn:producer" p:root="kept"><p:before p:v="1"/><x:documentProtection x:edit="{mode}" x:enforcement="{enforcement}" x:formatting="{formatting}" x:cryptProviderType="rsaAES" x:cryptAlgorithmClass="hash" x:cryptAlgorithmType="typeAny" x:cryptAlgorithmSid="14" x:cryptSpinCount="100000" x:hash="HASH-{mode}" x:salt="SALT-{mode}"/><p:after>keep me</p:after></x:settings>"#
        )
        .into_bytes()
    }

    #[test]
    fn document_protection_modes_and_metadata_parse_through_aliases() {
        for (name, expected) in [
            ("readOnly", ProtectionMode::ReadOnly),
            ("comments", ProtectionMode::Comments),
            ("trackedChanges", ProtectionMode::TrackedChanges),
            ("forms", ProtectionMode::Forms),
        ] {
            let parsed = CT_Settings::from_xml(&settings(name, "true", "0")).unwrap();
            let protection = parsed.document_protection().unwrap();
            assert_eq!(protection.mode, expected);
            assert_eq!(protection.enforcement, Some(true));
            assert_eq!(protection.formatting, Some(false));
            assert_eq!(protection.provider_type, Some(CryptProviderType::RsaAes));
            assert_eq!(protection.algorithm_class, Some(CryptAlgorithmClass::Hash));
            assert_eq!(protection.algorithm_type, Some(CryptAlgorithmType::Any));
            assert_eq!(protection.algorithm_sid, Some(14));
            assert_eq!(protection.spin_count, Some(100_000));
            assert_eq!(
                protection.hash.as_deref(),
                Some(format!("HASH-{name}").as_str())
            );
            assert_eq!(
                protection.salt.as_deref(),
                Some(format!("SALT-{name}").as_str())
            );
        }

        let false_and_on = CT_Settings::from_xml(&settings("forms", "false", "on")).unwrap();
        let protection = false_and_on.document_protection().unwrap();
        assert_eq!(protection.enforcement, Some(false));
        assert_eq!(protection.formatting, Some(true));
    }

    #[test]
    fn settings_keep_document_protection_and_unmodelled_children_byte_identical() {
        for mode in ["readOnly", "comments", "trackedChanges", "forms"] {
            let xml = settings(mode, "1", "off");
            let parsed = CT_Settings::from_xml(&xml).unwrap();
            assert_eq!(parsed.to_xml().unwrap(), xml);
        }
    }

    #[test]
    fn document_variables_are_alias_safe_and_leave_settings_bytes_unchanged() {
        let xml = format!(
            r#"<?xml version="1.0"?><q:settings xmlns:q="{W_NS}" xmlns:p="urn:producer"><q:docVars xmlns:v="{W_NS}"><v:docVar v:name="Customer" v:val="Ada"/><v:docVar v:name="Region" v:val="West"></v:docVar><p:docVar p:name="Foreign" p:val="ignored"/><v:docVar v:name="Malformed"/></q:docVars><p:after/></q:settings>"#
        )
        .into_bytes();
        let settings = CT_Settings::from_xml(&xml).unwrap();
        assert_eq!(
            settings.document_variables(),
            [
                DocumentVariable {
                    name: "Customer".to_owned(),
                    value: "Ada".to_owned(),
                },
                DocumentVariable {
                    name: "Region".to_owned(),
                    value: "West".to_owned(),
                },
            ]
        );
        assert_eq!(settings.to_xml().unwrap(), xml);
    }

    #[test]
    fn constructed_settings_use_fixed_prefix_and_schema_order() {
        let settings = CT_Settings {
            document_protection: Some(DocumentProtection {
                mode: ProtectionMode::ReadOnly,
                enforcement: Some(true),
                formatting: Some(false),
                provider_type: Some(CryptProviderType::RsaAes),
                algorithm_class: Some(CryptAlgorithmClass::Hash),
                algorithm_type: Some(CryptAlgorithmType::Any),
                algorithm_sid: Some(14),
                spin_count: Some(100_000),
                hash: Some("HASH".to_owned()),
                salt: Some("SALT".to_owned()),
            }),
            document_variables: Vec::new(),
            source_xml: None,
        };
        let xml = String::from_utf8(settings.to_xml().unwrap()).unwrap();
        assert!(xml.contains("<w:settings xmlns:w="));
        assert!(xml.contains("<w:documentProtection w:edit=\"readOnly\""));
        assert!(xml.find("w:formatting").unwrap() < xml.find("w:enforcement").unwrap());
        assert!(xml.find("w:cryptAlgorithmSid").unwrap() < xml.find("w:cryptSpinCount").unwrap());
        assert!(xml.find("w:cryptSpinCount").unwrap() < xml.find("w:hash").unwrap());
    }
}

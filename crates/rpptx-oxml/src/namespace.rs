use std::collections::HashMap;

use oxml_core::OxmlError;
use oxml_drawing::namespace::A_NS;
use quick_xml::XmlVersion;
use quick_xml::events::BytesStart;

/// PresentationML main namespace URI.
pub const P_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";

/// Fixed PresentationML prefix used when writing XML.
pub const P_PREFIX: &str = "p";

/// Office document relationships namespace URI.
pub const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Fixed office document relationships prefix used when writing XML.
pub const R_PREFIX: &str = "r";

/// Markup Compatibility namespace URI.
pub const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";

pub(crate) const FIXED_MODEL_PREFIXES: &[&str] = &["p", "a", "r"];
pub(crate) const FIXED_SHAPE_TREE_PREFIXES: &[&str] = &["p", "a", "r", "mc"];

#[derive(Clone, Debug, Default)]
pub(crate) struct NamespaceBindings {
    default: Option<String>,
    prefixes: HashMap<String, String>,
}

impl NamespaceBindings {
    pub(crate) fn from_entries(entries: &[(String, String)]) -> Self {
        let mut bindings = Self::default();
        for (prefix, uri) in entries {
            if prefix.is_empty() {
                bindings.default = Some(uri.clone());
            } else {
                bindings.prefixes.insert(prefix.clone(), uri.clone());
            }
        }
        bindings
    }

    pub(crate) fn with_start(&self, start: &BytesStart<'_>) -> Result<Self, OxmlError> {
        let mut bindings = self.clone();
        for (name, value) in all_attributes(start)? {
            if name == "xmlns" {
                bindings.default = Some(value);
            } else if let Some(prefix) = name.strip_prefix("xmlns:") {
                bindings.prefixes.insert(prefix.to_owned(), value);
            }
        }
        Ok(bindings)
    }

    pub(crate) fn element_uri<'a>(&'a self, name: &[u8]) -> Option<&'a str> {
        match qname_prefix(name) {
            Some(prefix) => self.prefixes.get(prefix).map(String::as_str),
            None => self.default.as_deref(),
        }
    }

    pub(crate) fn attribute_uri<'a>(&'a self, name: &[u8]) -> Option<&'a str> {
        qname_prefix(name).and_then(|prefix| self.prefixes.get(prefix).map(String::as_str))
    }

    pub(crate) fn entries(&self) -> Vec<(String, String)> {
        let mut entries: Vec<_> = self
            .prefixes
            .iter()
            .map(|(prefix, uri)| (prefix.clone(), uri.clone()))
            .collect();
        if let Some(uri) = &self.default {
            entries.push((String::new(), uri.clone()));
        }
        entries
    }

    pub(crate) fn reject_writer_conflicts(&self, fixed_prefixes: &[&str]) -> Result<(), OxmlError> {
        for prefix in fixed_prefixes {
            let expected = canonical_uri(prefix).expect("fixed prefixes have canonical URIs");
            if let Some(actual) = self.prefixes.get(*prefix)
                && actual != expected
            {
                return Err(OxmlError::InvalidValue(format!(
                    "xmlns:{prefix} conflicts with the fixed writer namespace"
                )));
            }
        }
        Ok(())
    }
}

pub(crate) fn all_attributes(start: &BytesStart<'_>) -> Result<Vec<(String, String)>, OxmlError> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let name = std::str::from_utf8(attribute.key.as_ref())?.to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?
            .replace(['\r', '\n', '\t'], " ");
        attributes.push((name, value));
    }
    Ok(attributes)
}

pub(crate) fn root_attributes(
    start: &BytesStart<'_>,
    fixed_prefixes: &[&str],
) -> Result<Vec<(String, String)>, OxmlError> {
    Ok(all_attributes(start)?
        .into_iter()
        .filter(|(name, _)| !is_fixed_xmlns(name, fixed_prefixes))
        .collect())
}

pub(crate) fn is_fixed_xmlns(name: &str, fixed_prefixes: &[&str]) -> bool {
    name.strip_prefix("xmlns:")
        .is_some_and(|prefix| fixed_prefixes.contains(&prefix))
}

fn canonical_uri(prefix: &str) -> Option<&'static str> {
    match prefix {
        "p" => Some(P_NS),
        "a" => Some(A_NS),
        "r" => Some(R_NS),
        "mc" => Some(MC_NS),
        _ => None,
    }
}

fn qname_prefix(name: &[u8]) -> Option<&str> {
    let position = name.iter().position(|byte| *byte == b':')?;
    std::str::from_utf8(&name[..position]).ok()
}

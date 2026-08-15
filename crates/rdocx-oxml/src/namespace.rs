//! OOXML namespace constants.

use oxml_core::raw_xml::NamespaceContext;
use quick_xml::events::BytesStart;

use crate::error::Result;

/// WordprocessingML main namespace
pub const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
/// WordprocessingML namespace prefix
pub const W_PREFIX: &[u8] = b"w";

pub use oxml_core::xml::{MC_NS, R_NS, matches_local_name};

/// Match a qualified name using its resolved namespace, with conventional
/// prefix fallback for fragments parsed without an ancestor context.
fn matches_resolved_namespace_name(
    name: &[u8],
    resolved: oxml_core::raw_xml::ResolvedName,
    conventional_prefix: &[u8],
    namespace: &str,
    expected_local: &[u8],
) -> bool {
    if resolved.local.as_bytes() != expected_local {
        return false;
    }
    match resolved.namespace_uri.as_deref() {
        Some(uri) => uri == namespace,
        None => name
            .iter()
            .position(|byte| *byte == b':')
            .is_some_and(|separator| &name[..separator] == conventional_prefix),
    }
}

/// Match an element name using XML namespace rules. A conventional prefix is
/// accepted only when no namespace context is available for a fragment.
pub fn matches_namespace_name(
    name: &[u8],
    context: &NamespaceContext,
    conventional_prefix: &[u8],
    namespace: &str,
    expected_local: &[u8],
) -> bool {
    matches_resolved_namespace_name(
        name,
        context.resolve_element(name),
        conventional_prefix,
        namespace,
        expected_local,
    )
}

/// Match an attribute name without applying the default namespace to an
/// unprefixed attribute.
pub fn matches_namespace_attribute(
    name: &[u8],
    context: &NamespaceContext,
    conventional_prefix: &[u8],
    namespace: &str,
    expected_local: &[u8],
) -> bool {
    matches_resolved_namespace_name(
        name,
        context.resolve_attribute(name),
        conventional_prefix,
        namespace,
        expected_local,
    )
}

/// Match an element after applying namespace declarations on that element.
pub fn matches_namespace_element(
    element: &BytesStart<'_>,
    parent_context: &NamespaceContext,
    conventional_prefix: &[u8],
    namespace: &str,
    expected_local: &[u8],
) -> bool {
    let context = parent_context.with_element(element);
    matches_namespace_name(
        element.name().as_ref(),
        &context,
        conventional_prefix,
        namespace,
        expected_local,
    )
}

pub fn matches_word_name(name: &[u8], context: &NamespaceContext, expected_local: &[u8]) -> bool {
    matches_namespace_name(name, context, W_PREFIX, W_NS, expected_local)
}

pub fn matches_word_attribute(
    name: &[u8],
    context: &NamespaceContext,
    expected_local: &[u8],
) -> bool {
    matches_namespace_attribute(name, context, W_PREFIX, W_NS, expected_local)
}

/// Whether an attribute belongs to the WordprocessingML namespace.
pub fn is_word_attribute(name: &[u8], context: &NamespaceContext) -> bool {
    let resolved = context.resolve_attribute(name);
    match resolved.namespace_uri.as_deref() {
        Some(uri) => uri == W_NS,
        None => name
            .iter()
            .position(|byte| *byte == b':')
            .is_some_and(|separator| &name[..separator] == W_PREFIX),
    }
}

pub fn matches_word_element(
    element: &BytesStart<'_>,
    parent_context: &NamespaceContext,
    expected_local: &[u8],
) -> bool {
    matches_namespace_element(element, parent_context, W_PREFIX, W_NS, expected_local)
}

/// Whether an element belongs to the WordprocessingML namespace.
///
/// Fragments without an ancestor namespace context retain the conventional
/// `w:` prefix fallback used by the specific-name matchers above.
pub fn is_word_element(element: &BytesStart<'_>, parent_context: &NamespaceContext) -> bool {
    let context = parent_context.with_element(element);
    let name = element.name();
    let resolved = context.resolve_element(name.as_ref());
    match resolved.namespace_uri.as_deref() {
        Some(uri) => uri == W_NS,
        None => name
            .as_ref()
            .iter()
            .position(|byte| *byte == b':')
            .is_some_and(|separator| &name.as_ref()[..separator] == W_PREFIX),
    }
}

/// Whether an element carries a non-declaration attribute outside the modeled
/// WordprocessingML and relationship attribute sets.
pub fn has_unmodeled_attributes(
    element: &BytesStart<'_>,
    parent_context: &NamespaceContext,
    word_attributes: &[&[u8]],
    relationship_attributes: &[&[u8]],
) -> Result<bool> {
    let context = parent_context.with_element(element);
    for attribute in element.attributes() {
        let attribute = attribute?;
        let name = attribute.key.as_ref();
        if name == b"xmlns" || name.starts_with(b"xmlns:") {
            continue;
        }
        if word_attributes
            .iter()
            .any(|expected| matches_word_attribute(name, &context, expected))
            || relationship_attributes
                .iter()
                .any(|expected| matches_namespace_attribute(name, &context, b"r", R_NS, expected))
        {
            continue;
        }
        return Ok(true);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use oxml_core::xml::StrictXmlDocument;
    use regex::Regex;

    use super::W_NS;

    const PARSER_CORPUS: &str = concat!(
        include_str!("properties.rs"),
        include_str!("text.rs"),
        include_str!("table.rs"),
        include_str!("numbering.rs"),
        include_str!("styles.rs"),
        include_str!("document.rs"),
        include_str!("drawing.rs"),
    );

    #[test]
    fn parser_corpus_names_obey_strict_spelling_and_namespace_properties() {
        let name_pattern = Regex::new(r#"(?:b)?\"([A-Za-z_][A-Za-z0-9_.-]*)\""#).unwrap();
        let names: BTreeSet<_> = name_pattern
            .captures_iter(PARSER_CORPUS)
            .map(|capture| capture[1].to_string())
            .collect();
        for required in [
            "pPr",
            "rPr",
            "p",
            "tbl",
            "tr",
            "tc",
            "numbering",
            "style",
            "sectPr",
            "drawing",
        ] {
            assert!(names.contains(required), "parser corpus omitted {required}");
        }

        for local in names {
            let empty =
                format!(r#"<w:root xmlns:w="{W_NS}"><w:{local} w:val="A &amp; B"/></w:root>"#);
            let expanded = format!(
                r#"<x:root xmlns:x="{W_NS}"><x:{local} x:val="A &amp; B"></x:{local}></x:root>"#
            );
            let foreign = format!(
                r#"<w:root xmlns:w="{W_NS}" xmlns:f="urn:foreign"><f:{local} f:val="A &amp; B"/></w:root>"#
            );
            let empty = StrictXmlDocument::parse(empty.as_bytes()).unwrap();
            let expanded = StrictXmlDocument::parse(expanded.as_bytes()).unwrap();
            let foreign = StrictXmlDocument::parse(foreign.as_bytes()).unwrap();

            assert_eq!(empty, expanded, "spelling or prefix changed {local}");
            assert_ne!(empty, foreign, "foreign namespace matched {local}");
        }
    }
}

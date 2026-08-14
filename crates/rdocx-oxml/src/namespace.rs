//! OOXML namespace constants.

use oxml_core::raw_xml::NamespaceContext;
use quick_xml::events::BytesStart;

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

pub fn matches_word_element(
    element: &BytesStart<'_>,
    parent_context: &NamespaceContext,
    expected_local: &[u8],
) -> bool {
    matches_namespace_element(element, parent_context, W_PREFIX, W_NS, expected_local)
}

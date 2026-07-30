//! Shared OOXML namespace and attribute helpers.

use quick_xml::XmlVersion;
use quick_xml::events::BytesStart;

use crate::error::Result;

/// Relationships namespace.
pub const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Markup Compatibility namespace.
pub const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";

/// Return the local portion of a possibly prefixed XML name.
pub fn local_name(name: &[u8]) -> &[u8] {
    match name.iter().position(|&byte| byte == b':') {
        Some(pos) => &name[pos + 1..],
        None => name,
    }
}

/// Check whether an XML name has the expected local portion.
pub fn matches_local_name(name: &[u8], expected: &[u8]) -> bool {
    local_name(name) == expected
}

/// Return a named attribute value, matching with or without a prefix.
pub fn get_attr(element: &BytesStart<'_>, name: &[u8]) -> Option<String> {
    element
        .attributes()
        .flatten()
        .find(|attr| matches_local_name(attr.key.as_ref(), name))
        .and_then(|attr| std::str::from_utf8(&attr.value).ok().map(str::to_owned))
}

/// Return non-`vt` prefixed namespace declarations needed by raw XML children.
pub(crate) fn extra_namespace_declarations(
    element: &BytesStart<'_>,
) -> Result<Vec<(String, String)>> {
    let mut declarations = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        if key.starts_with(b"xmlns:") && key != b"xmlns:vt" {
            let name = std::str::from_utf8(key)?.to_owned();
            let value = attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                .into_owned();
            declarations.push((name, value));
        }
    }
    Ok(declarations)
}

#[cfg(test)]
mod tests {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    use super::*;

    #[test]
    fn local_names_match_with_or_without_a_prefix() {
        assert_eq!(local_name(b"w:document"), b"document");
        assert_eq!(local_name(b"document"), b"document");
        assert!(matches_local_name(b"p:sld", b"sld"));
        assert!(!matches_local_name(b"p:sld", b"slide"));
    }

    #[test]
    fn attributes_match_with_or_without_a_prefix() {
        let mut reader = Reader::from_str(r#"<item r:id="rId7" plain="value"/>"#);
        let mut buf = Vec::new();
        let Event::Empty(element) = reader.read_event_into(&mut buf).unwrap() else {
            panic!("expected empty element");
        };

        assert_eq!(get_attr(&element, b"id").as_deref(), Some("rId7"));
        assert_eq!(get_attr(&element, b"plain").as_deref(), Some("value"));
        assert_eq!(get_attr(&element, b"missing"), None);
    }
}

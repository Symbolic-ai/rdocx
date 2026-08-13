//! Raw XML capture utilities for preserving unknown elements during round-trip.

use std::collections::HashSet;
use std::io::Write;

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::Result;

/// The resolved identity of an XML element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedName {
    /// The qualified name as it appeared in the source XML.
    pub qualified: String,
    /// The local part of the qualified name.
    pub local: String,
    /// The namespace URI resolved at the element's source location.
    pub namespace_uri: Option<String>,
}

/// Namespace bindings that are in scope at an XML node.
///
/// The empty string represents the default namespace. Keeping the complete
/// in-scope set makes a preserved subtree self-describing even when its
/// prefixes were declared by an ancestor outside the captured bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NamespaceContext {
    bindings: Vec<(String, String)>,
}

impl NamespaceContext {
    pub fn new(bindings: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut context = Self::default();
        for (prefix, uri) in bindings {
            context.bind(prefix, uri);
        }
        context
    }

    pub fn with_element(&self, element: &BytesStart<'_>) -> Self {
        let mut context = self.clone();
        for attribute in element.attributes().flatten() {
            let key = attribute.key.as_ref();
            let prefix = if key == b"xmlns" {
                Some(String::new())
            } else {
                key.strip_prefix(b"xmlns:")
                    .map(|value| String::from_utf8_lossy(value).into_owned())
            };
            if let Some(prefix) = prefix {
                context.bind(
                    prefix,
                    String::from_utf8_lossy(attribute.value.as_ref()).into_owned(),
                );
            }
        }
        context
    }

    pub fn resolve(&self, qualified_name: &[u8]) -> ResolvedName {
        let qualified = String::from_utf8_lossy(qualified_name).into_owned();
        let (prefix, local) = qualified
            .split_once(':')
            .map_or(("", qualified.as_str()), |(prefix, local)| (prefix, local));
        let local = local.to_string();
        let namespace_uri = self.namespace_uri(prefix).map(str::to_owned);
        ResolvedName {
            qualified,
            local,
            namespace_uri,
        }
    }

    pub fn namespace_uri(&self, prefix: &str) -> Option<&str> {
        self.bindings
            .iter()
            .find(|(candidate, _)| candidate == prefix)
            .map(|(_, uri)| uri.as_str())
    }

    pub fn bindings(&self) -> &[(String, String)] {
        &self.bindings
    }

    fn bind(&mut self, prefix: String, uri: String) {
        if let Some((_, current_uri)) = self
            .bindings
            .iter_mut()
            .find(|(candidate, _)| candidate == &prefix)
        {
            *current_uri = uri;
        } else {
            self.bindings.push((prefix, uri));
        }
    }
}

/// An unmodelled XML subtree together with its source namespace context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawXml {
    bytes: Vec<u8>,
    name: ResolvedName,
    namespaces: NamespaceContext,
}

impl RawXml {
    pub fn new(bytes: Vec<u8>, name: ResolvedName, namespaces: NamespaceContext) -> Self {
        Self {
            bytes,
            name,
            namespaces,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, qualified_name: &[u8], namespaces: NamespaceContext) -> Self {
        let name = namespaces.resolve(qualified_name);
        Self::new(bytes, name, namespaces)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn name(&self) -> &ResolvedName {
        &self.name
    }

    pub fn namespaces(&self) -> &NamespaceContext {
        &self.namespaces
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.bytes)
    }

    /// Write the preserved subtree in an output namespace context.
    ///
    /// The original bytes are emitted unchanged when the destination already
    /// provides every binding they use. Missing inherited declarations are
    /// added only when moving the subtree would otherwise leave a prefix
    /// unresolved.
    pub fn write_to_with_context<W: Write>(
        &self,
        writer: &mut W,
        output_context: &NamespaceContext,
    ) -> std::io::Result<()> {
        let mut reader = Reader::from_reader(self.bytes.as_slice());
        let mut buffer = Vec::new();
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(std::io::Error::other)?;
        let consumed = reader.buffer_position() as usize;

        match event {
            Event::Start(element) => {
                let mut element = element.into_owned();
                add_inherited_namespaces(
                    &mut element,
                    &self.bytes,
                    &self.namespaces,
                    output_context,
                );
                Writer::new(&mut *writer)
                    .write_event(Event::Start(element))
                    .map_err(std::io::Error::other)?;
                writer.write_all(&self.bytes[consumed..])
            }
            Event::Empty(element) => {
                let mut element = element.into_owned();
                add_inherited_namespaces(
                    &mut element,
                    &self.bytes,
                    &self.namespaces,
                    output_context,
                );
                Writer::new(writer)
                    .write_event(Event::Empty(element))
                    .map_err(std::io::Error::other)
            }
            _ => writer.write_all(&self.bytes),
        }
    }
}

fn add_inherited_namespaces(
    element: &mut BytesStart<'_>,
    raw_xml: &[u8],
    namespaces: &NamespaceContext,
    output_context: &NamespaceContext,
) {
    let declared = element
        .attributes()
        .flatten()
        .filter_map(|attribute| {
            let key = attribute.key.as_ref();
            if key == b"xmlns" {
                Some(String::new())
            } else {
                key.strip_prefix(b"xmlns:")
                    .map(|prefix| String::from_utf8_lossy(prefix).into_owned())
            }
        })
        .collect::<HashSet<_>>();
    let used = used_prefixes(raw_xml, namespaces);
    let declarations = namespaces
        .bindings()
        .iter()
        .filter(|(prefix, uri)| {
            prefix != "xml"
                && used.contains(prefix)
                && !declared.contains(prefix)
                && output_context.namespace_uri(prefix) != Some(uri.as_str())
        })
        .map(|(prefix, uri)| {
            let name = if prefix.is_empty() {
                "xmlns".to_string()
            } else {
                format!("xmlns:{prefix}")
            };
            (name, uri.clone())
        })
        .collect::<Vec<_>>();
    for (name, uri) in &declarations {
        element.push_attribute((name.as_str(), uri.as_str()));
    }
}

fn used_prefixes(raw_xml: &[u8], namespaces: &NamespaceContext) -> HashSet<String> {
    let known = namespaces
        .bindings()
        .iter()
        .map(|(prefix, _)| prefix.as_str())
        .collect::<HashSet<_>>();
    let mut used = HashSet::new();
    let mut reader = Reader::from_reader(raw_xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                record_name_prefix(element.name().as_ref(), true, &known, &mut used);
                for attribute in element.attributes().flatten() {
                    let name = attribute.key.as_ref();
                    if name == b"xmlns" || name.starts_with(b"xmlns:") {
                        continue;
                    }
                    record_name_prefix(name, false, &known, &mut used);
                    for token in attribute.value.split(|byte| byte.is_ascii_whitespace()) {
                        let prefix = token.split(|byte| *byte == b':').next().unwrap_or_default();
                        if let Ok(prefix) = std::str::from_utf8(prefix)
                            && known.contains(prefix)
                        {
                            used.insert(prefix.to_string());
                        }
                    }
                }
            }
            Ok(Event::End(element)) => {
                record_name_prefix(element.name().as_ref(), true, &known, &mut used);
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buffer.clear();
    }
    used
}

fn record_name_prefix(
    name: &[u8],
    element_name: bool,
    known: &HashSet<&str>,
    used: &mut HashSet<String>,
) {
    let prefix = name
        .iter()
        .position(|byte| *byte == b':')
        .map(|index| &name[..index])
        .unwrap_or_default();
    if let Ok(prefix) = std::str::from_utf8(prefix)
        && ((element_name && prefix.is_empty()) || known.contains(prefix))
    {
        used.insert(prefix.to_string());
    }
}

/// Capture a full XML subtree (from after the start tag through the matching end tag)
/// and return it as raw bytes. The returned bytes include the start tag, all children,
/// and the end tag.
pub fn capture_element(reader: &mut Reader<&[u8]>, start: &BytesStart) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());

    // Write the start tag
    writer.write_event(Event::Start(start.to_owned()))?;

    // Track nesting depth for the tag name
    let tag_name = start.name().as_ref().to_vec();
    let mut depth = 1u32;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == tag_name {
                    depth += 1;
                }
                writer.write_event(Event::Start(e.to_owned()))?;
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == tag_name {
                    depth -= 1;
                }
                writer.write_event(Event::End(e.to_owned()))?;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Empty(ref e)) => {
                writer.write_event(Event::Empty(e.to_owned()))?;
            }
            Ok(Event::Text(ref e)) => {
                writer.write_event(Event::Text(e.to_owned().into_owned()))?;
            }
            Ok(Event::CData(ref e)) => {
                writer.write_event(Event::CData(e.to_owned().into_owned()))?;
            }
            Ok(Event::Comment(ref e)) => {
                writer.write_event(Event::Comment(e.to_owned().into_owned()))?;
            }
            Ok(Event::PI(ref e)) => {
                writer.write_event(Event::PI(e.to_owned().into_owned()))?;
            }
            Ok(Event::Decl(ref e)) => {
                writer.write_event(Event::Decl(e.to_owned().into_owned()))?;
            }
            Ok(Event::DocType(ref e)) => {
                writer.write_event(Event::DocType(e.to_owned().into_owned()))?;
            }
            // Entity references are their own event since quick-xml 0.41;
            // re-emit them verbatim so captured markup round-trips unchanged.
            Ok(Event::GeneralRef(ref e)) => {
                writer.write_event(Event::GeneralRef(e.to_owned().into_owned()))?;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    Ok(writer.into_inner())
}

/// Capture an empty (self-closing) element as raw bytes.
pub fn capture_empty_element(e: &BytesStart) -> Result<Vec<u8>> {
    let mut writer = Writer::new(Vec::new());
    writer.write_event(Event::Empty(e.to_owned()))?;
    Ok(writer.into_inner())
}

/// Capture a full XML subtree and attach the namespace context in scope at
/// its start element.
pub fn capture_raw_element(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart,
    parent_context: &NamespaceContext,
) -> Result<RawXml> {
    let context = parent_context.with_element(start);
    let name = context.resolve(start.name().as_ref());
    Ok(RawXml::new(capture_element(reader, start)?, name, context))
}

/// Capture an empty element and attach the namespace context in scope at it.
pub fn capture_raw_empty_element(
    start: &BytesStart,
    parent_context: &NamespaceContext,
) -> Result<RawXml> {
    let context = parent_context.with_element(start);
    let name = context.resolve(start.name().as_ref());
    Ok(RawXml::new(capture_empty_element(start)?, name, context))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_simple_element() {
        let xml = r#"<root><child>text</child><sibling/></root>"#;
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        // Read past <root>
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"child" => {
                    let captured = capture_element(&mut reader, e).unwrap();
                    let s = String::from_utf8(captured).unwrap();
                    assert!(s.contains("<child>"));
                    assert!(s.contains("text"));
                    assert!(s.contains("</child>"));
                    return;
                }
                _ => {}
            }
            buf.clear();
        }
    }

    #[test]
    fn capture_empty() {
        let xml = r#"<item attr="val"/>"#;
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        loop {
            if let Ok(Event::Empty(ref e)) = reader.read_event_into(&mut buf) {
                let captured = capture_empty_element(e).unwrap();
                let s = String::from_utf8(captured).unwrap();
                assert!(s.contains("item"));
                assert!(s.contains("attr"));
                return;
            }
            buf.clear();
        }
    }

    #[test]
    fn capture_nested_element() {
        let xml = r#"<outer><inner><deep>data</deep></inner></outer>"#;
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"outer" => {
                    let captured = capture_element(&mut reader, e).unwrap();
                    let s = String::from_utf8(captured).unwrap();
                    assert!(s.contains("<outer>"));
                    assert!(s.contains("<inner>"));
                    assert!(s.contains("<deep>"));
                    assert!(s.contains("data"));
                    assert!(s.contains("</deep>"));
                    assert!(s.contains("</inner>"));
                    assert!(s.contains("</outer>"));
                    return;
                }
                _ => {}
            }
            buf.clear();
        }
    }

    #[test]
    fn unmodelled_subtree_is_preserved_byte_for_byte() {
        let xml = br#"<root><x:item x:id="7"><x:child>one &amp; two</x:child><!--note--></x:item></root>"#;
        let mut reader = Reader::from_reader(xml.as_slice());
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"x:item" => {
                    assert_eq!(
                        capture_element(&mut reader, e).unwrap(),
                        br#"<x:item x:id="7"><x:child>one &amp; two</x:child><!--note--></x:item>"#
                    );
                    return;
                }
                Ok(Event::Eof) => panic!("no unmodelled subtree"),
                _ => {}
            }
            buf.clear();
        }
    }

    #[test]
    fn captured_raw_xml_retains_inherited_and_local_namespace_context() {
        let xml = br#"<w:root xmlns:w="urn:word" xmlns:x="urn:outer"><x:item xmlns:y="urn:local"><y:child/></x:item></w:root>"#;
        let mut reader = Reader::from_reader(xml.as_slice());
        let mut root_context = NamespaceContext::default();
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) if element.name().as_ref() == b"w:root" => {
                    root_context = root_context.with_element(element);
                }
                Ok(Event::Start(ref element)) if element.name().as_ref() == b"x:item" => {
                    let raw = capture_raw_element(&mut reader, element, &root_context).unwrap();
                    assert_eq!(raw.name().qualified, "x:item");
                    assert_eq!(raw.name().namespace_uri.as_deref(), Some("urn:outer"));
                    assert_eq!(raw.namespaces().namespace_uri("w"), Some("urn:word"));
                    assert_eq!(raw.namespaces().namespace_uri("y"), Some("urn:local"));
                    assert_eq!(
                        raw.bytes(),
                        br#"<x:item xmlns:y="urn:local"><y:child/></x:item>"#
                    );
                    return;
                }
                Ok(Event::Eof) => panic!("raw element not found"),
                _ => {}
            }
            buf.clear();
        }
    }

    #[test]
    fn local_namespace_declaration_shadows_inherited_binding() {
        let parent = NamespaceContext::new([("x".to_string(), "urn:outer".to_string())]);
        let xml = br#"<x:item xmlns:x="urn:inner"/>"#;
        let mut reader = Reader::from_reader(xml.as_slice());
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref element)) => {
                    let raw = capture_raw_empty_element(element, &parent).unwrap();
                    assert_eq!(raw.name().namespace_uri.as_deref(), Some("urn:inner"));
                    assert_eq!(raw.namespaces().namespace_uri("x"), Some("urn:inner"));
                    return;
                }
                Ok(Event::Eof) => panic!("raw element not found"),
                _ => {}
            }
            buf.clear();
        }
    }

    #[test]
    fn writing_raw_xml_adds_only_inherited_namespaces_that_it_uses() {
        let context = NamespaceContext::new([
            ("z".to_string(), "urn:used".to_string()),
            ("unused".to_string(), "urn:unused".to_string()),
        ]);
        let raw = RawXml::from_bytes(b"<z:item value=\"kept\"/>".to_vec(), b"z:item", context);
        let mut output = Vec::new();
        raw.write_to_with_context(&mut output, &NamespaceContext::default())
            .unwrap();
        let output = String::from_utf8(output).unwrap();

        assert_eq!(output, r#"<z:item value="kept" xmlns:z="urn:used"/>"#);
        assert!(!output.contains("urn:unused"));
    }

    #[test]
    fn writing_raw_xml_in_a_matching_context_keeps_original_bytes() {
        let context = NamespaceContext::new([("z".to_string(), "urn:used".to_string())]);
        let raw = RawXml::from_bytes(b"<z:item/>".to_vec(), b"z:item", context.clone());
        let mut output = Vec::new();
        raw.write_to_with_context(&mut output, &context).unwrap();

        assert_eq!(output, raw.bytes());
    }

    #[test]
    fn writing_raw_xml_does_not_duplicate_local_namespace_declarations() {
        let context = NamespaceContext::new([("z".to_string(), "urn:used".to_string())]);
        let raw = RawXml::from_bytes(
            br#"<z:item xmlns:z="urn:used"/>"#.to_vec(),
            b"z:item",
            context,
        );
        let mut output = Vec::new();
        raw.write_to(&mut output).unwrap();

        assert_eq!(output, raw.bytes());
    }
}

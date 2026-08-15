//! Raw XML capture utilities for preserving unknown elements during round-trip.

use std::collections::HashSet;
use std::io::Write;
use std::ops::Range;
use std::sync::{Arc, OnceLock};

use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};

use crate::error::{OxmlError, Result};

const MAX_CAPTURE_DEPTH: usize = 64;
const XML_NAMESPACE_URI: &str = "http://www.w3.org/XML/1998/namespace";
const XMLNS_NAMESPACE_URI: &str = "http://www.w3.org/2000/xmlns/";

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
#[derive(Debug)]
struct NamespaceFrame {
    parent: Option<Arc<NamespaceFrame>>,
    declarations: Vec<(String, String)>,
}

#[derive(Clone)]
pub struct NamespaceContext {
    frame: Arc<NamespaceFrame>,
    flattened_bindings: Arc<OnceLock<Vec<(String, String)>>>,
}

impl Default for NamespaceContext {
    fn default() -> Self {
        Self {
            frame: Arc::new(NamespaceFrame {
                parent: None,
                declarations: Vec::new(),
            }),
            flattened_bindings: Arc::new(OnceLock::new()),
        }
    }
}

impl std::fmt::Debug for NamespaceContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NamespaceContext")
            .field("bindings", &self.bindings())
            .finish()
    }
}

impl PartialEq for NamespaceContext {
    fn eq(&self, other: &Self) -> bool {
        self.bindings() == other.bindings()
    }
}

impl Eq for NamespaceContext {}

impl NamespaceContext {
    pub fn new(bindings: impl IntoIterator<Item = (String, String)>) -> Self {
        Self::default().with_bindings(bindings.into_iter().collect())
    }

    pub fn with_element(&self, element: &BytesStart<'_>) -> Self {
        let mut declarations = Vec::new();
        for attribute in element.attributes().flatten() {
            let key = attribute.key.as_ref();
            let prefix = if key == b"xmlns" {
                Some(String::new())
            } else {
                key.strip_prefix(b"xmlns:")
                    .map(|value| String::from_utf8_lossy(value).into_owned())
            };
            if let Some(prefix) = prefix {
                declarations.push((
                    prefix,
                    String::from_utf8_lossy(attribute.value.as_ref()).into_owned(),
                ));
            }
        }
        self.with_bindings(declarations)
    }

    /// Extend this context with namespace declarations from `element`.
    ///
    /// Unlike [`Self::with_element`], this strict variant propagates malformed
    /// attributes and decodes namespace values before installing them. An
    /// empty namespace value removes the binding for that scope.
    pub fn try_with_element(&self, element: &BytesStart<'_>) -> Result<Self> {
        let mut declarations = Vec::new();
        for attribute in element.attributes() {
            let attribute = attribute?;
            let key = attribute.key.as_ref();
            let declared_prefix = if key == b"xmlns" {
                Some(None)
            } else {
                key.strip_prefix(b"xmlns:")
                    .map(|value| {
                        let prefix = std::str::from_utf8(value)?;
                        if !is_ncname(prefix) || prefix == "xmlns" {
                            return Err(OxmlError::InvalidValue(format!(
                                "invalid XML namespace prefix: {prefix}"
                            )));
                        }
                        Ok(Some(prefix.to_owned()))
                    })
                    .transpose()?
            };
            if let Some(declared_prefix) = declared_prefix {
                let uri = attribute
                    .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
                    .into_owned();
                validate_namespace_declaration(declared_prefix.as_deref(), &uri)?;
                declarations.push((declared_prefix.unwrap_or_default(), uri));
            }
        }
        Ok(self.with_bindings(declarations))
    }

    /// Resolve an element name. An unprefixed element inherits the default
    /// namespace, as required by XML Namespaces.
    pub fn resolve_element(&self, qualified_name: &[u8]) -> ResolvedName {
        self.resolve_name(qualified_name, true)
    }

    /// Resolve an attribute name. Default namespaces never apply to
    /// unprefixed attributes.
    pub fn resolve_attribute(&self, qualified_name: &[u8]) -> ResolvedName {
        self.resolve_name(qualified_name, false)
    }

    pub(crate) fn resolve_element_strict(&self, qualified_name: &[u8]) -> Result<ResolvedName> {
        self.resolve_name_strict(qualified_name, true)
    }

    pub(crate) fn resolve_attribute_strict(&self, qualified_name: &[u8]) -> Result<ResolvedName> {
        self.resolve_name_strict(qualified_name, false)
    }

    /// Resolve an element name.
    pub fn resolve(&self, qualified_name: &[u8]) -> ResolvedName {
        self.resolve_element(qualified_name)
    }

    fn resolve_name(&self, qualified_name: &[u8], use_default_namespace: bool) -> ResolvedName {
        let qualified = String::from_utf8_lossy(qualified_name).into_owned();
        let (prefix, local) = qualified.split_once(':').map_or_else(
            || (None, qualified.as_str()),
            |(prefix, local)| (Some(prefix), local),
        );
        let local = local.to_string();
        let namespace_uri = match prefix {
            Some(prefix) => self.namespace_uri(prefix),
            None if use_default_namespace => self.namespace_uri(""),
            None => None,
        }
        .map(str::to_owned);
        ResolvedName {
            qualified,
            local,
            namespace_uri,
        }
    }

    fn resolve_name_strict(
        &self,
        qualified_name: &[u8],
        use_default_namespace: bool,
    ) -> Result<ResolvedName> {
        let qualified = std::str::from_utf8(qualified_name)?;
        let (prefix, local) = parse_qname(qualified)?;
        let namespace_uri = match prefix {
            Some("xml") => Some(XML_NAMESPACE_URI),
            Some(prefix) => Some(self.namespace_uri(prefix).ok_or_else(|| {
                OxmlError::InvalidValue(format!("unbound XML prefix in {qualified}"))
            })?),
            None if use_default_namespace => self.namespace_uri(""),
            None => None,
        }
        .map(str::to_owned);
        Ok(ResolvedName {
            qualified: qualified.to_string(),
            local: local.to_string(),
            namespace_uri,
        })
    }

    pub fn namespace_uri(&self, prefix: &str) -> Option<&str> {
        let mut frame = Some(self.frame.as_ref());
        while let Some(current) = frame {
            if let Some((_, uri)) = current
                .declarations
                .iter()
                .rev()
                .find(|(candidate, _)| candidate == prefix)
            {
                return (!uri.is_empty()).then_some(uri.as_str());
            }
            frame = current.parent.as_deref();
        }
        None
    }

    pub fn bindings(&self) -> &[(String, String)] {
        self.flattened_bindings.get_or_init(|| {
            let mut frames = Vec::new();
            let mut frame = Some(self.frame.as_ref());
            while let Some(current) = frame {
                frames.push(current);
                frame = current.parent.as_deref();
            }

            let mut bindings = Vec::new();
            for frame in frames.into_iter().rev() {
                for (prefix, uri) in &frame.declarations {
                    if uri.is_empty() {
                        bindings.retain(|(candidate, _)| candidate != prefix);
                    } else if let Some((_, current_uri)) = bindings
                        .iter_mut()
                        .find(|(candidate, _)| candidate == prefix)
                    {
                        *current_uri = uri.clone();
                    } else {
                        bindings.push((prefix.clone(), uri.clone()));
                    }
                }
            }
            bindings
        })
    }

    fn with_bindings(&self, declarations: Vec<(String, String)>) -> Self {
        if declarations.is_empty() {
            return self.clone();
        }
        Self {
            frame: Arc::new(NamespaceFrame {
                parent: Some(Arc::clone(&self.frame)),
                declarations,
            }),
            flattened_bindings: Arc::new(OnceLock::new()),
        }
    }
}

fn validate_namespace_declaration(prefix: Option<&str>, uri: &str) -> Result<()> {
    if uri == XMLNS_NAMESPACE_URI {
        return Err(OxmlError::InvalidValue(
            "the xmlns namespace URI cannot be declared".to_string(),
        ));
    }
    match prefix {
        Some("xml") if uri != XML_NAMESPACE_URI => Err(OxmlError::InvalidValue(
            "the xml prefix must use its reserved namespace URI".to_string(),
        )),
        Some("xml") => Ok(()),
        Some(_) if uri == XML_NAMESPACE_URI => Err(OxmlError::InvalidValue(
            "only the xml prefix can use its reserved namespace URI".to_string(),
        )),
        Some(_) if uri.is_empty() => Err(OxmlError::InvalidValue(
            "a prefixed XML namespace cannot be empty".to_string(),
        )),
        None if uri == XML_NAMESPACE_URI => Err(OxmlError::InvalidValue(
            "the XML namespace URI cannot be the default namespace".to_string(),
        )),
        _ => Ok(()),
    }
}

fn parse_qname(qualified: &str) -> Result<(Option<&str>, &str)> {
    let mut parts = qualified.split(':');
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    if parts.next().is_some() {
        return Err(OxmlError::InvalidValue(format!(
            "XML qualified name contains multiple colons: {qualified}"
        )));
    }
    let (prefix, local) = second.map_or((None, first), |local| (Some(first), local));
    if prefix.is_some_and(|prefix| !is_ncname(prefix)) || !is_ncname(local) {
        return Err(OxmlError::InvalidValue(format!(
            "invalid XML qualified name: {qualified}"
        )));
    }
    Ok((prefix, local))
}

fn is_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(is_ncname_start)
        && characters.all(|character| is_ncname_start(character) || is_ncname_continue(character))
}

fn is_ncname_start(character: char) -> bool {
    matches!(
        character,
        'A'..='Z'
            | '_'
            | 'a'..='z'
            | '\u{00c0}'..='\u{00d6}'
            | '\u{00d8}'..='\u{00f6}'
            | '\u{00f8}'..='\u{02ff}'
            | '\u{0370}'..='\u{037d}'
            | '\u{037f}'..='\u{1fff}'
            | '\u{200c}'..='\u{200d}'
            | '\u{2070}'..='\u{218f}'
            | '\u{2c00}'..='\u{2fef}'
            | '\u{3001}'..='\u{d7ff}'
            | '\u{f900}'..='\u{fdcf}'
            | '\u{fdf0}'..='\u{fffd}'
            | '\u{10000}'..='\u{effff}'
    )
}

fn is_ncname_continue(character: char) -> bool {
    matches!(
        character,
        '-' | '.' | '0'..='9' | '\u{00b7}' | '\u{0300}'..='\u{036f}' | '\u{203f}'..='\u{2040}'
    )
}

pub(crate) fn is_xml_name(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|character| character == ':' || is_ncname_start(character))
        && characters.all(|character| {
            character == ':' || is_ncname_start(character) || is_ncname_continue(character)
        })
}

/// An unmodelled XML subtree together with its source namespace context.
#[derive(Debug, Clone)]
pub struct RawXml {
    source: Arc<[u8]>,
    range: Range<usize>,
    name: ResolvedName,
    namespaces: NamespaceContext,
}

impl RawXml {
    pub fn new(bytes: Vec<u8>, name: ResolvedName, namespaces: NamespaceContext) -> Self {
        let range = 0..bytes.len();
        Self {
            source: Arc::from(bytes),
            range,
            name,
            namespaces,
        }
    }

    pub(crate) fn from_shared_source(
        source: Arc<[u8]>,
        range: Range<usize>,
        name: ResolvedName,
        namespaces: NamespaceContext,
    ) -> Result<Self> {
        if range.start > range.end || range.end > source.len() {
            return Err(OxmlError::InvalidValue(
                "raw XML source range is out of bounds".to_string(),
            ));
        }
        Ok(Self {
            source,
            range,
            name,
            namespaces,
        })
    }

    pub fn from_bytes(bytes: Vec<u8>, qualified_name: &[u8], namespaces: NamespaceContext) -> Self {
        let name = namespaces.resolve(qualified_name);
        Self::new(bytes, name, namespaces)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.source[self.range.clone()]
    }

    #[cfg(test)]
    pub(crate) fn shares_source_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.source, &other.source)
    }

    pub fn name(&self) -> &ResolvedName {
        &self.name
    }

    pub fn namespaces(&self) -> &NamespaceContext {
        &self.namespaces
    }

    /// Whether the root contains nested elements or non-whitespace text.
    /// Malformed preserved XML is treated as content so callers fail closed.
    pub fn has_child_content(&self) -> bool {
        let mut reader = Reader::from_reader(self.bytes());
        reader.config_mut().trim_text(false);
        let mut saw_root = false;
        let mut depth = 0usize;
        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(_)) if !saw_root => {
                    saw_root = true;
                    depth = 1;
                }
                Ok(Event::Empty(_)) if !saw_root => return false,
                Ok(Event::Start(_) | Event::Empty(_)) => return true,
                Ok(Event::Text(text)) if depth > 0 => {
                    if !text.as_ref().iter().all(u8::is_ascii_whitespace) {
                        return true;
                    }
                }
                Ok(Event::CData(text)) if depth > 0 => {
                    if !text.as_ref().iter().all(u8::is_ascii_whitespace) {
                        return true;
                    }
                }
                Ok(Event::GeneralRef(_)) if depth > 0 => return true,
                Ok(Event::End(_)) if depth > 0 => depth -= 1,
                Ok(Event::Eof) => return false,
                Err(_) => return true,
                _ => {}
            }
            buffer.clear();
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(self.bytes())
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
        let raw_xml = self.bytes();
        let mut reader = Reader::from_reader(raw_xml);
        let mut buffer = Vec::new();
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(std::io::Error::other)?;
        let consumed = reader.buffer_position() as usize;

        match event {
            Event::Start(element) => {
                let mut element = element.into_owned();
                add_inherited_namespaces(&mut element, raw_xml, &self.namespaces, output_context);
                Writer::new(&mut *writer)
                    .write_event(Event::Start(element))
                    .map_err(std::io::Error::other)?;
                writer.write_all(&raw_xml[consumed..])
            }
            Event::Empty(element) => {
                let mut element = element.into_owned();
                add_inherited_namespaces(&mut element, raw_xml, &self.namespaces, output_context);
                Writer::new(writer)
                    .write_event(Event::Empty(element))
                    .map_err(std::io::Error::other)
            }
            _ => writer.write_all(raw_xml),
        }
    }
}

impl PartialEq for RawXml {
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
            && self.name == other.name
            && self.namespaces == other.namespaces
    }
}

impl Eq for RawXml {}

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

    // Bound and track the full subtree depth, not only repeated root names.
    let tag_name = start.name().as_ref().to_vec();
    let mut depth = 1usize;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                depth = depth.checked_add(1).ok_or_else(|| {
                    OxmlError::InvalidValue("XML nesting depth overflow".to_string())
                })?;
                if depth > MAX_CAPTURE_DEPTH {
                    return Err(OxmlError::InvalidValue(format!(
                        "XML nesting exceeds {MAX_CAPTURE_DEPTH} elements"
                    )));
                }
                writer.write_event(Event::Start(e.to_owned()))?;
            }
            Ok(Event::End(ref e)) => {
                depth = depth.checked_sub(1).ok_or_else(|| {
                    OxmlError::InvalidValue("XML nesting depth underflow".to_string())
                })?;
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
            Ok(Event::Eof) => {
                return Err(OxmlError::MissingElement(format!(
                    "closing {}",
                    String::from_utf8_lossy(&tag_name)
                )));
            }
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
    let name = context.resolve_element(start.name().as_ref());
    Ok(RawXml::new(capture_element(reader, start)?, name, context))
}

/// Capture an empty element and attach the namespace context in scope at it.
pub fn capture_raw_empty_element(
    start: &BytesStart,
    parent_context: &NamespaceContext,
) -> Result<RawXml> {
    let context = parent_context.with_element(start);
    let name = context.resolve_element(start.name().as_ref());
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
    fn raw_xml_reports_only_semantic_child_content() {
        let context = NamespaceContext::default();
        let expanded_empty = RawXml::from_bytes(
            b"<w:bookmarkStart> \n<!--kept--></w:bookmarkStart>".to_vec(),
            b"w:bookmarkStart",
            context.clone(),
        );
        let nested = RawXml::from_bytes(
            b"<w:bookmarkStart><w:r><w:t>visible</w:t></w:r></w:bookmarkStart>".to_vec(),
            b"w:bookmarkStart",
            context,
        );

        assert!(!expanded_empty.has_child_content());
        assert!(nested.has_child_content());
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

    #[test]
    fn default_namespace_applies_to_elements_but_not_attributes() {
        let context = NamespaceContext::new([("".to_string(), "urn:default".to_string())]);

        assert_eq!(
            context.resolve_element(b"item").namespace_uri.as_deref(),
            Some("urn:default")
        );
        assert_eq!(context.resolve_attribute(b"value").namespace_uri, None);
    }

    #[test]
    fn capturing_rejects_excessive_subtree_depth() {
        let xml = format!("{}value{}", "<n>".repeat(66), "</n>".repeat(66));
        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();
        let start = loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(element) => break element.into_owned(),
                _ => buf.clear(),
            }
        };

        assert!(matches!(
            capture_element(&mut reader, &start),
            Err(OxmlError::InvalidValue(_))
        ));
    }
}

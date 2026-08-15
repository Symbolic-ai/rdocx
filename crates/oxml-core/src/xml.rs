//! Shared OOXML namespace, attribute, and strict parsing helpers.

use std::collections::HashSet;
use std::sync::Arc;

use quick_xml::XmlVersion;
use quick_xml::events::{BytesCData, BytesDecl, BytesStart, Event};
use quick_xml::reader::Reader;

use crate::error::{OxmlError, Result};
use crate::raw_xml::{NamespaceContext, RawXml, ResolvedName, is_xml_name};
use crate::xml_text;

/// Default maximum element nesting accepted by the strict XML substrate.
pub const DEFAULT_MAX_XML_DEPTH: usize = 128;

/// Default maximum semantic XML nodes accepted in one part.
pub const DEFAULT_MAX_XML_NODES: usize = 1_000_000;

/// Intrinsic limits applied while parsing one complete XML part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrictXmlLimits {
    pub max_depth: usize,
    pub max_nodes: usize,
}

impl Default for StrictXmlLimits {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_XML_DEPTH,
            max_nodes: DEFAULT_MAX_XML_NODES,
        }
    }
}

/// One already-resolved and already-decoded XML attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictXmlAttribute {
    name: ResolvedName,
    value: String,
}

impl StrictXmlAttribute {
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_named(&self, namespace_uri: Option<&str>, local: &str) -> bool {
        resolved_name_matches(&self.name, namespace_uri, local)
    }
}

/// A normalized XML child. Character data and entity references are folded
/// into adjacent decoded text nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrictXmlNode {
    Element(Box<StrictXmlElement>),
    Text(String),
}

impl StrictXmlNode {
    pub fn as_element(&self) -> Option<&StrictXmlElement> {
        match self {
            Self::Element(element) => Some(element),
            Self::Text(_) => None,
        }
    }

    pub fn into_element(self) -> Option<StrictXmlElement> {
        match self {
            Self::Element(element) => Some(*element),
            Self::Text(_) => None,
        }
    }

    fn is_semantic(&self) -> bool {
        match self {
            Self::Element(_) => true,
            Self::Text(text) => !text.chars().all(char::is_whitespace),
        }
    }
}

/// One normalized XML element with its original subtree retained for lossless
/// preservation when a typed parser leaves it unconsumed.
#[derive(Debug, Clone)]
pub struct StrictXmlElement {
    name: ResolvedName,
    attributes: Vec<StrictXmlAttribute>,
    children: Vec<StrictXmlNode>,
    raw_xml: RawXml,
}

impl StrictXmlElement {
    pub fn is_named(&self, namespace_uri: Option<&str>, local: &str) -> bool {
        resolved_name_matches(&self.name, namespace_uri, local)
    }

    pub fn into_raw_xml(self) -> RawXml {
        self.raw_xml
    }

    pub fn parse<T>(
        self,
        parser: impl FnOnce(&mut StrictXmlCursor) -> Result<T>,
    ) -> Result<StrictXmlParsed<T>> {
        let mut cursor = StrictXmlCursor {
            name: self.name,
            attributes: self.attributes.into_iter().map(Some).collect(),
            children: self.children.into_iter().map(Some).collect(),
        };
        let value = parser(&mut cursor)?;
        Ok(StrictXmlParsed {
            value,
            leftovers: cursor.finish(),
        })
    }
}

impl PartialEq for StrictXmlElement {
    fn eq(&self, other: &Self) -> bool {
        semantic_name_eq(&self.name, &other.name)
            && self.attributes.len() == other.attributes.len()
            && self.attributes.iter().all(|attribute| {
                other.attributes.iter().any(|candidate| {
                    semantic_name_eq(&attribute.name, &candidate.name)
                        && attribute.value() == candidate.value()
                })
            })
            && self.children == other.children
    }
}

impl Eq for StrictXmlElement {}

/// A complete strict XML document containing exactly one root element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictXmlDocument {
    root: StrictXmlElement,
}

impl StrictXmlDocument {
    pub fn parse(xml: &[u8]) -> Result<Self> {
        Self::parse_with_limits(xml, StrictXmlLimits::default())
    }

    pub fn parse_with_limits(xml: &[u8], limits: StrictXmlLimits) -> Result<Self> {
        parse_strict_document(xml, limits)
    }

    pub fn root(&self) -> &StrictXmlElement {
        &self.root
    }

    pub fn into_root(self) -> StrictXmlElement {
        self.root
    }
}

/// Explicit unconsumed content returned by a typed element parser.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StrictXmlLeftovers {
    pub attributes: Vec<StrictXmlAttribute>,
    pub children: Vec<StrictXmlNode>,
}

/// The value produced by a typed parser and every semantic item it left
/// unconsumed. Construction is owned by [`StrictXmlElement::parse`], so a
/// successful typed parse cannot omit its leftovers result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictXmlParsed<T> {
    pub value: T,
    pub leftovers: StrictXmlLeftovers,
}

impl StrictXmlLeftovers {
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty() && self.children.is_empty()
    }
}

/// Consumption cursor used by typed parsers. Anything not taken from the
/// cursor is returned by [`Self::finish`] in source order.
#[derive(Debug)]
pub struct StrictXmlCursor {
    name: ResolvedName,
    attributes: Vec<Option<StrictXmlAttribute>>,
    children: Vec<Option<StrictXmlNode>>,
}

impl StrictXmlCursor {
    pub fn is_named(&self, namespace_uri: Option<&str>, local: &str) -> bool {
        resolved_name_matches(&self.name, namespace_uri, local)
    }

    pub fn take_attribute(&mut self, namespace_uri: Option<&str>, local: &str) -> Option<String> {
        self.attributes
            .iter_mut()
            .find(|slot| {
                slot.as_ref()
                    .is_some_and(|attribute| attribute.is_named(namespace_uri, local))
            })
            .and_then(Option::take)
            .map(|attribute| attribute.value)
    }

    pub fn child(&self, index: usize) -> Option<&StrictXmlNode> {
        self.children.get(index).and_then(Option::as_ref)
    }

    pub fn take_child(&mut self, index: usize) -> Option<StrictXmlNode> {
        self.children.get_mut(index).and_then(Option::take)
    }

    pub fn child_slots(&self) -> usize {
        self.children.len()
    }

    fn finish(self) -> StrictXmlLeftovers {
        StrictXmlLeftovers {
            attributes: self.attributes.into_iter().flatten().collect(),
            children: self
                .children
                .into_iter()
                .flatten()
                .filter(StrictXmlNode::is_semantic)
                .collect(),
        }
    }
}

struct ElementBuilder {
    name: ResolvedName,
    attributes: Vec<StrictXmlAttribute>,
    children: Vec<StrictXmlNode>,
    context: NamespaceContext,
    start_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocumentPhase {
    Start,
    Prolog,
    Root,
    Epilog,
}

fn parse_strict_document(xml: &[u8], limits: StrictXmlLimits) -> Result<StrictXmlDocument> {
    std::str::from_utf8(xml)?;
    let source: Arc<[u8]> = Arc::from(xml);
    let mut reader = Reader::from_reader(source.as_ref());
    reader.config_mut().trim_text(false);
    reader.config_mut().enable_all_checks(true);
    let mut stack: Vec<ElementBuilder> = Vec::new();
    let mut root = None;
    let mut saw_doctype = false;
    let mut phase = DocumentPhase::Start;
    let mut node_count = 0usize;
    let mut buffer = Vec::new();

    loop {
        let start_offset = reader.buffer_position() as usize;
        let event = reader.read_event_into(&mut buffer)?;
        let end_offset = reader.buffer_position() as usize;

        match event {
            Event::Start(element) => {
                reserve_node(&mut node_count, limits.max_nodes)?;
                begin_root_if_needed(&stack, &mut phase)?;
                let depth = stack.len().checked_add(1).ok_or_else(|| {
                    OxmlError::InvalidValue("XML nesting depth overflow".to_string())
                })?;
                if depth > limits.max_depth {
                    return Err(OxmlError::InvalidValue(format!(
                        "XML nesting depth exceeds {}",
                        limits.max_depth
                    )));
                }
                let parent_context = stack
                    .last()
                    .map(|builder| &builder.context)
                    .cloned()
                    .unwrap_or_default();
                reserve_attributes(&element, &mut node_count, limits.max_nodes)?;
                let context = parent_context.try_with_element(&element)?;
                let name = resolve_strict_element_name(&context, &element)?;
                let attributes = parse_strict_attributes(&context, &element)?;
                stack.push(ElementBuilder {
                    name,
                    attributes,
                    children: Vec::new(),
                    context,
                    start_offset,
                });
            }
            Event::Empty(element) => {
                reserve_node(&mut node_count, limits.max_nodes)?;
                let root_element = stack.is_empty();
                begin_root_if_needed(&stack, &mut phase)?;
                let depth = stack.len().checked_add(1).ok_or_else(|| {
                    OxmlError::InvalidValue("XML nesting depth overflow".to_string())
                })?;
                if depth > limits.max_depth {
                    return Err(OxmlError::InvalidValue(format!(
                        "XML nesting depth exceeds {}",
                        limits.max_depth
                    )));
                }
                let parent_context = stack
                    .last()
                    .map(|builder| &builder.context)
                    .cloned()
                    .unwrap_or_default();
                reserve_attributes(&element, &mut node_count, limits.max_nodes)?;
                let context = parent_context.try_with_element(&element)?;
                let name = resolve_strict_element_name(&context, &element)?;
                let attributes = parse_strict_attributes(&context, &element)?;
                let parsed = StrictXmlElement {
                    raw_xml: RawXml::from_shared_source(
                        Arc::clone(&source),
                        start_offset..end_offset,
                        name.clone(),
                        context,
                    )?,
                    name,
                    attributes,
                    children: Vec::new(),
                };
                append_element(&mut stack, &mut root, parsed)?;
                if root_element {
                    phase = DocumentPhase::Epilog;
                }
            }
            Event::End(_) => {
                let builder = stack.pop().ok_or_else(|| {
                    OxmlError::UnexpectedElement("closing element outside root".to_string())
                })?;
                let root_element = stack.is_empty();
                let parsed = StrictXmlElement {
                    raw_xml: RawXml::from_shared_source(
                        Arc::clone(&source),
                        builder.start_offset..end_offset,
                        builder.name.clone(),
                        builder.context,
                    )?,
                    name: builder.name,
                    attributes: builder.attributes,
                    children: builder.children,
                };
                append_element(&mut stack, &mut root, parsed)?;
                if root_element {
                    phase = DocumentPhase::Epilog;
                }
            }
            Event::Text(text) => {
                reserve_node(&mut node_count, limits.max_nodes)?;
                let decoded = xml_text::decode_plain(&text)?;
                if stack.is_empty() {
                    accept_literal_whitespace(&mut phase, &decoded)?;
                } else {
                    append_text(&mut stack, decoded)?;
                }
            }
            Event::CData(text) => {
                reserve_node(&mut node_count, limits.max_nodes)?;
                if stack.is_empty() {
                    return Err(OxmlError::UnexpectedElement(
                        "CDATA outside XML root".to_string(),
                    ));
                }
                let decoded = decode_cdata(&text)?;
                append_text(&mut stack, decoded)?;
            }
            Event::GeneralRef(reference) => {
                reserve_node(&mut node_count, limits.max_nodes)?;
                if stack.is_empty() {
                    return Err(OxmlError::UnexpectedElement(
                        "entity reference outside XML root".to_string(),
                    ));
                }
                let decoded = xml_text::resolve_entity(&reference)?;
                append_text(&mut stack, decoded)?;
            }
            Event::Decl(declaration) => {
                if phase != DocumentPhase::Start || !stack.is_empty() {
                    return Err(OxmlError::UnexpectedElement(
                        "misplaced XML declaration".to_string(),
                    ));
                }
                validate_declaration(&declaration)?;
                phase = DocumentPhase::Prolog;
            }
            Event::DocType(_) => {
                if !matches!(phase, DocumentPhase::Start | DocumentPhase::Prolog)
                    || !stack.is_empty()
                    || saw_doctype
                {
                    return Err(OxmlError::UnexpectedElement(
                        "misplaced document type".to_string(),
                    ));
                }
                saw_doctype = true;
                phase = DocumentPhase::Prolog;
            }
            Event::Comment(_) => {
                reserve_node(&mut node_count, limits.max_nodes)?;
                if stack.is_empty() && phase == DocumentPhase::Start {
                    phase = DocumentPhase::Prolog;
                }
            }
            Event::PI(instruction) => {
                reserve_node(&mut node_count, limits.max_nodes)?;
                validate_processing_instruction(instruction.target())?;
                if stack.is_empty() && phase == DocumentPhase::Start {
                    phase = DocumentPhase::Prolog;
                }
            }
            Event::Eof => break,
        }
        buffer.clear();
    }

    if !stack.is_empty() {
        return Err(OxmlError::MissingElement("closing XML root".to_string()));
    }
    let root = root.ok_or_else(|| OxmlError::MissingElement("XML root".to_string()))?;
    Ok(StrictXmlDocument { root })
}

fn reserve_node(count: &mut usize, max_nodes: usize) -> Result<()> {
    *count = count
        .checked_add(1)
        .ok_or_else(|| OxmlError::InvalidValue("XML node count overflow".to_string()))?;
    if *count > max_nodes {
        return Err(OxmlError::InvalidValue(format!(
            "XML node count exceeds {max_nodes}"
        )));
    }
    Ok(())
}

fn resolve_strict_element_name(
    context: &NamespaceContext,
    element: &BytesStart<'_>,
) -> Result<ResolvedName> {
    context.resolve_element_strict(element.name().as_ref())
}

fn reserve_attributes(
    element: &BytesStart<'_>,
    node_count: &mut usize,
    max_nodes: usize,
) -> Result<()> {
    for attribute in element.attributes() {
        attribute?;
        reserve_node(node_count, max_nodes)?;
    }
    Ok(())
}

fn parse_strict_attributes(
    context: &NamespaceContext,
    element: &BytesStart<'_>,
) -> Result<Vec<StrictXmlAttribute>> {
    let mut parsed = Vec::new();
    let mut expanded_names = HashSet::new();
    for attribute in element.attributes() {
        let attribute = attribute?;
        let qualified = attribute.key.as_ref();
        if qualified == b"xmlns" || qualified.starts_with(b"xmlns:") {
            continue;
        }
        let name = context.resolve_attribute_strict(qualified)?;
        let semantic_name = (name.namespace_uri.clone(), name.local.clone());
        if !expanded_names.insert(semantic_name) {
            return Err(OxmlError::InvalidValue(format!(
                "duplicate expanded XML attribute: {}",
                name.qualified
            )));
        }
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
            .into_owned();
        parsed.push(StrictXmlAttribute { name, value });
    }
    Ok(parsed)
}

fn begin_root_if_needed(stack: &[ElementBuilder], phase: &mut DocumentPhase) -> Result<()> {
    if !stack.is_empty() {
        return Ok(());
    }
    if *phase == DocumentPhase::Epilog {
        return Err(OxmlError::UnexpectedElement(
            "multiple XML roots".to_string(),
        ));
    }
    *phase = DocumentPhase::Root;
    Ok(())
}

fn accept_literal_whitespace(phase: &mut DocumentPhase, text: &str) -> Result<()> {
    if !text.chars().all(char::is_whitespace) {
        let location = if *phase == DocumentPhase::Epilog {
            "after"
        } else {
            "before"
        };
        return Err(OxmlError::UnexpectedElement(format!(
            "non-whitespace text {location} XML root"
        )));
    }
    if *phase == DocumentPhase::Start {
        *phase = DocumentPhase::Prolog;
    }
    Ok(())
}

fn validate_declaration(declaration: &BytesDecl<'_>) -> Result<()> {
    match declaration.version()?.as_ref() {
        b"1.0" => {}
        version => {
            return Err(OxmlError::InvalidValue(format!(
                "unsupported XML version: {}",
                String::from_utf8_lossy(version)
            )));
        }
    }
    if let Some(encoding) = declaration.encoding() {
        let encoding = encoding?;
        if !encoding.as_ref().eq_ignore_ascii_case(b"UTF-8") {
            return Err(OxmlError::InvalidValue(format!(
                "unsupported XML encoding: {}",
                String::from_utf8_lossy(encoding.as_ref())
            )));
        }
    }
    if let Some(standalone) = declaration.standalone() {
        match standalone?.as_ref() {
            b"yes" | b"no" => {}
            value => {
                return Err(OxmlError::InvalidValue(format!(
                    "invalid XML standalone value: {}",
                    String::from_utf8_lossy(value)
                )));
            }
        }
    }
    Ok(())
}

fn validate_processing_instruction(target: &[u8]) -> Result<()> {
    let target = std::str::from_utf8(target)?;
    if !is_xml_name(target) || target.eq_ignore_ascii_case("xml") {
        return Err(OxmlError::InvalidValue(format!(
            "invalid XML processing instruction target: {target}"
        )));
    }
    Ok(())
}

fn append_element(
    stack: &mut [ElementBuilder],
    root: &mut Option<StrictXmlElement>,
    element: StrictXmlElement,
) -> Result<()> {
    if let Some(parent) = stack.last_mut() {
        parent
            .children
            .push(StrictXmlNode::Element(Box::new(element)));
    } else if root.replace(element).is_some() {
        return Err(OxmlError::UnexpectedElement(
            "multiple XML roots".to_string(),
        ));
    }
    Ok(())
}

fn append_text(stack: &mut [ElementBuilder], text: String) -> Result<()> {
    let parent = stack
        .last_mut()
        .ok_or_else(|| OxmlError::UnexpectedElement("text outside XML root".to_string()))?;
    if let Some(StrictXmlNode::Text(current)) = parent.children.last_mut() {
        current.push_str(&text);
    } else {
        parent.children.push(StrictXmlNode::Text(text));
    }
    Ok(())
}

fn decode_cdata(text: &BytesCData<'_>) -> Result<String> {
    text.decode()
        .map(|value| value.into_owned())
        .map_err(|error| OxmlError::Xml(error.into()))
}

fn resolved_name_matches(name: &ResolvedName, namespace_uri: Option<&str>, local: &str) -> bool {
    name.namespace_uri.as_deref() == namespace_uri && name.local == local
}

fn semantic_name_eq(left: &ResolvedName, right: &ResolvedName) -> bool {
    left.namespace_uri == right.namespace_uri && left.local == right.local
}

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

    fn take_only_element_child(document: StrictXmlDocument) -> StrictXmlElement {
        let parsed = document
            .into_root()
            .parse(|cursor| {
                let index = (0..cursor.child_slots())
                    .find(|&index| {
                        cursor
                            .child(index)
                            .and_then(StrictXmlNode::as_element)
                            .is_some()
                    })
                    .ok_or_else(|| OxmlError::MissingElement("XML child".to_string()))?;
                cursor
                    .take_child(index)
                    .and_then(StrictXmlNode::into_element)
                    .ok_or_else(|| OxmlError::MissingElement("XML child".to_string()))
            })
            .unwrap();
        assert!(parsed.leftovers.is_empty());
        parsed.value
    }

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

    #[test]
    fn strict_xml_normalizes_empty_and_expanded_property_spellings() {
        let property_corpus = [
            "vanish",
            "numId",
            "ilvl",
            "gridBefore",
            "gridAfter",
            "gridSpan",
            "vMerge",
            "bottom",
            "headerReference",
            "footerReference",
        ];

        for local in property_corpus {
            let empty =
                format!(r#"<w:root xmlns:w="urn:word"><w:{local} w:val="A &amp; B"/></w:root>"#);
            let expanded = format!(
                r#"<x:root xmlns:x="urn:word"><x:{local} x:val="A &amp; B"></x:{local}></x:root>"#
            );
            let empty = StrictXmlDocument::parse(empty.as_bytes()).unwrap();
            let expanded = StrictXmlDocument::parse(expanded.as_bytes()).unwrap();

            assert_eq!(empty, expanded, "property {local}");
            let child = take_only_element_child(empty);
            let parsed = child
                .parse(|cursor| Ok(cursor.take_attribute(Some("urn:word"), "val")))
                .unwrap();
            assert_eq!(parsed.value.as_deref(), Some("A & B"));
            assert!(parsed.leftovers.is_empty());
        }
    }

    #[test]
    fn strict_xml_keeps_foreign_namespace_twins_semantically_distinct() {
        let word = StrictXmlDocument::parse(br#"<w:root xmlns:w="urn:word"><w:vanish/></w:root>"#)
            .unwrap();
        let foreign = StrictXmlDocument::parse(
            br#"<w:root xmlns:w="urn:word" xmlns:x="urn:foreign"><x:vanish/></w:root>"#,
        )
        .unwrap();

        assert_ne!(word, foreign);
        assert!(take_only_element_child(word).is_named(Some("urn:word"), "vanish"));
        assert!(take_only_element_child(foreign).is_named(Some("urn:foreign"), "vanish"));
    }

    #[test]
    fn strict_xml_rejects_unbound_names_and_duplicate_expanded_attributes() {
        for xml in [
            br#"<w:root/>"#.as_slice(),
            br#"<root xmlns:w="urn:word" xmlns:x="urn:word" w:val="one" x:val="two"/>"#.as_slice(),
        ] {
            assert!(StrictXmlDocument::parse(xml).is_err());
        }
    }

    #[test]
    fn strict_xml_enforces_document_shape_and_intrinsic_budgets() {
        for xml in [
            br#"<root/><second/>"#.as_slice(),
            br#"<root>"#.as_slice(),
            b"<root>before\xffafter</root>".as_slice(),
        ] {
            assert!(StrictXmlDocument::parse(xml).is_err());
        }

        assert!(
            StrictXmlDocument::parse_with_limits(
                br#"<root><child/></root>"#,
                StrictXmlLimits {
                    max_depth: 1,
                    max_nodes: 10,
                },
            )
            .is_err()
        );
        assert!(
            StrictXmlDocument::parse_with_limits(
                br#"<root><child/></root>"#,
                StrictXmlLimits {
                    max_depth: 10,
                    max_nodes: 1,
                },
            )
            .is_err()
        );

        StrictXmlDocument::parse_with_limits(
            br#"<root><child/></root>"#,
            StrictXmlLimits {
                max_depth: 2,
                max_nodes: 2,
            },
        )
        .unwrap();

        StrictXmlDocument::parse_with_limits(
            br#"<root value="one"/>"#,
            StrictXmlLimits {
                max_depth: 1,
                max_nodes: 2,
            },
        )
        .unwrap();
        assert!(
            StrictXmlDocument::parse_with_limits(
                br#"<root value="one"/>"#,
                StrictXmlLimits {
                    max_depth: 1,
                    max_nodes: 1,
                },
            )
            .is_err()
        );

        StrictXmlDocument::parse_with_limits(
            br#"<root xmlns:w="urn:word"/>"#,
            StrictXmlLimits {
                max_depth: 1,
                max_nodes: 2,
            },
        )
        .unwrap();
        assert!(
            StrictXmlDocument::parse_with_limits(
                br#"<root xmlns:w="urn:word"/>"#,
                StrictXmlLimits {
                    max_depth: 1,
                    max_nodes: 1,
                },
            )
            .is_err()
        );
    }

    #[test]
    fn strict_xml_rejects_misplaced_declarations_and_epilog_content() {
        for xml in [
            b" \n<?xml version=\"1.0\"?><root/>".as_slice(),
            b"<!--before--><?xml version=\"1.0\"?><root/>".as_slice(),
            b"<?before value?><?xml version=\"1.0\"?><root/>".as_slice(),
            b"<!DOCTYPE root><?xml version=\"1.0\"?><root/>".as_slice(),
            b"<root/><![CDATA[ ]]>".as_slice(),
            b"<root/>&#x20;".as_slice(),
            b"<?xml version=\"1.0\" standalone=\"maybe\"?><root/>".as_slice(),
        ] {
            assert!(StrictXmlDocument::parse(xml).is_err(), "{xml:?}");
        }
    }

    #[test]
    fn strict_xml_validates_encoding_comments_and_processing_instructions() {
        for xml in [
            b"<?xml version=\"1.1\"?><root/>".as_slice(),
            b"<?xml version=\"1.0\" encoding=\"UTF-16\"?><root/>".as_slice(),
            b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><root/>".as_slice(),
            b"<root/><!--a--b-->".as_slice(),
            b"<?XML value?><root/>".as_slice(),
            b"<?1bad value?><root/>".as_slice(),
            b"<root/><!--\xff-->".as_slice(),
            b"<?target \xff?><root/>".as_slice(),
            b"<!DOCTYPE root \xff><root/>".as_slice(),
        ] {
            assert!(StrictXmlDocument::parse(xml).is_err(), "{xml:?}");
        }

        StrictXmlDocument::parse(
            b"<?xml version=\"1.0\" encoding=\"utf-8\"?><?xml-stylesheet href=\"x\"?><root/>",
        )
        .unwrap();
        StrictXmlDocument::parse(b"<?a:b value?><root/>").unwrap();
    }

    #[test]
    fn strict_xml_validates_qnames_without_lossy_decoding() {
        let implicit_xml = StrictXmlDocument::parse(br#"<xml:root/>"#).unwrap();
        assert!(
            implicit_xml
                .root()
                .is_named(Some("http://www.w3.org/XML/1998/namespace"), "root")
        );

        let unicode = StrictXmlDocument::parse("<élement/>".as_bytes()).unwrap();
        assert!(unicode.root().is_named(None, "élement"));

        for xml in [
            br#"<w:a:b xmlns:w="urn:word"/>"#.as_slice(),
            br#"<:bad/>"#.as_slice(),
            br#"<bad:/>"#.as_slice(),
            br#"<root xmlns:xml="urn:not-xml"/>"#.as_slice(),
            br#"<root xmlns:other="http://www.w3.org/XML/1998/namespace"/>"#.as_slice(),
            br#"<root xmlns:xmlns="urn:other"/>"#.as_slice(),
            br#"<root xmlns:other=""/>"#.as_slice(),
            b"<\xff/>".as_slice(),
        ] {
            assert!(StrictXmlDocument::parse(xml).is_err(), "{xml:?}");
        }
    }

    #[test]
    fn strict_xml_cursor_returns_every_unconsumed_semantic_item() {
        let document = StrictXmlDocument::parse(
            br#"<w:p xmlns:w="urn:word" xmlns:x="urn:foreign" w:known="yes" x:extra="kept">
                <w:r/><x:visible/><w:empty>   </w:empty>
            </w:p>"#,
        )
        .unwrap();
        let parsed = document
            .into_root()
            .parse(|cursor| {
                assert_eq!(
                    cursor.take_attribute(Some("urn:word"), "known"),
                    Some("yes".to_string())
                );
                let run = cursor.take_child(1).unwrap().into_element().unwrap();
                assert!(run.is_named(Some("urn:word"), "r"));
                Ok(())
            })
            .unwrap();

        let leftovers = parsed.leftovers;
        assert_eq!(leftovers.attributes.len(), 1);
        assert_eq!(leftovers.attributes[0].value(), "kept");
        assert_eq!(leftovers.children.len(), 2);
        assert!(leftovers.children.iter().all(StrictXmlNode::is_semantic));
    }

    #[test]
    fn strict_xml_retains_original_subtree_bytes_for_leftovers() {
        let xml = br#"<root xmlns:x="urn:foreign"><x:child value="one"></x:child></root>"#;
        let document = StrictXmlDocument::parse(xml).unwrap();
        let child = take_only_element_child(document);

        assert_eq!(
            child.into_raw_xml().bytes(),
            br#"<x:child value="one"></x:child>"#
        );
    }

    #[test]
    fn strict_xml_subtrees_share_one_source_allocation() {
        let document = StrictXmlDocument::parse(br#"<root><child><leaf/></child></root>"#).unwrap();
        let child = document.root.children[0].as_element().unwrap();
        let leaf = child.children[0].as_element().unwrap();

        assert!(document.root.raw_xml.shares_source_with(&child.raw_xml));
        assert!(document.root.raw_xml.shares_source_with(&leaf.raw_xml));
    }
}

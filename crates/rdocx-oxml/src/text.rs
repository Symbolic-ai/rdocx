//! Ordered, lossless WordprocessingML text content.
//!
//! Paragraph and run children are represented by one vector in XML order.
//! Unmodelled nodes carry the namespace context from their source location,
//! and composite modeled nodes keep their original XML until deliberately
//! mutated through their child APIs.

use std::io::Write;

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::drawing::CT_Drawing;
use crate::error::{OxmlError, Result};
#[cfg(test)]
use crate::namespace::W_NS;
use crate::namespace::{
    MC_NS, R_NS, is_word_attribute, matches_local_name, matches_namespace_element,
    matches_namespace_name, matches_word_attribute, matches_word_element, matches_word_name,
};
use crate::properties::{CT_PPr, CT_RPr};
use crate::raw_xml::{NamespaceContext, RawXml, capture_raw_element, capture_raw_empty_element};

const MAX_COMPOSITE_DEPTH: usize = 8;

/// `CT_Text` — the text content of a run.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Text {
    pub text: String,
    pub preserve_space: bool,
}

impl CT_Text {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            preserve_space: text.starts_with(' ') || text.ends_with(' '),
        }
    }
}

/// A parsed value that can replay its untouched source element.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedWithRaw<T> {
    value: T,
    raw_xml: Option<RawXml>,
}

impl<T> ParsedWithRaw<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            raw_xml: None,
        }
    }

    pub fn from_parsed(value: T, raw_xml: RawXml) -> Self {
        Self {
            value,
            raw_xml: Some(raw_xml),
        }
    }

    pub fn raw_xml(&self) -> Option<&RawXml> {
        self.raw_xml.as_ref()
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Types of simple fields.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Page,
    NumPages,
    Other(String),
}

/// Types of breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakType {
    Line,
    Page,
    Column,
}

/// Content that can appear inside a run, in XML order.
#[derive(Debug, Clone, PartialEq)]
pub enum RunContent {
    Text(CT_Text),
    Tab,
    Break(ParsedWithRaw<BreakType>),
    Drawing(ParsedWithRaw<CT_Drawing>),
    FootnoteRef(ParsedWithRaw<i32>),
    EndnoteRef(ParsedWithRaw<i32>),
    Unsupported(RawXml),
}

/// `CT_R` — a run of text with uniform formatting.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_R {
    pub properties: Option<CT_RPr>,
    pub content: Vec<RunContent>,
}

#[allow(non_snake_case)]
impl CT_R {
    pub fn new(text: &str) -> Self {
        Self {
            properties: None,
            content: vec![RunContent::Text(CT_Text::new(text))],
        }
    }

    pub fn text(&self) -> String {
        let mut result = String::new();
        for item in &self.content {
            match item {
                RunContent::Text(text) => result.push_str(&text.text),
                RunContent::Tab => result.push('\t'),
                RunContent::Break(_) => result.push('\n'),
                RunContent::Drawing(_)
                | RunContent::FootnoteRef(_)
                | RunContent::EndnoteRef(_)
                | RunContent::Unsupported(_) => {}
            }
        }
        result
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut properties = None;
        let mut content = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) => {
                    let name = element.name();
                    if matches_word_element(element, context, b"rPr") {
                        let property_context = context.with_element(element);
                        properties =
                            Some(CT_RPr::from_xml_with_context(reader, &property_context)?);
                    } else if matches_word_element(element, context, b"t") {
                        let preserve_space = element.attributes().flatten().any(|attribute| {
                            attribute.key.as_ref() == b"xml:space"
                                && attribute.value.as_ref() == b"preserve"
                        });
                        let text = crate::xml_text::decode_escaped(&reader.read_text(name)?)?;
                        content.push(RunContent::Text(CT_Text {
                            text,
                            preserve_space,
                        }));
                    } else if matches_word_element(element, context, b"tab") {
                        reader.read_to_end_into(name, &mut Vec::new())?;
                        content.push(RunContent::Tab);
                    } else if matches_word_element(element, context, b"br") {
                        let raw = capture_raw_element(reader, element, context)?;
                        parse_break(element, context, raw, &mut content)?;
                    } else if matches_word_element(element, context, b"footnoteReference") {
                        let raw = capture_raw_element(reader, element, context)?;
                        parse_note_reference_with_raw(element, context, true, raw, &mut content);
                    } else if matches_word_element(element, context, b"endnoteReference") {
                        let raw = capture_raw_element(reader, element, context)?;
                        parse_note_reference_with_raw(element, context, false, raw, &mut content);
                    } else if matches_word_element(element, context, b"drawing") {
                        let raw = capture_raw_element(reader, element, context)?;
                        if let Some(drawing) = parse_drawing(raw.bytes())? {
                            content.push(RunContent::Drawing(ParsedWithRaw::from_parsed(
                                drawing, raw,
                            )));
                        } else {
                            content.push(RunContent::Unsupported(raw));
                        }
                    } else if matches_mc_element(element, context, b"AlternateContent") {
                        let raw = capture_raw_element(reader, element, context)?;
                        if let Some(drawing) = crate::drawing::parse_alternate_content(raw.bytes())
                        {
                            content.push(RunContent::Drawing(ParsedWithRaw::from_parsed(
                                drawing, raw,
                            )));
                        } else {
                            content.push(RunContent::Unsupported(raw));
                        }
                    } else {
                        content.push(RunContent::Unsupported(capture_raw_element(
                            reader, element, context,
                        )?));
                    }
                }
                Ok(Event::Empty(ref element)) => {
                    if matches_word_element(element, context, b"tab") {
                        content.push(RunContent::Tab);
                    } else if matches_word_element(element, context, b"t") {
                        content.push(RunContent::Text(CT_Text {
                            text: String::new(),
                            preserve_space: element.attributes().flatten().any(|attribute| {
                                attribute.key.as_ref() == b"xml:space"
                                    && attribute.value.as_ref() == b"preserve"
                            }),
                        }));
                    } else if matches_word_element(element, context, b"br") {
                        let raw = capture_raw_empty_element(element, context)?;
                        parse_break(element, context, raw, &mut content)?;
                    } else if matches_word_element(element, context, b"footnoteReference") {
                        parse_note_reference(element, context, true, &mut content)?;
                    } else if matches_word_element(element, context, b"endnoteReference") {
                        parse_note_reference(element, context, false, &mut content)?;
                    } else if matches_word_element(element, context, b"rPr") {
                        properties = Some(CT_RPr::default());
                    } else {
                        content.push(RunContent::Unsupported(capture_raw_empty_element(
                            element, context,
                        )?));
                    }
                }
                Ok(Event::End(ref element))
                    if matches_word_name(element.name().as_ref(), context, b"r") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:r".to_string()));
                }
                Err(error) => return Err(error.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(Self {
            properties,
            content,
        })
    }

    pub fn to_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.to_xml_with_context(writer, &NamespaceContext::default())
    }

    pub(crate) fn to_xml_with_context<W: Write>(
        &self,
        writer: &mut Writer<W>,
        context: &NamespaceContext,
    ) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:r")))?;
        if let Some(properties) = &self.properties {
            properties.to_xml(writer)?;
        }
        for item in &self.content {
            match item {
                RunContent::Text(text) => {
                    let mut element = BytesStart::new("w:t");
                    if text.preserve_space {
                        element.push_attribute(("xml:space", "preserve"));
                    }
                    writer.write_event(Event::Start(element))?;
                    writer.write_event(Event::Text(BytesText::new(&text.text)))?;
                    writer.write_event(Event::End(BytesEnd::new("w:t")))?;
                }
                RunContent::Tab => writer.write_event(Event::Empty(BytesStart::new("w:tab")))?,
                RunContent::Break(parsed) => {
                    if let Some(raw) = parsed.raw_xml() {
                        raw.write_to_with_context(writer.get_mut(), context)?;
                    } else {
                        let mut element = BytesStart::new("w:br");
                        match *parsed.value() {
                            BreakType::Page => element.push_attribute(("w:type", "page")),
                            BreakType::Column => element.push_attribute(("w:type", "column")),
                            BreakType::Line => {}
                        }
                        writer.write_event(Event::Empty(element))?;
                    }
                }
                RunContent::Drawing(parsed) => {
                    if let Some(raw) = parsed.raw_xml() {
                        raw.write_to_with_context(writer.get_mut(), context)?;
                    } else {
                        parsed.value().to_xml(writer)?;
                    }
                }
                RunContent::FootnoteRef(parsed) => {
                    write_parsed_note_reference(writer, "w:footnoteReference", parsed, context)?
                }
                RunContent::EndnoteRef(parsed) => {
                    write_parsed_note_reference(writer, "w:endnoteReference", parsed, context)?
                }
                RunContent::Unsupported(raw) => {
                    raw.write_to_with_context(writer.get_mut(), context)?
                }
            }
        }
        writer.write_event(Event::End(BytesEnd::new("w:r")))?;
        Ok(())
    }
}

/// Ordered content within a hyperlink or simple field.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineChild {
    Run(CT_R),
    SimpleField(CT_SimpleField),
    Unsupported(RawXml),
}

impl InlineChild {
    fn text(&self) -> String {
        match self {
            Self::Run(run) => run.text(),
            Self::SimpleField(field) => field.text(),
            Self::Unsupported(_) => String::new(),
        }
    }

    fn to_xml<W: Write>(&self, writer: &mut Writer<W>, context: &NamespaceContext) -> Result<()> {
        match self {
            Self::Run(run) => run.to_xml_with_context(writer, context),
            Self::SimpleField(field) => field.to_xml(writer, context),
            Self::Unsupported(raw) => Ok(raw.write_to_with_context(writer.get_mut(), context)?),
        }
    }
}

/// A paragraph-level simple field. Parsed fields replay their original XML
/// until their cached children are deliberately requested mutably.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_SimpleField {
    field_type: FieldType,
    instruction: String,
    children: Vec<InlineChild>,
    extra_attributes: Vec<(String, String)>,
    source_namespaces: NamespaceContext,
    raw_xml: Option<RawXml>,
}

impl CT_SimpleField {
    pub fn new(field_type: FieldType) -> Self {
        let instruction = match &field_type {
            FieldType::Page => " PAGE ".to_string(),
            FieldType::NumPages => " NUMPAGES ".to_string(),
            FieldType::Other(instruction) => instruction.clone(),
        };
        Self {
            field_type,
            instruction,
            children: vec![InlineChild::Run(CT_R::new("1"))],
            extra_attributes: Vec::new(),
            source_namespaces: NamespaceContext::default(),
            raw_xml: None,
        }
    }

    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    pub fn instruction(&self) -> &str {
        &self.instruction
    }

    pub fn children(&self) -> &[InlineChild] {
        &self.children
    }

    pub fn has_cached_content(&self) -> bool {
        !self.children.is_empty()
    }

    /// Whether Word marked the cached result as stale.
    pub fn dirty(&self) -> Option<bool> {
        extra_word_attribute(&self.extra_attributes, &self.source_namespaces, b"dirty")
            .and_then(parse_on_off_attribute)
    }

    /// Whether this field has a WordprocessingML attribute whose semantics
    /// are not represented by this type.
    pub fn has_unmodeled_semantic_attributes(&self) -> bool {
        self.extra_attributes.iter().any(|(name, value)| {
            let name = name.as_bytes();
            if !is_word_attribute(name, &self.source_namespaces) {
                return false;
            }
            !matches_word_attribute(name, &self.source_namespaces, b"dirty")
                || parse_on_off_attribute(value).is_none()
        })
    }

    pub fn children_mut(&mut self) -> &mut Vec<InlineChild> {
        self.raw_xml = None;
        &mut self.children
    }

    pub fn set_instruction(&mut self, instruction: impl Into<String>) {
        self.instruction = instruction.into();
        self.field_type = parse_field_instruction(&self.instruction);
        self.raw_xml = None;
    }

    pub fn text(&self) -> String {
        self.children.iter().map(InlineChild::text).collect()
    }

    pub fn run_count(&self) -> usize {
        count_inline_runs(&self.children)
    }

    fn from_raw(raw_xml: RawXml) -> Result<Self> {
        Self::from_raw_at_depth(raw_xml, 1)
    }

    fn from_raw_at_depth(raw_xml: RawXml, depth: usize) -> Result<Self> {
        let parsed = parse_composite_children(&raw_xml, b"fldSimple", depth)?;
        Ok(Self {
            field_type: parse_field_instruction(&parsed.instruction),
            instruction: parsed.instruction,
            children: parsed.children,
            extra_attributes: parsed.extra_attributes,
            source_namespaces: raw_xml.namespaces().clone(),
            raw_xml: Some(raw_xml),
        })
    }

    fn to_xml<W: Write>(&self, writer: &mut Writer<W>, context: &NamespaceContext) -> Result<()> {
        if let Some(raw) = &self.raw_xml {
            raw.write_to_with_context(writer.get_mut(), context)?;
            return Ok(());
        }
        let mut output = Vec::new();
        let mut generated = Writer::new(&mut output);
        let mut element = BytesStart::new("w:fldSimple");
        element.push_attribute(("w:instr", self.instruction.as_str()));
        for (name, value) in &self.extra_attributes {
            element.push_attribute((name.as_str(), value.as_str()));
        }
        generated.write_event(Event::Start(element))?;
        for child in &self.children {
            child.to_xml(&mut generated, &self.source_namespaces)?;
        }
        generated.write_event(Event::End(BytesEnd::new("w:fldSimple")))?;
        RawXml::from_bytes(output, b"w:fldSimple", self.source_namespaces.clone())
            .write_to_with_context(writer.get_mut(), context)?;
        Ok(())
    }
}

/// A paragraph-level hyperlink with ordered children.
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Hyperlink {
    rel_id: Option<String>,
    anchor: Option<String>,
    children: Vec<InlineChild>,
    extra_attributes: Vec<(String, String)>,
    source_namespaces: NamespaceContext,
    raw_xml: Option<RawXml>,
}

impl CT_Hyperlink {
    pub fn new(rel_id: Option<String>, anchor: Option<String>) -> Self {
        Self {
            rel_id,
            anchor,
            children: Vec::new(),
            extra_attributes: Vec::new(),
            source_namespaces: NamespaceContext::default(),
            raw_xml: None,
        }
    }

    pub fn rel_id(&self) -> Option<&str> {
        self.rel_id.as_deref()
    }

    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// User-facing hover text stored on the hyperlink.
    pub fn tooltip(&self) -> Option<&str> {
        extra_word_attribute(&self.extra_attributes, &self.source_namespaces, b"tooltip")
    }

    /// Location within the hyperlink target.
    pub fn doc_location(&self) -> Option<&str> {
        extra_word_attribute(
            &self.extra_attributes,
            &self.source_namespaces,
            b"docLocation",
        )
    }

    /// Whether this hyperlink has a WordprocessingML attribute whose
    /// semantics are not represented by this type.
    pub fn has_unmodeled_semantic_attributes(&self) -> bool {
        self.extra_attributes.iter().any(|(name, value)| {
            let name = name.as_bytes();
            if !is_word_attribute(name, &self.source_namespaces) {
                return false;
            }
            if matches_word_attribute(name, &self.source_namespaces, b"tooltip")
                || matches_word_attribute(name, &self.source_namespaces, b"docLocation")
            {
                return false;
            }
            !matches_word_attribute(name, &self.source_namespaces, b"history")
                || parse_on_off_attribute(value).is_none()
        })
    }

    pub fn children(&self) -> &[InlineChild] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<InlineChild> {
        self.raw_xml = None;
        &mut self.children
    }

    pub fn add_run(&mut self, text: &str) -> &mut CT_R {
        self.raw_xml = None;
        self.children.push(InlineChild::Run(CT_R::new(text)));
        match self.children.last_mut() {
            Some(InlineChild::Run(run)) => run,
            _ => unreachable!(),
        }
    }

    pub fn set_rel_id(&mut self, rel_id: Option<String>) {
        self.rel_id = rel_id;
        self.raw_xml = None;
    }

    pub fn set_anchor(&mut self, anchor: Option<String>) {
        self.anchor = anchor;
        self.raw_xml = None;
    }

    pub fn text(&self) -> String {
        self.children.iter().map(InlineChild::text).collect()
    }

    pub fn run_count(&self) -> usize {
        count_inline_runs(&self.children)
    }

    fn from_raw(raw_xml: RawXml) -> Result<Self> {
        let context = raw_xml.namespaces().clone();
        let mut reader = Reader::from_reader(raw_xml.bytes());
        reader.config_mut().trim_text(true);
        let mut rel_id = None;
        let mut anchor = None;
        let mut extra_attributes = Vec::new();
        let mut children = Vec::new();
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element))
                    if matches_word_element(element, &context, b"hyperlink") =>
                {
                    let element_context = context.with_element(element);
                    parse_hyperlink_attributes(
                        element,
                        &element_context,
                        &mut rel_id,
                        &mut anchor,
                        &mut extra_attributes,
                    );
                    children =
                        parse_inline_children(&mut reader, &element_context, b"hyperlink", 0)?;
                    break;
                }
                Ok(Event::Empty(ref element))
                    if matches_word_element(element, &context, b"hyperlink") =>
                {
                    let element_context = context.with_element(element);
                    parse_hyperlink_attributes(
                        element,
                        &element_context,
                        &mut rel_id,
                        &mut anchor,
                        &mut extra_attributes,
                    );
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("w:hyperlink".to_string()));
                }
                Err(error) => return Err(error.into()),
                _ => {}
            }
            buf.clear();
        }
        drop(reader);
        Ok(Self {
            rel_id,
            anchor,
            children,
            extra_attributes,
            source_namespaces: raw_xml.namespaces().clone(),
            raw_xml: Some(raw_xml),
        })
    }

    fn to_xml<W: Write>(&self, writer: &mut Writer<W>, context: &NamespaceContext) -> Result<()> {
        if let Some(raw) = &self.raw_xml {
            raw.write_to_with_context(writer.get_mut(), context)?;
            return Ok(());
        }
        let mut output = Vec::new();
        let mut generated = Writer::new(&mut output);
        let mut element = BytesStart::new("w:hyperlink");
        if let Some(rel_id) = &self.rel_id {
            element.push_attribute(("r:id", rel_id.as_str()));
        }
        if let Some(anchor) = &self.anchor {
            element.push_attribute(("w:anchor", anchor.as_str()));
        }
        for (name, value) in &self.extra_attributes {
            element.push_attribute((name.as_str(), value.as_str()));
        }
        generated.write_event(Event::Start(element))?;
        for child in &self.children {
            child.to_xml(&mut generated, &self.source_namespaces)?;
        }
        generated.write_event(Event::End(BytesEnd::new("w:hyperlink")))?;
        RawXml::from_bytes(output, b"w:hyperlink", self.source_namespaces.clone())
            .write_to_with_context(writer.get_mut(), context)?;
        Ok(())
    }
}

fn parse_hyperlink_attributes(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    rel_id: &mut Option<String>,
    anchor: &mut Option<String>,
    extra_attributes: &mut Vec<(String, String)>,
) {
    for attribute in element.attributes().flatten() {
        if matches_namespace_name(attribute.key.as_ref(), context, b"r", R_NS, b"id") {
            *rel_id = Some(String::from_utf8_lossy(attribute.value.as_ref()).into_owned());
        } else if matches_word_attribute(attribute.key.as_ref(), context, b"anchor") {
            *anchor = Some(String::from_utf8_lossy(attribute.value.as_ref()).into_owned());
        } else if attribute.key.as_ref() != b"xmlns"
            && !attribute.key.as_ref().starts_with(b"xmlns:")
        {
            extra_attributes.push((
                String::from_utf8_lossy(attribute.key.as_ref()).into_owned(),
                String::from_utf8_lossy(attribute.value.as_ref()).into_owned(),
            ));
        }
    }
}

fn extra_word_attribute<'a>(
    attributes: &'a [(String, String)],
    context: &NamespaceContext,
    expected_local: &[u8],
) -> Option<&'a str> {
    attributes.iter().find_map(|(name, value)| {
        matches_word_attribute(name.as_bytes(), context, expected_local).then_some(value.as_str())
    })
}

fn parse_on_off_attribute(value: &str) -> Option<bool> {
    match value {
        "true" | "1" | "on" => Some(true),
        "false" | "0" | "off" => Some(false),
        _ => None,
    }
}

/// Content that can appear directly inside a paragraph, in XML order.
#[derive(Debug, Clone, PartialEq)]
pub enum ParagraphChild {
    Run(CT_R),
    Hyperlink(CT_Hyperlink),
    SimpleField(CT_SimpleField),
    Unsupported(RawXml),
}

/// A compatibility projection over the ordered model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperlinkSpan {
    pub rel_id: Option<String>,
    pub anchor: Option<String>,
    pub run_start: usize,
    pub run_end: usize,
}

/// `CT_P` — a paragraph with one ordered child vector.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_P {
    pub properties: Option<CT_PPr>,
    pub content: Vec<ParagraphChild>,
}

#[allow(non_snake_case)]
impl CT_P {
    pub fn new() -> Self {
        Self {
            properties: None,
            content: Vec::new(),
        }
    }

    pub fn text(&self) -> String {
        self.content
            .iter()
            .map(|child| match child {
                ParagraphChild::Run(run) => run.text(),
                ParagraphChild::Hyperlink(hyperlink) => hyperlink.text(),
                ParagraphChild::SimpleField(field) => field.text(),
                ParagraphChild::Unsupported(_) => String::new(),
            })
            .collect()
    }

    pub fn add_run(&mut self, text: &str) -> &mut CT_R {
        self.content.push(ParagraphChild::Run(CT_R::new(text)));
        match self.content.last_mut() {
            Some(ParagraphChild::Run(run)) => run,
            _ => unreachable!(),
        }
    }

    pub fn add_hyperlink(&mut self, hyperlink: CT_Hyperlink) -> &mut CT_Hyperlink {
        self.content.push(ParagraphChild::Hyperlink(hyperlink));
        match self.content.last_mut() {
            Some(ParagraphChild::Hyperlink(hyperlink)) => hyperlink,
            _ => unreachable!(),
        }
    }

    pub fn add_simple_field(&mut self, field_type: FieldType) -> &mut CT_SimpleField {
        self.content
            .push(ParagraphChild::SimpleField(CT_SimpleField::new(field_type)));
        match self.content.last_mut() {
            Some(ParagraphChild::SimpleField(field)) => field,
            _ => unreachable!(),
        }
    }

    pub fn runs(&self) -> Vec<&CT_R> {
        let mut runs = Vec::new();
        for child in &self.content {
            collect_runs(child, &mut runs);
        }
        runs
    }

    pub fn run_count(&self) -> usize {
        self.content.iter().map(count_paragraph_runs).sum()
    }

    pub fn run(&self, mut index: usize) -> Option<&CT_R> {
        for child in &self.content {
            let count = count_paragraph_runs(child);
            if index < count {
                return paragraph_run(child, index);
            }
            index -= count;
        }
        None
    }

    pub fn run_mut(&mut self, mut index: usize) -> Option<&mut CT_R> {
        for child in &mut self.content {
            let count = count_paragraph_runs(child);
            if index < count {
                return paragraph_run_mut(child, index);
            }
            index -= count;
        }
        None
    }

    pub fn hyperlink_spans(&self) -> Vec<HyperlinkSpan> {
        let mut spans = Vec::new();
        let mut run_index = 0;
        for child in &self.content {
            match child {
                ParagraphChild::Run(_) => run_index += 1,
                ParagraphChild::Hyperlink(hyperlink) => {
                    let run_start = run_index;
                    run_index += count_inline_runs(hyperlink.children());
                    spans.push(HyperlinkSpan {
                        rel_id: hyperlink.rel_id.clone(),
                        anchor: hyperlink.anchor.clone(),
                        run_start,
                        run_end: run_index,
                    });
                }
                ParagraphChild::SimpleField(field) => {
                    run_index += count_inline_runs(field.children());
                }
                ParagraphChild::Unsupported(_) => {}
            }
        }
        spans
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_context(reader, &NamespaceContext::default())
    }

    pub fn from_xml_with_context(
        reader: &mut Reader<&[u8]>,
        context: &NamespaceContext,
    ) -> Result<Self> {
        let mut properties = None;
        let mut content = Vec::new();
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) => {
                    if matches_word_element(element, context, b"pPr") {
                        let property_context = context.with_element(element);
                        properties =
                            Some(CT_PPr::from_xml_with_context(reader, &property_context)?);
                    } else if matches_word_element(element, context, b"r") {
                        let child_context = context.with_element(element);
                        content.push(ParagraphChild::Run(CT_R::from_xml_with_context(
                            reader,
                            &child_context,
                        )?));
                    } else if matches_word_element(element, context, b"hyperlink") {
                        let raw = capture_raw_element(reader, element, context)?;
                        content.push(ParagraphChild::Hyperlink(CT_Hyperlink::from_raw(raw)?));
                    } else if matches_word_element(element, context, b"fldSimple") {
                        let raw = capture_raw_element(reader, element, context)?;
                        content.push(ParagraphChild::SimpleField(CT_SimpleField::from_raw(raw)?));
                    } else {
                        content.push(ParagraphChild::Unsupported(capture_raw_element(
                            reader, element, context,
                        )?));
                    }
                }
                Ok(Event::Empty(ref element)) => {
                    if matches_word_element(element, context, b"pPr") {
                        properties = Some(CT_PPr::default());
                    } else if matches_word_element(element, context, b"r") {
                        content.push(ParagraphChild::Run(CT_R {
                            properties: None,
                            content: Vec::new(),
                        }));
                    } else if matches_word_element(element, context, b"hyperlink") {
                        let raw = capture_raw_empty_element(element, context)?;
                        content.push(ParagraphChild::Hyperlink(CT_Hyperlink::from_raw(raw)?));
                    } else if matches_word_element(element, context, b"fldSimple") {
                        let raw = capture_raw_empty_element(element, context)?;
                        content.push(ParagraphChild::SimpleField(CT_SimpleField::from_raw(raw)?));
                    } else {
                        content.push(ParagraphChild::Unsupported(capture_raw_empty_element(
                            element, context,
                        )?));
                    }
                }
                Ok(Event::End(ref element))
                    if matches_word_name(element.name().as_ref(), context, b"p") =>
                {
                    break;
                }
                Ok(Event::Eof) => {
                    return Err(OxmlError::MissingElement("closing w:p".to_string()));
                }
                Err(error) => return Err(error.into()),
                _ => {}
            }
            buf.clear();
        }
        Ok(Self {
            properties,
            content,
        })
    }

    pub fn to_xml<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.to_xml_with_context(writer, &NamespaceContext::default())
    }

    pub(crate) fn to_xml_with_context<W: Write>(
        &self,
        writer: &mut Writer<W>,
        context: &NamespaceContext,
    ) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:p")))?;
        if let Some(properties) = &self.properties {
            properties.to_xml(writer)?;
        }
        for child in &self.content {
            match child {
                ParagraphChild::Run(run) => run.to_xml_with_context(writer, context)?,
                ParagraphChild::Hyperlink(hyperlink) => hyperlink.to_xml(writer, context)?,
                ParagraphChild::SimpleField(field) => field.to_xml(writer, context)?,
                ParagraphChild::Unsupported(raw) => {
                    raw.write_to_with_context(writer.get_mut(), context)?
                }
            }
        }
        writer.write_event(Event::End(BytesEnd::new("w:p")))?;
        Ok(())
    }
}

impl Default for CT_P {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_inline_children(
    reader: &mut Reader<&[u8]>,
    context: &NamespaceContext,
    end_name: &[u8],
    composite_depth: usize,
) -> Result<Vec<InlineChild>> {
    let mut children = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref element)) if matches_word_element(element, context, b"r") => {
                let child_context = context.with_element(element);
                children.push(InlineChild::Run(CT_R::from_xml_with_context(
                    reader,
                    &child_context,
                )?));
            }
            Ok(Event::Start(ref element))
                if matches_word_element(element, context, b"fldSimple") =>
            {
                if composite_depth >= MAX_COMPOSITE_DEPTH {
                    return Err(OxmlError::InvalidValue(format!(
                        "composite field nesting exceeds {MAX_COMPOSITE_DEPTH} levels"
                    )));
                }
                let raw = capture_raw_element(reader, element, context)?;
                children.push(InlineChild::SimpleField(CT_SimpleField::from_raw_at_depth(
                    raw,
                    composite_depth + 1,
                )?));
            }
            Ok(Event::Start(ref element)) => {
                children.push(InlineChild::Unsupported(capture_raw_element(
                    reader, element, context,
                )?));
            }
            Ok(Event::Empty(ref element)) => {
                if matches_word_element(element, context, b"r") {
                    children.push(InlineChild::Run(CT_R {
                        properties: None,
                        content: Vec::new(),
                    }));
                } else if matches_word_element(element, context, b"fldSimple") {
                    if composite_depth >= MAX_COMPOSITE_DEPTH {
                        return Err(OxmlError::InvalidValue(format!(
                            "composite field nesting exceeds {MAX_COMPOSITE_DEPTH} levels"
                        )));
                    }
                    let raw = capture_raw_empty_element(element, context)?;
                    children.push(InlineChild::SimpleField(CT_SimpleField::from_raw_at_depth(
                        raw,
                        composite_depth + 1,
                    )?));
                } else {
                    children.push(InlineChild::Unsupported(capture_raw_empty_element(
                        element, context,
                    )?));
                }
            }
            Ok(Event::End(ref element))
                if matches_word_name(element.name().as_ref(), context, end_name) =>
            {
                break;
            }
            Ok(Event::Eof) => {
                return Err(OxmlError::MissingElement(format!(
                    "closing w:{}",
                    String::from_utf8_lossy(end_name)
                )));
            }
            Err(error) => return Err(error.into()),
            _ => {}
        }
        buf.clear();
    }
    Ok(children)
}

struct ParsedCompositeChildren {
    instruction: String,
    children: Vec<InlineChild>,
    extra_attributes: Vec<(String, String)>,
}

fn parse_composite_children(
    raw: &RawXml,
    end_name: &[u8],
    composite_depth: usize,
) -> Result<ParsedCompositeChildren> {
    let context = raw.namespaces().clone();
    let mut reader = Reader::from_reader(raw.bytes());
    reader.config_mut().trim_text(true);
    let mut instruction = String::new();
    let mut children = Vec::new();
    let mut extra_attributes = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref element)) if matches_word_element(element, &context, end_name) => {
                let element_context = context.with_element(element);
                parse_field_attributes(
                    element,
                    &element_context,
                    &mut instruction,
                    &mut extra_attributes,
                );
                children = parse_inline_children(
                    &mut reader,
                    &element_context,
                    end_name,
                    composite_depth,
                )?;
                break;
            }
            Ok(Event::Empty(ref element)) if matches_word_element(element, &context, end_name) => {
                let element_context = context.with_element(element);
                parse_field_attributes(
                    element,
                    &element_context,
                    &mut instruction,
                    &mut extra_attributes,
                );
                break;
            }
            Ok(Event::Eof) => {
                return Err(OxmlError::MissingElement(format!(
                    "w:{}",
                    String::from_utf8_lossy(end_name)
                )));
            }
            Err(error) => return Err(error.into()),
            _ => {}
        }
        buf.clear();
    }
    Ok(ParsedCompositeChildren {
        instruction,
        children,
        extra_attributes,
    })
}

fn parse_field_attributes(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    instruction: &mut String,
    extra_attributes: &mut Vec<(String, String)>,
) {
    for attribute in element.attributes().flatten() {
        if matches_word_attribute(attribute.key.as_ref(), context, b"instr") {
            *instruction = String::from_utf8_lossy(attribute.value.as_ref()).into_owned();
        } else if attribute.key.as_ref() != b"xmlns"
            && !attribute.key.as_ref().starts_with(b"xmlns:")
        {
            extra_attributes.push((
                String::from_utf8_lossy(attribute.key.as_ref()).into_owned(),
                String::from_utf8_lossy(attribute.value.as_ref()).into_owned(),
            ));
        }
    }
}

fn parse_drawing(raw: &[u8]) -> Result<Option<CT_Drawing>> {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref element))
                if matches_local_name(element.name().as_ref(), b"drawing") =>
            {
                return Ok(Some(CT_Drawing::from_xml(&mut reader)?));
            }
            Ok(Event::Eof) => return Ok(None),
            Err(error) => return Err(error.into()),
            _ => {}
        }
        buf.clear();
    }
}

fn parse_note_reference(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    footnote: bool,
    content: &mut Vec<RunContent>,
) -> Result<()> {
    let raw = capture_raw_empty_element(element, context)?;
    parse_note_reference_with_raw(element, context, footnote, raw, content);
    Ok(())
}

fn parse_note_reference_with_raw(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    footnote: bool,
    raw: RawXml,
    content: &mut Vec<RunContent>,
) {
    let element_context = context.with_element(element);
    let id = element
        .attributes()
        .flatten()
        .find(|attribute| matches_word_attribute(attribute.key.as_ref(), &element_context, b"id"))
        .and_then(|attribute| {
            std::str::from_utf8(attribute.value.as_ref())
                .ok()?
                .parse::<i32>()
                .ok()
        });
    match (footnote, id) {
        (true, Some(id)) => {
            content.push(RunContent::FootnoteRef(ParsedWithRaw::from_parsed(id, raw)))
        }
        (false, Some(id)) => {
            content.push(RunContent::EndnoteRef(ParsedWithRaw::from_parsed(id, raw)))
        }
        (_, None) => content.push(RunContent::Unsupported(raw)),
    }
}

fn parse_break(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    raw: RawXml,
    content: &mut Vec<RunContent>,
) -> Result<()> {
    let element_context = context.with_element(element);
    let break_type = element
        .attributes()
        .flatten()
        .find(|attribute| matches_word_attribute(attribute.key.as_ref(), &element_context, b"type"))
        .map(|attribute| String::from_utf8_lossy(attribute.value.as_ref()).into_owned());
    match break_type.as_deref() {
        None | Some("textWrapping") => content.push(RunContent::Break(ParsedWithRaw::from_parsed(
            BreakType::Line,
            raw,
        ))),
        Some("page") => content.push(RunContent::Break(ParsedWithRaw::from_parsed(
            BreakType::Page,
            raw,
        ))),
        Some("column") => content.push(RunContent::Break(ParsedWithRaw::from_parsed(
            BreakType::Column,
            raw,
        ))),
        Some(_) => content.push(RunContent::Unsupported(raw)),
    }
    Ok(())
}

fn write_note_reference<W: Write>(writer: &mut Writer<W>, tag: &str, id: i32) -> Result<()> {
    let mut buffer = itoa::Buffer::new();
    let mut element = BytesStart::new(tag);
    element.push_attribute(("w:id", buffer.format(id)));
    writer.write_event(Event::Empty(element))?;
    Ok(())
}

fn write_parsed_note_reference<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    parsed: &ParsedWithRaw<i32>,
    context: &NamespaceContext,
) -> Result<()> {
    if let Some(raw) = parsed.raw_xml() {
        raw.write_to_with_context(writer.get_mut(), context)?;
        Ok(())
    } else {
        write_note_reference(writer, tag, *parsed.value())
    }
}

fn collect_runs<'a>(child: &'a ParagraphChild, runs: &mut Vec<&'a CT_R>) {
    match child {
        ParagraphChild::Run(run) => runs.push(run),
        ParagraphChild::Hyperlink(hyperlink) => collect_inline_runs(hyperlink.children(), runs),
        ParagraphChild::SimpleField(field) => collect_inline_runs(field.children(), runs),
        ParagraphChild::Unsupported(_) => {}
    }
}

fn collect_inline_runs<'a>(children: &'a [InlineChild], runs: &mut Vec<&'a CT_R>) {
    for child in children {
        match child {
            InlineChild::Run(run) => runs.push(run),
            InlineChild::SimpleField(field) => collect_inline_runs(field.children(), runs),
            InlineChild::Unsupported(_) => {}
        }
    }
}

fn count_inline_runs(children: &[InlineChild]) -> usize {
    children
        .iter()
        .map(|child| match child {
            InlineChild::Run(_) => 1,
            InlineChild::SimpleField(field) => count_inline_runs(field.children()),
            InlineChild::Unsupported(_) => 0,
        })
        .sum()
}

fn count_paragraph_runs(child: &ParagraphChild) -> usize {
    match child {
        ParagraphChild::Run(_) => 1,
        ParagraphChild::Hyperlink(hyperlink) => count_inline_runs(hyperlink.children()),
        ParagraphChild::SimpleField(field) => count_inline_runs(field.children()),
        ParagraphChild::Unsupported(_) => 0,
    }
}

fn paragraph_run(child: &ParagraphChild, index: usize) -> Option<&CT_R> {
    match child {
        ParagraphChild::Run(run) => (index == 0).then_some(run),
        ParagraphChild::Hyperlink(hyperlink) => inline_run(hyperlink.children(), index),
        ParagraphChild::SimpleField(field) => inline_run(field.children(), index),
        ParagraphChild::Unsupported(_) => None,
    }
}

fn inline_run(children: &[InlineChild], mut index: usize) -> Option<&CT_R> {
    for child in children {
        match child {
            InlineChild::Run(run) if index == 0 => return Some(run),
            InlineChild::Run(_) => index -= 1,
            InlineChild::SimpleField(field) => {
                let count = count_inline_runs(field.children());
                if index < count {
                    return inline_run(field.children(), index);
                }
                index -= count;
            }
            InlineChild::Unsupported(_) => {}
        }
    }
    None
}

fn paragraph_run_mut(child: &mut ParagraphChild, index: usize) -> Option<&mut CT_R> {
    match child {
        ParagraphChild::Run(run) => (index == 0).then_some(run),
        ParagraphChild::Hyperlink(hyperlink) => inline_run_mut(hyperlink.children_mut(), index),
        ParagraphChild::SimpleField(field) => inline_run_mut(field.children_mut(), index),
        ParagraphChild::Unsupported(_) => None,
    }
}

fn inline_run_mut(children: &mut [InlineChild], mut index: usize) -> Option<&mut CT_R> {
    for child in children {
        match child {
            InlineChild::Run(run) if index == 0 => return Some(run),
            InlineChild::Run(_) => index -= 1,
            InlineChild::SimpleField(field) => {
                let count = count_inline_runs(field.children());
                if index < count {
                    return inline_run_mut(field.children_mut(), index);
                }
                index -= count;
            }
            InlineChild::Unsupported(_) => {}
        }
    }
    None
}

fn matches_mc_element(
    element: &BytesStart<'_>,
    context: &NamespaceContext,
    expected: &[u8],
) -> bool {
    matches_namespace_element(element, context, b"mc", MC_NS, expected)
}

fn parse_field_instruction(instruction: &str) -> FieldType {
    match instruction
        .trim()
        .to_uppercase()
        .split_whitespace()
        .next()
        .unwrap_or("")
    {
        "PAGE" => FieldType::Page,
        "NUMPAGES" => FieldType::NumPages,
        _ => FieldType::Other(instruction.trim().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_paragraph(xml: &str) -> CT_P {
        let full =
            format!(r#"<w:p xmlns:w="{W_NS}" xmlns:r="{R_NS}" xmlns:mc="{MC_NS}">{xml}</w:p>"#);
        let mut reader = Reader::from_str(&full);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element))
                    if matches_local_name(element.name().as_ref(), b"p") =>
                {
                    let context = NamespaceContext::default().with_element(element);
                    return CT_P::from_xml_with_context(&mut reader, &context).unwrap();
                }
                Ok(Event::Eof) => panic!("paragraph not found"),
                _ => {}
            }
            buf.clear();
        }
    }

    fn serialize(paragraph: &CT_P) -> String {
        let mut output = Vec::new();
        paragraph.to_xml(&mut Writer::new(&mut output)).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn paragraph_children_remain_in_source_order() {
        let paragraph = parse_paragraph(concat!(
            r#"<w:r><w:t>before</w:t></w:r>"#,
            r#"<w:bookmarkStart w:id="1" w:name="mark"/>"#,
            r#"<w:hyperlink r:id="rId1"><w:r><w:t>link</w:t></w:r></w:hyperlink>"#,
            r#"<w:fldSimple w:instr=" PAGE "><w:r><w:t>4</w:t></w:r></w:fldSimple>"#,
            r#"<w:r><w:t>after</w:t></w:r>"#,
        ));
        assert!(matches!(paragraph.content[0], ParagraphChild::Run(_)));
        assert!(matches!(
            paragraph.content[1],
            ParagraphChild::Unsupported(_)
        ));
        assert!(matches!(paragraph.content[2], ParagraphChild::Hyperlink(_)));
        assert!(matches!(
            paragraph.content[3],
            ParagraphChild::SimpleField(_)
        ));
        assert!(matches!(paragraph.content[4], ParagraphChild::Run(_)));
    }

    #[test]
    fn unrelated_mutation_preserves_field_and_hyperlink_subtrees() {
        let mut paragraph = parse_paragraph(concat!(
            r#"<w:hyperlink r:id="rId5" w:history="1"><w:r><w:t>link</w:t></w:r></w:hyperlink>"#,
            r#"<w:fldSimple w:instr=" PAGE \\* MERGEFORMAT " w:dirty="true"><w:r><w:rPr><w:b/></w:rPr><w:t>12</w:t></w:r><x:cache xmlns:x="urn:producer">kept</x:cache></w:fldSimple>"#,
        ));
        paragraph.add_run("later");
        let xml = serialize(&paragraph);
        assert!(xml.contains(r#"w:history="1""#), "{xml}");
        assert!(xml.contains(r#"w:dirty="true""#), "{xml}");
        assert!(xml.contains(r#"<x:cache xmlns:x="urn:producer">kept</x:cache>"#));
    }

    #[test]
    fn composite_attributes_expose_import_semantics() {
        let paragraph = parse_paragraph(concat!(
            r#"<w:hyperlink r:id="rId5" w:tooltip="Open" w:docLocation="section" w:history="1" x:meta="kept" xmlns:x="urn:producer"><w:r><w:t>link</w:t></w:r></w:hyperlink>"#,
            r#"<w:hyperlink r:id="rId6" w:tgtFrame="_blank"><w:r><w:t>target</w:t></w:r></w:hyperlink>"#,
            r#"<w:fldSimple w:instr=" PAGE " w:dirty="on"><w:r><w:t>12</w:t></w:r></w:fldSimple>"#,
            r#"<w:fldSimple w:instr=" PAGE " w:fldLock="1"><w:r><w:t>13</w:t></w:r></w:fldSimple>"#,
        ));

        let ParagraphChild::Hyperlink(link) = &paragraph.content[0] else {
            panic!("expected hyperlink")
        };
        assert_eq!(link.tooltip(), Some("Open"));
        assert_eq!(link.doc_location(), Some("section"));
        assert!(!link.has_unmodeled_semantic_attributes());

        let ParagraphChild::Hyperlink(link) = &paragraph.content[1] else {
            panic!("expected hyperlink")
        };
        assert!(link.has_unmodeled_semantic_attributes());

        let ParagraphChild::SimpleField(field) = &paragraph.content[2] else {
            panic!("expected simple field")
        };
        assert_eq!(field.dirty(), Some(true));
        assert!(!field.has_unmodeled_semantic_attributes());

        let ParagraphChild::SimpleField(field) = &paragraph.content[3] else {
            panic!("expected simple field")
        };
        assert_eq!(field.dirty(), None);
        assert!(field.has_unmodeled_semantic_attributes());
    }

    #[test]
    fn mutating_hyperlink_children_preserves_unmodelled_outer_attributes() {
        let mut paragraph = parse_paragraph(concat!(
            r#"<w:hyperlink r:id="rId5" w:history="1"><w:r><w:t>link</w:t></w:r></w:hyperlink>"#,
            r#"<w:fldSimple w:instr=" PAGE " w:dirty="true"><w:r><w:t>12</w:t></w:r></w:fldSimple>"#,
        ));
        let ParagraphChild::Hyperlink(hyperlink) = &mut paragraph.content[0] else {
            panic!("expected hyperlink")
        };
        hyperlink
            .children_mut()
            .push(InlineChild::Run(CT_R::new("!")));
        let xml = serialize(&paragraph);
        assert!(xml.contains(r#"w:history="1""#), "{xml}");
        assert!(xml.contains(r#"w:dirty="true""#), "{xml}");
        assert!(xml.contains("<w:t>!</w:t>"), "{xml}");
    }

    #[test]
    fn mutating_field_children_preserves_its_and_neighbor_attributes() {
        let mut paragraph = parse_paragraph(concat!(
            r#"<w:hyperlink r:id="rId5" w:history="1"><w:r><w:t>link</w:t></w:r></w:hyperlink>"#,
            r#"<w:fldSimple w:instr=" PAGE " w:dirty="true"><w:r><w:t>12</w:t></w:r></w:fldSimple>"#,
        ));
        let ParagraphChild::SimpleField(field) = &mut paragraph.content[1] else {
            panic!("expected field")
        };
        field.children_mut().push(InlineChild::Run(CT_R::new("!")));
        let xml = serialize(&paragraph);
        assert!(xml.contains(r#"w:history="1""#), "{xml}");
        assert!(xml.contains(r#"w:dirty="true""#), "{xml}");
        assert!(xml.contains("<w:t>!</w:t>"), "{xml}");
    }

    #[test]
    fn run_mutation_does_not_rebase_unsupported_paragraph_children() {
        let mut paragraph = parse_paragraph(concat!(
            r#"<w:r><w:t>before</w:t></w:r>"#,
            r#"<w:bookmarkStart w:id="1" w:name="mark"/>"#,
            r#"<w:r><w:t>after</w:t></w:r>"#,
        ));
        let run = paragraph.run_mut(0).unwrap();
        let RunContent::Text(text) = &mut run.content[0] else {
            panic!("expected text")
        };
        text.text = "changed".to_string();
        let xml = serialize(&paragraph);
        let changed = xml.find("<w:t>changed</w:t>").unwrap();
        let bookmark = xml.find("<w:bookmarkStart").unwrap();
        let after = xml.find("<w:t>after</w:t>").unwrap();
        assert!(changed < bookmark && bookmark < after, "{xml}");
    }

    #[test]
    fn parsed_run_controls_are_immutable_and_keep_their_raw_elements() {
        let paragraph = parse_paragraph(concat!(
            r#"<w:r><w:br w:type="page" w:clear="all"/>"#,
            r#"<w:br w:type="column" w:clear="left"/></w:r>"#,
        ));
        let run = paragraph.run(0).unwrap();
        let RunContent::Break(first) = &run.content[0] else {
            panic!("expected first break")
        };
        assert_eq!(first.value(), &BreakType::Page);
        let xml = serialize(&paragraph);
        assert!(xml.contains(r#"w:clear="all""#), "{xml}");
        assert!(xml.contains(r#"w:clear="left""#), "{xml}");
    }

    #[test]
    fn expanded_run_controls_parse_like_empty_elements() {
        let paragraph = parse_paragraph(concat!(
            r#"<w:r><w:tab></w:tab><w:br w:type="page"></w:br>"#,
            r#"<w:footnoteReference w:id="7"></w:footnoteReference>"#,
            r#"<w:endnoteReference w:id="9"></w:endnoteReference></w:r>"#,
        ));
        let run = paragraph.run(0).unwrap();
        assert!(matches!(run.content[0], RunContent::Tab));
        assert!(matches!(
            run.content[1],
            RunContent::Break(ref parsed) if parsed.value() == &BreakType::Page
        ));
        assert!(matches!(
            run.content[2],
            RunContent::FootnoteRef(ref parsed) if parsed.value() == &7
        ));
        assert!(matches!(
            run.content[3],
            RunContent::EndnoteRef(ref parsed) if parsed.value() == &9
        ));
    }

    #[test]
    fn formatting_properties_require_the_word_namespace() {
        let inner = format!(
            concat!(
                r#"<w:r><w:rPr><x:vanish xmlns:x="urn:foreign"/></w:rPr><w:t>a</w:t></w:r>"#,
                r#"<w:r><w:rPr><z:vanish xmlns:z="{}"/></w:rPr><w:t>b</w:t></w:r>"#,
            ),
            W_NS
        );
        let paragraph = parse_paragraph(&inner);
        assert_eq!(
            paragraph
                .run(0)
                .unwrap()
                .properties
                .as_ref()
                .unwrap()
                .vanish,
            None
        );
        assert_eq!(
            paragraph
                .run(1)
                .unwrap()
                .properties
                .as_ref()
                .unwrap()
                .vanish,
            Some(true)
        );
    }

    #[test]
    fn parsed_note_ids_are_immutable_and_keep_their_raw_elements() {
        let paragraph = parse_paragraph(concat!(
            r#"<w:r><w:footnoteReference w:id="7" w:custom="first"/>"#,
            r#"<w:endnoteReference w:id="9" w:custom="second"/></w:r>"#,
        ));
        let run = paragraph.run(0).unwrap();
        let RunContent::FootnoteRef(note) = &run.content[0] else {
            panic!("expected footnote reference")
        };
        assert_eq!(note.value(), &7);

        let xml = serialize(&paragraph);
        assert!(xml.contains(r#"w:custom="first""#), "{xml}");
        assert!(xml.contains(r#"w:custom="second""#), "{xml}");
    }

    #[test]
    fn run_mutation_inside_composites_preserves_outer_and_opaque_data() {
        let mut paragraph = parse_paragraph(concat!(
            r#"<w:hyperlink r:id="rId5" w:history="1" w:tooltip="tip"><w:r><w:t>link</w:t></w:r><x:opaque xmlns:x="urn:producer"/></w:hyperlink>"#,
            r#"<w:fldSimple w:instr=" PAGE " w:dirty="true" w:fldLock="true"><w:r><w:t>12</w:t></w:r><x:cache xmlns:x="urn:producer">kept</x:cache></w:fldSimple>"#,
        ));
        for index in 0..2 {
            let run = paragraph.run_mut(index).unwrap();
            let RunContent::Text(text) = &mut run.content[0] else {
                panic!("expected text")
            };
            text.text.push('!');
        }

        let xml = serialize(&paragraph);
        for attribute in ["w:history", "w:tooltip", "w:dirty", "w:fldLock"] {
            assert!(xml.contains(attribute), "{attribute} missing from {xml}");
        }
        assert!(xml.contains("<x:opaque"), "{xml}");
        assert!(xml.contains("<x:cache"), "{xml}");
        assert!(xml.contains("<w:t>link!</w:t>"), "{xml}");
        assert!(xml.contains("<w:t>12!</w:t>"), "{xml}");
    }

    #[test]
    fn inherited_custom_word_prefix_is_resolved() {
        let xml = format!(
            r#"<z:p xmlns:z="{W_NS}" xmlns:f="urn:foreign"><z:r><z:t>text</z:t><f:item/></z:r></z:p>"#
        );
        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) if element.name().as_ref() == b"z:p" => {
                    let context = NamespaceContext::default().with_element(element);
                    let paragraph = CT_P::from_xml_with_context(&mut reader, &context).unwrap();
                    assert_eq!(paragraph.text(), "text");
                    let run = paragraph.runs()[0];
                    let RunContent::Unsupported(raw) = &run.content[1] else {
                        panic!("foreign element should be unsupported")
                    };
                    assert_eq!(raw.name().namespace_uri.as_deref(), Some("urn:foreign"));
                    assert_eq!(raw.namespaces().namespace_uri("z"), Some(W_NS));
                    assert!(serialize(&paragraph).contains(r#"<f:item xmlns:f="urn:foreign"/>"#));
                    return;
                }
                _ => {}
            }
            buf.clear();
        }
    }

    #[test]
    fn alternate_content_is_one_ordered_drawing_node() {
        let paragraph = parse_paragraph(concat!(
            r#"<w:r><mc:AlternateContent><mc:Choice Requires="wps">"#,
            r#"<w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" behindDoc="0"><wp:extent cx="1" cy="1"/></wp:anchor></w:drawing>"#,
            r#"</mc:Choice><mc:Fallback><w:pict/></mc:Fallback></mc:AlternateContent></w:r>"#,
        ));
        let run = paragraph.runs()[0];
        assert!(matches!(run.content.as_slice(), [RunContent::Drawing(_)]));
        let xml = serialize(&paragraph);
        assert_eq!(xml.matches("<mc:AlternateContent").count(), 1);
        assert!(xml.contains("<mc:Fallback>"));
    }

    #[test]
    fn malformed_note_reference_remains_ordered_raw_xml() {
        let paragraph =
            parse_paragraph(r#"<w:r><w:t>a</w:t><w:footnoteReference/><w:t>b</w:t></w:r>"#);
        let run = paragraph.runs()[0];
        assert!(matches!(run.content[1], RunContent::Unsupported(_)));
        let xml = serialize(&paragraph);
        assert!(xml.contains("<w:footnoteReference"));
        assert!(xml.contains(&format!(r#"xmlns:w="{W_NS}""#)));
    }

    #[test]
    fn paragraph_and_run_dispatch_respect_namespaces_and_empty_elements() {
        let paragraph = parse_paragraph(concat!(
            r#"<x:r xmlns:x="urn:foreign"><x:t>foreign</x:t></x:r>"#,
            r#"<w:r/><w:hyperlink r:id="rId1"/><w:fldSimple w:instr=" PAGE "/>"#,
            r#"<w:r><w:t/><w:br w:type="vendorSpecific"/></w:r>"#,
        ));

        assert!(matches!(
            paragraph.content[0],
            ParagraphChild::Unsupported(_)
        ));
        assert!(matches!(paragraph.content[1], ParagraphChild::Run(_)));
        assert!(matches!(paragraph.content[2], ParagraphChild::Hyperlink(_)));
        assert!(matches!(
            paragraph.content[3],
            ParagraphChild::SimpleField(_)
        ));
        let ParagraphChild::Run(run) = &paragraph.content[4] else {
            panic!("expected run")
        };
        assert!(matches!(run.content[0], RunContent::Text(_)));
        assert!(matches!(run.content[1], RunContent::Unsupported(_)));
    }

    #[test]
    fn simple_fields_report_cached_children() {
        let paragraph = parse_paragraph(concat!(
            r#"<w:fldSimple w:instr=" PAGE "/>"#,
            r#"<w:fldSimple w:instr=" PAGE "><w:r><w:t>7</w:t></w:r></w:fldSimple>"#,
        ));
        let ParagraphChild::SimpleField(empty) = &paragraph.content[0] else {
            panic!("expected empty field")
        };
        let ParagraphChild::SimpleField(cached) = &paragraph.content[1] else {
            panic!("expected cached field")
        };
        assert!(!empty.has_cached_content());
        assert!(cached.has_cached_content());
    }

    #[test]
    fn excessive_simple_field_nesting_is_rejected() {
        let xml = format!(
            "{}<w:r><w:t>value</w:t></w:r>{}",
            r#"<w:fldSimple w:instr=" PAGE ">"#.repeat(MAX_COMPOSITE_DEPTH + 1),
            "</w:fldSimple>".repeat(MAX_COMPOSITE_DEPTH + 1)
        );
        let full = format!(r#"<w:p xmlns:w="{W_NS}">{xml}</w:p>"#);
        let mut reader = Reader::from_str(&full);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element))
                    if matches_local_name(element.name().as_ref(), b"p") =>
                {
                    let context = NamespaceContext::default().with_element(element);
                    assert!(matches!(
                        CT_P::from_xml_with_context(&mut reader, &context),
                        Err(OxmlError::InvalidValue(_))
                    ));
                    return;
                }
                Ok(Event::Eof) => panic!("paragraph not found"),
                _ => {}
            }
            buf.clear();
        }
    }
}

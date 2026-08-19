//! Text content elements: `CT_P` (paragraph), `CT_R` (run), `CT_Text`.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::content_control::CT_Sdt;
use crate::drawing::CT_Drawing;
use crate::error::{OxmlError, Result};
use crate::namespace::{R_NS, matches_local_name};
use crate::numbering::{parse_scoped_ppr, word_prefixes_at};
use crate::properties::{CT_PPr, CT_RPr, is_word_element};
use crate::raw_xml::{capture_element, capture_empty_element};
use crate::revision::CT_Revision;

/// `CT_Text` — The text content of a run, with optional xml:space="preserve".
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Text {
    pub text: String,
    pub preserve_space: bool,
}

impl CT_Text {
    pub fn new(text: &str) -> Self {
        CT_Text {
            text: text.to_string(),
            preserve_space: text.starts_with(' ') || text.ends_with(' '),
        }
    }
}

/// Types of simple fields.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    /// Current page number (PAGE field).
    Page,
    /// Total number of pages (NUMPAGES field).
    NumPages,
    /// Text of the named bookmark (REF field).
    Ref {
        bookmark: String,
        instruction: String,
    },
    /// Page containing the named bookmark (PAGEREF field).
    PageRef {
        bookmark: String,
        instruction: String,
    },
    /// Any other field instruction.
    Other(String),
}

/// One parsed field switch, without the leading backslash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSwitch {
    pub name: String,
    pub argument: Option<String>,
}

/// A tokenized Word field instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInstruction {
    pub original: String,
    pub name: String,
    pub arguments: Vec<String>,
    pub switches: Vec<FieldSwitch>,
}

/// A complete complex field projected from `w:fldChar` and `w:instrText` runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexField {
    pub instruction: FieldInstruction,
    pub run_start: usize,
    pub run_end: usize,
    pub cached_text: String,
    pub dirty: bool,
    pub children: Vec<ComplexField>,
}

/// A namespace-resolved complex-field marker retained alongside raw run XML.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldMarker {
    Begin { dirty: bool },
    Instruction(String),
    Separate { dirty: bool },
    End { dirty: bool },
}

/// Content that can appear inside a run.
#[derive(Debug, Clone, PartialEq)]
pub enum RunContent {
    Text(CT_Text),
    /// Deleted text projected from `w:delText` inside a deletion wrapper.
    DeletedText(CT_Text),
    Tab,
    Break(BreakType),
    Drawing(CT_Drawing),
    /// A simple field (from `<w:fldSimple>`).
    Field {
        field_type: FieldType,
        /// Stored display text used when a target cannot be resolved.
        display: String,
    },
    /// A footnote reference (`<w:footnoteReference w:id="..."/>`).
    FootnoteRef {
        id: i32,
    },
    /// An endnote reference (`<w:endnoteReference w:id="..."/>`).
    EndnoteRef {
        id: i32,
    },
    /// A comment reference (`<w:commentReference w:id="..."/>`).
    CommentReference {
        id: i32,
        /// Number of raw run children that precede this reference.
        raw_before: usize,
    },
}

/// Semantic attributes preserved for a `w:fldSimple` result run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleFieldMetadata {
    pub dirty: Option<bool>,
    pub has_unmodeled_attributes: bool,
}

/// A typed comment range boundary at a run insertion point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentRangeMarker {
    Start {
        id: i32,
        run_index: usize,
        /// Number of raw children at this run boundary that precede the marker.
        raw_before: usize,
    },
    End {
        id: i32,
        run_index: usize,
        /// Number of raw children at this run boundary that precede the marker.
        raw_before: usize,
    },
}

/// Read projection of a bookmark marker retained at a run boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookmarkMarker {
    start: bool,
    id: Option<i32>,
    name: Option<String>,
    run_index: usize,
    raw_before: usize,
}

impl BookmarkMarker {
    fn new(
        start: bool,
        id: i32,
        name: Option<String>,
        run_index: usize,
        raw_before: usize,
    ) -> Self {
        Self {
            start,
            id: Some(id),
            name,
            run_index,
            raw_before,
        }
    }

    pub fn is_start(&self) -> bool {
        self.start
    }

    pub fn id(&self) -> Option<i32> {
        self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn run_index(&self) -> usize {
        self.run_index
    }

    /// Number of preserved raw children before this marker at its run boundary.
    #[doc(hidden)]
    pub fn raw_before(&self) -> usize {
        self.raw_before
    }
}

impl CommentRangeMarker {
    fn run_index(&self) -> usize {
        match self {
            Self::Start { run_index, .. } | Self::End { run_index, .. } => *run_index,
        }
    }

    fn raw_before(&self) -> usize {
        match self {
            Self::Start { raw_before, .. } | Self::End { raw_before, .. } => *raw_before,
        }
    }
}

/// Types of breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakType {
    Line,
    Page,
    Column,
}

/// `CT_R` — A run of text with uniform formatting.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_R {
    pub properties: Option<CT_RPr>,
    pub content: Vec<RunContent>,
    /// Attributes from the simple field represented by this synthetic result run.
    pub simple_field: Option<SimpleFieldMetadata>,
    /// Unknown child elements captured as raw XML.
    pub extra_xml: Vec<Vec<u8>>,
    /// Parsed raw-child positions among properties and typed run content.
    #[doc(hidden)]
    pub extra_xml_positions: Vec<usize>,
    /// Namespace-resolved field markers retained alongside `extra_xml`.
    #[doc(hidden)]
    pub field_markers: Vec<FieldMarker>,
    /// Drawings read out of an `mc:AlternateContent` block, for layout only.
    ///
    /// Never serialised. The verbatim copy in `extra_xml` is what gets
    /// written, so emitting these as well would duplicate the element.
    pub alt_drawings: Vec<CT_Drawing>,
}

#[allow(non_snake_case)]
impl CT_R {
    pub fn new(text: &str) -> Self {
        CT_R {
            properties: None,
            content: vec![RunContent::Text(CT_Text::new(text))],
            simple_field: None,
            extra_xml: Vec::new(),
            extra_xml_positions: Vec::new(),
            field_markers: Vec::new(),
            alt_drawings: Vec::new(),
        }
    }

    /// Get the combined text of all text content in this run.
    pub fn text(&self) -> String {
        let mut result = String::new();
        for item in &self.content {
            match item {
                RunContent::Text(t) | RunContent::DeletedText(t) => result.push_str(&t.text),
                RunContent::Tab => result.push('\t'),
                RunContent::Break(_) => result.push('\n'),
                RunContent::Drawing(_) => {} // Drawings have no text content
                RunContent::Field { .. } => {} // Fields have no static text
                RunContent::FootnoteRef { .. }
                | RunContent::EndnoteRef { .. }
                | RunContent::CommentReference { .. } => {}
            }
        }
        result
    }

    /// Replace typed run content while retaining every raw child boundary.
    #[doc(hidden)]
    pub fn replace_content(&mut self, content: Vec<RunContent>) {
        let property_boundary = usize::from(self.properties.is_some());
        let replacement_end = property_boundary + content.len();
        for position in &mut self.extra_xml_positions {
            if *position > property_boundary {
                *position = replacement_end;
            }
        }
        self.content = content;
    }

    /// Append typed content without moving raw children after existing content.
    #[doc(hidden)]
    pub fn append_content(&mut self, content: RunContent) {
        self.content.push(content);
    }

    /// Materialize run properties in their required first-child position.
    #[doc(hidden)]
    pub fn ensure_properties(&mut self) -> &mut CT_RPr {
        if self.properties.is_none() {
            for position in &mut self.extra_xml_positions {
                *position += 1;
            }
            self.properties = Some(CT_RPr::default());
        }
        self.properties.as_mut().expect("properties were inserted")
    }

    /// Remap raw boundaries after selected typed content children are removed.
    #[doc(hidden)]
    pub fn remap_removed_content(&mut self, removed: &[bool]) {
        let property_boundary = usize::from(self.properties.is_some());
        for position in &mut self.extra_xml_positions {
            let content_boundary = position.saturating_sub(property_boundary);
            *position = position.saturating_sub(
                removed
                    .iter()
                    .take(content_boundary.min(removed.len()))
                    .filter(|remove| **remove)
                    .count(),
            );
        }
    }

    /// Remove selected comment-reference content and retain surrounding raw XML.
    #[doc(hidden)]
    pub fn remove_comment_references(&mut self, ids: &[i32]) -> bool {
        let removed = self
            .content
            .iter()
            .map(|content| {
                matches!(content, RunContent::CommentReference { id, .. } if ids.contains(id))
            })
            .collect::<Vec<_>>();
        let removed_any = removed.iter().any(|remove| *remove);
        if removed_any {
            self.remap_removed_content(&removed);
            self.content = self
                .content
                .drain(..)
                .zip(removed)
                .filter_map(|(content, remove)| (!remove).then_some(content))
                .collect();
        }
        removed_any
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_prefixes(
            reader,
            &["w".to_owned(), format!("\0mc\0{}", crate::namespace::MC_NS)],
        )
    }

    pub(crate) fn from_xml_with_prefixes(
        reader: &mut Reader<&[u8]>,
        word_prefixes: &[String],
    ) -> Result<Self> {
        let mut properties = None;
        let mut content = Vec::new();
        let mut extra_xml = Vec::new();
        let mut extra_xml_positions = Vec::new();
        let mut field_markers = Vec::new();
        let mut alt_drawings = Vec::new();
        let mut modeled_children = 0usize;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if is_word_element(name.as_ref(), b"rPr", &prefixes) {
                        let raw = capture_element(reader, e)?;
                        properties = Some(crate::numbering::parse_scoped_rpr(&raw, word_prefixes)?);
                        modeled_children += 1;
                    } else if is_word_element(name.as_ref(), b"t", &prefixes) {
                        let preserve = e.attributes().any(|a| {
                            a.ok()
                                .map(|a| {
                                    a.key.as_ref() == b"xml:space"
                                        && a.value.as_ref() == b"preserve"
                                })
                                .unwrap_or(false)
                        });
                        // `read_text` returns the raw markup span, so entity
                        // references in it still need resolving.
                        let encoded = reader.read_text(name)?;
                        encoded
                            .decode()
                            .map_err(|error| crate::OxmlError::InvalidValue(error.to_string()))?;
                        let text = crate::xml_text::decode_escaped(&encoded);
                        content.push(RunContent::Text(CT_Text {
                            text,
                            preserve_space: preserve,
                        }));
                        modeled_children += 1;
                    } else if is_word_element(name.as_ref(), b"delText", &prefixes) {
                        let preserve = e.attributes().any(|a| {
                            a.ok().is_some_and(|a| {
                                a.key.as_ref() == b"xml:space" && a.value.as_ref() == b"preserve"
                            })
                        });
                        let encoded = reader.read_text(name)?;
                        encoded
                            .decode()
                            .map_err(|error| crate::OxmlError::InvalidValue(error.to_string()))?;
                        let text = crate::xml_text::decode_escaped(&encoded);
                        content.push(RunContent::DeletedText(CT_Text {
                            text,
                            preserve_space: preserve,
                        }));
                        modeled_children += 1;
                    } else if is_word_element(name.as_ref(), b"drawing", &prefixes) {
                        content.push(RunContent::Drawing(CT_Drawing::from_xml(reader)?));
                        modeled_children += 1;
                    } else if is_word_element(name.as_ref(), b"commentReference", &prefixes) {
                        let id = required_word_i32_attribute(e, b"id", &prefixes)?;
                        reader.read_to_end_into(name, &mut Vec::new())?;
                        content.push(RunContent::CommentReference {
                            id,
                            raw_before: extra_xml.len(),
                        });
                        modeled_children += 1;
                    } else if is_element_in_namespace(
                        name.as_ref(),
                        b"AlternateContent",
                        crate::namespace::MC_NS,
                        &prefixes,
                    ) {
                        // Keep the block verbatim so the VML fallback survives
                        // a write, and separately read the DrawingML out of it
                        // so layout can see the shape. alt_drawings is never
                        // serialised, the raw copy below is what gets written.
                        let raw = capture_element(reader, e)?;
                        if let Some(drawing) = crate::drawing::parse_alternate_content(&raw) {
                            alt_drawings.push(drawing);
                        }
                        extra_xml.push(raw);
                        extra_xml_positions.push(modeled_children);
                    } else {
                        // Capture unknown child elements as raw XML
                        let marker = field_marker_from_element(e, &prefixes)?;
                        let raw = capture_element(reader, e)?;
                        if let Some(marker) = marker {
                            field_markers.push(field_marker_with_instruction(marker, &raw)?);
                        }
                        extra_xml.push(raw);
                        extra_xml_positions.push(modeled_children);
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if is_word_element(name.as_ref(), b"tab", &prefixes) {
                        content.push(RunContent::Tab);
                        modeled_children += 1;
                    } else if is_word_element(name.as_ref(), b"br", &prefixes) {
                        let break_type = optional_word_attribute(e, b"type", &prefixes)
                            .map(|value| match value.as_bytes() {
                                b"page" => BreakType::Page,
                                b"column" => BreakType::Column,
                                _ => BreakType::Line,
                            })
                            .unwrap_or(BreakType::Line);
                        content.push(RunContent::Break(break_type));
                        modeled_children += 1;
                    } else if is_word_element(name.as_ref(), b"footnoteReference", &prefixes) {
                        let id = optional_word_attribute(e, b"id", &prefixes)
                            .and_then(|value| value.parse::<i32>().ok())
                            .unwrap_or(0);
                        content.push(RunContent::FootnoteRef { id });
                        modeled_children += 1;
                    } else if is_word_element(name.as_ref(), b"endnoteReference", &prefixes) {
                        let id = optional_word_attribute(e, b"id", &prefixes)
                            .and_then(|value| value.parse::<i32>().ok())
                            .unwrap_or(0);
                        content.push(RunContent::EndnoteRef { id });
                        modeled_children += 1;
                    } else if is_word_element(name.as_ref(), b"commentReference", &prefixes) {
                        let id = required_word_i32_attribute(e, b"id", &prefixes)?;
                        content.push(RunContent::CommentReference {
                            id,
                            raw_before: extra_xml.len(),
                        });
                        modeled_children += 1;
                    } else if !is_word_element(name.as_ref(), b"rPr", &prefixes) {
                        // Capture unknown empty child elements (e.g.
                        // w:commentReference) as raw XML, mirroring the
                        // Event::Start fallback above.
                        //
                        // A self-closing <w:rPr/> is deliberately skipped.
                        // extra_xml is re-emitted after the run content, but
                        // CT_R requires w:rPr to be the first child, so
                        // capturing it here would move it past <w:t> and
                        // produce schema-invalid output. An empty rPr carries
                        // no formatting, so dropping it loses nothing.
                        if let Some(marker) = field_marker_from_element(e, &prefixes)? {
                            field_markers.push(marker);
                        }
                        extra_xml.push(capture_empty_element(e)?);
                        extra_xml_positions.push(modeled_children);
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"r") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_R {
            properties,
            content,
            simple_field: None,
            extra_xml,
            extra_xml_positions,
            field_markers,
            alt_drawings,
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.to_xml_with_word_override(writer, None)
    }

    pub(crate) fn to_xml_with_word_override<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        foreign_word_namespace: Option<&str>,
    ) -> Result<()> {
        let mut run = BytesStart::new("w:r");
        if foreign_word_namespace.is_some() {
            run.push_attribute(("xmlns:w", crate::namespace::W_NS));
        }
        writer.write_event(Event::Start(run))?;

        let ordered_raw = self.extra_xml_positions.len() == self.extra_xml.len();
        let mut typed_boundary = 0usize;
        if ordered_raw {
            write_run_raw_boundary(writer, self, typed_boundary, foreign_word_namespace)?;
        }
        if let Some(ref props) = self.properties {
            props.to_xml_with_word_override(writer, foreign_word_namespace)?;
            typed_boundary += 1;
            if ordered_raw {
                write_run_raw_boundary(writer, self, typed_boundary, foreign_word_namespace)?;
            }
        }

        let mut raw_written = 0;
        for item in &self.content {
            match item {
                RunContent::Text(t) => {
                    let mut e = BytesStart::new("w:t");
                    if t.preserve_space {
                        e.push_attribute(("xml:space", "preserve"));
                    }
                    writer.write_event(Event::Start(e))?;
                    writer.write_event(Event::Text(BytesText::new(&t.text)))?;
                    writer.write_event(Event::End(BytesEnd::new("w:t")))?;
                }
                RunContent::DeletedText(t) => {
                    let mut e = BytesStart::new("w:delText");
                    if t.preserve_space {
                        e.push_attribute(("xml:space", "preserve"));
                    }
                    writer.write_event(Event::Start(e))?;
                    writer.write_event(Event::Text(BytesText::new(&t.text)))?;
                    writer.write_event(Event::End(BytesEnd::new("w:delText")))?;
                }
                RunContent::Tab => {
                    writer.write_event(Event::Empty(BytesStart::new("w:tab")))?;
                }
                RunContent::Break(bt) => {
                    let mut e = BytesStart::new("w:br");
                    match bt {
                        BreakType::Page => e.push_attribute(("w:type", "page")),
                        BreakType::Column => e.push_attribute(("w:type", "column")),
                        BreakType::Line => {}
                    }
                    writer.write_event(Event::Empty(e))?;
                }
                RunContent::Drawing(d) => {
                    d.to_xml(writer)?;
                }
                RunContent::Field { .. } => {
                    // Field runs are serialized at the paragraph level as <w:fldSimple>
                }
                RunContent::FootnoteRef { id } => {
                    let mut buf = itoa::Buffer::new();
                    let mut e = BytesStart::new("w:footnoteReference");
                    e.push_attribute(("w:id", buf.format(*id)));
                    writer.write_event(Event::Empty(e))?;
                }
                RunContent::EndnoteRef { id } => {
                    let mut buf = itoa::Buffer::new();
                    let mut e = BytesStart::new("w:endnoteReference");
                    e.push_attribute(("w:id", buf.format(*id)));
                    writer.write_event(Event::Empty(e))?;
                }
                RunContent::CommentReference { id, raw_before } => {
                    if !ordered_raw {
                        for raw in self
                            .extra_xml
                            .iter()
                            .take((*raw_before).min(self.extra_xml.len()))
                            .skip(raw_written)
                        {
                            write_raw_with_word_override(writer, raw, foreign_word_namespace)?;
                            raw_written += 1;
                        }
                    }
                    let mut buf = itoa::Buffer::new();
                    let mut e = BytesStart::new("w:commentReference");
                    e.push_attribute(("w:id", buf.format(*id)));
                    writer.write_event(Event::Empty(e))?;
                }
            }
            typed_boundary += 1;
            if ordered_raw {
                write_run_raw_boundary(writer, self, typed_boundary, foreign_word_namespace)?;
            }
        }

        // Write captured unknown child elements
        if !ordered_raw {
            for raw in self.extra_xml.iter().skip(raw_written) {
                write_raw_with_word_override(writer, raw, foreign_word_namespace)?;
            }
        }

        writer.write_event(Event::End(BytesEnd::new("w:r")))?;
        Ok(())
    }
}

fn write_run_raw_boundary<W: std::io::Write>(
    writer: &mut Writer<W>,
    run: &CT_R,
    boundary: usize,
    foreign_word_namespace: Option<&str>,
) -> Result<()> {
    for (position, raw) in run.extra_xml_positions.iter().zip(&run.extra_xml) {
        if *position == boundary {
            write_raw_with_word_override(writer, raw, foreign_word_namespace)?;
        }
    }
    Ok(())
}

/// A hyperlink span that wraps a range of runs.
#[derive(Debug, Clone, PartialEq)]
pub struct HyperlinkSpan {
    /// The relationship ID for the hyperlink target.
    pub rel_id: Option<String>,
    /// Optional anchor within the document (for internal links).
    pub anchor: Option<String>,
    /// Optional user-facing hover text.
    pub tooltip: Option<String>,
    /// Optional location in the hyperlink target document.
    pub doc_location: Option<String>,
    /// Index of the first run in the hyperlink (inclusive).
    pub run_start: usize,
    /// Index of the last run in the hyperlink (exclusive).
    pub run_end: usize,
    /// Unmodeled owner attributes retained when the hyperlink came from XML.
    #[doc(hidden)]
    pub extra_attributes: Vec<(String, String)>,
    /// Raw children at `(relative run boundary, typed revisions before, XML)`.
    #[doc(hidden)]
    pub extra_xml: Vec<(usize, usize, Vec<u8>)>,
    /// Parent raw slot for a revision-only hyperlink preserved as one subtree.
    #[doc(hidden)]
    pub preserved_raw_before: Option<usize>,
}

const HYPERLINK_REVISION_FLAG: usize = 1usize << (usize::BITS - 1);

struct ParsedHyperlinkChildren {
    runs: Vec<CT_R>,
    revisions: Vec<(usize, CT_Revision)>,
    extra_xml: Vec<(usize, usize, Vec<u8>)>,
}

/// A HYPERLINK represented by a valid complex field sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexFieldHyperlink {
    pub run_start: usize,
    pub run_end: usize,
    pub target: String,
}

#[derive(Debug)]
struct OpenComplexField {
    instruction: String,
    result_start: Option<usize>,
    dirty: bool,
    valid: bool,
    children: Vec<ComplexField>,
}

type ParsedHyperlinkAttributes = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Vec<(String, String)>,
);

pub(crate) fn hyperlink_revision_slot(hyperlink_index: usize) -> usize {
    HYPERLINK_REVISION_FLAG | hyperlink_index
}

#[doc(hidden)]
pub fn hyperlink_revision_index(slot: usize) -> Option<usize> {
    (slot & HYPERLINK_REVISION_FLAG != 0).then_some(slot & !HYPERLINK_REVISION_FLAG)
}

/// `CT_P` — A paragraph element containing runs and properties.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct CT_P {
    pub properties: Option<CT_PPr>,
    pub runs: Vec<CT_R>,
    /// Hyperlink spans referencing ranges of runs.
    pub hyperlinks: Vec<HyperlinkSpan>,
    /// Typed comment range boundaries at run insertion points.
    pub comment_ranges: Vec<CommentRangeMarker>,
    /// Typed projections of preserved bookmark markers.
    pub bookmark_markers: Vec<BookmarkMarker>,
    /// Unknown child elements captured as raw XML with their insertion position (run index).
    pub extra_xml: Vec<(usize, Vec<u8>)>,
    /// Typed run controls at
    /// `(run index, raw children before, comment markers before, control)`.
    pub content_controls: Vec<(usize, usize, usize, CT_Sdt)>,
    /// Read projections of revision wrappers retained at paragraph or hyperlink boundaries.
    pub revisions: Vec<(usize, usize, CT_Revision)>,
}

#[allow(non_snake_case)]
impl CT_P {
    pub fn new() -> Self {
        CT_P {
            properties: None,
            runs: Vec::new(),
            hyperlinks: Vec::new(),
            comment_ranges: Vec::new(),
            bookmark_markers: Vec::new(),
            extra_xml: Vec::new(),
            content_controls: Vec::new(),
            revisions: Vec::new(),
        }
    }

    /// Get the combined text of all runs in this paragraph.
    pub fn text(&self) -> String {
        self.runs().iter().map(|run| run.text()).collect()
    }

    /// Return recursively parsed complex fields with complete cached results.
    pub fn complex_fields(&self) -> Vec<ComplexField> {
        parse_complex_fields(&self.runs)
    }

    /// Return the cached-result spans of valid complex `HYPERLINK` fields.
    ///
    /// Complex fields stay in their original run form for round-trip writing.
    /// This projection supplies their external targets to reader consumers.
    pub fn complex_field_hyperlinks(&self) -> Vec<ComplexFieldHyperlink> {
        self.complex_fields()
            .into_iter()
            .filter(|field| {
                !field.dirty
                    && !field.cached_text.is_empty()
                    && field.instruction.name == "HYPERLINK"
            })
            .filter_map(|field| {
                field
                    .instruction
                    .arguments
                    .first()
                    .cloned()
                    .map(|target| ComplexFieldHyperlink {
                        run_start: field.run_start,
                        run_end: field.run_end,
                        target,
                    })
            })
            .collect()
    }

    /// Return direct and content-control-wrapped runs in document order.
    pub fn runs(&self) -> Vec<&CT_R> {
        let mut runs = Vec::new();
        self.collect_runs(&mut runs);
        runs
    }

    /// Add a run with the given text.
    pub fn add_run(&mut self, text: &str) -> &mut CT_R {
        self.runs.push(CT_R::new(text));
        self.runs.last_mut().unwrap()
    }

    /// Insert a direct run while keeping every paragraph boundary projection aligned.
    #[doc(hidden)]
    pub fn insert_unwrapped_run(&mut self, run_index: usize, run: CT_R) -> bool {
        if run_index > self.runs.len() {
            return false;
        }
        for marker in &mut self.comment_ranges {
            match marker {
                CommentRangeMarker::Start { run_index: at, .. }
                | CommentRangeMarker::End { run_index: at, .. }
                    if *at >= run_index =>
                {
                    *at += 1;
                }
                CommentRangeMarker::Start { .. } | CommentRangeMarker::End { .. } => {}
            }
        }
        for marker in &mut self.bookmark_markers {
            if marker.run_index >= run_index {
                marker.run_index += 1;
            }
        }
        for (at, _, _, _) in &mut self.content_controls {
            if *at >= run_index {
                *at += 1;
            }
        }
        for (at, _, _) in &mut self.revisions {
            if *at >= run_index {
                *at += 1;
            }
        }

        let mut hyperlinks = Vec::with_capacity(self.hyperlinks.len() + 1);
        let mut hyperlink_map = Vec::with_capacity(self.hyperlinks.len());
        for mut hyperlink in self.hyperlinks.drain(..) {
            if hyperlink.run_start >= run_index {
                hyperlink.run_start += 1;
                hyperlink.run_end += 1;
                hyperlink_map.push((hyperlinks.len(), None, false));
                hyperlinks.push(hyperlink);
            } else if hyperlink.run_end > run_index {
                let split_at = run_index - hyperlink.run_start;
                let suffix = HyperlinkSpan {
                    rel_id: hyperlink.rel_id.clone(),
                    anchor: hyperlink.anchor.clone(),
                    tooltip: hyperlink.tooltip.clone(),
                    doc_location: hyperlink.doc_location.clone(),
                    run_start: run_index + 1,
                    run_end: hyperlink.run_end + 1,
                    extra_attributes: hyperlink.extra_attributes.clone(),
                    extra_xml: hyperlink
                        .extra_xml
                        .iter()
                        .filter(|(boundary, _, _)| *boundary >= split_at)
                        .map(|(boundary, before, raw)| (boundary - split_at, *before, raw.clone()))
                        .collect(),
                    preserved_raw_before: None,
                };
                hyperlink
                    .extra_xml
                    .retain(|(boundary, _, _)| *boundary < split_at);
                hyperlink.run_end = run_index;
                let prefix_index = hyperlinks.len();
                hyperlinks.push(hyperlink);
                let suffix_index = hyperlinks.len();
                hyperlinks.push(suffix);
                hyperlink_map.push((prefix_index, Some(suffix_index), false));
            } else {
                hyperlink_map.push((hyperlinks.len(), None, hyperlink.run_end == run_index));
                hyperlinks.push(hyperlink);
            }
        }
        self.hyperlinks = hyperlinks;
        for (at, slot, _) in &mut self.revisions {
            let Some(old_index) = hyperlink_revision_index(*slot) else {
                continue;
            };
            let Some((prefix, suffix, ended_at_insertion)) = hyperlink_map.get(old_index) else {
                continue;
            };
            if *ended_at_insertion && *at == run_index + 1 {
                *at = run_index;
            }
            let new_index = suffix.filter(|_| *at > run_index).unwrap_or(*prefix);
            *slot = hyperlink_revision_slot(new_index);
        }
        for (position, _) in &mut self.extra_xml {
            if *position >= run_index {
                *position += 1;
            }
        }
        self.runs.insert(run_index, run);
        true
    }

    /// Remove selected comment anchors and remap every collapsed run boundary.
    #[doc(hidden)]
    pub fn remove_comment_anchors(&mut self, ids: &[i32]) {
        for (run_index, raw_before, markers_before, _) in &mut self.content_controls {
            let preceding_removed =
                self.comment_ranges
                    .iter()
                    .filter(|marker| {
                        marker.run_index() == *run_index && marker.raw_before() == *raw_before
                    })
                    .take(*markers_before)
                    .filter(|marker| match marker {
                        CommentRangeMarker::Start { id, .. }
                        | CommentRangeMarker::End { id, .. } => ids.contains(id),
                    })
                    .count();
            *markers_before = markers_before.saturating_sub(preceding_removed);
        }
        self.comment_ranges.retain(|marker| match marker {
            CommentRangeMarker::Start { id, .. } | CommentRangeMarker::End { id, .. } => {
                !ids.contains(id)
            }
        });

        let mut removed = Vec::with_capacity(self.runs.len());
        for run in &mut self.runs {
            let removed_reference = run.remove_comment_references(ids);
            removed.push(
                removed_reference
                    && run.content.is_empty()
                    && run.extra_xml.is_empty()
                    && run.alt_drawings.is_empty(),
            );
        }
        if removed.iter().all(|remove| !remove) {
            return;
        }

        let old_run_count = self.runs.len();
        let boundary_map = (0..=old_run_count)
            .map(|boundary| {
                boundary
                    - removed
                        .iter()
                        .take(boundary)
                        .filter(|remove| **remove)
                        .count()
            })
            .collect::<Vec<_>>();
        let mut raw_counts = vec![0usize; old_run_count + 1];
        for (position, _) in &self.extra_xml {
            raw_counts[(*position).min(old_run_count)] += 1;
        }
        let mut raw_prefixes = vec![0usize; old_run_count + 1];
        for boundary in 1..=old_run_count {
            if boundary_map[boundary] == boundary_map[boundary - 1] {
                raw_prefixes[boundary] = raw_prefixes[boundary - 1] + raw_counts[boundary - 1];
            }
        }

        self.extra_xml.sort_by_key(|(position, _)| *position);
        self.comment_ranges
            .sort_by_key(|marker| (marker.run_index(), marker.raw_before()));
        self.bookmark_markers.sort_by_key(|marker| marker.run_index);
        self.content_controls
            .sort_by_key(|(at, raw_before, markers_before, _)| (*at, *raw_before, *markers_before));

        for control_index in 0..self.content_controls.len() {
            let (run_index, raw_before, markers_before) = {
                let control = &self.content_controls[control_index];
                (control.0, control.1, control.2)
            };
            let old_boundary = run_index.min(old_run_count);
            let old_raw_slot = raw_before.min(raw_counts[old_boundary]);
            let new_boundary = boundary_map[old_boundary];
            let new_raw_slot = raw_prefixes[old_boundary] + old_raw_slot;
            let preceding_markers = self
                .comment_ranges
                .iter()
                .filter(|marker| {
                    let marker_boundary = marker.run_index().min(old_run_count);
                    marker_boundary < old_boundary
                        && boundary_map[marker_boundary] == new_boundary
                        && raw_prefixes[marker_boundary]
                            + marker.raw_before().min(raw_counts[marker_boundary])
                            == new_raw_slot
                })
                .count();
            let local_marker_count = self
                .comment_ranges
                .iter()
                .filter(|marker| {
                    marker.run_index() == old_boundary
                        && marker.raw_before().min(raw_counts[old_boundary]) == old_raw_slot
                })
                .count();
            let control = &mut self.content_controls[control_index];
            control.0 = new_boundary;
            control.1 = new_raw_slot;
            control.2 = preceding_markers + markers_before.min(local_marker_count);
        }
        for marker in &mut self.comment_ranges {
            match marker {
                CommentRangeMarker::Start {
                    run_index,
                    raw_before,
                    ..
                }
                | CommentRangeMarker::End {
                    run_index,
                    raw_before,
                    ..
                } => {
                    let old_boundary = (*run_index).min(old_run_count);
                    *run_index = boundary_map[old_boundary];
                    *raw_before =
                        raw_prefixes[old_boundary] + (*raw_before).min(raw_counts[old_boundary]);
                }
            }
        }
        for marker in &mut self.bookmark_markers {
            let old_boundary = marker.run_index.min(old_run_count);
            marker.run_index = boundary_map[old_boundary];
            marker.raw_before =
                raw_prefixes[old_boundary] + marker.raw_before.min(raw_counts[old_boundary]);
        }
        let hyperlink_revision_counts = self
            .hyperlinks
            .iter()
            .enumerate()
            .map(|(hyperlink_index, hyperlink)| {
                let start = hyperlink.run_start.min(old_run_count);
                let end = hyperlink.run_end.min(old_run_count);
                (start..=end)
                    .map(|boundary| {
                        self.revisions
                            .iter()
                            .filter(|(at, slot, _)| {
                                *at == boundary
                                    && hyperlink_revision_index(*slot) == Some(hyperlink_index)
                            })
                            .count()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for (run_index, raw_before, _) in &mut self.revisions {
            let old_boundary = (*run_index).min(old_run_count);
            *run_index = boundary_map[old_boundary];
            if hyperlink_revision_index(*raw_before).is_none() {
                *raw_before =
                    raw_prefixes[old_boundary] + (*raw_before).min(raw_counts[old_boundary]);
            }
        }
        for (position, _) in &mut self.extra_xml {
            *position = boundary_map[(*position).min(old_run_count)];
        }
        let old_hyperlinks = std::mem::take(&mut self.hyperlinks);
        let mut hyperlink_map = vec![None; old_hyperlinks.len()];
        for (old_index, mut hyperlink) in old_hyperlinks.into_iter().enumerate() {
            let old_start = hyperlink.run_start.min(old_run_count);
            let old_end = hyperlink.run_end.min(old_run_count);
            if let Some(raw_before) = hyperlink.preserved_raw_before {
                hyperlink.preserved_raw_before =
                    Some(raw_prefixes[old_start] + raw_before.min(raw_counts[old_start]));
            }
            let revision_counts = &hyperlink_revision_counts[old_index];
            let new_start = boundary_map[old_start];
            let new_end = boundary_map[old_end];
            for (boundary, revisions_before, _) in &mut hyperlink.extra_xml {
                let old_relative = (*boundary).min(old_end - old_start);
                let old_boundary = old_start + old_relative;
                let new_boundary = boundary_map[old_boundary];
                let collapsed_before = (0..old_relative)
                    .filter(|relative| boundary_map[old_start + relative] == new_boundary)
                    .map(|relative| revision_counts[relative])
                    .sum::<usize>();
                *boundary = new_boundary.saturating_sub(new_start);
                *revisions_before =
                    collapsed_before + (*revisions_before).min(revision_counts[old_relative]);
            }
            let owns_revision = self
                .revisions
                .iter()
                .any(|(_, slot, _)| hyperlink_revision_index(*slot) == Some(old_index));
            hyperlink.run_start = new_start;
            hyperlink.run_end = new_end;
            if new_start < new_end || owns_revision || !hyperlink.extra_xml.is_empty() {
                hyperlink_map[old_index] = Some(self.hyperlinks.len());
                self.hyperlinks.push(hyperlink);
            }
        }
        for (_, slot, _) in &mut self.revisions {
            let Some(old_index) = hyperlink_revision_index(*slot) else {
                continue;
            };
            if let Some(new_index) = hyperlink_map.get(old_index).copied().flatten() {
                *slot = hyperlink_revision_slot(new_index);
            }
        }
        self.runs = self
            .runs
            .drain(..)
            .zip(removed)
            .filter_map(|(run, remove)| (!remove).then_some(run))
            .collect();
    }

    /// Insert a canonical bookmark start marker at a direct-run boundary.
    pub fn insert_bookmark_start(&mut self, run_index: usize, id: i32, name: &str) -> bool {
        if run_index > self.runs.len() {
            return false;
        }
        let mut value = itoa::Buffer::new();
        let mut element = BytesStart::new("w:bookmarkStart");
        element.push_attribute(("w:id", value.format(id)));
        element.push_attribute(("w:name", name));
        let mut raw = Vec::new();
        if Writer::new(&mut raw)
            .write_event(Event::Empty(element))
            .is_err()
        {
            return false;
        }
        let raw_before = self
            .extra_xml
            .iter()
            .filter(|(at, _)| *at == run_index)
            .count();
        self.extra_xml.push((run_index, raw));
        self.bookmark_markers.push(BookmarkMarker::new(
            true,
            id,
            Some(name.to_owned()),
            run_index,
            raw_before,
        ));
        true
    }

    /// Insert a canonical bookmark end marker at a direct-run boundary.
    pub fn insert_bookmark_end(&mut self, run_index: usize, id: i32) -> bool {
        if run_index > self.runs.len() {
            return false;
        }
        let mut value = itoa::Buffer::new();
        let mut element = BytesStart::new("w:bookmarkEnd");
        element.push_attribute(("w:id", value.format(id)));
        let mut raw = Vec::new();
        if Writer::new(&mut raw)
            .write_event(Event::Empty(element))
            .is_err()
        {
            return false;
        }
        let raw_before = self
            .extra_xml
            .iter()
            .filter(|(at, _)| *at == run_index)
            .count();
        self.extra_xml.push((run_index, raw));
        self.bookmark_markers
            .push(BookmarkMarker::new(false, id, None, run_index, raw_before));
        true
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_prefixes(
            reader,
            &[
                "w".to_string(),
                format!("\0r\0{R_NS}"),
                format!("\0mc\0{}", crate::namespace::MC_NS),
            ],
        )
    }

    pub(crate) fn from_xml_with_prefixes(
        reader: &mut Reader<&[u8]>,
        word_prefixes: &[String],
    ) -> Result<Self> {
        reader.config_mut().trim_text(false);
        let mut properties = None;
        let mut runs = Vec::new();
        let mut hyperlinks = Vec::new();
        let mut comment_ranges = Vec::new();
        let mut bookmark_markers = Vec::new();
        let mut extra_xml = Vec::new();
        let mut content_controls = Vec::new();
        let mut revisions = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if is_word_element(name.as_ref(), b"pPr", &prefixes) {
                        let raw = capture_element(reader, e)?;
                        properties = Some(parse_scoped_ppr(&raw, word_prefixes)?);
                    } else if is_word_element(name.as_ref(), b"r", &prefixes) {
                        runs.push(CT_R::from_xml_with_prefixes(reader, &prefixes)?);
                    } else if matches_local_name(name.as_ref(), b"r")
                        && !element_prefix_has_binding(name.as_ref(), &prefixes)
                    {
                        let prefixes = prefixes_with_assumed_word_owner(name.as_ref(), &prefixes)?;
                        runs.push(CT_R::from_xml_with_prefixes(reader, &prefixes)?);
                    } else if is_word_element(name.as_ref(), b"sdt", &prefixes) {
                        let raw = capture_element(reader, e)?;
                        if let Some(sdt) = CT_Sdt::from_raw(&raw, &prefixes) {
                            let raw_before =
                                extra_xml.iter().filter(|(at, _)| *at == runs.len()).count();
                            let markers_before = comment_ranges
                                .iter()
                                .filter(|marker: &&CommentRangeMarker| {
                                    marker.run_index() == runs.len()
                                        && marker.raw_before() == raw_before
                                })
                                .count();
                            content_controls.push((runs.len(), raw_before, markers_before, sdt));
                        } else {
                            extra_xml.push((runs.len(), raw));
                        }
                    } else if is_word_element(name.as_ref(), b"hyperlink", &prefixes) {
                        let (rel_id, anchor, tooltip, doc_location, extra_attributes) =
                            parse_hyperlink_attributes(e, &prefixes)?;
                        let raw = capture_element(reader, e)?;
                        let parsed = parse_hyperlink_children(&raw, &prefixes)?;
                        let run_start = runs.len();
                        if parsed.runs.is_empty() && parsed.revisions.is_empty() {
                            extra_xml.push((run_start, raw));
                        } else {
                            let hyperlink_index = hyperlinks.len();
                            let run_end = run_start + parsed.runs.len();
                            let preserved_raw_before = parsed.runs.is_empty().then(|| {
                                extra_xml.iter().filter(|(at, _)| *at == run_start).count()
                            });
                            runs.extend(parsed.runs);
                            hyperlinks.push(HyperlinkSpan {
                                rel_id,
                                anchor,
                                tooltip,
                                doc_location,
                                run_start,
                                run_end,
                                extra_attributes,
                                extra_xml: parsed.extra_xml,
                                preserved_raw_before,
                            });
                            revisions.extend(parsed.revisions.into_iter().map(|(at, revision)| {
                                (
                                    run_start + at,
                                    hyperlink_revision_slot(hyperlink_index),
                                    revision,
                                )
                            }));
                            if preserved_raw_before.is_some() {
                                extra_xml.push((run_start, raw));
                            }
                        }
                    } else if is_word_element(name.as_ref(), b"fldSimple", &prefixes) {
                        // Parse simple field: extract w:instr attribute
                        let instr =
                            optional_word_attribute(e, b"instr", &prefixes).unwrap_or_default();

                        let field_type = parse_field_instruction(&instr);
                        let mut dirty = None;
                        let mut has_unmodeled_attributes = false;
                        for attribute in e.attributes() {
                            let attribute = attribute?;
                            let name = attribute.key.as_ref();
                            let value = attribute
                                .decoded_and_normalized_value(XmlVersion::Implicit1_0, e.decoder())?
                                .into_owned();
                            if attribute_in_namespace(
                                name,
                                b"instr",
                                crate::namespace::W_NS,
                                &prefixes,
                            ) {
                                continue;
                            }
                            if attribute_in_namespace(
                                name,
                                b"dirty",
                                crate::namespace::W_NS,
                                &prefixes,
                            ) {
                                dirty = Some(matches!(value.as_str(), "true" | "1" | "on"));
                            } else {
                                has_unmodeled_attributes = true;
                            }
                        }

                        let mut display = String::new();
                        let mut inner_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut inner_buf) {
                                Ok(Event::Start(ref child)) => {
                                    let child_prefixes = word_prefixes_at(child, &prefixes)?;
                                    if is_word_element(child.name().as_ref(), b"r", &child_prefixes)
                                    {
                                        display.push_str(
                                            &CT_R::from_xml_with_prefixes(reader, &child_prefixes)?
                                                .text(),
                                        );
                                    } else {
                                        reader.read_to_end_into(child.name(), &mut Vec::new())?;
                                    }
                                }
                                Ok(Event::End(ref child))
                                    if matches_local_name(child.name().as_ref(), b"fldSimple") =>
                                {
                                    break;
                                }
                                Ok(Event::Eof) => break,
                                Err(error) => return Err(error.into()),
                                _ => {}
                            }
                            inner_buf.clear();
                        }

                        // Add a synthetic run with the field content
                        runs.push(CT_R {
                            properties: None,
                            content: vec![RunContent::Field {
                                field_type,
                                display,
                            }],
                            simple_field: Some(SimpleFieldMetadata {
                                dirty,
                                has_unmodeled_attributes,
                            }),
                            extra_xml: Vec::new(),
                            extra_xml_positions: Vec::new(),
                            field_markers: Vec::new(),
                            alt_drawings: Vec::new(),
                        });
                    } else if is_word_element(name.as_ref(), b"commentRangeStart", &prefixes)
                        || is_word_element(name.as_ref(), b"commentRangeEnd", &prefixes)
                    {
                        let id = required_word_i32_attribute(e, b"id", &prefixes)?;
                        reader.read_to_end_into(name, &mut Vec::new())?;
                        push_comment_marker(
                            &mut comment_ranges,
                            &extra_xml,
                            runs.len(),
                            id,
                            is_word_element(name.as_ref(), b"commentRangeStart", &prefixes),
                        );
                    } else if is_word_element(name.as_ref(), b"bookmarkStart", &prefixes)
                        || is_word_element(name.as_ref(), b"bookmarkEnd", &prefixes)
                    {
                        let start = is_word_element(name.as_ref(), b"bookmarkStart", &prefixes);
                        let id = optional_word_attribute(e, b"id", &prefixes)
                            .and_then(|value| value.parse().ok());
                        let bookmark_name = optional_word_attribute(e, b"name", &prefixes);
                        bookmark_markers.push(BookmarkMarker {
                            start,
                            id,
                            name: bookmark_name,
                            run_index: runs.len(),
                            raw_before: extra_xml
                                .iter()
                                .filter(|(at, _)| *at == runs.len())
                                .count(),
                        });
                        extra_xml.push((runs.len(), capture_element(reader, e)?));
                    } else if is_word_element(name.as_ref(), b"ins", &prefixes)
                        || is_word_element(name.as_ref(), b"del", &prefixes)
                        || is_word_element(name.as_ref(), b"moveFrom", &prefixes)
                        || is_word_element(name.as_ref(), b"moveTo", &prefixes)
                    {
                        let raw_before =
                            extra_xml.iter().filter(|(at, _)| *at == runs.len()).count();
                        let raw = capture_element(reader, e)?;
                        if let Some(revision) =
                            CT_Revision::from_raw_content(raw.clone(), &prefixes)
                        {
                            revisions.push((runs.len(), raw_before, revision));
                        }
                        extra_xml.push((runs.len(), raw));
                    } else {
                        // Capture unknown elements (bookmarks, comments, etc.) as raw XML
                        extra_xml.push((runs.len(), capture_element(reader, e)?));
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if is_word_element(name.as_ref(), b"commentRangeStart", &prefixes)
                        || is_word_element(name.as_ref(), b"commentRangeEnd", &prefixes)
                    {
                        let id = required_word_i32_attribute(e, b"id", &prefixes)?;
                        push_comment_marker(
                            &mut comment_ranges,
                            &extra_xml,
                            runs.len(),
                            id,
                            is_word_element(name.as_ref(), b"commentRangeStart", &prefixes),
                        );
                    } else if is_word_element(name.as_ref(), b"bookmarkStart", &prefixes)
                        || is_word_element(name.as_ref(), b"bookmarkEnd", &prefixes)
                    {
                        let start = is_word_element(name.as_ref(), b"bookmarkStart", &prefixes);
                        let id = optional_word_attribute(e, b"id", &prefixes)
                            .and_then(|value| value.parse().ok());
                        let bookmark_name = optional_word_attribute(e, b"name", &prefixes);
                        bookmark_markers.push(BookmarkMarker {
                            start,
                            id,
                            name: bookmark_name,
                            run_index: runs.len(),
                            raw_before: extra_xml
                                .iter()
                                .filter(|(at, _)| *at == runs.len())
                                .count(),
                        });
                        extra_xml.push((runs.len(), capture_empty_element(e)?));
                    } else if is_word_element(name.as_ref(), b"fldSimple", &prefixes) {
                        let instruction =
                            optional_word_attribute(e, b"instr", &prefixes).unwrap_or_default();
                        let mut dirty = None;
                        let mut has_unmodeled_attributes = false;
                        for attribute in e.attributes() {
                            let attribute = attribute?;
                            let name = attribute.key.as_ref();
                            let value = attribute
                                .decoded_and_normalized_value(XmlVersion::Implicit1_0, e.decoder())?
                                .into_owned();
                            if attribute_in_namespace(
                                name,
                                b"instr",
                                crate::namespace::W_NS,
                                &prefixes,
                            ) {
                                continue;
                            }
                            if attribute_in_namespace(
                                name,
                                b"dirty",
                                crate::namespace::W_NS,
                                &prefixes,
                            ) {
                                dirty = Some(matches!(value.as_str(), "true" | "1" | "on"));
                            } else {
                                has_unmodeled_attributes = true;
                            }
                        }
                        runs.push(CT_R {
                            properties: None,
                            content: vec![RunContent::Field {
                                field_type: parse_field_instruction(&instruction),
                                display: String::new(),
                            }],
                            simple_field: Some(SimpleFieldMetadata {
                                dirty,
                                has_unmodeled_attributes,
                            }),
                            extra_xml: Vec::new(),
                            extra_xml_positions: Vec::new(),
                            field_markers: Vec::new(),
                            alt_drawings: Vec::new(),
                        });
                    } else if !matches_local_name(name.as_ref(), b"p") {
                        let raw_before =
                            extra_xml.iter().filter(|(at, _)| *at == runs.len()).count();
                        let raw = capture_empty_element(e)?;
                        if (is_word_element(name.as_ref(), b"ins", &prefixes)
                            || is_word_element(name.as_ref(), b"del", &prefixes)
                            || is_word_element(name.as_ref(), b"moveFrom", &prefixes)
                            || is_word_element(name.as_ref(), b"moveTo", &prefixes))
                            && let Some(revision) =
                                CT_Revision::from_raw_content(raw.clone(), &prefixes)
                        {
                            revisions.push((runs.len(), raw_before, revision));
                        }
                        extra_xml.push((runs.len(), raw));
                    }
                }
                Ok(Event::End(ref e)) if matches_local_name(e.name().as_ref(), b"p") => {
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(CT_P {
            properties,
            runs,
            hyperlinks,
            comment_ranges,
            bookmark_markers,
            extra_xml,
            content_controls,
            revisions,
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        self.to_xml_with_para_id(writer, None)
    }

    pub(crate) fn to_xml_with_para_id<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        para_id: Option<&str>,
    ) -> Result<()> {
        let mut start = BytesStart::new("w:p");
        if let Some(para_id) = para_id {
            start.push_attribute(("w14:paraId", para_id));
        }
        writer.write_event(Event::Start(start))?;

        if let Some(ref props) = self.properties {
            props.to_xml(writer)?;
        }

        // Build a set of run indices that are inside hyperlinks
        let mut hyperlink_runs: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for (hl_idx, hl) in self.hyperlinks.iter().enumerate() {
            for run_idx in hl.run_start..hl.run_end {
                hyperlink_runs.insert(run_idx, hl_idx);
            }
        }

        let mut current_hyperlink: Option<usize> = None;
        for (run_idx, run) in self.runs.iter().enumerate() {
            let in_hl = hyperlink_runs.get(&run_idx).copied();

            // Paragraph boundary content is a sibling of the hyperlink.
            if current_hyperlink.is_some() && current_hyperlink != in_hl {
                let hyperlink_index = current_hyperlink.expect("open hyperlink exists");
                write_hyperlink_boundary(
                    writer,
                    &self.revisions,
                    hyperlink_index,
                    &self.hyperlinks[hyperlink_index],
                    run_idx,
                )?;
                writer.write_event(Event::End(BytesEnd::new(hyperlink_qname(
                    &self.hyperlinks[hyperlink_index],
                ))))?;
                current_hyperlink = None;
            }

            write_paragraph_boundary(
                writer,
                &self.extra_xml,
                &self.content_controls,
                &self.comment_ranges,
                &self.hyperlinks,
                &self.revisions,
                run_idx,
            )?;

            write_empty_hyperlinks(writer, &self.hyperlinks, &self.revisions, run_idx)?;

            // Open hyperlink if entering one
            if let Some(hl_idx) = in_hl
                && current_hyperlink != in_hl
            {
                let hl = &self.hyperlinks[hl_idx];
                write_hyperlink_start(writer, hl)?;
                current_hyperlink = in_hl;
            }

            if let Some(hyperlink_index) = current_hyperlink {
                write_hyperlink_boundary(
                    writer,
                    &self.revisions,
                    hyperlink_index,
                    &self.hyperlinks[hyperlink_index],
                    run_idx,
                )?;
            }

            // Check if this run is a field run
            if run.content.len() == 1
                && let RunContent::Field {
                    field_type,
                    display,
                } = &run.content[0]
            {
                let instr = match field_type {
                    FieldType::Page => " PAGE ",
                    FieldType::NumPages => " NUMPAGES ",
                    FieldType::Ref { instruction, .. } | FieldType::PageRef { instruction, .. } => {
                        instruction.as_str()
                    }
                    FieldType::Other(s) => s.as_str(),
                };
                let mut fld = BytesStart::new("w:fldSimple");
                if current_hyperlink
                    .and_then(|index| shadowed_word_namespace(&self.hyperlinks[index]))
                    .is_some()
                {
                    fld.push_attribute(("xmlns:w", crate::namespace::W_NS));
                }
                fld.push_attribute(("w:instr", instr));
                writer.write_event(Event::Start(fld))?;
                // Emit a default display run
                writer.write_event(Event::Start(BytesStart::new("w:r")))?;
                writer.write_event(Event::Start(BytesStart::new("w:t")))?;
                let display = match field_type {
                    FieldType::Page | FieldType::NumPages if display.is_empty() => "1",
                    _ => display.as_str(),
                };
                writer.write_event(Event::Text(BytesText::new(display)))?;
                writer.write_event(Event::End(BytesEnd::new("w:t")))?;
                writer.write_event(Event::End(BytesEnd::new("w:r")))?;
                writer.write_event(Event::End(BytesEnd::new("w:fldSimple")))?;
                continue;
            }

            if let Some(hyperlink_index) = current_hyperlink {
                run.to_xml_with_word_override(
                    writer,
                    shadowed_word_namespace(&self.hyperlinks[hyperlink_index]),
                )?;
            } else {
                run.to_xml(writer)?;
            }
        }

        // Close any remaining open hyperlink
        if let Some(hyperlink_index) = current_hyperlink {
            write_hyperlink_boundary(
                writer,
                &self.revisions,
                hyperlink_index,
                &self.hyperlinks[hyperlink_index],
                self.runs.len(),
            )?;
            writer.write_event(Event::End(BytesEnd::new(hyperlink_qname(
                &self.hyperlinks[hyperlink_index],
            ))))?;
        }

        write_paragraph_boundary(
            writer,
            &self.extra_xml,
            &self.content_controls,
            &self.comment_ranges,
            &self.hyperlinks,
            &self.revisions,
            self.runs.len(),
        )?;
        write_empty_hyperlinks(writer, &self.hyperlinks, &self.revisions, self.runs.len())?;

        writer.write_event(Event::End(BytesEnd::new("w:p")))?;
        Ok(())
    }

    pub(crate) fn collect_runs<'a>(&'a self, runs: &mut Vec<&'a CT_R>) {
        for index in 0..=self.runs.len() {
            for (_, _, _, sdt) in self
                .content_controls
                .iter()
                .filter(|(at, _, _, _)| *at == index)
            {
                sdt.collect_runs(runs);
            }
            if let Some(run) = self.runs.get(index) {
                runs.push(run);
            }
        }
    }

    pub(crate) fn collect_controls<'a>(&'a self, controls: &mut Vec<&'a CT_Sdt>) {
        for (_, _, _, sdt) in &self.content_controls {
            controls.push(sdt);
            sdt.collect_controls(controls);
        }
    }
}

fn push_comment_marker(
    markers: &mut Vec<CommentRangeMarker>,
    extra_xml: &[(usize, Vec<u8>)],
    run_index: usize,
    id: i32,
    start: bool,
) {
    let raw_before = extra_xml
        .iter()
        .filter(|(position, _)| *position == run_index)
        .count();
    let marker = if start {
        CommentRangeMarker::Start {
            id,
            run_index,
            raw_before,
        }
    } else {
        CommentRangeMarker::End {
            id,
            run_index,
            raw_before,
        }
    };
    markers.push(marker);
}

fn parse_hyperlink_children(
    raw: &[u8],
    word_prefixes: &[String],
) -> Result<ParsedHyperlinkChildren> {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(false);
    let mut runs = Vec::new();
    let mut revisions = Vec::new();
    let mut extra_xml = Vec::new();
    let mut revisions_at_boundary = 0usize;
    let mut buffer = Vec::new();
    let mut inside = false;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if !inside => {
                inside = true;
                let _ = word_prefixes_at(&element, word_prefixes)?;
            }
            Event::Start(element) => {
                let prefixes = word_prefixes_at(&element, word_prefixes)?;
                if is_word_element(element.name().as_ref(), b"r", &prefixes) {
                    runs.push(CT_R::from_xml_with_prefixes(&mut reader, &prefixes)?);
                    revisions_at_boundary = 0;
                } else if is_content_revision_element(element.name().as_ref(), &prefixes) {
                    let raw = capture_element(&mut reader, &element)?;
                    if let Some(revision) = CT_Revision::from_raw(raw.clone(), &prefixes) {
                        revisions.push((runs.len(), revision));
                        revisions_at_boundary += 1;
                    } else {
                        extra_xml.push((runs.len(), revisions_at_boundary, raw));
                    }
                } else {
                    extra_xml.push((
                        runs.len(),
                        revisions_at_boundary,
                        capture_element(&mut reader, &element)?,
                    ));
                }
            }
            Event::Empty(element) => {
                let prefixes = word_prefixes_at(&element, word_prefixes)?;
                let raw = capture_empty_element(&element)?;
                if is_content_revision_element(element.name().as_ref(), &prefixes) {
                    if let Some(revision) = CT_Revision::from_raw(raw.clone(), &prefixes) {
                        revisions.push((runs.len(), revision));
                        revisions_at_boundary += 1;
                    } else {
                        extra_xml.push((runs.len(), revisions_at_boundary, raw));
                    }
                } else {
                    extra_xml.push((runs.len(), revisions_at_boundary, raw));
                }
            }
            Event::End(element)
                if inside && matches_local_name(element.name().as_ref(), b"hyperlink") =>
            {
                break;
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(ParsedHyperlinkChildren {
        runs,
        revisions,
        extra_xml,
    })
}

fn parse_hyperlink_attributes(
    element: &BytesStart<'_>,
    scope: &[String],
) -> Result<ParsedHyperlinkAttributes> {
    let mut rel_id = None;
    let mut anchor = None;
    let mut tooltip = None;
    let mut doc_location = None;
    let mut extra = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute?;
        let name = attribute.key.as_ref();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
            .into_owned();
        if attribute_in_namespace(name, b"id", R_NS, scope) {
            rel_id = Some(value);
        } else if attribute_in_namespace(name, b"anchor", crate::namespace::W_NS, scope) {
            anchor = Some(value);
        } else if attribute_in_namespace(name, b"tooltip", crate::namespace::W_NS, scope) {
            tooltip = Some(value);
        } else if attribute_in_namespace(name, b"docLocation", crate::namespace::W_NS, scope) {
            doc_location = Some(value);
        } else {
            extra.push((std::str::from_utf8(name)?.to_owned(), value));
        }
    }
    Ok((rel_id, anchor, tooltip, doc_location, extra))
}

fn attribute_in_namespace(name: &[u8], local: &[u8], namespace: &str, scope: &[String]) -> bool {
    let Some(separator) = name.iter().position(|byte| *byte == b':') else {
        return false;
    };
    if name.get(separator + 1..) != Some(local) {
        return false;
    }
    let prefix = &name[..separator];
    let bound_namespace = scope.iter().find_map(|binding| {
        binding
            .strip_prefix('\0')
            .and_then(|binding| binding.split_once('\0'))
            .filter(|(candidate, _)| candidate.as_bytes() == prefix)
            .map(|(_, value)| value)
    });
    match bound_namespace {
        Some(value) => value == namespace,
        None if namespace == crate::namespace::W_NS => {
            scope.iter().any(|candidate| candidate.as_bytes() == prefix)
        }
        None => false,
    }
}

fn is_element_in_namespace(name: &[u8], local: &[u8], namespace: &str, scope: &[String]) -> bool {
    let (prefix, actual_local) = name
        .iter()
        .position(|byte| *byte == b':')
        .map_or((&b""[..], name), |separator| {
            (&name[..separator], &name[separator + 1..])
        });
    actual_local == local
        && scope.iter().any(|binding| {
            binding
                .strip_prefix('\0')
                .and_then(|binding| binding.split_once('\0'))
                .is_some_and(|(candidate, value)| {
                    candidate.as_bytes() == prefix && value == namespace
                })
        })
}

fn element_prefix_has_binding(name: &[u8], scope: &[String]) -> bool {
    let prefix = name
        .iter()
        .position(|byte| *byte == b':')
        .map_or(&b""[..], |separator| &name[..separator]);
    scope.iter().any(|binding| {
        binding
            .strip_prefix('\0')
            .and_then(|binding| binding.split_once('\0'))
            .is_some_and(|(candidate, _)| candidate.as_bytes() == prefix)
    })
}

fn prefixes_with_assumed_word_owner(name: &[u8], scope: &[String]) -> Result<Vec<String>> {
    let prefix = name
        .iter()
        .position(|byte| *byte == b':')
        .map_or(&b""[..], |separator| &name[..separator]);
    let mut prefixes = scope.to_vec();
    prefixes.push(std::str::from_utf8(prefix)?.to_owned());
    Ok(prefixes)
}

fn is_content_revision_element(name: &[u8], word_prefixes: &[String]) -> bool {
    is_word_element(name, b"ins", word_prefixes)
        || is_word_element(name, b"del", word_prefixes)
        || is_word_element(name, b"moveFrom", word_prefixes)
        || is_word_element(name, b"moveTo", word_prefixes)
}

fn write_paragraph_boundary<W: std::io::Write>(
    writer: &mut Writer<W>,
    extra_xml: &[(usize, Vec<u8>)],
    content_controls: &[(usize, usize, usize, CT_Sdt)],
    markers: &[CommentRangeMarker],
    hyperlinks: &[HyperlinkSpan],
    revisions: &[(usize, usize, CT_Revision)],
    run_index: usize,
) -> Result<()> {
    let extras = extra_xml
        .iter()
        .filter(|(position, _)| *position == run_index)
        .map(|(_, raw)| raw)
        .collect::<Vec<_>>();
    for raw_index in 0..=extras.len() {
        let boundary_markers = markers
            .iter()
            .filter(|marker| {
                marker.run_index() == run_index
                    && marker.raw_before().min(extras.len()) == raw_index
            })
            .collect::<Vec<_>>();
        for marker_index in 0..=boundary_markers.len() {
            for (_, _, _, sdt) in
                content_controls
                    .iter()
                    .filter(|(at, raw_before, markers_before, _)| {
                        *at == run_index
                            && (*raw_before).min(extras.len()) == raw_index
                            && (*markers_before).min(boundary_markers.len()) == marker_index
                    })
            {
                sdt.to_xml(writer)?;
            }
            if let Some(marker) = boundary_markers.get(marker_index) {
                let (tag, id) = match marker {
                    CommentRangeMarker::Start { id, .. } => ("w:commentRangeStart", *id),
                    CommentRangeMarker::End { id, .. } => ("w:commentRangeEnd", *id),
                };
                let mut value = itoa::Buffer::new();
                let mut element = BytesStart::new(tag);
                element.push_attribute(("w:id", value.format(id)));
                writer.write_event(Event::Empty(element))?;
            }
        }
        if let Some(raw) = extras.get(raw_index) {
            if let Some((hyperlink_index, hyperlink)) =
                hyperlinks.iter().enumerate().find(|(_, hyperlink)| {
                    hyperlink.run_start == run_index
                        && hyperlink.run_end == run_index
                        && hyperlink.preserved_raw_before == Some(raw_index)
                })
            {
                let mut replacement = Vec::new();
                let mut replacement_writer = Writer::new(&mut replacement);
                write_hyperlink_start(&mut replacement_writer, hyperlink)?;
                write_hyperlink_boundary(
                    &mut replacement_writer,
                    revisions,
                    hyperlink_index,
                    hyperlink,
                    run_index,
                )?;
                replacement_writer
                    .write_event(Event::End(BytesEnd::new(hyperlink_qname(hyperlink))))?;
                writer.get_mut().write_all(&replacement)?;
            } else {
                writer.get_mut().write_all(raw)?;
            }
        }
    }
    Ok(())
}

fn write_hyperlink_boundary<W: std::io::Write>(
    writer: &mut Writer<W>,
    revisions: &[(usize, usize, CT_Revision)],
    hyperlink_index: usize,
    hyperlink: &HyperlinkSpan,
    run_index: usize,
) -> Result<()> {
    let boundary_revisions = revisions
        .iter()
        .filter(|(at, slot, _)| {
            *at == run_index && hyperlink_revision_index(*slot) == Some(hyperlink_index)
        })
        .map(|(_, _, revision)| revision)
        .collect::<Vec<_>>();
    let relative_boundary = run_index.saturating_sub(hyperlink.run_start);
    for revision_index in 0..=boundary_revisions.len() {
        for (_, _, raw) in hyperlink.extra_xml.iter().filter(|(boundary, before, _)| {
            *boundary == relative_boundary
                && (*before).min(boundary_revisions.len()) == revision_index
        }) {
            writer.get_mut().write_all(raw)?;
        }
        if let Some(revision) = boundary_revisions.get(revision_index) {
            revision.write_xml(writer)?;
        }
    }
    Ok(())
}

fn write_hyperlink_start<W: std::io::Write>(
    writer: &mut Writer<W>,
    hyperlink: &HyperlinkSpan,
) -> Result<()> {
    let (word_prefix, declare_word) =
        safe_hyperlink_prefix(hyperlink, "w", "rdocxW", crate::namespace::W_NS);
    let (relationship_prefix, declare_relationship) =
        safe_hyperlink_prefix(hyperlink, "r", "rdocxR", R_NS);
    let mut element = BytesStart::new(format!("{word_prefix}:hyperlink"));
    let word_declaration = format!("xmlns:{word_prefix}");
    let relationship_declaration = format!("xmlns:{relationship_prefix}");
    let relationship_id = format!("{relationship_prefix}:id");
    let anchor_name = format!("{word_prefix}:anchor");
    if declare_word {
        element.push_attribute((word_declaration.as_str(), crate::namespace::W_NS));
    }
    if hyperlink.rel_id.is_some() && declare_relationship {
        element.push_attribute((relationship_declaration.as_str(), R_NS));
    }
    if let Some(rel_id) = &hyperlink.rel_id {
        element.push_attribute((relationship_id.as_str(), rel_id.as_str()));
    }
    if let Some(anchor) = &hyperlink.anchor {
        element.push_attribute((anchor_name.as_str(), anchor.as_str()));
    }
    if let Some(tooltip) = &hyperlink.tooltip {
        let tooltip_name = format!("{word_prefix}:tooltip");
        element.push_attribute((tooltip_name.as_str(), tooltip.as_str()));
    }
    if let Some(doc_location) = &hyperlink.doc_location {
        let doc_location_name = format!("{word_prefix}:docLocation");
        element.push_attribute((doc_location_name.as_str(), doc_location.as_str()));
    }
    for (name, value) in &hyperlink.extra_attributes {
        element.push_attribute((name.as_str(), value.as_str()));
    }
    writer.write_event(Event::Start(element))?;
    Ok(())
}

fn hyperlink_qname(hyperlink: &HyperlinkSpan) -> String {
    let (prefix, _) = safe_hyperlink_prefix(hyperlink, "w", "rdocxW", crate::namespace::W_NS);
    format!("{prefix}:hyperlink")
}

fn safe_hyperlink_prefix(
    hyperlink: &HyperlinkSpan,
    preferred: &str,
    fallback: &str,
    namespace: &str,
) -> (String, bool) {
    let preferred_declaration = format!("xmlns:{preferred}");
    match hyperlink
        .extra_attributes
        .iter()
        .find(|(name, _)| name == &preferred_declaration)
    {
        None => return (preferred.to_owned(), false),
        Some((_, value)) if value == namespace => return (preferred.to_owned(), false),
        Some(_) => {}
    }
    for suffix in 0usize.. {
        let candidate = if suffix == 0 {
            fallback.to_owned()
        } else {
            format!("{fallback}{suffix}")
        };
        let declaration = format!("xmlns:{candidate}");
        if !hyperlink
            .extra_attributes
            .iter()
            .any(|(name, _)| name == &declaration)
        {
            return (candidate, true);
        }
    }
    unreachable!("the finite attribute set cannot occupy every prefix")
}

fn shadowed_word_namespace(hyperlink: &HyperlinkSpan) -> Option<&str> {
    hyperlink
        .extra_attributes
        .iter()
        .find(|(name, value)| name == "xmlns:w" && value != crate::namespace::W_NS)
        .map(|(_, value)| value.as_str())
}

pub(crate) fn write_raw_with_word_override<W: std::io::Write>(
    writer: &mut Writer<W>,
    raw: &[u8],
    foreign_word_namespace: Option<&str>,
) -> Result<()> {
    let Some(namespace) = foreign_word_namespace else {
        writer.get_mut().write_all(raw)?;
        return Ok(());
    };
    if !raw_uses_external_word_binding(raw) {
        writer.get_mut().write_all(raw)?;
        return Ok(());
    }
    writer
        .get_mut()
        .write_all(&raw_with_root_word_binding(raw, namespace)?)?;
    Ok(())
}

pub(crate) fn raw_with_external_bindings(
    raw: &[u8],
    external_bindings: &[(String, String)],
) -> Result<Vec<u8>> {
    if external_bindings.is_empty() {
        return Ok(raw.to_vec());
    }

    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(false);
    let mut scopes = Vec::<Vec<(String, String)>>::new();
    let mut required = Vec::<String>::new();
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let declarations = raw_namespace_declarations(&element)?;
                record_external_bindings_used(
                    &element,
                    &scopes,
                    &declarations,
                    external_bindings,
                    &mut required,
                );
                scopes.push(declarations);
            }
            Event::Empty(element) => {
                let declarations = raw_namespace_declarations(&element)?;
                record_external_bindings_used(
                    &element,
                    &scopes,
                    &declarations,
                    external_bindings,
                    &mut required,
                );
            }
            Event::End(_) => {
                scopes.pop();
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }

    if required.is_empty() {
        return Ok(raw.to_vec());
    }

    let bindings = external_bindings
        .iter()
        .filter(|(prefix, _)| required.contains(prefix))
        .cloned()
        .collect::<Vec<_>>();
    raw_with_root_bindings(raw, &bindings)
}

fn raw_namespace_declarations(element: &BytesStart<'_>) -> Result<Vec<(String, String)>> {
    let mut declarations = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute?;
        let name = attribute.key.as_ref();
        let prefix = if name == b"xmlns" {
            ""
        } else if let Some(prefix) = name.strip_prefix(b"xmlns:") {
            std::str::from_utf8(prefix)?
        } else {
            continue;
        };
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, element.decoder())?
            .into_owned();
        declarations.push((prefix.to_owned(), value));
    }
    Ok(declarations)
}

fn record_external_bindings_used(
    element: &BytesStart<'_>,
    scopes: &[Vec<(String, String)>],
    declarations: &[(String, String)],
    external_bindings: &[(String, String)],
    required: &mut Vec<String>,
) {
    let element_name = element.name();
    let element_prefix = qualified_name_prefix(element_name.as_ref()).unwrap_or("");
    record_external_prefix_used(
        element_prefix,
        scopes,
        declarations,
        external_bindings,
        required,
    );
    for attribute in element.attributes().filter_map(|attribute| attribute.ok()) {
        let name = attribute.key.as_ref();
        if name == b"xmlns" || name.starts_with(b"xmlns:") {
            continue;
        }
        if let Some(prefix) = qualified_name_prefix(name) {
            record_external_prefix_used(prefix, scopes, declarations, external_bindings, required);
        }
    }
}

fn record_external_prefix_used(
    prefix: &str,
    scopes: &[Vec<(String, String)>],
    declarations: &[(String, String)],
    external_bindings: &[(String, String)],
    required: &mut Vec<String>,
) {
    let internally_bound = declarations
        .iter()
        .rev()
        .any(|(candidate, _)| candidate == prefix)
        || scopes
            .iter()
            .rev()
            .any(|scope| scope.iter().rev().any(|(candidate, _)| candidate == prefix));
    if !internally_bound
        && external_bindings
            .iter()
            .any(|(candidate, _)| candidate == prefix)
        && !required.iter().any(|candidate| candidate == prefix)
    {
        required.push(prefix.to_owned());
    }
}

fn qualified_name_prefix(name: &[u8]) -> Option<&str> {
    let separator = name.iter().position(|byte| *byte == b':')?;
    std::str::from_utf8(&name[..separator]).ok()
}

fn raw_with_root_bindings(raw: &[u8], bindings: &[(String, String)]) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    loop {
        let start = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let end = reader.buffer_position() as usize;
                let mut element = element.into_owned();
                let names = bindings
                    .iter()
                    .map(|(prefix, _)| {
                        if prefix.is_empty() {
                            "xmlns".to_owned()
                        } else {
                            format!("xmlns:{prefix}")
                        }
                    })
                    .collect::<Vec<_>>();
                for ((_, namespace), name) in bindings.iter().zip(&names) {
                    element.push_attribute((name.as_str(), namespace.as_str()));
                }
                let mut output = raw[..start].to_vec();
                Writer::new(&mut output).write_event(Event::Start(element))?;
                output.extend_from_slice(&raw[end..]);
                return Ok(output);
            }
            Event::Empty(element) => {
                let end = reader.buffer_position() as usize;
                let mut element = element.into_owned();
                let names = bindings
                    .iter()
                    .map(|(prefix, _)| {
                        if prefix.is_empty() {
                            "xmlns".to_owned()
                        } else {
                            format!("xmlns:{prefix}")
                        }
                    })
                    .collect::<Vec<_>>();
                for ((_, namespace), name) in bindings.iter().zip(&names) {
                    element.push_attribute((name.as_str(), namespace.as_str()));
                }
                let mut output = raw[..start].to_vec();
                Writer::new(&mut output).write_event(Event::Empty(element))?;
                output.extend_from_slice(&raw[end..]);
                return Ok(output);
            }
            Event::Eof => return Ok(raw.to_vec()),
            _ => {}
        }
        buffer.clear();
    }
}

fn raw_uses_external_word_binding(raw: &[u8]) -> bool {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(false);
    let mut external_binding = vec![true];
    let mut saw_root = false;
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(element)) => {
                let (declares_word, uses_word) = raw_word_binding_use(&element);
                if !saw_root {
                    saw_root = true;
                    if declares_word {
                        return false;
                    }
                }
                let uses_external =
                    external_binding.last().copied().unwrap_or(true) && !declares_word;
                if uses_external && uses_word {
                    return true;
                }
                external_binding.push(uses_external);
            }
            Ok(Event::Empty(element)) => {
                let (declares_word, uses_word) = raw_word_binding_use(&element);
                if !saw_root {
                    saw_root = true;
                    if declares_word {
                        return false;
                    }
                }
                let uses_external =
                    external_binding.last().copied().unwrap_or(true) && !declares_word;
                if uses_external && uses_word {
                    return true;
                }
            }
            Ok(Event::End(_)) => {
                if external_binding.len() > 1 {
                    external_binding.pop();
                }
            }
            Ok(Event::Eof) => return false,
            Err(_) => return false,
            _ => {}
        }
        buffer.clear();
    }
}

fn raw_word_binding_use(element: &BytesStart<'_>) -> (bool, bool) {
    let declares_word = element
        .attributes()
        .filter_map(|attribute| attribute.ok())
        .any(|attribute| attribute.key.as_ref() == b"xmlns:w");
    let uses_word = qualified_name_uses_prefix(element.name().as_ref(), b"w")
        || element
            .attributes()
            .filter_map(|attribute| attribute.ok())
            .any(|attribute| qualified_name_uses_prefix(attribute.key.as_ref(), b"w"));
    (declares_word, uses_word)
}

fn qualified_name_uses_prefix(name: &[u8], prefix: &[u8]) -> bool {
    name.iter()
        .position(|byte| *byte == b':')
        .is_some_and(|separator| &name[..separator] == prefix)
}

fn raw_with_root_word_binding(raw: &[u8], namespace: &str) -> Result<Vec<u8>> {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    loop {
        let start = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let end = reader.buffer_position() as usize;
                let mut element = element.into_owned();
                element.push_attribute(("xmlns:w", namespace));
                let mut output = raw[..start].to_vec();
                Writer::new(&mut output).write_event(Event::Start(element))?;
                output.extend_from_slice(&raw[end..]);
                return Ok(output);
            }
            Event::Empty(element) => {
                let end = reader.buffer_position() as usize;
                let mut element = element.into_owned();
                element.push_attribute(("xmlns:w", namespace));
                let mut output = raw[..start].to_vec();
                Writer::new(&mut output).write_event(Event::Empty(element))?;
                output.extend_from_slice(&raw[end..]);
                return Ok(output);
            }
            Event::Eof => return Ok(raw.to_vec()),
            _ => {}
        }
        buffer.clear();
    }
}

fn write_empty_hyperlinks<W: std::io::Write>(
    writer: &mut Writer<W>,
    hyperlinks: &[HyperlinkSpan],
    revisions: &[(usize, usize, CT_Revision)],
    run_index: usize,
) -> Result<()> {
    for (hyperlink_index, hyperlink) in hyperlinks.iter().enumerate().filter(|(_, hyperlink)| {
        hyperlink.run_start == run_index
            && hyperlink.run_end == run_index
            && hyperlink.preserved_raw_before.is_none()
    }) {
        write_hyperlink_start(writer, hyperlink)?;
        write_hyperlink_boundary(writer, revisions, hyperlink_index, hyperlink, run_index)?;
        writer.write_event(Event::End(BytesEnd::new(hyperlink_qname(hyperlink))))?;
    }
    Ok(())
}

fn required_word_i32_attribute(
    element: &BytesStart<'_>,
    local: &[u8],
    word_prefixes: &[String],
) -> Result<i32> {
    for attribute in element.attributes() {
        let attribute = attribute?;
        let key = attribute.key.as_ref();
        let Some(separator) = key.iter().position(|byte| *byte == b':') else {
            continue;
        };
        if key.get(separator + 1..) == Some(local)
            && word_prefixes
                .iter()
                .any(|prefix| prefix.as_bytes() == &key[..separator])
        {
            return Ok(std::str::from_utf8(&attribute.value)?.parse()?);
        }
    }
    Err(OxmlError::MissingElement(format!(
        "{} attribute",
        String::from_utf8_lossy(local)
    )))
}

fn optional_word_attribute(
    element: &BytesStart<'_>,
    local: &[u8],
    word_prefixes: &[String],
) -> Option<String> {
    for attribute in element.attributes().flatten() {
        let key = attribute.key.as_ref();
        let Some(separator) = key.iter().position(|byte| *byte == b':') else {
            continue;
        };
        if key.get(separator + 1..) == Some(local)
            && word_prefixes
                .iter()
                .any(|prefix| prefix.as_bytes() == &key[..separator])
        {
            return std::str::from_utf8(&attribute.value)
                .ok()
                .map(str::to_owned);
        }
    }
    None
}

impl FieldInstruction {
    fn parse(instruction: &str) -> Self {
        let original = instruction.to_owned();
        let tokens = field_instruction_tokens(instruction);
        let name = tokens
            .first()
            .map(|token| token.to_ascii_uppercase())
            .unwrap_or_default();
        let mut arguments = Vec::new();
        let mut switches = Vec::new();
        let mut index = 1;

        while let Some(token) = tokens.get(index) {
            if let Some(name) = token.strip_prefix('\\') {
                index += 1;
                let argument = tokens
                    .get(index)
                    .filter(|next| !next.starts_with('\\'))
                    .cloned();
                if argument.is_some() {
                    index += 1;
                }
                switches.push(FieldSwitch {
                    name: name.to_owned(),
                    argument,
                });
            } else {
                arguments.push(token.clone());
                index += 1;
            }
        }

        Self {
            original,
            name,
            arguments,
            switches,
        }
    }
}

fn field_instruction_tokens(instruction: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quoted = false;

    for character in instruction.chars() {
        match character {
            '"' => quoted = !quoted,
            character if character.is_whitespace() && !quoted => {
                if !token.is_empty() {
                    tokens.push(std::mem::take(&mut token));
                }
            }
            _ => token.push(character),
        }
    }
    if !token.is_empty() {
        tokens.push(token);
    }
    tokens
}

/// Parse a field instruction string into the existing renderer field type.
fn parse_field_instruction(instr: &str) -> FieldType {
    let parsed = FieldInstruction::parse(instr.trim());
    let bookmark = parsed.arguments.first().cloned().unwrap_or_default();
    match parsed.name.as_str() {
        "PAGE" => FieldType::Page,
        "NUMPAGES" => FieldType::NumPages,
        "REF" if !bookmark.is_empty() => FieldType::Ref {
            bookmark,
            instruction: parsed.original,
        },
        "PAGEREF" if !bookmark.is_empty() => FieldType::PageRef {
            bookmark,
            instruction: parsed.original,
        },
        _ => FieldType::Other(parsed.original.trim().to_owned()),
    }
}

fn parse_complex_fields(runs: &[CT_R]) -> Vec<ComplexField> {
    let mut roots = Vec::new();
    let mut stack = Vec::<OpenComplexField>::new();

    for (run_index, run) in runs.iter().enumerate() {
        for marker in &run.field_markers {
            match marker {
                FieldMarker::Begin { dirty } => stack.push(OpenComplexField {
                    instruction: String::new(),
                    result_start: None,
                    dirty: *dirty,
                    valid: true,
                    children: Vec::new(),
                }),
                FieldMarker::Instruction(instruction) => {
                    if let Some(field) = stack.last_mut() {
                        if field.result_start.is_some() {
                            field.valid = false;
                        } else {
                            field.instruction.push_str(instruction);
                        }
                    }
                }
                FieldMarker::Separate { dirty } => {
                    if let Some(field) = stack.last_mut() {
                        if field.result_start.replace(run_index + 1).is_some() {
                            field.valid = false;
                        }
                        field.dirty |= *dirty;
                    }
                }
                FieldMarker::End { dirty } => {
                    let Some(mut field) = stack.pop() else {
                        continue;
                    };
                    field.dirty |= *dirty;
                    let parsed = field.result_start.and_then(|result_start| {
                        let instruction = FieldInstruction::parse(&field.instruction);
                        (!instruction.name.is_empty() && field.valid).then(|| ComplexField {
                            instruction,
                            run_start: result_start,
                            run_end: run_index + 1,
                            cached_text: runs[result_start..run_index + 1]
                                .iter()
                                .map(CT_R::text)
                                .collect(),
                            dirty: field.dirty,
                            children: field.children,
                        })
                    });
                    if let Some(parent) = stack.last_mut() {
                        if let Some(parsed) = parsed {
                            parent.children.push(parsed);
                        } else {
                            parent.valid = false;
                        }
                    } else if let Some(parsed) = parsed {
                        roots.push(parsed);
                    }
                }
            }
        }
    }
    roots
}

fn field_marker_from_element(
    element: &BytesStart<'_>,
    word_prefixes: &[String],
) -> Result<Option<FieldMarker>> {
    if is_word_element(element.name().as_ref(), b"instrText", word_prefixes) {
        return Ok(Some(FieldMarker::Instruction(String::new())));
    }
    if !is_word_element(element.name().as_ref(), b"fldChar", word_prefixes) {
        return Ok(None);
    }

    let dirty = optional_word_attribute(element, b"dirty", word_prefixes)
        .is_some_and(|value| matches!(value.as_str(), "true" | "1" | "on"));
    Ok(
        match optional_word_attribute(element, b"fldCharType", word_prefixes).as_deref() {
            Some("begin") => Some(FieldMarker::Begin { dirty }),
            Some("separate") => Some(FieldMarker::Separate { dirty }),
            Some("end") => Some(FieldMarker::End { dirty }),
            _ => None,
        },
    )
}

fn field_marker_with_instruction(marker: FieldMarker, raw: &[u8]) -> Result<FieldMarker> {
    let FieldMarker::Instruction(_) = marker else {
        return Ok(marker);
    };
    let mut reader = Reader::from_reader(raw);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) => {
                let instruction = reader
                    .read_text(element.name())
                    .map(|text| crate::xml_text::decode_escaped(&text))?;
                return Ok(FieldMarker::Instruction(instruction));
            }
            Event::Empty(_) | Event::Eof => return Ok(FieldMarker::Instruction(String::new())),
            _ => {}
        }
        buffer.clear();
    }
}

impl Default for CT_P {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_paragraph(xml: &str) -> CT_P {
        let full = format!("<w:p>{xml}</w:p>");
        let mut reader = Reader::from_str(&full);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"p") => break,
                _ => {}
            }
            buf.clear();
        }
        CT_P::from_xml(&mut reader).unwrap()
    }

    #[test]
    fn parse_simple_paragraph() {
        let p = parse_paragraph(r#"<w:r><w:t>Hello World</w:t></w:r>"#);
        assert_eq!(p.text(), "Hello World");
        assert_eq!(p.runs.len(), 1);
    }

    #[test]
    fn parse_paragraph_rejects_undecodable_visible_text() {
        let xml = b"<w:p><w:r><w:t>before\xff</w:t></w:r></w:p>";
        let mut reader = Reader::from_reader(xml.as_slice());
        let mut buffer = Vec::new();
        assert!(matches!(
            reader.read_event_into(&mut buffer),
            Ok(Event::Start(_))
        ));
        assert!(CT_P::from_xml(&mut reader).is_err());
    }

    #[test]
    fn complex_hyperlink_field_exposes_its_target_and_cached_text() {
        let p = parse_paragraph(concat!(
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
            r#"<w:r><w:instrText xml:space="preserve"> HYPERLINK &quot;https://example.test/path&quot; </w:instrText></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
            r#"<w:r><w:t>Example link</w:t></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        ));

        assert_eq!(p.text(), "Example link");
        assert_eq!(
            p.complex_field_hyperlinks(),
            vec![ComplexFieldHyperlink {
                run_start: 3,
                run_end: 5,
                target: "https://example.test/path".to_owned(),
            }]
        );
    }

    #[test]
    fn form_text_field_imports_its_cached_plain_text() {
        let p = parse_paragraph(concat!(
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
            r#"<w:r><w:instrText> FORMTEXT </w:instrText></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
            r#"<w:r><w:t>Entered value</w:t></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        ));

        assert_eq!(p.text(), "Entered value");
        assert!(p.complex_field_hyperlinks().is_empty());
    }

    #[test]
    fn complex_fields_parse_nested_operands_and_split_instructions() {
        let p = parse_paragraph(concat!(
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
            r#"<w:r><w:instrText> HYPER</w:instrText></w:r>"#,
            r#"<w:r><w:instrText>LINK </w:instrText></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
            r#"<w:r><w:instrText> REF destination </w:instrText></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
            r#"<w:r><w:t>https://example.test</w:t></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
            r#"<w:r><w:t>Example link</w:t></w:r>"#,
            r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
        ));

        assert_eq!(p.text(), "https://example.testExample link");
        let field = &p.complex_fields()[0];
        assert_eq!(field.instruction.name, "HYPERLINK");
        assert_eq!(field.children[0].instruction.name, "REF");
    }

    #[test]
    fn simple_and_complex_fields_share_the_instruction_tokenizer() {
        let instruction = r#" REF "destination name" \h \* MERGEFORMAT "#;
        let simple = parse_field_instruction(instruction);
        let parsed = FieldInstruction::parse(instruction);

        assert!(matches!(simple, FieldType::Ref { .. }));
        assert_eq!(parsed.arguments, vec!["destination name"]);
        assert_eq!(
            parsed.switches,
            vec![
                FieldSwitch {
                    name: "h".to_owned(),
                    argument: None,
                },
                FieldSwitch {
                    name: "*".to_owned(),
                    argument: Some("MERGEFORMAT".to_owned()),
                },
            ]
        );
    }

    #[test]
    fn complex_fields_accept_aliases_and_default_word_namespaces() {
        let word_namespace = crate::namespace::W_NS;
        let alias = parse_paragraph(&format!(
            r#"<q:r xmlns:q="{word_namespace}"><q:fldChar q:fldCharType="begin"/></q:r><q:r xmlns:q="{word_namespace}"><q:instrText> HYPERLINK &quot;https://alias.test&quot; </q:instrText></q:r><q:r xmlns:q="{word_namespace}"><q:fldChar q:fldCharType="separate"/></q:r><q:r xmlns:q="{word_namespace}"><q:t>Alias</q:t></q:r><q:r xmlns:q="{word_namespace}"><q:fldChar q:fldCharType="end"/></q:r>"#
        ));
        let default = parse_paragraph(&format!(
            r#"<r xmlns="{word_namespace}" xmlns:w="{word_namespace}"><fldChar w:fldCharType="begin"/></r><r xmlns="{word_namespace}"><instrText> HYPERLINK &quot;https://default.test&quot; </instrText></r><r xmlns="{word_namespace}" xmlns:w="{word_namespace}"><fldChar w:fldCharType="separate"/></r><r xmlns="{word_namespace}"><t>Default</t></r><r xmlns="{word_namespace}" xmlns:w="{word_namespace}"><fldChar w:fldCharType="end"/></r>"#
        ));

        assert_eq!(
            alias.complex_field_hyperlinks()[0].target,
            "https://alias.test"
        );
        assert_eq!(
            default.complex_field_hyperlinks()[0].target,
            "https://default.test"
        );
    }

    #[test]
    fn foreign_field_marker_collisions_remain_opaque() {
        let p = parse_paragraph(concat!(
            r#"<ext:r xmlns:ext="urn:producer"><ext:fldChar ext:fldCharType="begin"/></ext:r>"#,
            r#"<ext:r xmlns:ext="urn:producer"><ext:instrText> HYPERLINK https://foreign.test </ext:instrText></ext:r>"#,
        ));

        assert!(p.complex_fields().is_empty());
        let mut output = Vec::new();
        p.to_xml(&mut Writer::new(&mut output))
            .expect("paragraph writes");
        assert!(
            String::from_utf8(output)
                .expect("XML is UTF-8")
                .contains(r#"<ext:fldChar ext:fldCharType="begin"/>"#)
        );
    }

    #[test]
    fn unsupported_complex_fields_remain_raw_without_failing_document_open() {
        let xml = concat!(
            r#"<w:p><w:r><w:fldChar w:fldCharType="begin" w:dirty="true"/></w:r>"#,
            r#"<w:r><w:instrText> HYPERLINK &quot;https://example.test&quot; </w:instrText></w:r>"#,
            r#"</w:p>"#,
        );
        let mut reader = Reader::from_str(xml);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(element)) if matches_local_name(element.name().as_ref(), b"p") => {
                    let paragraph = CT_P::from_xml(&mut reader).expect("paragraph opens");
                    let mut output = Vec::new();
                    paragraph
                        .to_xml(&mut Writer::new(&mut output))
                        .expect("paragraph writes");
                    assert!(
                        String::from_utf8(output)
                            .expect("XML is UTF-8")
                            .contains(r#"<w:fldChar w:fldCharType="begin" w:dirty="true"/>"#)
                    );
                    break;
                }
                Ok(Event::Eof) => panic!("missing paragraph"),
                Ok(_) => {}
                Err(error) => panic!("invalid fixture: {error}"),
            }
            buffer.clear();
        }
    }

    #[test]
    fn unsafe_complex_fields_do_not_expose_hyperlinks() {
        let cases = [
            concat!(
                r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
                r#"<w:r><w:instrText> HYPERLINK &quot;https://example.test&quot; </w:instrText></w:r>"#,
            ),
            concat!(
                r#"<w:r><w:fldChar w:fldCharType="begin" w:dirty="true"/></w:r>"#,
                r#"<w:r><w:instrText> HYPERLINK &quot;https://example.test&quot; </w:instrText></w:r>"#,
                r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
                r#"<w:r><w:t>Cached</w:t></w:r>"#,
                r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            concat!(
                r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
                r#"<w:r><w:instrText> HYPERLINK </w:instrText></w:r>"#,
                r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
                r#"<w:r><w:t>Cached</w:t></w:r>"#,
                r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            concat!(
                r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
                r#"<w:r><w:instrText> HYPERLINK &quot;https://example.test&quot; </w:instrText></w:r>"#,
                r#"<w:r><w:fldChar w:fldCharType="separate" w:dirty="true"/></w:r>"#,
                r#"<w:r><w:t>Cached</w:t></w:r>"#,
                r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
            concat!(
                r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r>"#,
                r#"<w:r><w:instrText> HYPERLINK &quot;https://example.test&quot; </w:instrText></w:r>"#,
                r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r>"#,
                r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#,
            ),
        ];

        for case in cases {
            let xml = format!("<w:p>{case}</w:p>");
            let mut reader = Reader::from_str(&xml);
            let mut buffer = Vec::new();
            loop {
                match reader.read_event_into(&mut buffer) {
                    Ok(Event::Start(element))
                        if matches_local_name(element.name().as_ref(), b"p") =>
                    {
                        let paragraph = CT_P::from_xml(&mut reader).expect("paragraph opens");
                        assert!(paragraph.complex_field_hyperlinks().is_empty(), "{case}");
                        break;
                    }
                    Ok(Event::Eof) => panic!("missing paragraph"),
                    Ok(_) => {}
                    Err(error) => panic!("invalid fixture: {error}"),
                }
                buffer.clear();
            }
        }
    }

    #[test]
    fn parse_paragraph_with_properties() {
        let p = parse_paragraph(
            r#"<w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Centered</w:t></w:r>"#,
        );
        assert_eq!(p.text(), "Centered");
        assert!(p.properties.is_some());
        assert_eq!(
            p.properties.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn direct_paragraph_parser_accepts_explicit_property_binding() {
        let xml = format!(
            r#"<outer xmlns:ext="urn:producer"><q:p xmlns:q="{}"><ext:pPr><ext:jc ext:val="right"/></ext:pPr><q:pPr xmlns:q="{}"><ext:jc ext:val="right"/><q:jc q:val="center"/></q:pPr><q:r><q:t>Direct</q:t></q:r></q:p></outer>"#,
            crate::namespace::W_NS,
            crate::namespace::W_NS
        );
        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();
        let parsed = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) if element.local_name().as_ref() == b"p" => {
                    break CT_P::from_xml(&mut reader).unwrap();
                }
                Ok(Event::Eof) => panic!("missing paragraph"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        };
        assert_eq!(parsed.text(), "Direct");
        assert_eq!(
            parsed.properties.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn direct_paragraph_parser_does_not_invent_foreign_word_identity() {
        let xml = r#"<outer><ext:p xmlns:ext="urn:producer"><ext:pPr xmlns:ext="urn:producer"><ext:jc ext:val="right"/></ext:pPr><ext:r><ext:t>Foreign</ext:t></ext:r></ext:p></outer>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let parsed = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) if element.local_name().as_ref() == b"p" => {
                    break CT_P::from_xml(&mut reader).unwrap();
                }
                Ok(Event::Eof) => panic!("missing paragraph"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        };
        assert!(parsed.properties.is_none());
    }

    #[test]
    fn direct_paragraph_parser_accepts_default_word_namespace() {
        let xml = format!(
            r#"<outer xmlns:ext="urn:producer"><p xmlns="{0}" xmlns:w="{0}"><ext:pPr><ext:jc ext:val="right"/></ext:pPr><pPr xmlns="{0}" xmlns:w="{0}"><ext:jc ext:val="right"/><jc w:val="center"/></pPr><r><t>Direct</t></r></p></outer>"#,
            crate::namespace::W_NS
        );
        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();
        let parsed = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element)) if element.local_name().as_ref() == b"p" => {
                    break CT_P::from_xml(&mut reader).unwrap();
                }
                Ok(Event::Eof) => panic!("missing paragraph"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        };
        assert_eq!(parsed.text(), "Direct");
        assert_eq!(
            parsed.properties.as_ref().unwrap().jc,
            Some(crate::shared::ST_Jc::Center)
        );
    }

    #[test]
    fn parse_run_with_formatting() {
        let p = parse_paragraph(r#"<w:r><w:rPr><w:b/><w:i/></w:rPr><w:t>Bold Italic</w:t></w:r>"#);
        let run = &p.runs[0];
        let rpr = run.properties.as_ref().unwrap();
        assert_eq!(rpr.bold, Some(true));
        assert_eq!(rpr.italic, Some(true));
    }

    #[test]
    fn parse_multiple_runs() {
        let p = parse_paragraph(r#"<w:r><w:t>Hello </w:t></w:r><w:r><w:t>World</w:t></w:r>"#);
        assert_eq!(p.runs.len(), 2);
        assert_eq!(p.text(), "Hello World");
    }

    #[test]
    fn parse_hyperlink() {
        let p = parse_paragraph(
            r#"<w:hyperlink r:id="rId5"><w:r><w:t>Click here</w:t></w:r></w:hyperlink>"#,
        );
        assert_eq!(p.runs.len(), 1);
        assert_eq!(p.text(), "Click here");
        assert_eq!(p.hyperlinks.len(), 1);
        assert_eq!(p.hyperlinks[0].rel_id, Some("rId5".to_string()));
        assert_eq!(p.hyperlinks[0].run_start, 0);
        assert_eq!(p.hyperlinks[0].run_end, 1);
    }

    #[test]
    fn parse_hyperlink_with_anchor() {
        let p = parse_paragraph(
            r#"<w:hyperlink w:anchor="section1" w:tooltip="Jump" w:docLocation="target"><w:r><w:t>Go to section</w:t></w:r></w:hyperlink>"#,
        );
        assert_eq!(p.hyperlinks.len(), 1);
        assert_eq!(p.hyperlinks[0].anchor, Some("section1".to_string()));
        assert_eq!(p.hyperlinks[0].tooltip.as_deref(), Some("Jump"));
        assert_eq!(p.hyperlinks[0].doc_location.as_deref(), Some("target"));
        assert!(p.hyperlinks[0].rel_id.is_none());
    }

    #[test]
    fn parse_hyperlink_multiple_runs() {
        let p = parse_paragraph(
            r#"<w:r><w:t>Before </w:t></w:r><w:hyperlink r:id="rId6"><w:r><w:t>link </w:t></w:r><w:r><w:rPr><w:b/></w:rPr><w:t>text</w:t></w:r></w:hyperlink><w:r><w:t> after</w:t></w:r>"#,
        );
        assert_eq!(p.runs.len(), 4);
        assert_eq!(p.text(), "Before link text after");
        assert_eq!(p.hyperlinks.len(), 1);
        assert_eq!(p.hyperlinks[0].run_start, 1);
        assert_eq!(p.hyperlinks[0].run_end, 3);
    }

    #[test]
    fn round_trip_hyperlink() {
        let mut p = CT_P::new();
        p.add_run("Before ");
        p.add_run("link text");
        p.add_run(" after");
        p.hyperlinks.push(HyperlinkSpan {
            rel_id: Some("rId7".to_string()),
            anchor: None,
            tooltip: None,
            doc_location: None,
            run_start: 1,
            run_end: 2,
            extra_attributes: Vec::new(),
            extra_xml: Vec::new(),
            preserved_raw_before: None,
        });

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        p.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let parsed = parse_paragraph(
            xml.strip_prefix("<w:p>")
                .unwrap()
                .strip_suffix("</w:p>")
                .unwrap(),
        );
        assert_eq!(parsed.text(), "Before link text after");
        assert_eq!(parsed.hyperlinks.len(), 1);
        assert_eq!(parsed.hyperlinks[0].rel_id, Some("rId7".to_string()));
        assert_eq!(parsed.hyperlinks[0].run_start, 1);
        assert_eq!(parsed.hyperlinks[0].run_end, 2);
    }

    #[test]
    fn comment_reference_is_typed_and_round_trips() {
        let p = parse_paragraph(
            r#"<w:r><w:t>flagged</w:t></w:r><w:r><w:commentReference w:id="1"/></w:r>"#,
        );
        assert_eq!(p.runs.len(), 2);
        assert!(matches!(
            p.runs[1].content.as_slice(),
            [RunContent::CommentReference { id: 1, .. }]
        ));

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        p.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();
        assert!(
            xml.contains(r#"<w:commentReference w:id="1"/>"#),
            "comment reference must survive round-trip: {xml}"
        );
        assert!(xml.contains("flagged"));
    }

    #[test]
    fn alternate_content_is_preserved_once_and_parsed_for_layout() {
        // A shape as Word writes it: DrawingML in mc:Choice, VML in the
        // fallback. The block has to come back out verbatim, exactly once,
        // while still being visible to layout.
        let src = concat!(
            r#"<w:r><mc:AlternateContent><mc:Choice Requires="wps">"#,
            r#"<w:drawing><wp:anchor behindDoc="0">"#,
            r#"<wp:positionH relativeFrom="column"><wp:posOffset>914400</wp:posOffset></wp:positionH>"#,
            r#"<wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV>"#,
            r#"<wp:extent cx="914400" cy="457200"/>"#,
            r#"<a:graphic><a:graphicData><wps:wsp><wps:spPr>"#,
            r#"<a:prstGeom prst="rect"/><a:solidFill><a:srgbClr val="729FCF"/></a:solidFill>"#,
            r#"<a:ln><a:solidFill><a:srgbClr val="000000"/></a:solidFill></a:ln>"#,
            r#"</wps:spPr><wps:txbx><w:txbxContent>"#,
            r#"<w:p><w:r><w:t>boxed</w:t></w:r></w:p>"#,
            r#"</w:txbxContent></wps:txbx></wps:wsp></a:graphicData></a:graphic>"#,
            r#"</wp:anchor></w:drawing></mc:Choice>"#,
            r#"<mc:Fallback><w:pict><v:rect/></w:pict></mc:Fallback>"#,
            r#"</mc:AlternateContent></w:r>"#,
        );
        let p = parse_paragraph(src);
        assert_eq!(p.runs.len(), 1);
        let run = &p.runs[0];

        // Visible to layout.
        assert_eq!(
            run.alt_drawings.len(),
            1,
            "the drawing must reach the model"
        );
        let anchor = run.alt_drawings[0]
            .anchor
            .as_ref()
            .expect("should be an anchored drawing");
        let shape = anchor.shape.as_ref().expect("should carry shape content");
        assert_eq!(shape.preset.as_deref(), Some("rect"));
        assert_eq!(
            shape.solid_fill.as_deref(),
            Some("729FCF"),
            "the fill colour must win over the outline colour"
        );
        assert_eq!(shape.text.len(), 1);
        assert_eq!(shape.text[0].text(), "boxed");

        // Preserved verbatim, exactly once.
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        p.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();
        assert_eq!(
            xml.matches("<mc:AlternateContent").count(),
            1,
            "the block must not be duplicated: {xml}"
        );
        assert_eq!(
            xml.matches("<mc:Fallback").count(),
            1,
            "the VML fallback must survive"
        );
        assert!(xml.contains(r#"<a:prstGeom prst="rect"/>"#));
    }

    #[test]
    fn vml_only_alternate_content_is_preserved_without_a_layout_projection() {
        let p = parse_paragraph(concat!(
            r#"<w:r><mc:AlternateContent><mc:Choice Requires="vml">"#,
            r#"<w:pict><v:shape id="legacy"/></w:pict>"#,
            r#"</mc:Choice><mc:Fallback><w:pict><v:shape id="fallback"/></w:pict>"#,
            r#"</mc:Fallback></mc:AlternateContent></w:r>"#,
        ));

        assert!(p.runs[0].alt_drawings.is_empty());
        let mut output = Vec::new();
        p.to_xml(&mut Writer::new(&mut output))
            .expect("paragraph writes");
        let xml = String::from_utf8(output).expect("XML is UTF-8");
        assert_eq!(xml.matches("<mc:AlternateContent").count(), 1, "{xml}");
        assert!(xml.contains(r#"<v:shape id="legacy"/>"#), "{xml}");
    }

    #[test]
    fn empty_run_properties_are_not_moved_after_content() {
        // extra_xml is written after the run content, and CT_R requires
        // w:rPr first, so a self-closing <w:rPr/> must not be captured.
        // Capturing it would emit <w:t> before <w:rPr/> and break the schema.
        let p = parse_paragraph(r#"<w:r><w:rPr/><w:t>x</w:t></w:r>"#);
        assert_eq!(p.runs.len(), 1);
        assert!(
            p.runs[0].extra_xml.is_empty(),
            "an empty w:rPr must not be captured as extra_xml"
        );

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        p.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();
        assert!(
            !xml.contains(r#"<w:t>x</w:t><w:rPr/>"#),
            "w:rPr must never follow run content: {xml}"
        );
    }

    #[test]
    fn parse_fld_simple_page() {
        let p = parse_paragraph(
            r#"<w:fldSimple w:instr=" PAGE "><w:r><w:t>1</w:t></w:r></w:fldSimple>"#,
        );
        assert_eq!(p.runs.len(), 1);
        assert_eq!(p.runs[0].content.len(), 1);
        assert!(matches!(
            p.runs[0].content[0],
            RunContent::Field {
                field_type: FieldType::Page,
                ..
            }
        ));
    }

    #[test]
    fn parse_fld_simple_numpages() {
        let p = parse_paragraph(
            r#"<w:fldSimple w:instr=" NUMPAGES \* MERGEFORMAT "><w:r><w:t>5</w:t></w:r></w:fldSimple>"#,
        );
        assert_eq!(p.runs.len(), 1);
        assert!(matches!(
            p.runs[0].content[0],
            RunContent::Field {
                field_type: FieldType::NumPages,
                ..
            }
        ));
    }

    #[test]
    fn ref_and_pageref_instructions_keep_targets_and_switches() {
        let ref_field = parse_paragraph(
            r#"<w:fldSimple w:instr=" REF destination \h \* MERGEFORMAT "><w:r><w:t>cached text</w:t></w:r></w:fldSimple>"#,
        );
        let page_ref = parse_paragraph(
            r#"<w:fldSimple w:instr=" PAGEREF destination \p "><w:r><w:t>7</w:t></w:r></w:fldSimple>"#,
        );

        assert!(matches!(
            &ref_field.runs[0].content[0],
            RunContent::Field {
                field_type: FieldType::Ref { bookmark, instruction },
                display,
            } if bookmark == "destination"
                && instruction == r"REF destination \h \* MERGEFORMAT"
                && display == "cached text"
        ));
        assert!(matches!(
            &page_ref.runs[0].content[0],
            RunContent::Field {
                field_type: FieldType::PageRef { bookmark, instruction },
                display,
            } if bookmark == "destination"
                && instruction == r"PAGEREF destination \p"
                && display == "7"
        ));

        let mut output = Vec::new();
        ref_field.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains(r#"w:instr="REF destination \h \* MERGEFORMAT""#));
        assert!(output.contains("<w:t>cached text</w:t>"));
    }

    #[test]
    fn empty_cross_reference_displays_remain_empty() {
        let paragraph = parse_paragraph(
            r#"<w:fldSimple w:instr=" REF destination "><w:r><w:t></w:t></w:r></w:fldSimple><w:fldSimple w:instr=" PAGEREF destination "><w:r><w:t></w:t></w:r></w:fldSimple>"#,
        );

        let mut output = Vec::new();
        paragraph.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output.matches("<w:t></w:t>").count(), 2, "{output}");
        assert!(!output.contains("<w:t>1</w:t>"), "{output}");
    }

    #[test]
    fn field_parser_uses_expanded_names_and_accepts_word_aliases() {
        let word_namespace = crate::namespace::W_NS;
        let paragraph = parse_paragraph(&format!(
            r#"<x:fldSimple xmlns:x="urn:producer" x:instr="REF foreign"><x:r><x:t>foreign</x:t></x:r></x:fldSimple><q:fldSimple xmlns:q="{word_namespace}" q:instr="REF destination"><q:r><q:t>cached</q:t></q:r></q:fldSimple>"#,
        ));

        assert_eq!(paragraph.runs.len(), 1);
        assert!(matches!(
            &paragraph.runs[0].content[0],
            RunContent::Field {
                field_type: FieldType::Ref { bookmark, .. },
                display,
            } if bookmark == "destination" && display == "cached"
        ));
        let mut output = Vec::new();
        paragraph.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains("<x:fldSimple"), "{output}");
        assert!(output.contains("x:instr=\"REF foreign\""), "{output}");
        assert!(output.contains("<w:fldSimple"), "{output}");
    }

    #[test]
    fn bookmark_markers_keep_range_order_and_unmodelled_neighbours() {
        let p = parse_paragraph(
            r#"<w:customBefore/><w:bookmarkStart w:id="4" w:name="destination"/><w:r><w:t>inside</w:t></w:r><w:bookmarkEnd w:id="4"/><w:customAfter/>"#,
        );

        assert_eq!(p.bookmark_markers.len(), 2);
        assert!(p.bookmark_markers[0].is_start());
        assert_eq!(p.bookmark_markers[0].id(), Some(4));
        assert_eq!(p.bookmark_markers[0].name(), Some("destination"));
        assert_eq!(p.bookmark_markers[0].run_index(), 0);
        assert!(!p.bookmark_markers[1].is_start());
        assert_eq!(p.bookmark_markers[1].run_index(), 1);

        let mut output = Vec::new();
        p.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output = String::from_utf8(output).unwrap();
        let before = output.find("<w:customBefore/>").unwrap();
        let start = output.find("<w:bookmarkStart").unwrap();
        let run = output.find("<w:r>").unwrap();
        let end = output.find("<w:bookmarkEnd").unwrap();
        let after = output.find("<w:customAfter/>").unwrap();
        assert!(before < start && start < run && run < end && end < after);
    }

    #[test]
    fn parse_fld_simple_mixed_with_text() {
        let p = parse_paragraph(
            r#"<w:r><w:t>Page </w:t></w:r><w:fldSimple w:instr=" PAGE "><w:r><w:t>1</w:t></w:r></w:fldSimple><w:r><w:t> of </w:t></w:r><w:fldSimple w:instr=" NUMPAGES "><w:r><w:t>5</w:t></w:r></w:fldSimple>"#,
        );
        assert_eq!(p.runs.len(), 4);
        assert_eq!(p.text(), "Page  of ");
        assert!(matches!(
            p.runs[1].content[0],
            RunContent::Field {
                field_type: FieldType::Page,
                ..
            }
        ));
        assert!(matches!(
            p.runs[3].content[0],
            RunContent::Field {
                field_type: FieldType::NumPages,
                ..
            }
        ));
    }

    #[test]
    fn round_trip_fld_simple() {
        let mut p = CT_P::new();
        p.add_run("Page ");
        p.runs.push(CT_R {
            properties: None,
            content: vec![RunContent::Field {
                field_type: FieldType::Page,
                display: "1".to_owned(),
            }],
            simple_field: None,
            extra_xml: Vec::new(),
            extra_xml_positions: Vec::new(),
            field_markers: Vec::new(),
            alt_drawings: Vec::new(),
        });

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        p.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let parsed = parse_paragraph(
            xml.strip_prefix("<w:p>")
                .unwrap()
                .strip_suffix("</w:p>")
                .unwrap(),
        );
        assert_eq!(parsed.runs.len(), 2);
        assert!(matches!(
            parsed.runs[1].content[0],
            RunContent::Field {
                field_type: FieldType::Page,
                ..
            }
        ));
    }

    #[test]
    fn round_trip_paragraph() {
        let mut p = CT_P::new();
        p.add_run("Hello ");
        let run = p.add_run("World");
        run.properties = Some(CT_RPr {
            bold: Some(true),
            ..Default::default()
        });

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        p.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let parsed = parse_paragraph(
            xml.strip_prefix("<w:p>")
                .unwrap()
                .strip_suffix("</w:p>")
                .unwrap(),
        );
        assert_eq!(parsed.text(), "Hello World");
        assert_eq!(parsed.runs.len(), 2);
        assert_eq!(parsed.runs[1].properties.as_ref().unwrap().bold, Some(true));
    }

    #[test]
    fn parse_footnote_reference() {
        let p = parse_paragraph(
            r#"<w:r><w:t>Some text</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r>"#,
        );
        assert_eq!(p.runs.len(), 2);
        assert_eq!(p.runs[0].text(), "Some text");
        assert_eq!(p.runs[1].content.len(), 1);
        assert!(matches!(
            p.runs[1].content[0],
            RunContent::FootnoteRef { id: 1 }
        ));
    }

    #[test]
    fn parse_endnote_reference() {
        let p = parse_paragraph(r#"<w:r><w:endnoteReference w:id="3"/></w:r>"#);
        assert_eq!(p.runs.len(), 1);
        assert!(matches!(
            p.runs[0].content[0],
            RunContent::EndnoteRef { id: 3 }
        ));
    }

    #[test]
    fn round_trip_footnote_reference() {
        let mut p = CT_P::new();
        p.add_run("Text before");
        p.runs.push(CT_R {
            properties: None,
            content: vec![RunContent::FootnoteRef { id: 2 }],
            simple_field: None,
            extra_xml: Vec::new(),
            extra_xml_positions: Vec::new(),
            field_markers: Vec::new(),
            alt_drawings: Vec::new(),
        });
        p.add_run(" text after");

        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output);
        p.to_xml(&mut writer).unwrap();
        let xml = String::from_utf8(output).unwrap();

        let parsed = parse_paragraph(
            xml.strip_prefix("<w:p>")
                .unwrap()
                .strip_suffix("</w:p>")
                .unwrap(),
        );
        assert_eq!(parsed.runs.len(), 3);
        assert!(matches!(
            parsed.runs[1].content[0],
            RunContent::FootnoteRef { id: 2 }
        ));
    }

    #[test]
    fn comment_anchors_are_typed_without_moving_neighbouring_xml() {
        let word_namespace = crate::namespace::W_NS;
        let p = parse_paragraph(&format!(
            r#"<w:r><w:t>before</w:t></w:r><ext:marker xmlns:ext="urn:producer"/><q:commentRangeStart xmlns:q="{word_namespace}" q:id="7"/><w:r><w:t>inside</w:t></w:r><q:commentRangeEnd xmlns:q="{word_namespace}" q:id="7"/><w:r><ext:runBefore xmlns:ext="urn:producer"/><q:commentReference xmlns:q="{word_namespace}" q:id="7"/><ext:runAfter xmlns:ext="urn:producer"/></w:r><ext:tail xmlns:ext="urn:producer"/>"#
        ));

        assert_eq!(p.comment_ranges.len(), 2);
        assert!(matches!(
            p.comment_ranges[0],
            CommentRangeMarker::Start {
                id: 7,
                run_index: 1,
                ..
            }
        ));
        assert!(matches!(
            p.comment_ranges[1],
            CommentRangeMarker::End {
                id: 7,
                run_index: 2,
                ..
            }
        ));
        assert!(matches!(
            p.runs[2].content[0],
            RunContent::CommentReference { id: 7, .. }
        ));

        let mut output = Vec::new();
        p.to_xml(&mut Writer::new(&mut output)).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert!(output.contains(
            r#"<ext:marker xmlns:ext="urn:producer"/><w:commentRangeStart w:id="7"/><w:r><w:t>inside</w:t></w:r><w:commentRangeEnd w:id="7"/><w:r><ext:runBefore xmlns:ext="urn:producer"/><w:commentReference w:id="7"/><ext:runAfter xmlns:ext="urn:producer"/></w:r><ext:tail xmlns:ext="urn:producer"/>"#
        ));
    }

    #[test]
    fn malformed_comment_anchor_id_is_rejected() {
        let full = format!(
            r#"<w:p xmlns:w="{}"><w:commentRangeStart w:id="not-a-number"/></w:p>"#,
            crate::namespace::W_NS
        );
        let mut reader = Reader::from_str(&full);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref element))
                    if matches_local_name(element.name().as_ref(), b"p") =>
                {
                    break;
                }
                Ok(Event::Eof) => panic!("missing paragraph"),
                event => {
                    event.unwrap();
                }
            }
            buf.clear();
        }
        assert!(CT_P::from_xml(&mut reader).is_err());
    }
}

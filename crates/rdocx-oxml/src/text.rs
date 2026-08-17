//! Text content elements: `CT_P` (paragraph), `CT_R` (run), `CT_Text`.

use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::content_control::CT_Sdt;
use crate::drawing::CT_Drawing;
use crate::error::{OxmlError, Result};
use crate::namespace::matches_local_name;
use crate::numbering::{parse_scoped_ppr, word_prefixes_at};
use crate::properties::{CT_PPr, CT_RPr, is_word_element};
use crate::raw_xml::{capture_element, capture_empty_element};

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

/// Content that can appear inside a run.
#[derive(Debug, Clone, PartialEq)]
pub enum RunContent {
    Text(CT_Text),
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
}

impl BookmarkMarker {
    fn new(start: bool, id: i32, name: Option<String>, run_index: usize) -> Self {
        Self {
            start,
            id: Some(id),
            name,
            run_index,
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
    /// Unknown child elements captured as raw XML.
    pub extra_xml: Vec<Vec<u8>>,
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
            extra_xml: Vec::new(),
            alt_drawings: Vec::new(),
        }
    }

    /// Get the combined text of all text content in this run.
    pub fn text(&self) -> String {
        let mut result = String::new();
        for item in &self.content {
            match item {
                RunContent::Text(t) => result.push_str(&t.text),
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

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_prefixes(reader, &["w".to_owned()])
    }

    pub(crate) fn from_xml_with_prefixes(
        reader: &mut Reader<&[u8]>,
        word_prefixes: &[String],
    ) -> Result<Self> {
        let mut properties = None;
        let mut content = Vec::new();
        let mut extra_xml = Vec::new();
        let mut alt_drawings = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if matches_local_name(name.as_ref(), b"rPr") {
                        properties = Some(CT_RPr::from_xml(reader)?);
                    } else if matches_local_name(name.as_ref(), b"t") {
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
                        let text = reader
                            .read_text(name)
                            .map(|t| crate::xml_text::decode_escaped(&t))
                            .unwrap_or_default();
                        content.push(RunContent::Text(CT_Text {
                            text,
                            preserve_space: preserve,
                        }));
                    } else if matches_local_name(name.as_ref(), b"drawing") {
                        content.push(RunContent::Drawing(CT_Drawing::from_xml(reader)?));
                    } else if matches_local_name(name.as_ref(), b"commentReference")
                        && is_word_element(name.as_ref(), b"commentReference", &prefixes)
                    {
                        let id = required_word_i32_attribute(e, b"id", &prefixes)?;
                        reader.read_to_end_into(name, &mut Vec::new())?;
                        content.push(RunContent::CommentReference {
                            id,
                            raw_before: extra_xml.len(),
                        });
                    } else if matches_local_name(name.as_ref(), b"AlternateContent") {
                        // Keep the block verbatim so the VML fallback survives
                        // a write, and separately read the DrawingML out of it
                        // so layout can see the shape. alt_drawings is never
                        // serialised, the raw copy below is what gets written.
                        let raw = capture_element(reader, e)?;
                        if let Some(drawing) = crate::drawing::parse_alternate_content(&raw) {
                            alt_drawings.push(drawing);
                        }
                        extra_xml.push(raw);
                    } else {
                        // Capture unknown child elements as raw XML
                        extra_xml.push(capture_element(reader, e)?);
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if matches_local_name(name.as_ref(), b"tab") {
                        content.push(RunContent::Tab);
                    } else if matches_local_name(name.as_ref(), b"br") {
                        let break_type = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .find(|a| matches_local_name(a.key.as_ref(), b"type"))
                            .map(|a| match a.value.as_ref() {
                                b"page" => BreakType::Page,
                                b"column" => BreakType::Column,
                                _ => BreakType::Line,
                            })
                            .unwrap_or(BreakType::Line);
                        content.push(RunContent::Break(break_type));
                    } else if matches_local_name(name.as_ref(), b"footnoteReference") {
                        let id = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .find(|a| matches_local_name(a.key.as_ref(), b"id"))
                            .and_then(|a| std::str::from_utf8(&a.value).ok()?.parse::<i32>().ok())
                            .unwrap_or(0);
                        content.push(RunContent::FootnoteRef { id });
                    } else if matches_local_name(name.as_ref(), b"endnoteReference") {
                        let id = e
                            .attributes()
                            .filter_map(|a| a.ok())
                            .find(|a| matches_local_name(a.key.as_ref(), b"id"))
                            .and_then(|a| std::str::from_utf8(&a.value).ok()?.parse::<i32>().ok())
                            .unwrap_or(0);
                        content.push(RunContent::EndnoteRef { id });
                    } else if is_word_element(name.as_ref(), b"commentReference", &prefixes) {
                        let id = required_word_i32_attribute(e, b"id", &prefixes)?;
                        content.push(RunContent::CommentReference {
                            id,
                            raw_before: extra_xml.len(),
                        });
                    } else if !matches_local_name(name.as_ref(), b"rPr") {
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
                        extra_xml.push(capture_empty_element(e)?);
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
            extra_xml,
            alt_drawings,
        })
    }

    pub fn to_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        writer.write_event(Event::Start(BytesStart::new("w:r")))?;

        if let Some(ref props) = self.properties {
            props.to_xml(writer)?;
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
                    for raw in self
                        .extra_xml
                        .iter()
                        .take((*raw_before).min(self.extra_xml.len()))
                        .skip(raw_written)
                    {
                        writer.get_mut().write_all(raw)?;
                        raw_written += 1;
                    }
                    let mut buf = itoa::Buffer::new();
                    let mut e = BytesStart::new("w:commentReference");
                    e.push_attribute(("w:id", buf.format(*id)));
                    writer.write_event(Event::Empty(e))?;
                }
            }
        }

        // Write captured unknown child elements
        for raw in self.extra_xml.iter().skip(raw_written) {
            writer.get_mut().write_all(raw)?;
        }

        writer.write_event(Event::End(BytesEnd::new("w:r")))?;
        Ok(())
    }
}

/// A hyperlink span that wraps a range of runs.
#[derive(Debug, Clone, PartialEq)]
pub struct HyperlinkSpan {
    /// The relationship ID for the hyperlink target.
    pub rel_id: Option<String>,
    /// Optional anchor within the document (for internal links).
    pub anchor: Option<String>,
    /// Index of the first run in the hyperlink (inclusive).
    pub run_start: usize,
    /// Index of the last run in the hyperlink (exclusive).
    pub run_end: usize,
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
        }
    }

    /// Get the combined text of all runs in this paragraph.
    pub fn text(&self) -> String {
        self.runs().iter().map(|run| run.text()).collect()
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
        self.extra_xml.push((run_index, raw));
        self.bookmark_markers.push(BookmarkMarker::new(
            true,
            id,
            Some(name.to_owned()),
            run_index,
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
        self.extra_xml.push((run_index, raw));
        self.bookmark_markers
            .push(BookmarkMarker::new(false, id, None, run_index));
        true
    }

    pub fn from_xml(reader: &mut Reader<&[u8]>) -> Result<Self> {
        Self::from_xml_with_prefixes(reader, &["w".to_string()])
    }

    pub(crate) fn from_xml_with_prefixes(
        reader: &mut Reader<&[u8]>,
        word_prefixes: &[String],
    ) -> Result<Self> {
        let mut properties = None;
        let mut runs = Vec::new();
        let mut hyperlinks = Vec::new();
        let mut comment_ranges = Vec::new();
        let mut bookmark_markers = Vec::new();
        let mut extra_xml = Vec::new();
        let mut content_controls = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let prefixes = word_prefixes_at(e, word_prefixes)?;
                    if is_word_element(name.as_ref(), b"pPr", &prefixes) {
                        let raw = capture_element(reader, e)?;
                        properties = Some(parse_scoped_ppr(&raw, &prefixes)?);
                    } else if matches_local_name(name.as_ref(), b"r") {
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
                    } else if matches_local_name(name.as_ref(), b"hyperlink") {
                        // Parse hyperlink: extract r:id and/or w:anchor, then parse child runs
                        let mut rel_id = None;
                        let mut anchor = None;
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if matches_local_name(key, b"id") {
                                rel_id = Some(
                                    std::str::from_utf8(&attr.value).unwrap_or("").to_string(),
                                );
                            } else if matches_local_name(key, b"anchor") {
                                anchor = Some(
                                    std::str::from_utf8(&attr.value).unwrap_or("").to_string(),
                                );
                            }
                        }

                        let run_start = runs.len();
                        // Parse child runs within the hyperlink
                        let mut inner_buf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut inner_buf) {
                                Ok(Event::Start(ref ie)) => {
                                    let iname = ie.name();
                                    if matches_local_name(iname.as_ref(), b"r") {
                                        let run_prefixes = word_prefixes_at(ie, &prefixes)?;
                                        runs.push(CT_R::from_xml_with_prefixes(
                                            reader,
                                            &run_prefixes,
                                        )?);
                                    } else {
                                        reader.read_to_end_into(iname, &mut Vec::new())?;
                                    }
                                }
                                Ok(Event::End(ref ie))
                                    if matches_local_name(ie.name().as_ref(), b"hyperlink") =>
                                {
                                    break;
                                }
                                Ok(Event::Eof) => break,
                                Err(e) => return Err(e.into()),
                                _ => {}
                            }
                            inner_buf.clear();
                        }

                        let run_end = runs.len();
                        if run_start < run_end && (rel_id.is_some() || anchor.is_some()) {
                            hyperlinks.push(HyperlinkSpan {
                                rel_id,
                                anchor,
                                run_start,
                                run_end,
                            });
                        }
                    } else if is_word_element(name.as_ref(), b"fldSimple", &prefixes) {
                        // Parse simple field: extract w:instr attribute
                        let instr =
                            optional_word_attribute(e, b"instr", &prefixes).unwrap_or_default();

                        let field_type = parse_field_instruction(&instr);

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
                            extra_xml: Vec::new(),
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
                        });
                        extra_xml.push((runs.len(), capture_element(reader, e)?));
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
                        });
                        extra_xml.push((runs.len(), capture_empty_element(e)?));
                    } else if !matches_local_name(name.as_ref(), b"p") {
                        extra_xml.push((runs.len(), capture_empty_element(e)?));
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
                writer.write_event(Event::End(BytesEnd::new("w:hyperlink")))?;
                current_hyperlink = None;
            }

            write_paragraph_boundary(
                writer,
                &self.extra_xml,
                &self.content_controls,
                &self.comment_ranges,
                run_idx,
            )?;

            // Open hyperlink if entering one
            if let Some(hl_idx) = in_hl
                && current_hyperlink != in_hl
            {
                let hl = &self.hyperlinks[hl_idx];
                let mut e = BytesStart::new("w:hyperlink");
                if let Some(ref rid) = hl.rel_id {
                    e.push_attribute(("r:id", rid.as_str()));
                }
                if let Some(ref anchor) = hl.anchor {
                    e.push_attribute(("w:anchor", anchor.as_str()));
                }
                writer.write_event(Event::Start(e))?;
                current_hyperlink = in_hl;
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

            run.to_xml(writer)?;
        }

        // Close any remaining open hyperlink
        if current_hyperlink.is_some() {
            writer.write_event(Event::End(BytesEnd::new("w:hyperlink")))?;
        }

        write_paragraph_boundary(
            writer,
            &self.extra_xml,
            &self.content_controls,
            &self.comment_ranges,
            self.runs.len(),
        )?;

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

fn write_paragraph_boundary<W: std::io::Write>(
    writer: &mut Writer<W>,
    extra_xml: &[(usize, Vec<u8>)],
    content_controls: &[(usize, usize, usize, CT_Sdt)],
    markers: &[CommentRangeMarker],
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
            writer.get_mut().write_all(raw)?;
        }
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

/// Parse a field instruction string into a FieldType.
fn parse_field_instruction(instr: &str) -> FieldType {
    let instruction = instr.trim();
    let trimmed = instruction.to_uppercase();
    // Field instruction may have switches like "PAGE \* MERGEFORMAT"
    let keyword = trimmed.split_whitespace().next().unwrap_or("");
    match keyword {
        "PAGE" => FieldType::Page,
        "NUMPAGES" => FieldType::NumPages,
        "REF" | "PAGEREF" => {
            let bookmark = instruction
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim_matches('"')
                .to_owned();
            if bookmark.is_empty() {
                FieldType::Other(instruction.to_owned())
            } else if keyword == "REF" {
                FieldType::Ref {
                    bookmark,
                    instruction: instruction.to_owned(),
                }
            } else {
                FieldType::PageRef {
                    bookmark,
                    instruction: instruction.to_owned(),
                }
            }
        }
        _ => FieldType::Other(instruction.to_owned()),
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
            r#"<w:hyperlink w:anchor="section1"><w:r><w:t>Go to section</w:t></w:r></w:hyperlink>"#,
        );
        assert_eq!(p.hyperlinks.len(), 1);
        assert_eq!(p.hyperlinks[0].anchor, Some("section1".to_string()));
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
            run_start: 1,
            run_end: 2,
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
            extra_xml: Vec::new(),
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
            extra_xml: Vec::new(),
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

//! Footnote and endnote content, laid out once before pagination.
//!
//! Notes used to be laid out inside the post-pagination pass that drew them,
//! which meant pagination could not know how much room they would need and
//! drew body text straight over them. Laying them out here, ahead of
//! pagination, lets the paginator reserve exactly the height it will later
//! draw. Reserve and render read the same lines, so they cannot disagree.
//!
//! The marker is shaped here too. The paginator only holds `&FontManager` and
//! shaping needs `&mut`, so a note that arrives pre-shaped is a note the
//! paginator can place without touching a font.

use std::collections::HashMap;

use rdocx_oxml::styles::CT_Styles;

use crate::engine::layout_paragraph;
use crate::input::{LayoutInput, MediaRegistry};
use crate::style_resolver::NumberingState;
use oxml_layout::{Color, FontManager, LayoutLine, NoteRef, NoteStream, Result, TextSegment};

/// Point size notes are set at.
pub const NOTE_FONT_SIZE: f64 = 8.0;
/// Horizontal space reserved for the marker, to the left of note text.
///
/// Notes are both line-broken and drawn against this, so the two agree.
pub const NOTE_INDENT: f64 = 12.0;
/// Vertical gap between the separator rule and the first note line.
pub const NOTE_SEPARATOR_OFFSET: f64 = 6.0;
/// Width of the rule above a note that starts on its own page, as a fraction
/// of the content width.
pub const SEPARATOR_WIDTH_FRACTION: f64 = 0.33;

/// One note, laid out and ready to place.
#[derive(Debug, Clone)]
pub struct NoteLayout {
    /// The pre-shaped superscript number drawn at the start of the note.
    pub marker: TextSegment,
    /// How far above the baseline the marker sits.
    pub marker_rise: f64,
    /// The note's lines, flattened across its paragraphs.
    pub lines: Vec<LayoutLine>,
}

impl NoteLayout {
    /// Height of a range of this note's lines.
    pub fn height_of(&self, first: usize, count: usize) -> f64 {
        self.lines
            .iter()
            .skip(first)
            .take(count)
            .map(|line| line.height)
            .sum()
    }

    /// Height of every line from `first` onward.
    pub fn height_from(&self, first: usize) -> f64 {
        self.height_of(first, self.lines.len())
    }

    /// Total height of every line.
    pub fn height(&self) -> f64 {
        self.height_from(0)
    }
}

/// Every note the document defines, laid out once.
#[derive(Debug, Clone, Default)]
pub struct NoteRegistry {
    notes: HashMap<NoteRef, NoteLayout>,
    continuation_separator: bool,
}

impl NoteRegistry {
    /// Lay out every note in the footnote and endnote streams.
    ///
    /// `content_width` is the page's content width. Notes are broken at
    /// `content_width - NOTE_INDENT`, because that is where they are drawn.
    pub fn build(
        input: &LayoutInput,
        styles: &CT_Styles,
        media: &MediaRegistry,
        fm: &mut FontManager,
        num_state: &mut NumberingState,
        content_width: f64,
    ) -> Result<Self> {
        let mut notes = HashMap::new();
        let mut continuation_separator = false;
        let note_width = (content_width - NOTE_INDENT).max(1.0);

        // Each stream is keyed separately, so a document numbering a footnote
        // and an endnote alike keeps both.
        for (kind, stream) in [
            (NoteStream::Footnote, input.footnotes.as_ref()),
            (NoteStream::Endnote, input.endnotes.as_ref()),
        ]
        .into_iter()
        .filter_map(|(kind, stream)| stream.map(|stream| (kind, stream)))
        {
            if stream.has_continuation_separator() {
                continuation_separator = true;
            }

            for note in &stream.footnotes {
                // `get_by_id` is the authority on what counts as a real note,
                // so separators never reach the registry.
                let key = NoteRef {
                    stream: kind,
                    id: note.id,
                };
                if stream.get_by_id(note.id).is_none() || notes.contains_key(&key) {
                    continue;
                }

                let mut lines = Vec::new();
                for paragraph in &note.paragraphs {
                    let block = layout_paragraph(
                        paragraph, note_width, styles, input, media, fm, num_state,
                    )?;
                    lines.extend(block.lines);
                }

                let Some(marker) = shape_marker(note.id, fm)? else {
                    continue;
                };

                notes.insert(
                    key,
                    NoteLayout {
                        marker,
                        marker_rise: NOTE_FONT_SIZE * 0.33,
                        lines,
                    },
                );
            }
        }

        Ok(NoteRegistry {
            notes,
            continuation_separator,
        })
    }

    pub fn get(&self, note: NoteRef) -> Option<&NoteLayout> {
        self.notes.get(&note)
    }

    /// Whether either stream defined the rule drawn above a carried note.
    pub fn has_continuation_separator(&self) -> bool {
        self.continuation_separator
    }

    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }
}

/// Shape a note's number as the superscript marker drawn beside it.
fn shape_marker(id: i32, fm: &mut FontManager) -> Result<Option<TextSegment>> {
    let text = id.to_string();
    let size = NOTE_FONT_SIZE * 0.58;

    let Ok(font_id) = fm.resolve_font(Some("serif"), false, false) else {
        return Ok(None);
    };
    let Ok(shaped) = fm.shape_text(font_id, &text, size) else {
        return Ok(None);
    };
    let metrics = fm.metrics(font_id, size)?;

    Ok(Some(TextSegment {
        text,
        font_id,
        font_size: size,
        glyph_ids: shaped.glyph_ids,
        advances: shaped.advances,
        width: shaped.width,
        ascent: metrics.ascent,
        descent: metrics.descent,
        line_gap: 0.0,
        color: Color::BLACK,
        bold: false,
        italic: false,
        underline: None,
        strike: false,
        dstrike: false,
        highlight: None,
        baseline_offset: 0.0,
        hyperlink_url: None,
        field_kind: None,
        note: None,
    }))
}

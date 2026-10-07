#![doc = include_str!("../README.md")]
#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

pub mod block;
mod convert;
pub mod engine;
pub mod input;
mod math;
pub mod notes;
pub mod paginator;
pub mod style_resolver;
pub mod table;

pub use input::{ImageData, LayoutInput, MediaRegistry, RevisionView};
use oxml_layout::Diagnostic;
pub use oxml_layout::{
    Color, DocumentMetadata, FontData, FontFile, FontId, GlyphRun, LayoutError, LayoutResult,
    PageFrame, Point, PositionedElement, Rect, Result, SourceNodeId, SourceSpan,
};
use rdocx_oxml::document::BodyContent;

/// Word story containing a source paragraph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WordStory {
    Document,
    Header {
        relationship_id: String,
    },
    Footer {
        relationship_id: String,
    },
    Footnote {
        id: i32,
    },
    Endnote {
        id: i32,
    },
    TextBox {
        part_name: String,
        owner_children: Vec<usize>,
    },
}

/// Modeled path to one paragraph in a Word story.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WordSourcePath {
    pub story: WordStory,
    pub children: Vec<usize>,
}

/// Point-space extent occupied by one top-level Word body item on one page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WordBodyLayoutFragment {
    /// One-based physical page number in the produced layout.
    pub physical_page: usize,
    /// One-based displayed page number after section page-number restarts.
    pub displayed_page: usize,
    /// Left edge in points from the physical page origin.
    pub x: f64,
    /// Top edge in points from the physical page origin.
    pub y: f64,
    /// Width in points.
    pub width: f64,
    /// Height in points.
    pub height: f64,
}

/// One physical page occupied by a section in this immutable layout snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordPageSection {
    pub physical_page: usize,
    pub displayed_page: usize,
    pub section_index: usize,
}

/// One occurrence of a field in this immutable layout snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordFieldPlacement {
    pub source: oxml_layout::FieldSource,
    /// Physical page on which this field display is painted.
    pub physical_page: usize,
    /// Word's semantic PAGE value. Note fields use their unique reference page.
    pub displayed_page: usize,
    /// Owning section, following the unique reference for note fields.
    pub section_index: usize,
}

/// Complete layout output plus its result-local Word source map.
#[derive(Debug)]
pub struct WordLayoutResult {
    pub layout: LayoutResult,
    pub revision_view: RevisionView,
    source_nodes: Vec<WordSourcePath>,
    body_fragments: Vec<Vec<WordBodyLayoutFragment>>,
    numbering_by_source: Vec<Option<style_resolver::ResolvedNumbering>>,
    page_reference_names: Vec<String>,
    page_sections: Vec<WordPageSection>,
    field_placements: Vec<WordFieldPlacement>,
    bookmark_pages: std::collections::HashMap<usize, WordPageSection>,
    field_source_xml: std::collections::HashMap<oxml_layout::FieldSource, Vec<u8>>,
    text_box_owner_indices: std::collections::HashMap<WordStory, usize>,
}

impl WordLayoutResult {
    /// Physical and displayed section placement from the same pagination pass.
    pub fn page_sections(&self) -> &[WordPageSection] {
        &self.page_sections
    }

    /// Exact parsed text-box field source used to bind a physical story paragraph.
    #[doc(hidden)]
    pub fn source_field_xml(&self, source: oxml_layout::FieldSource) -> Option<&[u8]> {
        self.field_source_xml.get(&source).map(Vec::as_slice)
    }

    /// Physical selected text-box owner identity in its actual OPC part.
    #[doc(hidden)]
    pub fn text_box_owner_index(&self, story: &WordStory) -> Option<usize> {
        self.text_box_owner_indices.get(story).copied()
    }

    /// Physical paragraph preorder within one selected text-box owner.
    #[doc(hidden)]
    pub fn text_box_paragraph_index(&self, node: SourceNodeId) -> Option<usize> {
        let index = node.get() as usize - 1;
        let path = self.source_node(node)?;
        if !matches!(path.story, WordStory::TextBox { .. }) {
            return None;
        }
        Some(
            self.source_nodes[..index]
                .iter()
                .filter(|other| other.story == path.story)
                .count(),
        )
    }

    /// Every placed field occurrence, including shared story repetitions.
    pub fn field_placements(&self) -> &[WordFieldPlacement] {
        &self.field_placements
    }

    /// Displayed page of a unique, placed bookmark target.
    pub fn bookmark_page(&self, name: &str) -> Option<usize> {
        self.bookmark_page_section(name)
            .map(|record| record.displayed_page)
    }

    /// Physical page, displayed number and owning section of a unique placed target.
    pub fn bookmark_page_section(&self, name: &str) -> Option<WordPageSection> {
        let index = self
            .page_reference_names
            .iter()
            .position(|value| value == name)?;
        self.bookmark_pages.get(&index).copied()
    }

    /// Return placed fragments for one direct body item by its source index.
    ///
    /// Modeled items return one fragment per occupied page. Preserved content
    /// that does not enter layout returns an empty slice. An out-of-range body
    /// index returns `None`.
    pub fn body_layout_fragments(&self, body_index: usize) -> Option<&[WordBodyLayoutFragment]> {
        self.body_fragments.get(body_index).map(Vec::as_slice)
    }

    /// Resolve a result-local source identity.
    pub fn source_node(&self, id: SourceNodeId) -> Option<&WordSourcePath> {
        self.source_nodes.get(id.get() as usize - 1)
    }

    /// Resolve the visible numbering marker assigned to a source paragraph.
    pub fn paragraph_numbering(
        &self,
        id: SourceNodeId,
    ) -> Option<&style_resolver::ResolvedNumbering> {
        self.numbering_by_source
            .get(id.get() as usize - 1)
            .and_then(Option::as_ref)
    }

    /// Resolve numbering by flattened main-story paragraph order.
    #[doc(hidden)]
    pub fn document_paragraph_numbering(
        &self,
        paragraph_index: usize,
    ) -> Option<&style_resolver::ResolvedNumbering> {
        let source_index = self
            .source_nodes
            .iter()
            .enumerate()
            .filter(|(_, path)| path.story == WordStory::Document)
            .nth(paragraph_index)
            .map(|(index, _)| index)?;
        self.numbering_by_source
            .get(source_index)
            .and_then(Option::as_ref)
    }

    /// Resolve numbering for a top-level body paragraph by its body index.
    #[doc(hidden)]
    pub fn document_body_paragraph_numbering(
        &self,
        body_index: usize,
    ) -> Option<&style_resolver::ResolvedNumbering> {
        let source_index = self.source_nodes.iter().position(|path| {
            path.story == WordStory::Document && path.children.as_slice() == [body_index]
        })?;
        self.numbering_by_source
            .get(source_index)
            .and_then(Option::as_ref)
    }

    /// Discard the Word source map and return the backend-neutral layout.
    pub fn into_layout_result(self) -> LayoutResult {
        self.layout
    }

    /// Resolve the bookmark name assigned to a result-local page target.
    #[doc(hidden)]
    pub fn page_reference_name(&self, target: usize) -> Option<&str> {
        self.page_reference_names.get(target).map(String::as_str)
    }
}

/// Lay out a complete DOCX document, producing positioned page frames.
pub fn layout_document(input: &LayoutInput) -> Result<LayoutResult> {
    engine::Engine::new().layout(input)
}

/// Lay out a complete DOCX and retain exact Word paragraph provenance.
pub fn layout_document_with_provenance(input: &LayoutInput) -> Result<WordLayoutResult> {
    let mut engine = engine::Engine::new();
    let (layout, source_nodes) = engine.layout_with_provenance(input)?;
    let body_fragments = engine.take_body_fragments();
    let (page_sections, field_placements, bookmark_pages) = engine.take_field_snapshot();
    let numbering_by_source = engine.numbering_by_source(source_nodes.len());
    Ok(WordLayoutResult {
        layout,
        revision_view: input.revision_view,
        source_nodes,
        body_fragments,
        page_sections,
        field_placements,
        bookmark_pages,
        field_source_xml: engine.take_field_source_xml(),
        text_box_owner_indices: engine.take_text_box_owner_indices(),
        numbering_by_source,
        page_reference_names: engine::page_reference_names(input),
    })
}

/// Lay out a DOCX with a reusable normal-font engine.
///
/// This hidden facade hook lets `rdocx::Document` retain expensive normal-font
/// work without exposing cache ownership as a second public abstraction.
#[doc(hidden)]
pub fn layout_document_with_reusable_engine(
    engine: &mut engine::Engine,
    input: &LayoutInput,
) -> Result<WordLayoutResult> {
    let (layout, source_nodes) = engine.layout_with_provenance(input)?;
    let body_fragments = engine.take_body_fragments();
    let (page_sections, field_placements, bookmark_pages) = engine.take_field_snapshot();
    let numbering_by_source = engine.numbering_by_source(source_nodes.len());
    Ok(WordLayoutResult {
        layout,
        revision_view: input.revision_view,
        source_nodes,
        body_fragments,
        page_sections,
        field_placements,
        bookmark_pages,
        field_source_xml: engine.take_field_source_xml(),
        text_box_owner_indices: engine.take_text_box_owner_indices(),
        numbering_by_source,
        page_reference_names: engine::page_reference_names(input),
    })
}

/// Lay out a DOCX using only caller-supplied and document-embedded fonts.
#[doc(hidden)]
pub fn layout_document_with_caller_fonts_and_provenance(
    input: &LayoutInput,
) -> Result<WordLayoutResult> {
    let mut engine = engine::Engine::new_with_caller_fonts();
    let (layout, source_nodes) = engine.layout_with_provenance(input)?;
    let body_fragments = engine.take_body_fragments();
    let (page_sections, field_placements, bookmark_pages) = engine.take_field_snapshot();
    let numbering_by_source = engine.numbering_by_source(source_nodes.len());
    Ok(WordLayoutResult {
        layout,
        revision_view: input.revision_view,
        source_nodes,
        body_fragments,
        page_sections,
        field_placements,
        bookmark_pages,
        field_source_xml: engine.take_field_source_xml(),
        text_box_owner_indices: engine.take_text_box_owner_indices(),
        numbering_by_source,
        page_reference_names: engine::page_reference_names(input),
    })
}

/// Lay out a DOCX using bundled fonts without system font discovery.
pub fn layout_document_deterministic(input: &LayoutInput) -> Result<LayoutResult> {
    engine::Engine::new_deterministic()?.layout(input)
}

/// Measure one supported Word block with the deterministic production engine.
///
/// This hidden facade hook keeps caller-width measurement on the same font,
/// paragraph, table, media, and numbering path as whole-document layout.
#[doc(hidden)]
pub fn measure_content_deterministic(
    input: &LayoutInput,
    content: &BodyContent,
    available_width: f64,
    related_story_scope: Option<&str>,
) -> Result<(f64, Vec<Diagnostic>)> {
    engine::Engine::new_deterministic()?.measure_content(
        input,
        content,
        available_width,
        related_story_scope,
    )
}

/// Lay out a DOCX deterministically and retain exact Word paragraph provenance.
pub fn layout_document_deterministic_with_provenance(
    input: &LayoutInput,
) -> Result<WordLayoutResult> {
    let mut engine = engine::Engine::new_deterministic()?;
    let (layout, source_nodes) = engine.layout_with_provenance(input)?;
    let body_fragments = engine.take_body_fragments();
    let (page_sections, field_placements, bookmark_pages) = engine.take_field_snapshot();
    let numbering_by_source = engine.numbering_by_source(source_nodes.len());
    Ok(WordLayoutResult {
        layout,
        revision_view: input.revision_view,
        source_nodes,
        body_fragments,
        page_sections,
        field_placements,
        bookmark_pages,
        field_source_xml: engine.take_field_source_xml(),
        text_box_owner_indices: engine.take_text_box_owner_indices(),
        numbering_by_source,
        page_reference_names: engine::page_reference_names(input),
    })
}

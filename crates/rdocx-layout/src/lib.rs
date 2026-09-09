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
pub use oxml_layout::{
    Color, DocumentMetadata, FontData, FontFile, FontId, GlyphRun, LayoutError, LayoutResult,
    PageFrame, Point, PositionedElement, Rect, Result, SourceNodeId, SourceSpan,
};

/// Word story containing a source paragraph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WordStory {
    Document,
    Header { relationship_id: String },
    Footer { relationship_id: String },
    Footnote { id: i32 },
    Endnote { id: i32 },
}

/// Modeled path to one paragraph in a Word story.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WordSourcePath {
    pub story: WordStory,
    pub children: Vec<usize>,
}

/// Complete layout output plus its result-local Word source map.
#[derive(Debug)]
pub struct WordLayoutResult {
    pub layout: LayoutResult,
    pub revision_view: RevisionView,
    source_nodes: Vec<WordSourcePath>,
    numbering_by_source: Vec<Option<style_resolver::ResolvedNumbering>>,
    page_reference_names: Vec<String>,
}

impl WordLayoutResult {
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
    let numbering_by_source = engine.numbering_by_source(source_nodes.len());
    Ok(WordLayoutResult {
        layout,
        revision_view: input.revision_view,
        source_nodes,
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
    let numbering_by_source = engine.numbering_by_source(source_nodes.len());
    Ok(WordLayoutResult {
        layout,
        revision_view: input.revision_view,
        source_nodes,
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
    let numbering_by_source = engine.numbering_by_source(source_nodes.len());
    Ok(WordLayoutResult {
        layout,
        revision_view: input.revision_view,
        source_nodes,
        numbering_by_source,
        page_reference_names: engine::page_reference_names(input),
    })
}

/// Lay out a DOCX using bundled fonts without system font discovery.
pub fn layout_document_deterministic(input: &LayoutInput) -> Result<LayoutResult> {
    engine::Engine::new_deterministic()?.layout(input)
}

/// Lay out a DOCX deterministically and retain exact Word paragraph provenance.
pub fn layout_document_deterministic_with_provenance(
    input: &LayoutInput,
) -> Result<WordLayoutResult> {
    let mut engine = engine::Engine::new_deterministic()?;
    let (layout, source_nodes) = engine.layout_with_provenance(input)?;
    let numbering_by_source = engine.numbering_by_source(source_nodes.len());
    Ok(WordLayoutResult {
        layout,
        revision_view: input.revision_view,
        source_nodes,
        numbering_by_source,
        page_reference_names: engine::page_reference_names(input),
    })
}

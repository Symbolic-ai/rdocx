//! Layout engine for converting DOCX flow model to positioned page frames.
//!
//! This crate implements style resolution, font loading, text shaping,
//! line breaking, block/table layout, and pagination to produce
//! [`LayoutResult`] containing [`PageFrame`]s with absolutely-positioned elements.

#![allow(non_camel_case_types)]
#![allow(clippy::too_many_arguments)]

pub mod block;
mod convert;
pub mod engine;
pub mod input;
pub mod paginator;
pub mod style_resolver;
pub mod table;

pub use input::{ImageData, LayoutInput, MediaRegistry};
pub use oxml_layout::{
    Color, DocumentMetadata, FontData, FontFile, FontId, GlyphRun, LayoutError, LayoutResult,
    PageFrame, Point, PositionedElement, Rect, Result,
};

/// Lay out a complete DOCX document, producing positioned page frames.
pub fn layout_document(input: &LayoutInput) -> Result<LayoutResult> {
    engine::Engine::new().layout(input)
}

/// Lay out a DOCX using bundled fonts without system font discovery.
pub fn layout_document_deterministic(input: &LayoutInput) -> Result<LayoutResult> {
    engine::Engine::new_deterministic()?.layout(input)
}

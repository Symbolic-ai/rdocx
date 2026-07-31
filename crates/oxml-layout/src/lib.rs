//! Format-neutral layout output and font primitives.

pub mod bundled_fonts;
pub mod error;
pub mod font;
pub mod output;

pub use error::{LayoutError, Result};
pub use font::{FontFile, FontManager, FontMetrics, ShapedText};
pub use output::{
    Color, DocumentMetadata, FieldKind, FontData, FontId, GlyphRun, LayoutResult, OutlineEntry,
    PageFrame, Point, PositionedElement, Rect,
};

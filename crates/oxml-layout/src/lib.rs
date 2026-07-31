//! Format-neutral layout output and font primitives.

pub mod bundled_fonts;
pub mod error;
pub mod font;
pub mod line;
pub mod output;
pub mod path;
pub mod transform;

pub use error::{LayoutError, Result};
pub use font::{FontFile, FontManager, FontMetrics, ShapedText};
pub use line::{
    Align, InlineItem, LayoutLine, LineBreakParams, LineItem, LineSpacing, TabAlign, TabLeader,
    TabStop, TextSegment, Underline, break_into_lines,
};
pub use output::{
    Color, DocumentMetadata, FieldKind, FontData, FontId, GlyphRun, LayoutResult, OutlineEntry,
    PageFrame, Point, PositionedElement, Rect,
};
pub use path::{FillRule, Path, PathCommand};
pub use transform::Transform;

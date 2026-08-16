//! Block-level layout: paragraphs and tables as positioned blocks.

use crate::table::TableBlock;
use oxml_layout::{Align, Color, LayoutLine, MediaId};
use rdocx_oxml::borders::CT_PBdr;
use rdocx_oxml::drawing::{
    AnchorAlignH, AnchorAlignV, ST_RelativeFromH, ST_RelativeFromV, WrapType,
};

/// A floating drawing anchored to a paragraph.
///
/// Offsets are kept in points alongside the frame they are measured from. A
/// `wp:anchor` offset is meaningless on its own: the same number means a
/// different place depending on whether it is relative to the page, the
/// margin, the text column or the paragraph.
#[derive(Debug, Clone)]
pub struct AnchoredDrawing {
    /// Render underneath the text rather than on top of it.
    pub behind_doc: bool,
    /// Frame the horizontal offset is measured from.
    pub rel_h: ST_RelativeFromH,
    /// Horizontal offset in points.
    pub off_h: f64,
    /// Frame the vertical offset is measured from.
    pub rel_v: ST_RelativeFromV,
    /// Vertical offset in points.
    pub off_v: f64,
    /// Width in points.
    pub width: f64,
    /// Height in points.
    pub height: f64,
    /// How text flows around the drawing.
    pub wrap: WrapType,
    /// Space kept between the drawing and the text wrapping around it, in
    /// points.
    pub dist_top: f64,
    pub dist_bottom: f64,
    pub dist_left: f64,
    pub dist_right: f64,
    /// Horizontal alignment, used instead of the offset when present.
    pub align_h: Option<AnchorAlignH>,
    /// Vertical alignment, used instead of the offset when present.
    pub align_v: Option<AnchorAlignV>,
    /// What the drawing actually holds.
    pub content: AnchoredContent,
}

/// The drawable content of an anchored drawing.
#[derive(Debug, Clone)]
pub enum AnchoredContent {
    /// A picture resolved to its content-addressed shared media identity.
    Image { media_id: MediaId },
    /// A shape: preset geometry, an optional fill, and optional text.
    ///
    /// The text arrives already laid out, because breaking it into lines needs
    /// a font manager and that only exists in the engine.
    Shape {
        /// Preset geometry we recognise.
        preset: ShapePreset,
        /// Fill colour, or `None` for `a:noFill` and for fills we cannot
        /// resolve. An unfilled shape draws no body, only its text.
        fill: Option<Color>,
        /// Laid-out paragraphs of the shape's text box.
        text: Vec<ParagraphBlock>,
    },
}

/// The preset geometries we can draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapePreset {
    /// A rectangle, drawn as a filled box.
    Rect,
    /// A straight line, drawn along the top edge of the extent.
    Line,
    /// Anything else. The body is not drawn, but text still is.
    Unsupported,
}

impl ShapePreset {
    /// Map an `a:prstGeom@prst` value onto what we can draw.
    pub fn from_prst(prst: Option<&str>) -> Self {
        match prst {
            Some("rect") => ShapePreset::Rect,
            Some("line") | Some("straightConnector1") => ShapePreset::Line,
            _ => ShapePreset::Unsupported,
        }
    }
}

/// A laid-out block element (paragraph or table).
#[derive(Debug, Clone)]
pub enum LayoutBlock {
    Paragraph(ParagraphBlock),
    Table(TableBlock),
}

impl LayoutBlock {
    /// Total height including spacing.
    pub fn total_height(&self) -> f64 {
        match self {
            LayoutBlock::Paragraph(p) => p.total_height(),
            LayoutBlock::Table(t) => t.total_height(),
        }
    }

    /// Content height without spacing.
    pub fn content_height(&self) -> f64 {
        match self {
            LayoutBlock::Paragraph(p) => p.content_height(),
            LayoutBlock::Table(t) => t.content_height(),
        }
    }

    pub fn space_before(&self) -> f64 {
        match self {
            LayoutBlock::Paragraph(p) => p.space_before,
            LayoutBlock::Table(_) => 0.0,
        }
    }

    pub fn space_after(&self) -> f64 {
        match self {
            LayoutBlock::Paragraph(p) => p.space_after,
            LayoutBlock::Table(_) => 0.0,
        }
    }

    pub fn keep_next(&self) -> bool {
        match self {
            LayoutBlock::Paragraph(p) => p.keep_next,
            LayoutBlock::Table(_) => false,
        }
    }

    pub fn keep_lines(&self) -> bool {
        match self {
            LayoutBlock::Paragraph(p) => p.keep_lines,
            LayoutBlock::Table(_) => false,
        }
    }

    pub fn page_break_before(&self) -> bool {
        match self {
            LayoutBlock::Paragraph(p) => p.page_break_before,
            LayoutBlock::Table(_) => false,
        }
    }

    pub fn widow_control(&self) -> bool {
        match self {
            LayoutBlock::Paragraph(p) => p.widow_control,
            LayoutBlock::Table(_) => false,
        }
    }
}

/// A laid-out paragraph with its lines and spacing.
#[derive(Debug, Clone)]
pub struct ParagraphBlock {
    /// Laid-out lines.
    pub lines: Vec<LayoutLine>,
    /// Floating drawings anchored to this paragraph.
    ///
    /// These travel with the paragraph so the paginator can resolve a
    /// paragraph-relative or line-relative offset once it knows where the
    /// paragraph actually landed.
    pub anchored: Vec<AnchoredDrawing>,
    /// Space before the paragraph in points.
    pub space_before: f64,
    /// Space after the paragraph in points.
    pub space_after: f64,
    /// Paragraph borders.
    pub borders: Option<CT_PBdr>,
    /// Background shading color.
    pub shading: Option<Color>,
    /// Left indent in points.
    pub indent_left: f64,
    /// Right indent in points.
    pub indent_right: f64,
    /// Paragraph justification.
    pub jc: Option<Align>,
    /// Keep with next paragraph.
    pub keep_next: bool,
    /// Keep all lines together on one page.
    pub keep_lines: bool,
    /// Force page break before this paragraph.
    pub page_break_before: bool,
    /// Widow/orphan control.
    pub widow_control: bool,
    /// Heading level (1-9) if this is a heading paragraph, for outline generation.
    pub heading_level: Option<u32>,
    /// Heading text for outline generation.
    pub heading_text: Option<String>,
}

impl ParagraphBlock {
    /// Total height of the paragraph lines (not including before/after spacing).
    pub fn content_height(&self) -> f64 {
        self.lines.iter().map(|l| l.height).sum()
    }

    /// Total height including spacing.
    pub fn total_height(&self) -> f64 {
        self.space_before + self.content_height() + self.space_after
    }

    /// Number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

/// Build a ParagraphBlock from resolved properties and layout lines.
pub fn build_paragraph_block(
    lines: Vec<LayoutLine>,
    space_before: f64,
    space_after: f64,
    borders: Option<CT_PBdr>,
    shading: Option<Color>,
    indent_left: f64,
    indent_right: f64,
    jc: Option<Align>,
    keep_next: bool,
    keep_lines: bool,
    page_break_before: bool,
    widow_control: bool,
) -> ParagraphBlock {
    ParagraphBlock {
        lines,
        anchored: Vec::new(),
        space_before,
        space_after,
        borders,
        shading,
        indent_left,
        indent_right,
        jc,
        keep_next,
        keep_lines,
        page_break_before,
        widow_control,
        heading_level: None,
        heading_text: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph_block_height() {
        let block = ParagraphBlock {
            anchored: Vec::new(),
            lines: vec![
                LayoutLine {
                    items: vec![],
                    width: 0.0,
                    ascent: 10.0,
                    descent: 3.0,
                    line_gap: 0.0,
                    height: 13.0,
                    indent_left: 0.0,
                    available_width: 468.0,
                    is_last: false,
                },
                LayoutLine {
                    items: vec![],
                    width: 0.0,
                    ascent: 10.0,
                    descent: 3.0,
                    line_gap: 0.0,
                    height: 13.0,
                    indent_left: 0.0,
                    available_width: 468.0,
                    is_last: true,
                },
            ],
            space_before: 6.0,
            space_after: 8.0,
            borders: None,
            shading: None,
            indent_left: 0.0,
            indent_right: 0.0,
            jc: None,
            keep_next: false,
            keep_lines: false,
            page_break_before: false,
            widow_control: true,
            heading_level: None,
            heading_text: None,
        };
        assert!((block.content_height() - 26.0).abs() < 0.01);
        assert!((block.total_height() - 40.0).abs() < 0.01);
    }

    /// Only the presets we can actually draw map to a drawable body. Anything
    /// else still renders its text, so it must not be treated as a rectangle.
    #[test]
    fn shape_presets_map_to_what_we_can_draw() {
        assert_eq!(ShapePreset::from_prst(Some("rect")), ShapePreset::Rect);
        assert_eq!(ShapePreset::from_prst(Some("line")), ShapePreset::Line);
        assert_eq!(
            ShapePreset::from_prst(Some("straightConnector1")),
            ShapePreset::Line
        );
        assert_eq!(
            ShapePreset::from_prst(Some("roundRect")),
            ShapePreset::Unsupported,
            "an unhandled preset must not silently draw as a plain rectangle"
        );
        assert_eq!(ShapePreset::from_prst(None), ShapePreset::Unsupported);
    }
}

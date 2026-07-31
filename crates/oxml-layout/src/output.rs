//! Output types for the layout engine: positioned page frames, glyph runs, etc.

/// A point in 2D space (in typographic points from the top-left corner).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// An axis-aligned rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// An RGBA color with components in [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    /// Parse a hex color string like "FF0000" to Color.
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
            Color { r, g, b, a: 1.0 }
        } else {
            Color::BLACK
        }
    }
}

/// Opaque font identifier assigned by FontManager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);

/// Stable content-addressed media key for renderer-local reuse.
///
/// This compact key is not a collision-free content guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MediaId(pub u64);

impl MediaId {
    /// Derive a stable key from raw media bytes using 64-bit FNV-1a.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self(hash)
    }
}

/// Kind of field for post-pagination substitution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Current page number.
    Page,
    /// Total number of pages.
    NumPages,
}

/// A positioned run of shaped glyphs.
#[derive(Debug, Clone)]
pub struct GlyphRun {
    /// Baseline origin of the first glyph (in points).
    pub origin: Point,
    /// Font identifier (from FontManager).
    pub font_id: FontId,
    /// Font size in points.
    pub font_size: f64,
    /// Shaped glyph IDs.
    pub glyph_ids: Vec<u16>,
    /// Per-glyph advances in points.
    pub advances: Vec<f64>,
    /// Original text (for PDF ToUnicode mapping).
    pub text: String,
    /// Text color.
    pub color: Color,
    /// Whether the font is bold.
    pub bold: bool,
    /// Whether the font is italic.
    pub italic: bool,
    /// If this glyph run is a field placeholder, the kind of field.
    pub field_kind: Option<FieldKind>,
    /// If this glyph run is a footnote/endnote reference marker, its ID.
    pub footnote_id: Option<i32>,
}

/// A positioned element on a page.
#[derive(Debug, Clone)]
pub enum PositionedElement {
    /// A run of shaped text glyphs.
    Text(GlyphRun),
    /// A line segment (for borders, underlines, strikethrough).
    Line {
        start: Point,
        end: Point,
        width: f64,
        color: Color,
        /// Optional dash pattern (dash_on, dash_off) in points. None = solid line.
        dash_pattern: Option<(f64, f64)>,
    },
    /// A filled rectangle (for shading, highlights).
    FilledRect { rect: Rect, color: Color },
    /// An inline image.
    Image {
        rect: Rect,
        data: Vec<u8>,
        content_type: String,
        media_id: MediaId,
    },
    /// A link annotation (hyperlink).
    LinkAnnotation { rect: Rect, url: String },
}

/// A single page of laid-out content.
#[derive(Debug, Clone)]
pub struct PageFrame {
    /// 1-based page number.
    pub page_number: usize,
    /// Page width in points.
    pub width: f64,
    /// Page height in points.
    pub height: f64,
    /// All positioned elements on this page.
    pub elements: Vec<PositionedElement>,
}

/// Font data for embedding in PDF output.
#[derive(Debug, Clone)]
pub struct FontData {
    /// Font identifier.
    pub id: FontId,
    /// Font family name.
    pub family: String,
    /// Raw TTF/OTF bytes for PDF embedding.
    pub data: Vec<u8>,
    /// Face index within a font collection.
    pub face_index: u32,
    /// Whether this is a bold variant.
    pub bold: bool,
    /// Whether this is an italic variant.
    pub italic: bool,
}

/// Document metadata to pass through to PDF output.
#[derive(Debug, Clone, Default)]
pub struct DocumentMetadata {
    /// Document title.
    pub title: Option<String>,
    /// Document author.
    pub author: Option<String>,
    /// Document subject.
    pub subject: Option<String>,
    /// Document keywords.
    pub keywords: Option<String>,
    /// Creator application.
    pub creator: Option<String>,
}

/// An outline/bookmark entry for PDF generation.
#[derive(Debug, Clone)]
pub struct OutlineEntry {
    /// The heading text.
    pub title: String,
    /// Heading level (1 for Heading1, 2 for Heading2, etc.).
    pub level: u32,
    /// 0-based page index this heading appears on.
    pub page_index: usize,
    /// Y position on the page (in points from top).
    pub y_position: f64,
}

/// The complete result of laying out a document.
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// Laid-out pages.
    pub pages: Vec<PageFrame>,
    /// Font data for all fonts used.
    pub fonts: Vec<FontData>,
    /// Optional document metadata for PDF output.
    pub metadata: Option<DocumentMetadata>,
    /// Outline/bookmark entries from headings.
    pub outlines: Vec<OutlineEntry>,
}

#[cfg(test)]
mod media_id_tests {
    use std::collections::HashSet;

    use super::{MediaId, PositionedElement, Rect};

    #[test]
    fn the_same_image_bytes_inserted_twice_produce_one_media_id() {
        let ids = HashSet::from([
            MediaId::from_bytes(b"same image"),
            MediaId::from_bytes(b"same image"),
        ]);
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn media_id_depends_on_bytes_not_relationship_context() {
        assert_eq!(
            MediaId::from_bytes(b"image bytes"),
            MediaId::from_bytes(b"image bytes")
        );
    }

    #[test]
    fn different_image_bytes_have_different_fixture_ids() {
        assert_ne!(
            MediaId::from_bytes(b"first image"),
            MediaId::from_bytes(b"second image")
        );
    }

    #[test]
    fn staged_output_image_uses_media_id_instead_of_embed_id() {
        let media_id = MediaId::from_bytes(b"image bytes");
        let image = PositionedElement::Image {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 20.0,
            },
            data: b"image bytes".to_vec(),
            content_type: "image/png".to_owned(),
            media_id,
        };
        let PositionedElement::Image {
            media_id: actual, ..
        } = image
        else {
            panic!("constructed image should remain an image");
        };
        assert_eq!(actual, media_id);
    }
}

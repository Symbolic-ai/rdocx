//! PDF renderer for format-neutral OOXML layout output.
//!
//! Converts `LayoutResult` (positioned page frames with glyph runs, lines,
//! rectangles, and images) into a PDF document.

mod font;
mod image;
pub mod raster;
mod structure;
mod writer;

use oxml_layout::LayoutResult;

/// Render a laid-out document to PDF bytes.
///
/// The `LayoutResult` must contain all pages, fonts, metadata, and outlines
/// produced by a format-specific layout engine.
pub fn render_to_pdf(layout: &LayoutResult) -> Vec<u8> {
    writer::write_pdf(layout)
}

/// Render a single page to PNG bytes.
///
/// # Arguments
/// * `layout` - The format-neutral layout result to rasterise
/// * `page_index` - 0-based page index
/// * `dpi` - Resolution (72 = 1:1, 150 = standard, 300 = high quality)
pub fn render_page_to_png(layout: &LayoutResult, page_index: usize, dpi: f64) -> Option<Vec<u8>> {
    raster::render_page_to_png(layout, page_index, dpi)
}

/// Render all pages to PNG bytes.
pub fn render_all_pages(layout: &LayoutResult, dpi: f64) -> Vec<Vec<u8>> {
    raster::render_all_pages(layout, dpi)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxml_layout::{
        Color, DocumentMetadata, DocumentStructure, LayoutResult, OutlineEntry, PageFrame, Point,
        PositionedElement, Rect, StructureId, StructureNode, StructureRole,
    };

    fn layout_with(elements: Vec<PositionedElement>) -> LayoutResult {
        LayoutResult::new(
            vec![PageFrame::new(1, 612.0, 792.0, elements).into()],
            vec![],
            None,
            vec![],
        )
    }

    #[test]
    fn render_empty_layout() {
        let pdf = render_to_pdf(&layout_with(vec![]));
        assert!(pdf.starts_with(b"%PDF"));
        assert!(String::from_utf8_lossy(&pdf).contains("%%EOF"));
    }

    #[test]
    fn render_with_lines_and_rects() {
        let elements = vec![
            PositionedElement::Line {
                start: Point { x: 72.0, y: 72.0 },
                end: Point { x: 540.0, y: 72.0 },
                width: 1.0,
                color: Color::BLACK,
                dash_pattern: None,
            },
            PositionedElement::FilledRect {
                rect: Rect {
                    x: 72.0,
                    y: 100.0,
                    width: 468.0,
                    height: 20.0,
                },
                color: Color {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                },
            },
        ];
        let pdf = render_to_pdf(&layout_with(elements));
        assert!(pdf.starts_with(b"%PDF"));
        assert!(pdf.len() > 100);
    }

    #[test]
    fn tagging_preserves_visible_pdf_and_raster_output() {
        let leaf = PositionedElement::FilledRect {
            rect: Rect {
                x: 72.0,
                y: 100.0,
                width: 120.0,
                height: 30.0,
            },
            color: Color {
                r: 0.25,
                g: 0.5,
                b: 0.75,
                a: 1.0,
            },
        };
        let plain = layout_with(vec![leaf.clone()]);
        let document = StructureId::new(1).expect("non-zero document id");
        let paragraph = StructureId::new(2).expect("non-zero paragraph id");
        let mut tagged = layout_with(vec![PositionedElement::MarkedContent {
            structure: Some(paragraph),
            children: vec![leaf],
        }]);
        tagged.structure = Some(DocumentStructure {
            root: document,
            nodes: vec![
                StructureNode {
                    id: document,
                    role: StructureRole::Document,
                    children: vec![paragraph],
                    alternate_text: None,
                },
                StructureNode {
                    id: paragraph,
                    role: StructureRole::Paragraph,
                    children: Vec::new(),
                    alternate_text: None,
                },
            ],
        });

        assert_eq!(
            render_page_to_png(&plain, 0, 144.0),
            render_page_to_png(&tagged, 0, 144.0)
        );
        assert_ne!(render_to_pdf(&plain), render_to_pdf(&tagged));
    }

    #[test]
    fn render_with_metadata() {
        let layout = LayoutResult::new(
            vec![PageFrame::new(1, 612.0, 792.0, vec![]).into()],
            vec![],
            Some(DocumentMetadata {
                title: Some("Test Title".to_owned()),
                author: Some("Test Author".to_owned()),
                subject: None,
                keywords: None,
                creator: Some("oxml-pdf".to_owned()),
            }),
            vec![],
        );
        let pdf = render_to_pdf(&layout);
        let pdf = String::from_utf8_lossy(&pdf);
        assert!(pdf.contains("Test Title"));
        assert!(pdf.contains("Test Author"));
    }

    #[test]
    fn render_with_link_annotation() {
        let layout = layout_with(vec![PositionedElement::LinkAnnotation {
            rect: Rect {
                x: 72.0,
                y: 100.0,
                width: 100.0,
                height: 15.0,
            },
            url: "https://example.com".to_owned(),
        }]);
        let pdf = render_to_pdf(&layout);
        let pdf = String::from_utf8_lossy(&pdf);
        assert!(pdf.contains("example.com"));
    }

    #[test]
    fn render_with_outlines() {
        let layout = LayoutResult::new(
            vec![PageFrame::new(1, 612.0, 792.0, vec![]).into()],
            vec![],
            None,
            vec![
                OutlineEntry {
                    title: "Chapter 1".to_owned(),
                    level: 1,
                    page_index: 0,
                    y_position: 72.0,
                },
                OutlineEntry {
                    title: "Section 1.1".to_owned(),
                    level: 2,
                    page_index: 0,
                    y_position: 200.0,
                },
            ],
        );
        let pdf = render_to_pdf(&layout);
        let pdf = String::from_utf8_lossy(&pdf);
        assert!(pdf.contains("Chapter 1"));
        assert!(pdf.contains("Section 1.1"));
    }
}

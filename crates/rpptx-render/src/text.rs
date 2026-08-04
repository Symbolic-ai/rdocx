use oxml_layout::{
    Color, FontManager, InlineItem, LayoutError, Paint, Rect, TextSegment, Underline,
};
use rpptx_layout::{
    ResolvedParagraph, ResolvedRunStyle, ResolvedShape, ResolvedTextBody, ResolvedTextRun,
};

const DEFAULT_FONT_SIZE: f64 = 18.0;

pub(super) fn inline_items(
    font_manager: &mut FontManager,
    paragraph: &ResolvedParagraph,
) -> Result<Vec<InlineItem>, LayoutError> {
    let mut items = Vec::with_capacity(paragraph.runs.len());
    for run in &paragraph.runs {
        match run {
            ResolvedTextRun::Text { text, style } | ResolvedTextRun::Field { text, style, .. } => {
                if !text.is_empty() {
                    items.push(InlineItem::Text(shape_run(font_manager, text, style)?));
                }
            }
            ResolvedTextRun::Break => items.push(InlineItem::LineBreak),
        }
    }
    Ok(items)
}

fn shape_run(
    font_manager: &mut FontManager,
    text: &str,
    style: &ResolvedRunStyle,
) -> Result<TextSegment, LayoutError> {
    let font_size = style.font_size.unwrap_or(DEFAULT_FONT_SIZE);
    let typeface = typeface_for_text(style, text);
    let font_id = font_manager.resolve_font_for_text(typeface, style.bold, style.italic, text)?;
    let metrics = font_manager.metrics(font_id, font_size)?;
    let mut shaped = font_manager.shape_text(font_id, text, font_size)?;
    if let Some(spacing) = style.spacing {
        for advance in &mut shaped.advances {
            *advance += spacing;
        }
        shaped.width += spacing * shaped.advances.len() as f64;
    }

    Ok(TextSegment {
        text: text.to_owned(),
        font_id,
        font_size,
        glyph_ids: shaped.glyph_ids,
        advances: shaped.advances,
        width: shaped.width,
        ascent: metrics.ascent,
        descent: metrics.descent,
        color: text_color(style.fill.as_ref()),
        bold: style.bold,
        italic: style.italic,
        underline: style.underline.then_some(Underline::Single),
        strike: style.strike,
        dstrike: false,
        highlight: None,
        baseline_offset: style.baseline.unwrap_or(0.0) * font_size,
        hyperlink_url: None,
        field_kind: None,
        footnote_id: None,
    })
}

fn typeface_for_text<'a>(style: &'a ResolvedRunStyle, text: &str) -> Option<&'a str> {
    let preferred = if text.chars().any(is_symbol_character) {
        style.symbol_typeface.as_deref()
    } else if text.chars().any(is_east_asian_character) {
        style.east_asian_typeface.as_deref()
    } else if text.chars().any(is_complex_script_character) {
        style.complex_script_typeface.as_deref()
    } else {
        style.latin_typeface.as_deref()
    };
    preferred
        .or(style.latin_typeface.as_deref())
        .or(style.east_asian_typeface.as_deref())
        .or(style.complex_script_typeface.as_deref())
        .or(style.symbol_typeface.as_deref())
}

fn is_symbol_character(character: char) -> bool {
    matches!(character as u32, 0xE000..=0xF8FF)
}

fn is_east_asian_character(character: char) -> bool {
    matches!(
        character as u32,
        0x1100..=0x11FF
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE4F
            | 0xFF00..=0xFFEF
            | 0x20000..=0x323AF
    )
}

fn is_complex_script_character(character: char) -> bool {
    matches!(
        character as u32,
        0x0590..=0x10FF | 0x1780..=0x17FF | 0xFB1D..=0xFDFF | 0xFE70..=0xFEFF
    )
}

fn text_color(fill: Option<&Paint>) -> Color {
    match fill {
        Some(Paint::Solid(color)) => *color,
        Some(Paint::Linear { stops, .. }) | Some(Paint::Radial { stops, .. }) => {
            stops.first().map_or(Color::BLACK, |stop| stop.color)
        }
        Some(Paint::Tile { .. }) | None => Color::BLACK,
    }
}

pub(super) fn content_box(shape: &ResolvedShape, text: &ResolvedTextBody) -> Rect {
    let boundary = match &shape.geometry {
        rpptx_layout::ResolvedGeometry::Custom {
            text_rect: Some(rect),
            ..
        } => *rect,
        _ => Rect {
            x: 0.0,
            y: 0.0,
            width: shape.bounds.width,
            height: shape.bounds.height,
        },
    };
    Rect {
        x: boundary.x + text.insets.left,
        y: boundary.y + text.insets.top,
        width: (boundary.width - text.insets.left - text.insets.right).max(0.0),
        height: (boundary.height - text.insets.top - text.insets.bottom).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use oxml_layout::{Color, Paint, Transform};
    use rpptx_layout::{
        ResolvedAutofit, ResolvedContent, ResolvedGeometry, ResolvedRunStyle, ResolvedSlide,
        ResolvedTextRun, TextAnchor, TextDirection, TextInsets,
    };

    use crate::{RenderInput, RenderInputError, layout_presentation, layout_slide_with_fonts};

    fn style() -> ResolvedRunStyle {
        ResolvedRunStyle {
            font_size: Some(22.0),
            bold: true,
            italic: true,
            underline: true,
            strike: true,
            spacing: Some(1.5),
            baseline: Some(0.25),
            fill: Some(Paint::Solid(Color {
                r: 0.8,
                g: 0.1,
                b: 0.2,
                a: 1.0,
            })),
            latin_typeface: Some("Carlito".to_owned()),
            ..ResolvedRunStyle::default()
        }
    }

    fn shape(bounds: Rect, geometry: ResolvedGeometry) -> ResolvedShape {
        ResolvedShape {
            group_transform: Transform::IDENTITY,
            bounds,
            rotation_deg: 0.0,
            flip_h: false,
            flip_v: false,
            geometry,
            fill: None,
            line: None,
            head_end: None,
            tail_end: None,
            shadow: None,
            content: ResolvedContent::None,
            unsupported: None,
        }
    }

    fn text_body(insets: TextInsets) -> ResolvedTextBody {
        ResolvedTextBody {
            insets,
            anchor: TextAnchor::Top,
            wrap: true,
            vertical: TextDirection::Horizontal,
            autofit: ResolvedAutofit::None,
            paragraphs: Vec::new(),
        }
    }

    #[test]
    fn preset_text_rectangle_minus_unequal_insets_produces_the_computed_content_box() {
        let shape = shape(
            Rect {
                x: 30.0,
                y: 40.0,
                width: 120.0,
                height: 70.0,
            },
            ResolvedGeometry::Custom {
                paths: Vec::new(),
                text_rect: Some(Rect {
                    x: 10.0,
                    y: 15.0,
                    width: 80.0,
                    height: 40.0,
                }),
            },
        );
        let text = text_body(TextInsets {
            left: 3.0,
            top: 5.0,
            right: 7.0,
            bottom: 11.0,
        });

        assert_eq!(
            content_box(&shape, &text),
            Rect {
                x: 13.0,
                y: 20.0,
                width: 70.0,
                height: 24.0,
            }
        );
    }

    #[test]
    fn missing_text_rectangle_falls_back_to_local_shape_bounds() {
        let text = text_body(TextInsets {
            left: 4.0,
            top: 6.0,
            right: 8.0,
            bottom: 10.0,
        });

        for geometry in [
            ResolvedGeometry::Rectangle,
            ResolvedGeometry::BoundsFallback,
        ] {
            let shape = shape(
                Rect {
                    x: 30.0,
                    y: 40.0,
                    width: 120.0,
                    height: 70.0,
                },
                geometry,
            );
            assert_eq!(
                content_box(&shape, &text),
                Rect {
                    x: 4.0,
                    y: 6.0,
                    width: 108.0,
                    height: 54.0,
                }
            );
        }
    }

    #[test]
    fn insets_larger_than_the_text_rectangle_do_not_create_negative_extents() {
        let shape = shape(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 30.0,
                height: 20.0,
            },
            ResolvedGeometry::Custom {
                paths: Vec::new(),
                text_rect: Some(Rect {
                    x: 1.0,
                    y: 2.0,
                    width: 10.0,
                    height: 8.0,
                }),
            },
        );
        let text = text_body(TextInsets {
            left: 8.0,
            top: 9.0,
            right: 7.0,
            bottom: 6.0,
        });

        let content = content_box(&shape, &text);
        assert_eq!(
            content,
            Rect {
                x: 9.0,
                y: 11.0,
                width: 0.0,
                height: 0.0,
            }
        );
        assert!(content.x.is_finite());
        assert!(content.y.is_finite());
        assert!(content.width.is_finite());
        assert!(content.height.is_finite());
    }

    #[test]
    fn resolved_runs_emit_glyph_items_with_concrete_style_and_break_boundaries() {
        let paragraph = ResolvedParagraph {
            runs: vec![
                ResolvedTextRun::Text {
                    text: "Alpha".to_owned(),
                    style: style(),
                },
                ResolvedTextRun::Break,
                ResolvedTextRun::Field {
                    text: "42".to_owned(),
                    field_type: Some("slidenum".to_owned()),
                    style: style(),
                },
            ],
            ..ResolvedParagraph::default()
        };
        let mut fonts = FontManager::new_deterministic().expect("deterministic fonts");

        let items = inline_items(&mut fonts, &paragraph).expect("shape resolved runs");

        assert_eq!(items.len(), 3);
        let InlineItem::Text(first) = &items[0] else {
            panic!("text run should become a text item");
        };
        assert_eq!(first.text, "Alpha");
        assert_eq!(first.font_size, 22.0);
        assert_eq!(first.color.r, 0.8);
        assert!(first.bold);
        assert!(first.italic);
        assert!(first.underline.is_some());
        assert!(first.strike);
        assert_eq!(first.baseline_offset, 5.5);
        assert!(first.glyph_ids.iter().any(|glyph| *glyph != 0));
        assert!(matches!(items[1], InlineItem::LineBreak));
        let InlineItem::Text(field) = &items[2] else {
            panic!("field should retain its display text as a text item");
        };
        assert_eq!(field.text, "42");
        assert_eq!(field.font_size, 22.0);
    }

    #[test]
    fn script_specific_text_selects_its_resolved_concrete_typeface() {
        let style = ResolvedRunStyle {
            latin_typeface: Some("Latin face".to_owned()),
            east_asian_typeface: Some("East Asian face".to_owned()),
            complex_script_typeface: Some("Complex face".to_owned()),
            symbol_typeface: Some("Symbol face".to_owned()),
            ..ResolvedRunStyle::default()
        };

        assert_eq!(typeface_for_text(&style, "Latin"), Some("Latin face"));
        assert_eq!(typeface_for_text(&style, "漢字"), Some("East Asian face"));
        assert_eq!(typeface_for_text(&style, "مرحبا"), Some("Complex face"));
        assert_eq!(typeface_for_text(&style, "\u{F0B7}"), Some("Symbol face"));
    }

    #[test]
    fn missing_run_size_and_fill_use_visible_renderer_defaults() {
        let paragraph = ResolvedParagraph {
            runs: vec![ResolvedTextRun::Text {
                text: "Visible".to_owned(),
                style: ResolvedRunStyle::default(),
            }],
            ..ResolvedParagraph::default()
        };
        let mut fonts = FontManager::new_deterministic().expect("deterministic fonts");

        let items = inline_items(&mut fonts, &paragraph).expect("shape default run");

        let InlineItem::Text(segment) = &items[0] else {
            panic!("text run should become a text item");
        };
        assert_eq!(segment.font_size, 18.0);
        assert_eq!(segment.color, Color::BLACK);
    }

    #[test]
    fn text_shaping_failures_return_a_render_error_without_panicking() {
        let text = ResolvedTextBody {
            insets: TextInsets::default(),
            anchor: TextAnchor::Top,
            wrap: true,
            vertical: TextDirection::Horizontal,
            autofit: ResolvedAutofit::None,
            paragraphs: vec![ResolvedParagraph {
                runs: vec![ResolvedTextRun::Text {
                    text: "No font".to_owned(),
                    style: ResolvedRunStyle::default(),
                }],
                ..ResolvedParagraph::default()
            }],
        };
        let mut text_shape = shape(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
            ResolvedGeometry::Rectangle,
        );
        text_shape.content = ResolvedContent::Text(text);
        let input = RenderInput {
            slides: vec![ResolvedSlide {
                size: (100.0, 50.0),
                background: None,
                shapes: vec![text_shape],
                diagnostics: Vec::new(),
            }],
            media: HashMap::new(),
            fonts: Vec::new(),
            metadata: None,
        };
        let mut fonts = FontManager::new_with_fonts(Vec::new());

        let error =
            layout_slide_with_fonts(&input, 0, &mut fonts).expect_err("missing font should fail");

        assert!(matches!(error, RenderInputError::TextLayout { .. }));
    }

    #[test]
    fn presentation_layout_collects_every_font_used_by_shape_text() {
        let text = ResolvedTextBody {
            insets: TextInsets::default(),
            anchor: TextAnchor::Top,
            wrap: true,
            vertical: TextDirection::Horizontal,
            autofit: ResolvedAutofit::None,
            paragraphs: vec![ResolvedParagraph {
                runs: vec![
                    ResolvedTextRun::Text {
                        text: "Regular".to_owned(),
                        style: ResolvedRunStyle {
                            latin_typeface: Some("Carlito".to_owned()),
                            ..ResolvedRunStyle::default()
                        },
                    },
                    ResolvedTextRun::Text {
                        text: "Bold".to_owned(),
                        style: ResolvedRunStyle {
                            bold: true,
                            latin_typeface: Some("Carlito".to_owned()),
                            ..ResolvedRunStyle::default()
                        },
                    },
                ],
                ..ResolvedParagraph::default()
            }],
        };
        let mut text_shape = shape(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
            },
            ResolvedGeometry::Rectangle,
        );
        text_shape.content = ResolvedContent::Text(text);
        let input = RenderInput {
            slides: vec![ResolvedSlide {
                size: (100.0, 50.0),
                background: None,
                shapes: vec![text_shape],
                diagnostics: Vec::new(),
            }],
            media: HashMap::new(),
            fonts: Vec::new(),
            metadata: None,
        };

        let layout = layout_presentation(&input).expect("layout presentation text");

        assert_eq!(layout.fonts.len(), 2);
        assert!(layout.fonts.iter().any(|font| !font.bold));
        assert!(layout.fonts.iter().any(|font| font.bold));
    }
}

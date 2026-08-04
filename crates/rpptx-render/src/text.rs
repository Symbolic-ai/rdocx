use oxml_layout::Rect;
use rpptx_layout::{ResolvedShape, ResolvedTextBody};

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
    use oxml_layout::Transform;
    use rpptx_layout::{
        ResolvedAutofit, ResolvedContent, ResolvedGeometry, TextAnchor, TextDirection, TextInsets,
    };

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
}

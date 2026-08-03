//! Presentation rendering inputs and package assembly helpers.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use oxml_drawing::theme::CT_OfficeStyleSheet;
use oxml_layout::{
    Color, DocumentMetadata, FontFile, GroupElement, LayoutResult, MediaId, PageFrame, Paint, Path,
    PathElement, PositionedElement, Rect, Stroke, Transform,
};
use rpptx_layout::{ResolvedGeometry, ResolvedShape, ResolvedSlide};
use rpptx_oxml::notes_parts::CT_NotesSlide;
use rpptx_oxml::slide_parts::{CT_Slide, CT_SlideLayout, CT_SlideMaster};

/// The source part whose relationship map owns an identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelScope {
    Slide,
    Layout,
    Master,
}

impl fmt::Display for RelScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Slide => "slide",
            Self::Layout => "layout",
            Self::Master => "master",
        })
    }
}

/// A package relationship after its target has been resolved against its source part.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedRel {
    pub target: String,
    pub relationship_type: String,
}

/// Relationship maps kept separate by their source-part scope.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RelScopes {
    pub slide: HashMap<String, ResolvedRel>,
    pub layout: HashMap<String, ResolvedRel>,
    pub master: HashMap<String, ResolvedRel>,
}

impl RelScopes {
    /// Look up a relationship only in the explicitly selected source-part scope.
    pub fn get(
        &self,
        scope: RelScope,
        relationship_id: &str,
    ) -> Result<&ResolvedRel, RenderInputError> {
        let relationships = match scope {
            RelScope::Slide => &self.slide,
            RelScope::Layout => &self.layout,
            RelScope::Master => &self.master,
        };
        relationships
            .get(relationship_id)
            .ok_or_else(|| RenderInputError::MissingRelationship {
                scope,
                relationship_id: relationship_id.to_owned(),
            })
    }
}

/// Media bytes available to a renderer, with their package content type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaData {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

/// Package assembly failures that retain relationship source context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenderInputError {
    MissingRelationship {
        scope: RelScope,
        relationship_id: String,
    },
    MissingMediaTarget {
        scope: RelScope,
        relationship_id: String,
        target: String,
    },
    SlideIndexOutOfBounds {
        index: usize,
        slide_count: usize,
    },
}

impl fmt::Display for RenderInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRelationship {
                scope,
                relationship_id,
            } => write!(formatter, "missing {scope} relationship {relationship_id}"),
            Self::MissingMediaTarget {
                scope,
                relationship_id,
                target,
            } => write!(
                formatter,
                "missing media target {target} for {scope} relationship {relationship_id}"
            ),
            Self::SlideIndexOutOfBounds { index, slide_count } => write!(
                formatter,
                "slide index {index} is out of bounds for {slide_count} slides"
            ),
        }
    }
}

impl Error for RenderInputError {}

/// Raw package parts assembled before inheritance resolution.
#[derive(Clone, Debug)]
pub struct SlideBundle {
    pub slide: CT_Slide,
    pub layout: Arc<CT_SlideLayout>,
    pub master: Arc<CT_SlideMaster>,
    pub theme: Arc<CT_OfficeStyleSheet>,
    pub notes: Option<CT_NotesSlide>,
    pub hidden: bool,
    pub relationships: RelScopes,
}

/// Frozen, format-neutral input consumed by the rendering stage.
#[derive(Clone, Debug)]
pub struct RenderInput {
    pub slides: Vec<ResolvedSlide>,
    pub media: HashMap<MediaId, MediaData>,
    pub fonts: Vec<FontFile>,
    pub metadata: Option<DocumentMetadata>,
}

/// Lower every resolved slide to one fixed-size page in presentation order.
pub fn layout_presentation(input: &RenderInput) -> Result<LayoutResult, RenderInputError> {
    let pages = (0..input.slides.len())
        .map(|index| layout_slide(input, index))
        .collect::<Result<Vec<_>, _>>()?;
    let diagnostics = input
        .slides
        .iter()
        .flat_map(|slide| slide.diagnostics.iter().cloned())
        .collect();
    let mut layout = LayoutResult::new(pages, Vec::new(), input.metadata.clone(), Vec::new());
    layout.diagnostics = diagnostics;
    Ok(layout)
}

/// Lower one zero-based resolved slide to a fixed-size page.
pub fn layout_slide(input: &RenderInput, index: usize) -> Result<PageFrame, RenderInputError> {
    let slide = input
        .slides
        .get(index)
        .ok_or(RenderInputError::SlideIndexOutOfBounds {
            index,
            slide_count: input.slides.len(),
        })?;
    let elements = slide.shapes.iter().map(lower_shape).collect();
    Ok(PageFrame::new(
        index + 1,
        slide.size.0,
        slide.size.1,
        elements,
    ))
}

fn lower_shape(shape: &ResolvedShape) -> PositionedElement {
    let paths = match &shape.geometry {
        ResolvedGeometry::Rectangle | ResolvedGeometry::BoundsFallback => vec![Path::rect(Rect {
            x: 0.0,
            y: 0.0,
            width: shape.bounds.width,
            height: shape.bounds.height,
        })],
        ResolvedGeometry::Custom { paths, .. } => paths.clone(),
    };
    let stroke = match (&shape.geometry, &shape.fill, &shape.line) {
        (ResolvedGeometry::BoundsFallback, None, None) => {
            Some(Stroke::new(Paint::Solid(Color::BLACK), 1.0))
        }
        _ => shape.line.clone(),
    };
    let children = paths
        .into_iter()
        .map(|path| {
            PositionedElement::Path(PathElement {
                path,
                fill: shape.fill.clone(),
                stroke: stroke.clone(),
            })
        })
        .collect();
    PositionedElement::Group(GroupElement {
        transform: shape_transform(shape),
        clip: None,
        opacity: 1.0,
        effects: Vec::new(),
        children,
    })
}

fn shape_transform(shape: &ResolvedShape) -> Transform {
    let center_x = shape.bounds.width / 2.0;
    let center_y = shape.bounds.height / 2.0;
    let rotation = Transform::rotate_about(shape.rotation_deg, center_x, center_y);
    let flip = Transform {
        a: if shape.flip_h { -1.0 } else { 1.0 },
        b: 0.0,
        c: 0.0,
        d: if shape.flip_v { -1.0 } else { 1.0 },
        e: if shape.flip_h {
            shape.bounds.width
        } else {
            0.0
        },
        f: if shape.flip_v {
            shape.bounds.height
        } else {
            0.0
        },
    };
    let translation = Transform {
        e: shape.bounds.x,
        f: shape.bounds.y,
        ..Transform::IDENTITY
    };
    rotation
        .then(flip)
        .then(translation)
        .then(shape.group_transform)
}

/// Resolve one scoped media relationship into the deck's content-addressed store.
pub fn resolve_media_relationship(
    relationships: &RelScopes,
    scope: RelScope,
    relationship_id: &str,
    package_media: &HashMap<String, MediaData>,
    deck_media: &mut HashMap<MediaId, MediaData>,
) -> Result<MediaId, RenderInputError> {
    let relationship = relationships.get(scope, relationship_id)?;
    let media = package_media.get(&relationship.target).ok_or_else(|| {
        RenderInputError::MissingMediaTarget {
            scope,
            relationship_id: relationship_id.to_owned(),
            target: relationship.target.clone(),
        }
    })?;
    let media_id = MediaId::from_bytes(&media.bytes);
    deck_media.entry(media_id).or_insert_with(|| media.clone());
    Ok(media_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxml_layout::{
        Color, Diagnostic, FillRule, GradientStop, GroupElement, Paint, Path, PathCommand, Point,
        PositionedElement, Rect, Stroke, Transform,
    };
    use rpptx_layout::{ResolvedContent, ResolvedGeometry, ResolvedShape};

    const IMAGE_RELATIONSHIP: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

    fn media(bytes: &[u8]) -> MediaData {
        MediaData {
            bytes: bytes.to_vec(),
            content_type: "image/png".to_owned(),
        }
    }

    fn relationship(target: &str) -> ResolvedRel {
        ResolvedRel {
            target: target.to_owned(),
            relationship_type: IMAGE_RELATIONSHIP.to_owned(),
        }
    }

    fn color(red: f64, green: f64, blue: f64) -> Color {
        Color {
            r: red,
            g: green,
            b: blue,
            a: 1.0,
        }
    }

    fn shape(
        bounds: Rect,
        geometry: ResolvedGeometry,
        fill: Option<Paint>,
        line: Option<Stroke>,
    ) -> ResolvedShape {
        ResolvedShape {
            group_transform: Transform::IDENTITY,
            bounds,
            rotation_deg: 0.0,
            flip_h: false,
            flip_v: false,
            geometry,
            fill,
            line,
            shadow: None,
            content: ResolvedContent::None,
            unsupported: None,
        }
    }

    fn slide(size: (f64, f64), shapes: Vec<ResolvedShape>) -> ResolvedSlide {
        ResolvedSlide {
            size,
            background: None,
            shapes,
            diagnostics: Vec::new(),
        }
    }

    fn render_input(slides: Vec<ResolvedSlide>) -> RenderInput {
        RenderInput {
            slides,
            media: HashMap::new(),
            fonts: Vec::new(),
            metadata: None,
        }
    }

    fn only_group(element: &PositionedElement) -> &GroupElement {
        let PositionedElement::Group(group) = element else {
            panic!("shape should lower to one group");
        };
        group
    }

    fn assert_point_close(actual: Point, expected: Point) {
        const EPSILON: f64 = 1.0e-10;
        assert!(
            (actual.x - expected.x).abs() < EPSILON && (actual.y - expected.y).abs() < EPSILON,
            "expected ({}, {}), got ({}, {})",
            expected.x,
            expected.y,
            actual.x,
            actual.y
        );
    }

    #[test]
    fn rotated_shape_corners_match_hand_computed_coordinates() {
        let mut rotated = shape(
            Rect {
                x: 10.0,
                y: 20.0,
                width: 8.0,
                height: 4.0,
            },
            ResolvedGeometry::Rectangle,
            Some(Paint::Solid(Color::BLACK)),
            None,
        );
        rotated.rotation_deg = 30.0;
        let page = layout_slide(&render_input(vec![slide((40.0, 40.0), vec![rotated])]), 0)
            .expect("lower rotated shape");
        let transform = only_group(&page.elements[0]).transform;
        let radians = 30.0_f64.to_radians();
        let (sin, cos) = radians.sin_cos();

        for corner in [
            Point { x: 0.0, y: 0.0 },
            Point { x: 8.0, y: 0.0 },
            Point { x: 0.0, y: 4.0 },
            Point { x: 8.0, y: 4.0 },
        ] {
            let dx = corner.x - 4.0;
            let dy = corner.y - 2.0;
            let expected = Point {
                x: 10.0 + 4.0 + cos * dx - sin * dy,
                y: 20.0 + 2.0 + sin * dx + cos * dy,
            };
            assert_point_close(transform.apply(corner), expected);
        }
    }

    #[test]
    fn horizontal_and_vertical_flips_are_about_the_shape_centre() {
        let bounds = Rect {
            x: 10.0,
            y: 20.0,
            width: 8.0,
            height: 4.0,
        };
        let mut horizontal = shape(
            bounds,
            ResolvedGeometry::Rectangle,
            Some(Paint::Solid(Color::BLACK)),
            None,
        );
        horizontal.flip_h = true;
        let mut vertical = horizontal.clone();
        vertical.flip_h = false;
        vertical.flip_v = true;
        let page = layout_slide(
            &render_input(vec![slide((40.0, 40.0), vec![horizontal, vertical])]),
            0,
        )
        .expect("lower flipped shapes");
        let horizontal = only_group(&page.elements[0]).transform;
        let vertical = only_group(&page.elements[1]).transform;

        assert_point_close(
            horizontal.apply(Point { x: 4.0, y: 2.0 }),
            Point { x: 14.0, y: 22.0 },
        );
        assert_point_close(
            horizontal.apply(Point { x: 0.0, y: 0.0 }),
            Point { x: 18.0, y: 20.0 },
        );
        assert_point_close(
            horizontal.apply(Point { x: 8.0, y: 4.0 }),
            Point { x: 10.0, y: 24.0 },
        );
        assert_point_close(
            vertical.apply(Point { x: 4.0, y: 2.0 }),
            Point { x: 14.0, y: 22.0 },
        );
        assert_point_close(
            vertical.apply(Point { x: 0.0, y: 0.0 }),
            Point { x: 10.0, y: 24.0 },
        );
        assert_point_close(
            vertical.apply(Point { x: 8.0, y: 4.0 }),
            Point { x: 18.0, y: 20.0 },
        );
    }

    #[test]
    fn nested_group_transform_applies_child_before_parent() {
        let mut nested = shape(
            Rect {
                x: 10.0,
                y: 20.0,
                width: 8.0,
                height: 4.0,
            },
            ResolvedGeometry::Rectangle,
            Some(Paint::Solid(Color::BLACK)),
            None,
        );
        nested.rotation_deg = 90.0;
        nested.flip_h = true;
        nested.group_transform = Transform {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 3.0,
            e: 5.0,
            f: 7.0,
        };
        let page = layout_slide(&render_input(vec![slide((80.0, 100.0), vec![nested])]), 0)
            .expect("lower nested shape");
        let transform = only_group(&page.elements[0]).transform;

        assert_point_close(
            transform.apply(Point { x: 0.0, y: 0.0 }),
            Point { x: 29.0, y: 61.0 },
        );
        assert_point_close(
            transform.apply(Point { x: 8.0, y: 4.0 }),
            Point { x: 37.0, y: 85.0 },
        );
    }

    #[test]
    fn rotated_gradient_and_outline_share_the_shape_transform() {
        let red = color(1.0, 0.0, 0.0);
        let blue = color(0.0, 0.0, 1.0);
        let mut rotated = shape(
            Rect {
                x: 8.0,
                y: 8.0,
                width: 12.0,
                height: 6.0,
            },
            ResolvedGeometry::Rectangle,
            Some(Paint::linear(
                Point { x: 0.0, y: 0.0 },
                Point { x: 12.0, y: 0.0 },
                vec![
                    GradientStop {
                        offset: 0.0,
                        color: red,
                    },
                    GradientStop {
                        offset: 0.49,
                        color: red,
                    },
                    GradientStop {
                        offset: 0.51,
                        color: blue,
                    },
                    GradientStop {
                        offset: 1.0,
                        color: blue,
                    },
                ],
                (true, true),
            )),
            Some(Stroke::new(Paint::Solid(Color::BLACK), 2.0)),
        );
        rotated.rotation_deg = 90.0;
        let layout = layout_presentation(&render_input(vec![slide((28.0, 24.0), vec![rotated])]))
            .expect("lower rotated gradient");
        let png =
            oxml_pdf::render_page_to_png(&layout, 0, 72.0).expect("rasterise rotated gradient");
        let pixmap = tiny_skia::Pixmap::decode_png(&png).expect("decode rotated gradient");
        let rgb = |x, y| {
            let pixel = pixmap.pixel(x, y).expect("sample lies inside page");
            (pixel.red(), pixel.green(), pixel.blue())
        };

        assert_eq!(rgb(14, 7), (255, 0, 0));
        assert_eq!(rgb(14, 15), (0, 0, 255));
        assert_eq!(rgb(11, 11), (0, 0, 0));
        assert_eq!(rgb(8, 8), (255, 255, 255));
    }

    #[test]
    fn solid_gradient_and_outlined_shapes_rasterise_at_sampled_pixels() {
        let red = color(1.0, 0.0, 0.0);
        let blue = color(0.0, 0.0, 1.0);
        let green = color(0.0, 1.0, 0.0);
        let gradient = Paint::linear(
            Point { x: 0.0, y: 0.0 },
            Point { x: 8.0, y: 0.0 },
            vec![
                GradientStop {
                    offset: 0.0,
                    color: red,
                },
                GradientStop {
                    offset: 0.49,
                    color: red,
                },
                GradientStop {
                    offset: 0.51,
                    color: blue,
                },
                GradientStop {
                    offset: 1.0,
                    color: blue,
                },
            ],
            (true, true),
        );
        let input = render_input(vec![slide(
            (40.0, 14.0),
            vec![
                shape(
                    Rect {
                        x: 2.0,
                        y: 2.0,
                        width: 8.0,
                        height: 8.0,
                    },
                    ResolvedGeometry::Rectangle,
                    Some(Paint::Solid(red)),
                    None,
                ),
                shape(
                    Rect {
                        x: 14.0,
                        y: 2.0,
                        width: 8.0,
                        height: 8.0,
                    },
                    ResolvedGeometry::Rectangle,
                    Some(gradient),
                    None,
                ),
                shape(
                    Rect {
                        x: 26.0,
                        y: 2.0,
                        width: 8.0,
                        height: 8.0,
                    },
                    ResolvedGeometry::Rectangle,
                    None,
                    Some(Stroke::new(Paint::Solid(green), 2.0)),
                ),
            ],
        )]);

        let layout = layout_presentation(&input).expect("lower shape slide");
        let png = oxml_pdf::render_page_to_png(&layout, 0, 72.0).expect("rasterise shape slide");
        let pixmap = tiny_skia::Pixmap::decode_png(&png).expect("decode shape slide");
        let rgb = |x, y| {
            let pixel = pixmap.pixel(x, y).expect("sample lies inside page");
            (pixel.red(), pixel.green(), pixel.blue())
        };

        assert_eq!(rgb(5, 5), (255, 0, 0));
        assert_eq!(rgb(15, 5), (255, 0, 0));
        assert_eq!(rgb(20, 5), (0, 0, 255));
        assert_eq!(rgb(26, 5), (0, 255, 0));
        assert_eq!(rgb(30, 5), (255, 255, 255));
        assert_eq!(rgb(38, 5), (255, 255, 255));
    }

    #[test]
    fn preset_and_custom_geometry_lower_to_ordered_paths() {
        let first = Path {
            commands: vec![
                PathCommand::MoveTo(Point { x: 0.0, y: 0.0 }),
                PathCommand::LineTo(Point { x: 4.0, y: 0.0 }),
            ],
            fill_rule: FillRule::NonZero,
        };
        let second = Path {
            commands: vec![
                PathCommand::MoveTo(Point { x: 0.0, y: 1.0 }),
                PathCommand::LineTo(Point { x: 4.0, y: 1.0 }),
            ],
            fill_rule: FillRule::EvenOdd,
        };
        let fill = Paint::Solid(Color::BLACK);
        let line = Stroke::new(Paint::Solid(Color::WHITE), 2.0);
        let input = render_input(vec![slide(
            (20.0, 20.0),
            vec![
                shape(
                    Rect {
                        x: 2.0,
                        y: 3.0,
                        width: 4.0,
                        height: 5.0,
                    },
                    ResolvedGeometry::Rectangle,
                    Some(fill.clone()),
                    Some(line.clone()),
                ),
                shape(
                    Rect {
                        x: 8.0,
                        y: 9.0,
                        width: 4.0,
                        height: 5.0,
                    },
                    ResolvedGeometry::Custom {
                        paths: vec![first.clone(), second.clone()],
                        text_rect: None,
                    },
                    Some(fill.clone()),
                    Some(line.clone()),
                ),
            ],
        )]);

        let page = layout_slide(&input, 0).expect("lower first slide");
        assert_eq!(page.elements.len(), 2);
        let rectangle = only_group(&page.elements[0]);
        assert_eq!(
            rectangle.transform,
            Transform {
                e: 2.0,
                f: 3.0,
                ..Transform::IDENTITY
            }
        );
        let PositionedElement::Path(rectangle) = &rectangle.children[0] else {
            panic!("rectangle should lower to a path");
        };
        assert_eq!(
            rectangle.path,
            Path::rect(Rect {
                x: 0.0,
                y: 0.0,
                width: 4.0,
                height: 5.0
            })
        );
        assert_eq!(rectangle.fill, Some(fill.clone()));
        assert_eq!(rectangle.stroke, Some(line.clone()));

        let custom = only_group(&page.elements[1]);
        assert_eq!(custom.children.len(), 2);
        for (element, expected) in custom.children.iter().zip([first, second]) {
            let PositionedElement::Path(element) = element else {
                panic!("custom geometry should lower to paths");
            };
            assert_eq!(element.path, expected);
            assert_eq!(element.fill, Some(fill.clone()));
            assert_eq!(element.stroke, Some(line.clone()));
        }
    }

    #[test]
    fn bounds_fallback_emits_a_visible_black_outline() {
        let input = render_input(vec![slide(
            (20.0, 20.0),
            vec![shape(
                Rect {
                    x: 2.0,
                    y: 3.0,
                    width: 4.0,
                    height: 5.0,
                },
                ResolvedGeometry::BoundsFallback,
                None,
                None,
            )],
        )]);

        let page = layout_slide(&input, 0).expect("lower fallback slide");
        let group = only_group(&page.elements[0]);
        let PositionedElement::Path(path) = &group.children[0] else {
            panic!("fallback should lower to a path");
        };
        assert_eq!(path.fill, None);
        assert_eq!(
            path.stroke,
            Some(Stroke::new(Paint::Solid(Color::BLACK), 1.0))
        );
    }

    #[test]
    fn layout_slide_rejects_an_out_of_range_index() {
        let input = render_input(vec![slide((20.0, 10.0), Vec::new())]);

        assert_eq!(
            layout_slide(&input, 4).unwrap_err(),
            RenderInputError::SlideIndexOutOfBounds {
                index: 4,
                slide_count: 1,
            }
        );
    }

    #[test]
    fn layout_presentation_preserves_page_order_and_diagnostics() {
        let mut first = slide((20.0, 10.0), Vec::new());
        first.diagnostics.push(Diagnostic {
            message: "first diagnostic".to_owned(),
        });
        let mut second = slide((30.0, 15.0), Vec::new());
        second.diagnostics.push(Diagnostic {
            message: "second diagnostic".to_owned(),
        });
        let mut input = render_input(vec![first, second]);
        input.metadata = Some(DocumentMetadata {
            title: Some("shape deck".to_owned()),
            author: Some("rpptx-render".to_owned()),
            ..DocumentMetadata::default()
        });

        let layout = layout_presentation(&input).expect("lower presentation");
        assert_eq!(layout.pages.len(), 2);
        assert_eq!(
            (
                layout.pages[0].page_number,
                layout.pages[0].width,
                layout.pages[0].height
            ),
            (1, 20.0, 10.0)
        );
        assert_eq!(
            (
                layout.pages[1].page_number,
                layout.pages[1].width,
                layout.pages[1].height
            ),
            (2, 30.0, 15.0)
        );
        assert_eq!(
            layout
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.title.as_deref()),
            Some("shape deck")
        );
        assert_eq!(
            layout
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec!["first diagnostic", "second diagnostic"]
        );
        assert!(layout.fonts.is_empty());
        assert!(layout.outlines.is_empty());
    }

    #[test]
    fn same_relationship_id_resolves_independently_in_all_three_scopes() {
        let relationships = RelScopes {
            slide: HashMap::from([("rId2".to_owned(), relationship("slide.png"))]),
            layout: HashMap::from([("rId2".to_owned(), relationship("layout.png"))]),
            master: HashMap::from([("rId2".to_owned(), relationship("master.png"))]),
        };
        let package_media = HashMap::from([
            ("slide.png".to_owned(), media(b"slide image")),
            ("layout.png".to_owned(), media(b"layout image")),
            ("master.png".to_owned(), media(b"master image")),
        ]);
        let mut deck_media = HashMap::new();

        let slide = resolve_media_relationship(
            &relationships,
            RelScope::Slide,
            "rId2",
            &package_media,
            &mut deck_media,
        )
        .unwrap();
        let layout = resolve_media_relationship(
            &relationships,
            RelScope::Layout,
            "rId2",
            &package_media,
            &mut deck_media,
        )
        .unwrap();
        let master = resolve_media_relationship(
            &relationships,
            RelScope::Master,
            "rId2",
            &package_media,
            &mut deck_media,
        )
        .unwrap();

        assert_eq!(slide, MediaId::from_bytes(b"slide image"));
        assert_eq!(layout, MediaId::from_bytes(b"layout image"));
        assert_eq!(master, MediaId::from_bytes(b"master image"));
        assert_eq!(deck_media.len(), 3);
    }

    #[test]
    fn equal_media_bytes_deduplicate_to_one_media_entry() {
        let relationships = RelScopes {
            slide: HashMap::from([
                ("rId1".to_owned(), relationship("logo-a.png")),
                ("rId2".to_owned(), relationship("logo-b.png")),
            ]),
            ..RelScopes::default()
        };
        let package_media = HashMap::from([
            ("logo-a.png".to_owned(), media(b"shared logo")),
            ("logo-b.png".to_owned(), media(b"shared logo")),
        ]);
        let mut deck_media = HashMap::new();

        let first = resolve_media_relationship(
            &relationships,
            RelScope::Slide,
            "rId1",
            &package_media,
            &mut deck_media,
        )
        .unwrap();
        let second = resolve_media_relationship(
            &relationships,
            RelScope::Slide,
            "rId2",
            &package_media,
            &mut deck_media,
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(deck_media.len(), 1);
    }

    #[test]
    fn missing_relationship_reports_scope_and_id() {
        let error = resolve_media_relationship(
            &RelScopes::default(),
            RelScope::Layout,
            "rId9",
            &HashMap::new(),
            &mut HashMap::new(),
        )
        .unwrap_err();

        assert_eq!(
            error,
            RenderInputError::MissingRelationship {
                scope: RelScope::Layout,
                relationship_id: "rId9".to_owned(),
            }
        );
        assert!(error.to_string().contains("layout"));
        assert!(error.to_string().contains("rId9"));
    }

    #[test]
    fn render_input_contains_only_resolved_slides() {
        let input = RenderInput {
            slides: Vec::<ResolvedSlide>::new(),
            media: HashMap::new(),
            fonts: Vec::new(),
            metadata: None,
        };

        assert!(input.slides.is_empty());
        assert_eq!(
            std::any::type_name_of_val(&input.slides),
            "alloc::vec::Vec<rpptx_layout::ResolvedSlide>"
        );
    }

    #[test]
    fn rpptx_render_dependency_direction_is_one_way() {
        let manifest = include_str!("../Cargo.toml");
        for dependency in [
            "oxml-drawing.workspace = true",
            "oxml-layout.workspace = true",
            "oxml-media.workspace = true",
            "rpptx-layout.workspace = true",
            "rpptx-oxml.workspace = true",
        ] {
            assert!(manifest.contains(dependency), "missing {dependency}");
        }
        for oxml_manifest in [
            include_str!("../../oxml-core/Cargo.toml"),
            include_str!("../../oxml-drawing/Cargo.toml"),
            include_str!("../../oxml-layout/Cargo.toml"),
            include_str!("../../oxml-media/Cargo.toml"),
            include_str!("../../oxml-opc/Cargo.toml"),
            include_str!("../../oxml-pdf/Cargo.toml"),
        ] {
            assert!(!oxml_manifest.contains("rpptx-render"));
        }
        assert!(manifest.contains("version = \"0.0.0\""));
        assert!(manifest.contains("publish = false"));
    }
}

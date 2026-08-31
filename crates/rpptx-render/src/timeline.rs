//! Composition of evaluated slide frames and transitions.

use oxml_layout::{
    Diagnostic, FontManager, GroupElement, PageFrame, Path, PathCommand, PathElement,
    PositionedElement, Rect, Transform,
};
use rpptx_layout::timeline::{EvaluatedTransition, ResolvedTimelineSlide};
use rpptx_layout::{ResolvedGeometry, ResolvedShape, ResolvedSlideTextDirections};
use rpptx_oxml::timing::TransitionEffect;

use crate::{
    RenderInput, RenderInputError, layout_resolved_slide_with_fonts_text_directions_and_states,
};

/// One composed page plus stable approximation diagnostics.
#[derive(Clone, Debug)]
pub struct TimelinePageFrame {
    pub page: PageFrame,
    pub diagnostics: Vec<Diagnostic>,
}

/// Lower one evaluated slide through the existing deterministic shape and text path.
pub fn layout_timeline_slide_deterministic(
    input: &RenderInput,
    index: usize,
    timeline: &ResolvedTimelineSlide,
    text_directions: Option<&ResolvedSlideTextDirections>,
) -> Result<PageFrame, RenderInputError> {
    let mut fonts =
        FontManager::new_deterministic().map_err(|error| RenderInputError::TextLayout {
            detail: error.to_string(),
        })?;
    fonts.load_additional_fonts(&input.fonts);
    layout_timeline_slide_with_font_manager(input, index, timeline, text_directions, &mut fonts)
}

/// Lower one evaluated slide while retaining the caller-owned deterministic font identity.
pub fn layout_timeline_slide_with_font_manager(
    input: &RenderInput,
    index: usize,
    timeline: &ResolvedTimelineSlide,
    text_directions: Option<&ResolvedSlideTextDirections>,
    fonts: &mut FontManager,
) -> Result<PageFrame, RenderInputError> {
    if index >= input.slides.len() {
        return Err(RenderInputError::SlideIndexOutOfBounds {
            index,
            slide_count: input.slides.len(),
        });
    }
    layout_resolved_slide_with_fonts_text_directions_and_states(
        input,
        &timeline.slide,
        index + 1,
        fonts,
        text_directions.map(Vec::as_slice),
        Some(&timeline.shape_states),
    )
}

/// Compose an evaluated incoming page with an optional outgoing page.
pub fn compose_transition(
    mut incoming: PageFrame,
    outgoing: Option<&PageFrame>,
    transition: Option<&EvaluatedTransition>,
) -> TimelinePageFrame {
    let Some(transition) = transition else {
        return TimelinePageFrame {
            page: incoming,
            diagnostics: Vec::new(),
        };
    };
    if matches!(transition.effect, TransitionEffect::Other(_)) {
        return TimelinePageFrame {
            page: incoming,
            diagnostics: vec![Diagnostic {
                message: "unsupported slide transition".to_owned(),
            }],
        };
    }
    let direction = match transition_direction(transition) {
        Ok(direction) => direction,
        Err(message) => {
            return TimelinePageFrame {
                page: incoming,
                diagnostics: vec![Diagnostic { message }],
            };
        }
    };
    if matches!(transition.effect, TransitionEffect::Cut) || transition.progress >= 1.0 {
        return TimelinePageFrame {
            page: incoming,
            diagnostics: Vec::new(),
        };
    }
    let Some(outgoing) = outgoing else {
        return TimelinePageFrame {
            page: incoming,
            diagnostics: vec![Diagnostic {
                message: "transition requires an outgoing slide".to_owned(),
            }],
        };
    };
    let progress = f64::from(transition.progress.clamp(0.0, 1.0));
    let incoming_elements = take_page_content(&mut incoming);
    let outgoing_elements = page_content(outgoing);
    let groups = match transition.effect {
        TransitionEffect::Fade | TransitionEffect::Morph => vec![
            page_group(outgoing_elements, Transform::IDENTITY, 1.0, None),
            page_group(incoming_elements, Transform::IDENTITY, progress, None),
        ],
        TransitionEffect::Wipe => vec![
            page_group(outgoing_elements, Transform::IDENTITY, 1.0, None),
            page_group(
                incoming_elements,
                Transform::IDENTITY,
                1.0,
                Some(Path::rect(wipe_rect(
                    incoming.width,
                    incoming.height,
                    progress,
                    direction,
                ))),
            ),
        ],
        TransitionEffect::Push => vec![
            page_group(
                outgoing_elements,
                push_translation(incoming.width, incoming.height, progress, direction, false),
                1.0,
                None,
            ),
            page_group(
                incoming_elements,
                push_translation(incoming.width, incoming.height, progress, direction, true),
                1.0,
                None,
            ),
        ],
        TransitionEffect::Zoom => vec![
            if direction == "out" {
                page_group(
                    outgoing_elements,
                    scale_about(
                        1.0 - 0.5 * progress,
                        incoming.width / 2.0,
                        incoming.height / 2.0,
                    ),
                    1.0 - progress,
                    None,
                )
            } else {
                page_group(outgoing_elements, Transform::IDENTITY, 1.0 - progress, None)
            },
            if direction == "out" {
                page_group(incoming_elements, Transform::IDENTITY, progress, None)
            } else {
                page_group(
                    incoming_elements,
                    scale_about(
                        0.5 + 0.5 * progress,
                        incoming.width / 2.0,
                        incoming.height / 2.0,
                    ),
                    progress,
                    None,
                )
            },
        ],
        TransitionEffect::Cut => unreachable!("cut returned before composition"),
        TransitionEffect::Other(_) => {
            return TimelinePageFrame {
                page: incoming,
                diagnostics: vec![Diagnostic {
                    message: "unsupported slide transition".to_owned(),
                }],
            };
        }
    };
    incoming.elements = groups;
    let diagnostics = if matches!(transition.effect, TransitionEffect::Morph) {
        vec![Diagnostic {
            message: "morph has no compatible explicit-name pairs and uses crossfade".to_owned(),
        }]
    } else {
        Vec::new()
    };
    TimelinePageFrame {
        page: incoming,
        diagnostics,
    }
}

fn transition_direction(transition: &EvaluatedTransition) -> Result<&str, String> {
    let explicit = transition
        .parameters
        .iter()
        .find(|parameter| parameter.name == "dir")
        .map(|parameter| parameter.value.as_str());
    let (default, supported): (&str, &[&str]) = match transition.effect {
        TransitionEffect::Wipe | TransitionEffect::Push => ("l", &["l", "r", "u", "d"]),
        TransitionEffect::Zoom => ("in", &["in", "out"]),
        _ => return Ok(""),
    };
    let direction = explicit.unwrap_or(default);
    if supported.contains(&direction) {
        Ok(direction)
    } else {
        Err(format!(
            "unsupported {} transition direction {direction}",
            match transition.effect {
                TransitionEffect::Wipe => "wipe",
                TransitionEffect::Push => "push",
                TransitionEffect::Zoom => "zoom",
                _ => unreachable!("non-directional transition returned above"),
            }
        ))
    }
}

fn wipe_rect(width: f64, height: f64, progress: f64, direction: &str) -> Rect {
    match direction {
        "r" => Rect {
            x: width * (1.0 - progress),
            y: 0.0,
            width: width * progress,
            height,
        },
        "u" => Rect {
            x: 0.0,
            y: height * (1.0 - progress),
            width,
            height: height * progress,
        },
        "d" => Rect {
            x: 0.0,
            y: 0.0,
            width,
            height: height * progress,
        },
        _ => Rect {
            x: 0.0,
            y: 0.0,
            width: width * progress,
            height,
        },
    }
}

fn push_translation(
    width: f64,
    height: f64,
    progress: f64,
    direction: &str,
    incoming: bool,
) -> Transform {
    let remaining = 1.0 - progress;
    let (x, y) = match (direction, incoming) {
        ("r", false) => (width * progress, 0.0),
        ("r", true) => (-width * remaining, 0.0),
        ("u", false) => (0.0, -height * progress),
        ("u", true) => (0.0, height * remaining),
        ("d", false) => (0.0, height * progress),
        ("d", true) => (0.0, -height * remaining),
        (_, false) => (-width * progress, 0.0),
        (_, true) => (width * remaining, 0.0),
    };
    translation(x, y)
}

/// Compose the bounded explicit-name morph subset, crossfading every fallback.
pub fn compose_morph_transition(
    mut incoming_page: PageFrame,
    incoming: &ResolvedTimelineSlide,
    outgoing_page: &PageFrame,
    outgoing: &ResolvedTimelineSlide,
    transition: &EvaluatedTransition,
) -> TimelinePageFrame {
    if !matches!(transition.morph_option.as_deref(), None | Some("byObject")) {
        let option = transition.morph_option.as_deref().unwrap_or_default();
        let mut fallback = transition.clone();
        fallback.effect = TransitionEffect::Fade;
        let mut frame = compose_transition(incoming_page, Some(outgoing_page), Some(&fallback));
        frame.diagnostics.push(Diagnostic {
            message: format!("unsupported morph option {option} uses crossfade"),
        });
        return frame;
    }
    let progress = f64::from(transition.progress.clamp(0.0, 1.0));
    if progress >= 1.0 {
        return TimelinePageFrame {
            page: incoming_page,
            diagnostics: Vec::new(),
        };
    }
    let incoming_offset = incoming_page
        .elements
        .len()
        .saturating_sub(incoming.identities.len());
    let outgoing_offset = outgoing_page
        .elements
        .len()
        .saturating_sub(outgoing.identities.len());
    let mut used_incoming = vec![false; incoming.identities.len()];
    let mut elements = Vec::new();
    let mut diagnostics = Vec::new();
    if let Some(background) = outgoing_page.background.clone() {
        elements.push(with_opacity(
            background_element(outgoing_page.width, outgoing_page.height, background),
            1.0,
        ));
    }
    if let Some(background) = incoming_page.background.take() {
        elements.push(with_opacity(
            background_element(incoming_page.width, incoming_page.height, background),
            progress,
        ));
    }

    for (outgoing_index, outgoing_identity) in outgoing.identities.iter().enumerate() {
        let Some(name) = outgoing_identity
            .name
            .as_deref()
            .filter(|name| name.starts_with("!!"))
        else {
            if let Some(element) = outgoing_page.elements.get(outgoing_offset + outgoing_index) {
                elements.push(with_opacity(element.clone(), 1.0 - progress));
            }
            continue;
        };
        let matching = incoming
            .identities
            .iter()
            .enumerate()
            .find(|(index, identity)| {
                !used_incoming[*index]
                    && identity.name.as_deref() == Some(name)
                    && identity.source == outgoing_identity.source
            });
        let Some((incoming_index, _)) = matching else {
            diagnostics.push(Diagnostic {
                message: format!("morph shape {name} has no incoming match and uses crossfade"),
            });
            if let Some(element) = outgoing_page.elements.get(outgoing_offset + outgoing_index) {
                elements.push(with_opacity(element.clone(), 1.0 - progress));
            }
            continue;
        };
        used_incoming[incoming_index] = true;
        let resolved_pair = outgoing
            .slide
            .shapes
            .get(outgoing_index)
            .zip(incoming.slide.shapes.get(incoming_index));
        let compatible = resolved_pair.is_some_and(geometry_compatible);
        let pair = outgoing_page
            .elements
            .get(outgoing_offset + outgoing_index)
            .zip(incoming_page.elements.get(incoming_offset + incoming_index));
        if compatible
            && let Some((outgoing_element, incoming_element)) = pair
            && let (
                PositionedElement::Group(outgoing_group),
                PositionedElement::Group(incoming_group),
            ) = (outgoing_element, incoming_element)
        {
            let (outgoing_shape, incoming_shape) = resolved_pair.expect("compatible pair exists");
            let width = interpolate(
                outgoing_shape.bounds.width,
                incoming_shape.bounds.width,
                progress,
            );
            let height = interpolate(
                outgoing_shape.bounds.height,
                incoming_shape.bounds.height,
                progress,
            );
            let transform =
                interpolate_transform(outgoing_group.transform, incoming_group.transform, progress);
            let mut outgoing_group = outgoing_group.clone();
            outgoing_group.transform = Transform {
                a: width / outgoing_shape.bounds.width,
                d: height / outgoing_shape.bounds.height,
                ..Transform::IDENTITY
            }
            .then(transform);
            let mut incoming_group = incoming_group.clone();
            incoming_group.transform = Transform {
                a: width / incoming_shape.bounds.width,
                d: height / incoming_shape.bounds.height,
                ..Transform::IDENTITY
            }
            .then(transform);
            incoming_group.opacity *= progress;
            elements.push(page_group(
                vec![
                    PositionedElement::Group(outgoing_group),
                    PositionedElement::Group(incoming_group),
                ],
                Transform::IDENTITY,
                1.0,
                None,
            ));
            continue;
        }
        diagnostics.push(Diagnostic {
            message: format!("morph shape {name} has incompatible geometry and uses crossfade"),
        });
        if let Some((outgoing_element, incoming_element)) = pair {
            elements.push(with_opacity(outgoing_element.clone(), 1.0 - progress));
            elements.push(with_opacity(incoming_element.clone(), progress));
        }
    }
    for (index, used) in used_incoming.into_iter().enumerate() {
        if !used && let Some(element) = incoming_page.elements.get(incoming_offset + index) {
            if let Some(name) = incoming.identities[index]
                .name
                .as_deref()
                .filter(|name| name.starts_with("!!"))
            {
                diagnostics.push(Diagnostic {
                    message: format!("morph shape {name} has no outgoing match and uses crossfade"),
                });
            }
            elements.push(with_opacity(element.clone(), progress));
        }
    }
    incoming_page.elements = elements;
    TimelinePageFrame {
        page: incoming_page,
        diagnostics,
    }
}

fn rect_is_finite(rect: Rect) -> bool {
    [rect.x, rect.y, rect.width, rect.height]
        .into_iter()
        .all(f64::is_finite)
}

fn geometry_compatible((outgoing, incoming): (&ResolvedShape, &ResolvedShape)) -> bool {
    if !rect_is_finite(outgoing.bounds)
        || !rect_is_finite(incoming.bounds)
        || outgoing.bounds.width <= 0.0
        || outgoing.bounds.height <= 0.0
        || incoming.bounds.width <= 0.0
        || incoming.bounds.height <= 0.0
    {
        return false;
    }
    match (&outgoing.geometry, &incoming.geometry) {
        (ResolvedGeometry::Rectangle, ResolvedGeometry::Rectangle)
        | (ResolvedGeometry::BoundsFallback, ResolvedGeometry::BoundsFallback) => true,
        (
            ResolvedGeometry::Custom {
                paths: outgoing_paths,
                text_rect: outgoing_text_rect,
            },
            ResolvedGeometry::Custom {
                paths: incoming_paths,
                text_rect: incoming_text_rect,
            },
        ) => {
            outgoing_paths.len() == incoming_paths.len()
                && outgoing_paths.iter().zip(incoming_paths).all(
                    |(outgoing_path, incoming_path)| {
                        outgoing_path.fill_rule == incoming_path.fill_rule
                            && outgoing_path.commands.len() == incoming_path.commands.len()
                            && outgoing_path
                                .commands
                                .iter()
                                .zip(&incoming_path.commands)
                                .all(|(outgoing_command, incoming_command)| {
                                    path_command_compatible(
                                        outgoing_command,
                                        outgoing.bounds,
                                        incoming_command,
                                        incoming.bounds,
                                    )
                                })
                    },
                )
                && optional_rect_compatible(
                    *outgoing_text_rect,
                    outgoing.bounds,
                    *incoming_text_rect,
                    incoming.bounds,
                )
        }
        _ => false,
    }
}

fn path_command_compatible(
    outgoing: &PathCommand,
    outgoing_bounds: Rect,
    incoming: &PathCommand,
    incoming_bounds: Rect,
) -> bool {
    match (outgoing, incoming) {
        (PathCommand::MoveTo(outgoing), PathCommand::MoveTo(incoming))
        | (PathCommand::LineTo(outgoing), PathCommand::LineTo(incoming)) => {
            point_compatible(*outgoing, outgoing_bounds, *incoming, incoming_bounds)
        }
        (
            PathCommand::CurveTo {
                c1: outgoing_c1,
                c2: outgoing_c2,
                to: outgoing_to,
            },
            PathCommand::CurveTo {
                c1: incoming_c1,
                c2: incoming_c2,
                to: incoming_to,
            },
        ) => {
            point_compatible(*outgoing_c1, outgoing_bounds, *incoming_c1, incoming_bounds)
                && point_compatible(*outgoing_c2, outgoing_bounds, *incoming_c2, incoming_bounds)
                && point_compatible(*outgoing_to, outgoing_bounds, *incoming_to, incoming_bounds)
        }
        (PathCommand::Close, PathCommand::Close) => true,
        _ => false,
    }
}

fn point_compatible(
    outgoing: oxml_layout::Point,
    outgoing_bounds: Rect,
    incoming: oxml_layout::Point,
    incoming_bounds: Rect,
) -> bool {
    normalized_close(
        outgoing.x / outgoing_bounds.width,
        incoming.x / incoming_bounds.width,
    ) && normalized_close(
        outgoing.y / outgoing_bounds.height,
        incoming.y / incoming_bounds.height,
    )
}

fn optional_rect_compatible(
    outgoing: Option<Rect>,
    outgoing_bounds: Rect,
    incoming: Option<Rect>,
    incoming_bounds: Rect,
) -> bool {
    match (outgoing, incoming) {
        (None, None) => true,
        (Some(outgoing), Some(incoming)) => {
            normalized_close(
                outgoing.x / outgoing_bounds.width,
                incoming.x / incoming_bounds.width,
            ) && normalized_close(
                outgoing.y / outgoing_bounds.height,
                incoming.y / incoming_bounds.height,
            ) && normalized_close(
                outgoing.width / outgoing_bounds.width,
                incoming.width / incoming_bounds.width,
            ) && normalized_close(
                outgoing.height / outgoing_bounds.height,
                incoming.height / incoming_bounds.height,
            )
        }
        _ => false,
    }
}

fn normalized_close(left: f64, right: f64) -> bool {
    left.is_finite() && right.is_finite() && (left - right).abs() <= 1e-9
}

fn interpolate(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress
}

fn interpolate_transform(from: Transform, to: Transform, progress: f64) -> Transform {
    Transform {
        a: interpolate(from.a, to.a, progress),
        b: interpolate(from.b, to.b, progress),
        c: interpolate(from.c, to.c, progress),
        d: interpolate(from.d, to.d, progress),
        e: interpolate(from.e, to.e, progress),
        f: interpolate(from.f, to.f, progress),
    }
}

fn with_opacity(mut element: PositionedElement, opacity: f64) -> PositionedElement {
    if let PositionedElement::Group(group) = &mut element {
        group.opacity *= opacity;
        return element;
    }
    page_group(vec![element], Transform::IDENTITY, opacity, None)
}

fn page_content(page: &PageFrame) -> Vec<PositionedElement> {
    let mut elements = page
        .background
        .clone()
        .map(|background| background_element(page.width, page.height, background))
        .into_iter()
        .collect::<Vec<_>>();
    elements.extend(page.elements.clone());
    elements
}

fn take_page_content(page: &mut PageFrame) -> Vec<PositionedElement> {
    let mut elements = page
        .background
        .take()
        .map(|background| background_element(page.width, page.height, background))
        .into_iter()
        .collect::<Vec<_>>();
    elements.append(&mut page.elements);
    elements
}

fn background_element(
    width: f64,
    height: f64,
    background: oxml_layout::Paint,
) -> PositionedElement {
    PositionedElement::Path(PathElement {
        path: Path::rect(Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }),
        fill: Some(background),
        stroke: None,
    })
}

fn page_group(
    children: Vec<PositionedElement>,
    transform: Transform,
    opacity: f64,
    clip: Option<Path>,
) -> PositionedElement {
    PositionedElement::Group(GroupElement {
        transform,
        clip,
        opacity,
        effects: Vec::new(),
        children,
    })
}

fn translation(x: f64, y: f64) -> Transform {
    Transform {
        e: x,
        f: y,
        ..Transform::IDENTITY
    }
}

fn scale_about(scale: f64, center_x: f64, center_y: f64) -> Transform {
    Transform {
        a: scale,
        d: scale,
        e: center_x * (1.0 - scale),
        f: center_y * (1.0 - scale),
        ..Transform::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use oxml_layout::{
        FillRule, GroupElement, PageFrame, Path, PathCommand, Point, PositionedElement, Rect,
        Transform,
    };
    use rpptx_layout::timeline::{
        EvaluatedFrameState, EvaluatedShapeState, EvaluatedTransition, ResolvedShapeIdentity,
        ResolvedTimelineSlide,
    };
    use rpptx_layout::{
        FlattenedSource, ResolvedContent, ResolvedGeometry, ResolvedShape, ResolvedSlide,
    };
    use rpptx_oxml::timing::{TransitionEffect, TransitionParameter};

    use super::{compose_morph_transition, compose_transition};

    #[test]
    fn fade_transition_keeps_outgoing_opaque_and_fades_incoming_at_exact_progress() {
        let outgoing = PageFrame::new(
            1,
            100.0,
            50.0,
            vec![PositionedElement::Group(GroupElement {
                transform: Transform::IDENTITY,
                clip: None,
                opacity: 1.0,
                effects: Vec::new(),
                children: Vec::new(),
            })],
        );
        let incoming = PageFrame::new(2, 100.0, 50.0, Vec::new());
        let frame = compose_transition(
            incoming,
            Some(&outgoing),
            Some(&EvaluatedTransition {
                effect: TransitionEffect::Fade,
                progress: 0.25,
                duration_ms: 1000,
                parameters: Vec::new(),
                morph_option: None,
            }),
        );

        assert_eq!(frame.page.elements.len(), 2);
        let PositionedElement::Group(first) = &frame.page.elements[0] else {
            panic!("expected outgoing group")
        };
        let PositionedElement::Group(second) = &frame.page.elements[1] else {
            panic!("expected incoming group")
        };
        assert_eq!(first.opacity, 1.0);
        assert_eq!(second.opacity, 0.25);
    }

    #[test]
    fn cut_wipe_push_and_zoom_have_finite_bounded_compositions() {
        for effect in [
            TransitionEffect::Cut,
            TransitionEffect::Wipe,
            TransitionEffect::Push,
            TransitionEffect::Zoom,
        ] {
            let outgoing = PageFrame::new(1, 100.0, 50.0, Vec::new());
            let incoming = PageFrame::new(2, 100.0, 50.0, Vec::new());
            let frame = compose_transition(
                incoming,
                Some(&outgoing),
                Some(&EvaluatedTransition {
                    effect: effect.clone(),
                    progress: 0.5,
                    duration_ms: 1000,
                    parameters: Vec::new(),
                    morph_option: None,
                }),
            );
            assert!(frame.diagnostics.is_empty());
            if effect != TransitionEffect::Cut {
                assert_eq!(frame.page.elements.len(), 2);
            }
        }
    }

    #[test]
    fn invalid_transition_directions_diagnose_without_composition() {
        for (effect, name) in [
            (TransitionEffect::Wipe, "wipe"),
            (TransitionEffect::Push, "push"),
            (TransitionEffect::Zoom, "zoom"),
        ] {
            for progress in [0.5, 1.0] {
                let incoming = PageFrame::new(
                    2,
                    100.0,
                    50.0,
                    vec![PositionedElement::Group(GroupElement {
                        transform: Transform::IDENTITY,
                        clip: None,
                        opacity: 1.0,
                        effects: Vec::new(),
                        children: Vec::new(),
                    })],
                );
                let frame = compose_transition(
                    incoming.clone(),
                    Some(&PageFrame::new(1, 100.0, 50.0, Vec::new())),
                    Some(&EvaluatedTransition {
                        effect: effect.clone(),
                        progress,
                        duration_ms: 1000,
                        parameters: vec![TransitionParameter {
                            name: "dir".to_owned(),
                            value: "diagonal".to_owned(),
                        }],
                        morph_option: None,
                    }),
                );

                assert_eq!(frame.page.width, incoming.width);
                assert_eq!(frame.page.height, incoming.height);
                assert_eq!(frame.page.elements, incoming.elements);
                assert_eq!(
                    frame.diagnostics[0].message,
                    format!("unsupported {name} transition direction diagonal")
                );
            }
        }

        let other = compose_transition(
            PageFrame::new(2, 100.0, 50.0, Vec::new()),
            None,
            Some(&EvaluatedTransition {
                effect: TransitionEffect::Other("producer".to_owned()),
                progress: 1.0,
                duration_ms: 1000,
                parameters: Vec::new(),
                morph_option: None,
            }),
        );
        assert_eq!(other.diagnostics[0].message, "unsupported slide transition");
    }

    #[test]
    fn explicit_name_morph_interpolates_compatible_shape_groups() {
        let mut outgoing_timeline = timeline_slide("!!Hero", 0.0);
        outgoing_timeline.slide.shapes[0].bounds.width = 20.0;
        outgoing_timeline.slide.shapes[0].bounds.height = 30.0;
        let incoming_timeline = timeline_slide("!!Hero", 100.0);
        let outgoing_page = page_with_translation(0.0);
        let incoming_page = page_with_translation(100.0);
        let frame = compose_morph_transition(
            incoming_page,
            &incoming_timeline,
            &outgoing_page,
            &outgoing_timeline,
            &EvaluatedTransition {
                effect: TransitionEffect::Morph,
                progress: 0.5,
                duration_ms: 1000,
                parameters: Vec::new(),
                morph_option: Some("byObject".to_owned()),
            },
        );

        assert!(frame.diagnostics.is_empty());
        let PositionedElement::Group(group) = &frame.page.elements[0] else {
            panic!("morph pair should remain one group")
        };
        assert_eq!(group.transform, Transform::IDENTITY);
        assert_eq!(group.children.len(), 2);
        let PositionedElement::Group(outgoing_group) = &group.children[0] else {
            panic!("morph pair should retain outgoing content")
        };
        let PositionedElement::Group(incoming_group) = &group.children[1] else {
            panic!("morph pair should retain incoming content")
        };
        assert_eq!(outgoing_group.transform.e, 50.0);
        assert_eq!(outgoing_group.transform.a, 0.75);
        assert_eq!(outgoing_group.transform.d, 2.0 / 3.0);
        assert_eq!(outgoing_group.opacity, 1.0);
        assert_eq!(incoming_group.transform.e, 50.0);
        assert_eq!(incoming_group.transform.a, 1.5);
        assert_eq!(incoming_group.transform.d, 2.0);
        assert_eq!(incoming_group.opacity, 0.5);
    }

    #[test]
    fn compatible_morph_keeps_outgoing_content_at_progress_zero() {
        let outgoing_timeline = timeline_slide("!!Hero", 0.0);
        let incoming_timeline = timeline_slide("!!Hero", 100.0);
        let outgoing_page = page_with_marker(0.0, 7.0);
        let incoming_page = page_with_marker(100.0, 9.0);
        let frame = compose_morph_transition(
            incoming_page,
            &incoming_timeline,
            &outgoing_page,
            &outgoing_timeline,
            &EvaluatedTransition {
                effect: TransitionEffect::Morph,
                progress: 0.0,
                duration_ms: 1000,
                parameters: Vec::new(),
                morph_option: Some("byObject".to_owned()),
            },
        );

        let PositionedElement::Group(pair) = &frame.page.elements[0] else {
            panic!("morph pair should remain one group")
        };
        let PositionedElement::Group(outgoing) = &pair.children[0] else {
            panic!("morph pair should retain outgoing content")
        };
        let PositionedElement::Group(incoming) = &pair.children[1] else {
            panic!("morph pair should retain incoming content")
        };
        let PositionedElement::Group(outgoing_marker) = &outgoing.children[0] else {
            panic!("expected outgoing marker")
        };
        let PositionedElement::Group(incoming_marker) = &incoming.children[0] else {
            panic!("expected incoming marker")
        };
        assert_eq!(outgoing.opacity, 1.0);
        assert_eq!(incoming.opacity, 0.0);
        assert_eq!(outgoing_marker.transform.e, 7.0);
        assert_eq!(incoming_marker.transform.e, 9.0);
    }

    #[test]
    fn word_and_character_morph_options_crossfade_with_diagnostics() {
        for option in ["byWord", "byChar"] {
            let outgoing_timeline = timeline_slide("!!Hero", 0.0);
            let incoming_timeline = timeline_slide("!!Hero", 100.0);
            let frame = compose_morph_transition(
                page_with_translation(100.0),
                &incoming_timeline,
                &page_with_translation(0.0),
                &outgoing_timeline,
                &EvaluatedTransition {
                    effect: TransitionEffect::Morph,
                    progress: 0.25,
                    duration_ms: 1000,
                    parameters: Vec::new(),
                    morph_option: Some(option.to_owned()),
                },
            );

            assert_eq!(frame.page.elements.len(), 2);
            let PositionedElement::Group(outgoing) = &frame.page.elements[0] else {
                panic!("expected outgoing crossfade group")
            };
            let PositionedElement::Group(incoming) = &frame.page.elements[1] else {
                panic!("expected incoming crossfade group")
            };
            assert_eq!(outgoing.opacity, 1.0);
            assert_eq!(incoming.opacity, 0.25);
            assert_eq!(
                frame.diagnostics[0].message,
                format!("unsupported morph option {option} uses crossfade")
            );
        }
    }

    #[test]
    fn zoom_in_and_out_transform_opposite_pages() {
        let compose = |direction: &str| {
            compose_transition(
                PageFrame::new(2, 100.0, 50.0, Vec::new()),
                Some(&PageFrame::new(1, 100.0, 50.0, Vec::new())),
                Some(&EvaluatedTransition {
                    effect: TransitionEffect::Zoom,
                    progress: 0.5,
                    duration_ms: 1000,
                    parameters: vec![TransitionParameter {
                        name: "dir".to_owned(),
                        value: direction.to_owned(),
                    }],
                    morph_option: None,
                }),
            )
        };

        let zoom_in = compose("in");
        let zoom_out = compose("out");
        let PositionedElement::Group(in_outgoing) = &zoom_in.page.elements[0] else {
            panic!("expected zoom-in outgoing group")
        };
        let PositionedElement::Group(in_incoming) = &zoom_in.page.elements[1] else {
            panic!("expected zoom-in incoming group")
        };
        let PositionedElement::Group(out_outgoing) = &zoom_out.page.elements[0] else {
            panic!("expected zoom-out outgoing group")
        };
        let PositionedElement::Group(out_incoming) = &zoom_out.page.elements[1] else {
            panic!("expected zoom-out incoming group")
        };
        assert_eq!(in_outgoing.transform.a, 1.0);
        assert_eq!(in_incoming.transform.a, 0.75);
        assert_eq!(out_outgoing.transform.a, 0.75);
        assert_eq!(out_incoming.transform.a, 1.0);
    }

    #[test]
    fn morph_reports_explicit_names_missing_on_either_slide() {
        let outgoing_timeline = timeline_slide("!!Outgoing", 0.0);
        let incoming_timeline = timeline_slide("!!Incoming", 100.0);
        let frame = compose_morph_transition(
            page_with_translation(100.0),
            &incoming_timeline,
            &page_with_translation(0.0),
            &outgoing_timeline,
            &EvaluatedTransition {
                effect: TransitionEffect::Morph,
                progress: 0.5,
                duration_ms: 1000,
                parameters: Vec::new(),
                morph_option: Some("byObject".to_owned()),
            },
        );

        assert_eq!(frame.page.elements.len(), 2);
        assert_eq!(frame.diagnostics.len(), 2);
        assert!(
            frame
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("!!Outgoing")
                    && diagnostic.message.contains("no incoming match"))
        );
        assert!(
            frame
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("!!Incoming")
                    && diagnostic.message.contains("no outgoing match"))
        );
    }

    #[test]
    fn morph_crossfades_custom_geometries_with_different_path_structure() {
        let mut outgoing_timeline = timeline_slide("!!Hero", 0.0);
        outgoing_timeline.slide.shapes[0].geometry = ResolvedGeometry::Custom {
            paths: vec![Path {
                commands: vec![
                    PathCommand::MoveTo(Point { x: 0.0, y: 0.0 }),
                    PathCommand::LineTo(Point { x: 10.0, y: 0.0 }),
                    PathCommand::LineTo(Point { x: 5.0, y: 10.0 }),
                    PathCommand::Close,
                ],
                fill_rule: FillRule::NonZero,
            }],
            text_rect: None,
        };
        let mut incoming_timeline = timeline_slide("!!Hero", 100.0);
        incoming_timeline.slide.shapes[0].geometry = ResolvedGeometry::Custom {
            paths: vec![Path::rect(Rect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            })],
            text_rect: None,
        };
        let frame = compose_morph_transition(
            page_with_translation(100.0),
            &incoming_timeline,
            &page_with_translation(0.0),
            &outgoing_timeline,
            &EvaluatedTransition {
                effect: TransitionEffect::Morph,
                progress: 0.5,
                duration_ms: 1000,
                parameters: Vec::new(),
                morph_option: Some("byObject".to_owned()),
            },
        );

        assert_eq!(frame.page.elements.len(), 2);
        assert_eq!(frame.diagnostics.len(), 1);
        assert!(
            frame.diagnostics[0]
                .message
                .contains("incompatible geometry")
        );
    }

    fn timeline_slide(name: &str, x: f64) -> ResolvedTimelineSlide {
        ResolvedTimelineSlide {
            slide: ResolvedSlide {
                size: (100.0, 50.0),
                background: None,
                shapes: vec![ResolvedShape {
                    group_transform: Transform::IDENTITY,
                    bounds: Rect {
                        x,
                        y: 0.0,
                        width: 10.0,
                        height: 10.0,
                    },
                    rotation_deg: 0.0,
                    flip_h: false,
                    flip_v: false,
                    geometry: ResolvedGeometry::Rectangle,
                    fill: None,
                    image_fill: None,
                    line: None,
                    head_end: None,
                    tail_end: None,
                    shadow: None,
                    content: ResolvedContent::None,
                    unsupported: None,
                }],
                diagnostics: Vec::new(),
            },
            identities: vec![ResolvedShapeIdentity {
                source: FlattenedSource::Slide,
                shape_id: Some(2),
                containing_group_ids: Vec::new(),
                name: Some(name.to_owned()),
            }],
            shape_states: vec![EvaluatedShapeState::default()],
            state: EvaluatedFrameState::default(),
        }
    }

    fn page_with_translation(x: f64) -> PageFrame {
        PageFrame::new(
            1,
            100.0,
            50.0,
            vec![PositionedElement::Group(GroupElement {
                transform: Transform {
                    e: x,
                    ..Transform::IDENTITY
                },
                clip: None,
                opacity: 1.0,
                effects: Vec::new(),
                children: Vec::new(),
            })],
        )
    }

    fn page_with_marker(x: f64, marker_x: f64) -> PageFrame {
        let mut page = page_with_translation(x);
        let PositionedElement::Group(group) = &mut page.elements[0] else {
            unreachable!("page helper creates a group")
        };
        group.children.push(PositionedElement::Group(GroupElement {
            transform: Transform {
                e: marker_x,
                ..Transform::IDENTITY
            },
            clip: None,
            opacity: 1.0,
            effects: Vec::new(),
            children: Vec::new(),
        }));
        page
    }
}

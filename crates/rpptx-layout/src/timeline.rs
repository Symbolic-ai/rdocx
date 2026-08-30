//! Deterministic evaluation of PresentationML timing trees.

use std::collections::HashMap;

use oxml_layout::{Diagnostic, Path, PathCommand, Point, Rect, Transform};
use rpptx_oxml::timing::{
    CT_SlideTransition, CT_Timing, CommonTimeNode, TimingAnimate, TimingCondition, TimingContainer,
    TimingDuration, TimingEffect, TimingEvent, TimingFill, TimingMotionPath, TimingNode,
    TimingNodeType, TimingSequence, TimingSet, TimingTarget, TransitionEffect, TransitionParameter,
    TransitionSpeed,
};

use crate::{FlattenedSource, ResolveError, ResolvedSlide};

/// Explicit slide-local timeline input supplied by a caller.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimelinePosition {
    pub elapsed_ms: u64,
    pub click_count: u32,
}

/// Evaluated renderer state for one shape target.
#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatedShapeState {
    pub visible: bool,
    pub opacity: f32,
    pub transform: Transform,
    /// Optional shape-local normalized reveal rectangle.
    pub clip: Option<Rect>,
    animation: EvaluatedAnimationTransform,
    oriented_clips: Vec<[Point; 4]>,
}

impl Default for EvaluatedShapeState {
    fn default() -> Self {
        Self {
            visible: true,
            opacity: 1.0,
            transform: Transform::IDENTITY,
            clip: None,
            animation: EvaluatedAnimationTransform::default(),
            oriented_clips: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EvaluatedAnimationTransform {
    scale_x: f64,
    scale_y: f64,
    rotation_deg: f64,
    motion: Option<EvaluatedMotion>,
}

impl Default for EvaluatedAnimationTransform {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_deg: 0.0,
            motion: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct EvaluatedMotion {
    x: f64,
    y: f64,
    origin: MotionOrigin,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum MotionOrigin {
    Parent,
    Layout,
}

impl EvaluatedShapeState {
    pub(crate) fn push_oriented_clip(&mut self, points: [Point; 4]) {
        self.oriented_clips.push(points);
    }

    /// Resolve timeline reveal clips into shape-local point geometry.
    pub fn local_clip_paths(&self, size: (f64, f64)) -> Vec<Path> {
        let scale = |point: Point| Point {
            x: point.x * size.0,
            y: point.y * size.1,
        };
        let mut paths = Vec::new();
        if let Some(clip) = self.clip {
            paths.push(Path::rect(Rect {
                x: clip.x * size.0,
                y: clip.y * size.1,
                width: clip.width * size.0,
                height: clip.height * size.1,
            }));
        }
        for points in &self.oriented_clips {
            paths.push(Path {
                commands: vec![
                    PathCommand::MoveTo(scale(points[0])),
                    PathCommand::LineTo(scale(points[1])),
                    PathCommand::LineTo(scale(points[2])),
                    PathCommand::LineTo(scale(points[3])),
                    PathCommand::Close,
                ],
                fill_rule: oxml_layout::FillRule::NonZero,
            });
        }
        paths
    }

    fn rebuild_public_transform(&mut self) {
        let scale = scale_about(self.animation.scale_x, self.animation.scale_y, 0.5, 0.5);
        let rotation = Transform::rotate_about(self.animation.rotation_deg, 0.5, 0.5);
        let motion = self.animation.motion.map_or(Transform::IDENTITY, |motion| {
            translation(motion.x, motion.y)
        });
        self.transform = scale.then(rotation).then(motion);
    }

    pub(crate) fn resolved_shape_geometry(&self) -> (f64, f64, f64) {
        (
            self.animation.scale_x,
            self.animation.scale_y,
            self.animation.rotation_deg,
        )
    }

    pub(crate) fn resolved_page_transform(
        &self,
        center: (f64, f64),
        slide_size: (f64, f64),
    ) -> Transform {
        let scale = scale_about(
            self.animation.scale_x,
            self.animation.scale_y,
            center.0,
            center.1,
        );
        let rotation = Transform::rotate_about(self.animation.rotation_deg, center.0, center.1);
        let motion = self.resolved_motion_translation(center, slide_size);
        scale.then(rotation).then(motion)
    }

    pub(crate) fn resolved_motion_translation(
        &self,
        center: (f64, f64),
        slide_size: (f64, f64),
    ) -> Transform {
        self.animation.motion.map_or(Transform::IDENTITY, |motion| {
            let (x, y) = match motion.origin {
                MotionOrigin::Parent => (
                    motion.x * slide_size.0 - center.0,
                    motion.y * slide_size.1 - center.1,
                ),
                MotionOrigin::Layout => (motion.x * slide_size.0, motion.y * slide_size.1),
            };
            translation(x, y)
        })
    }

    pub(crate) fn is_finite(&self) -> bool {
        self.opacity.is_finite()
            && transform_is_finite(self.transform)
            && self.clip.is_none_or(rect_is_finite)
            && self.oriented_clips.iter().all(|points| {
                points
                    .iter()
                    .all(|point| point.x.is_finite() && point.y.is_finite())
            })
            && [
                self.animation.scale_x,
                self.animation.scale_y,
                self.animation.rotation_deg,
            ]
            .into_iter()
            .all(f64::is_finite)
            && self
                .animation
                .motion
                .is_none_or(|motion| motion.x.is_finite() && motion.y.is_finite())
    }
}

/// One evaluated incoming-slide transition.
#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatedTransition {
    pub effect: TransitionEffect,
    pub progress: f32,
    pub duration_ms: u64,
    pub parameters: Vec<TransitionParameter>,
    pub morph_option: Option<String>,
}

/// Complete frame state at one explicit slide-local position.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EvaluatedFrameState {
    pub shapes: HashMap<u32, EvaluatedShapeState>,
    pub transition: Option<EvaluatedTransition>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Stable source identity parallel to one resolved leaf shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedShapeIdentity {
    pub source: FlattenedSource,
    pub shape_id: Option<u32>,
    pub containing_group_ids: Vec<u32>,
    pub name: Option<String>,
}

/// A resolved slide plus timeline state parallel to its leaf shapes.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedTimelineSlide {
    pub slide: ResolvedSlide,
    pub identities: Vec<ResolvedShapeIdentity>,
    pub shape_states: Vec<EvaluatedShapeState>,
    pub state: EvaluatedFrameState,
}

/// Evaluate supported timing and transition values at one explicit position.
pub fn evaluate_timeline(
    timing: Option<&CT_Timing>,
    transition: Option<&CT_SlideTransition>,
    position: TimelinePosition,
) -> Result<EvaluatedFrameState, ResolveError> {
    let mut state = EvaluatedFrameState {
        transition: transition.and_then(|transition| evaluate_transition(transition, position)),
        ..EvaluatedFrameState::default()
    };
    if let Some(timing) = timing {
        let mut context = EvaluationContext {
            timing,
            position,
            click_ordinal: 0,
            state: &mut state,
        };
        evaluate_nodes(
            timing.nodes(),
            0,
            u64::MAX,
            Scheduling::Parallel,
            &mut context,
        );
    }
    let invalid_targets = state
        .shapes
        .iter()
        .filter_map(|(target, evaluated)| (!evaluated.is_finite()).then_some(*target))
        .collect::<Vec<_>>();
    for target in invalid_targets {
        state.shapes.insert(target, EvaluatedShapeState::default());
        state.diagnostics.push(Diagnostic {
            message: format!("non-finite timeline state ignored for target {target}"),
        });
    }
    Ok(state)
}

#[derive(Clone, Copy)]
enum Scheduling {
    Parallel,
    Sequence,
}

struct EvaluationContext<'a> {
    timing: &'a CT_Timing,
    position: TimelinePosition,
    click_ordinal: u32,
    state: &'a mut EvaluatedFrameState,
}

fn evaluate_nodes(
    nodes: &[TimingNode],
    parent_start: u64,
    parent_end: u64,
    scheduling: Scheduling,
    context: &mut EvaluationContext<'_>,
) -> u64 {
    let mut cursor = parent_start;
    let mut previous = None;
    let mut latest_end = parent_start;
    for node in nodes {
        if let TimingNode::Unsupported(node) = node {
            context.state.diagnostics.push(Diagnostic {
                message: format!("unsupported timing node {}", node.local_name),
            });
            continue;
        }
        let common = common(node);
        let suggested = match (scheduling, common.node_type.as_ref(), previous) {
            (Scheduling::Sequence, Some(TimingNodeType::WithEffect), Some((start, _))) => start,
            (Scheduling::Sequence, Some(TimingNodeType::AfterEffect), Some((_, end))) => end,
            (Scheduling::Sequence, _, _) => cursor,
            (Scheduling::Parallel, _, _) => parent_start,
        };
        let start = scheduled_start(common, suggested, context);
        let end = evaluate_node(node, start, parent_end, context);
        previous = Some((start, end));
        latest_end = latest_end.max(end);
        if matches!(scheduling, Scheduling::Sequence)
            && !matches!(common.node_type, Some(TimingNodeType::WithEffect))
        {
            cursor = end;
        }
    }
    latest_end.min(parent_end)
}

fn common(node: &TimingNode) -> &CommonTimeNode {
    match node {
        TimingNode::Parallel(node) => &node.common,
        TimingNode::Sequence(node) => &node.common,
        TimingNode::Set(node) => &node.common,
        TimingNode::Animate(node) => &node.common,
        TimingNode::Effect(node) => &node.common,
        TimingNode::Motion(node) => &node.common,
        TimingNode::Unsupported(_) => unreachable!("unsupported nodes are diagnosed by the caller"),
    }
}

fn scheduled_start(
    common: &CommonTimeNode,
    suggested: u64,
    context: &mut EvaluationContext<'_>,
) -> u64 {
    let click_effect = matches!(common.node_type, Some(TimingNodeType::ClickEffect));
    let has_click_condition = common.start_conditions.iter().any(|condition| {
        matches!(
            condition.event,
            Some(TimingEvent::OnClick | TimingEvent::OnNext)
        )
    });
    let click_available = if click_effect || has_click_condition {
        context.click_ordinal = context.click_ordinal.saturating_add(1);
        context.position.click_count >= context.click_ordinal
    } else {
        false
    };
    if click_effect && !click_available {
        return u64::MAX;
    }
    if common.start_conditions.is_empty() {
        return suggested;
    }
    common
        .start_conditions
        .iter()
        .enumerate()
        .filter(|(_, condition)| {
            !matches!(
                condition.event,
                Some(TimingEvent::OnClick | TimingEvent::OnNext)
            ) || click_available
        })
        .filter_map(|(index, condition)| {
            condition_start(
                condition,
                suggested,
                context
                    .timing
                    .condition_has_explicit_target(common.id, false, index),
                context.state,
            )
        })
        .min()
        .unwrap_or(u64::MAX)
}

fn condition_start(
    condition: &TimingCondition,
    suggested: u64,
    explicit_target: Option<bool>,
    state: &mut EvaluatedFrameState,
) -> Option<u64> {
    let supported_event = matches!(
        condition.event,
        None | Some(TimingEvent::OnBegin) | Some(TimingEvent::OnClick) | Some(TimingEvent::OnNext)
    );
    let supported_target = matches!(condition.target, TimingTarget::Slide)
        || matches!(condition.target, TimingTarget::Unsupported) && explicit_target == Some(false);
    if !supported_event || !supported_target {
        state.diagnostics.push(Diagnostic {
            message: "unsupported timing condition trigger".to_owned(),
        });
        return None;
    }
    match condition.delay {
        TimingDuration::Finite(delay) => Some(suggested.saturating_add(delay)),
        TimingDuration::Indefinite => None,
    }
}

fn evaluate_node(
    node: &TimingNode,
    start: u64,
    parent_end: u64,
    context: &mut EvaluationContext<'_>,
) -> u64 {
    let common = common(node);
    let condition_end = declared_condition_end(common, start, context);
    let container_end = match common.duration {
        TimingDuration::Finite(duration) => start.saturating_add(duration),
        TimingDuration::Indefinite => u64::MAX,
    }
    .min(condition_end.unwrap_or(u64::MAX))
    .min(parent_end);
    match node {
        TimingNode::Parallel(TimingContainer { common }) => {
            let child_end = evaluate_nodes(
                &common.children,
                start,
                container_end,
                Scheduling::Parallel,
                context,
            );
            declared_end(common, start, child_end, parent_end, condition_end)
        }
        TimingNode::Sequence(TimingSequence { common, .. }) => {
            let child_end = evaluate_nodes(
                &common.children,
                start,
                container_end,
                Scheduling::Sequence,
                context,
            );
            declared_end(common, start, child_end, parent_end, condition_end)
        }
        TimingNode::Set(node) => {
            let phase_end = declared_end(&node.common, start, start, u64::MAX, condition_end);
            let end = phase_end.min(parent_end);
            if start < parent_end && context.position.elapsed_ms <= parent_end {
                evaluate_set(node, start, phase_end, context);
            }
            end
        }
        TimingNode::Animate(node) => {
            let phase_end = declared_end(&node.common, start, start, u64::MAX, condition_end);
            let end = phase_end.min(parent_end);
            if start < parent_end && context.position.elapsed_ms <= parent_end {
                evaluate_animate(node, start, phase_end, context);
            }
            end
        }
        TimingNode::Effect(node) => {
            let phase_end = declared_end(&node.common, start, start, u64::MAX, condition_end);
            let end = phase_end.min(parent_end);
            if start < parent_end && context.position.elapsed_ms <= parent_end {
                evaluate_effect(node, start, phase_end, context);
            }
            end
        }
        TimingNode::Motion(node) => {
            let phase_end = declared_end(&node.common, start, start, u64::MAX, condition_end);
            let end = phase_end.min(parent_end);
            if start < parent_end && context.position.elapsed_ms <= parent_end {
                evaluate_motion(node, start, phase_end, context);
            }
            end
        }
        TimingNode::Unsupported(_) => start,
    }
}

fn declared_end(
    common: &CommonTimeNode,
    start: u64,
    child_end: u64,
    parent_end: u64,
    condition_end: Option<u64>,
) -> u64 {
    let natural_end = match common.duration {
        TimingDuration::Finite(duration) => start.saturating_add(duration),
        TimingDuration::Indefinite if child_end > start => child_end,
        TimingDuration::Indefinite => u64::MAX,
    };
    natural_end
        .min(condition_end.unwrap_or(u64::MAX))
        .min(parent_end)
}

fn declared_condition_end(
    common: &CommonTimeNode,
    start: u64,
    context: &mut EvaluationContext<'_>,
) -> Option<u64> {
    common
        .end_conditions
        .iter()
        .enumerate()
        .filter_map(|(index, condition)| {
            end_condition_time(
                condition,
                start,
                context
                    .timing
                    .condition_has_explicit_target(common.id, true, index),
                context.state,
            )
        })
        .min()
}

fn end_condition_time(
    condition: &TimingCondition,
    start: u64,
    explicit_target: Option<bool>,
    state: &mut EvaluatedFrameState,
) -> Option<u64> {
    if !matches!(condition.event, None | Some(TimingEvent::OnBegin))
        || !(matches!(condition.target, TimingTarget::Slide)
            || matches!(condition.target, TimingTarget::Unsupported)
                && explicit_target == Some(false))
    {
        state.diagnostics.push(Diagnostic {
            message: "unsupported timing condition trigger".to_owned(),
        });
        return None;
    }
    match condition.delay {
        TimingDuration::Finite(delay) => Some(start.saturating_add(delay)),
        TimingDuration::Indefinite => None,
    }
}

#[derive(Clone, Copy)]
struct Phase {
    progress: f64,
    applies: bool,
    before: bool,
}

fn phase(common: &CommonTimeNode, start: u64, end: u64, elapsed: u64) -> Phase {
    if start == u64::MAX || elapsed < start {
        return Phase {
            progress: 0.0,
            applies: false,
            before: true,
        };
    }
    if end == u64::MAX {
        return Phase {
            progress: 0.0,
            applies: true,
            before: false,
        };
    }
    let duration = end.saturating_sub(start);
    if elapsed <= end {
        return Phase {
            progress: if duration == 0 {
                1.0
            } else {
                (elapsed - start) as f64 / duration as f64
            }
            .clamp(0.0, 1.0),
            applies: true,
            before: false,
        };
    }
    Phase {
        progress: 1.0,
        applies: matches!(common.fill, Some(TimingFill::Hold | TimingFill::Freeze)),
        before: false,
    }
}

fn target_shape(target: &TimingTarget, state: &mut EvaluatedFrameState) -> Option<u32> {
    match target {
        TimingTarget::Shape(id) => Some(*id),
        _ => {
            state.diagnostics.push(Diagnostic {
                message: "unsupported non-shape timing target".to_owned(),
            });
            None
        }
    }
}

fn shape_state(state: &mut EvaluatedFrameState, id: u32) -> &mut EvaluatedShapeState {
    state.shapes.entry(id).or_default()
}

fn evaluate_set(node: &TimingSet, start: u64, end: u64, context: &mut EvaluationContext<'_>) {
    let phase = phase(&node.common, start, end, context.position.elapsed_ms);
    if !phase.applies {
        return;
    }
    let Some(id) = target_shape(&node.target, context.state) else {
        return;
    };
    let value = node.value.as_deref().unwrap_or_default();
    match node.attribute_name.as_deref().unwrap_or_default() {
        "style.visibility" | "visibility" => {
            shape_state(context.state, id).visible = !matches!(value, "hidden" | "false" | "0")
        }
        "style.opacity" | "opacity" => {
            if let Some(value) = parse_number(value) {
                shape_state(context.state, id).opacity = normalize_fraction(value) as f32;
            } else {
                context.state.diagnostics.push(Diagnostic {
                    message: "invalid non-finite set value ignored".to_owned(),
                });
            }
        }
        attribute => context.state.diagnostics.push(Diagnostic {
            message: format!("unsupported set attribute {attribute}"),
        }),
    }
}

fn evaluate_animate(
    node: &TimingAnimate,
    start: u64,
    end: u64,
    context: &mut EvaluationContext<'_>,
) {
    let phase = phase(&node.common, start, end, context.position.elapsed_ms);
    if !phase.applies {
        return;
    }
    let Some(id) = target_shape(&node.target, context.state) else {
        return;
    };
    if node
        .from
        .iter()
        .chain(&node.to)
        .chain(&node.by)
        .any(|value| parse_number(value).is_none())
    {
        context.state.diagnostics.push(Diagnostic {
            message: "invalid non-finite animate value ignored".to_owned(),
        });
        return;
    }
    let from = node.from.as_deref().and_then(parse_number).unwrap_or(0.0);
    let to = if let Some(to) = node.to.as_deref() {
        parse_number(to)
    } else if let Some(by) = node.by.as_deref() {
        parse_number(by)
            .map(|by| from + by)
            .filter(|value| value.is_finite())
    } else {
        Some(from)
    };
    let Some(value) = to.and_then(|to| checked_interpolate(from, to, phase.progress)) else {
        context.state.diagnostics.push(Diagnostic {
            message: "non-finite animate arithmetic ignored".to_owned(),
        });
        return;
    };
    match node.attribute_name.as_deref().unwrap_or_default() {
        "style.opacity" | "opacity" => {
            shape_state(context.state, id).opacity = normalize_fraction(value) as f32
        }
        "scale" => {
            let state = shape_state(context.state, id);
            let value = normalize_percentage(value);
            state.animation.scale_x = value;
            state.animation.scale_y = value;
            state.rebuild_public_transform();
        }
        "ppt_x" | "ppt_y" => context.state.diagnostics.push(Diagnostic {
            message: format!(
                "unsupported animate attribute {}",
                node.attribute_name.as_deref().unwrap_or_default()
            ),
        }),
        "rotation" | "spin" => {
            let state = shape_state(context.state, id);
            let degrees = if value.abs() > 360.0 {
                value / 60_000.0
            } else {
                value
            };
            state.animation.rotation_deg = degrees;
            state.rebuild_public_transform();
        }
        attribute => context.state.diagnostics.push(Diagnostic {
            message: format!("unsupported animate attribute {attribute}"),
        }),
    }
}

fn evaluate_effect(node: &TimingEffect, start: u64, end: u64, context: &mut EvaluationContext<'_>) {
    let phase = phase(&node.common, start, end, context.position.elapsed_ms);
    let Some(id) = target_shape(&node.target, context.state) else {
        return;
    };
    let transition = node.transition.as_deref().unwrap_or("in");
    if !matches!(transition, "in" | "out") {
        context.state.diagnostics.push(Diagnostic {
            message: format!("unsupported timing effect transition {transition}"),
        });
        return;
    }
    let filter = node.filter.as_deref().unwrap_or("fade");
    let Some(effect) = shape_effect(filter) else {
        context.state.diagnostics.push(Diagnostic {
            message: format!("unsupported timing effect {filter}"),
        });
        return;
    };
    let state = shape_state(context.state, id);
    if phase.before {
        if transition == "in" {
            state.visible = false;
            state.opacity = 0.0;
        }
        return;
    }
    if !phase.applies {
        return;
    }
    let progress = if matches!(effect, ShapeEffect::Appear) {
        1.0
    } else {
        phase.progress
    };
    match (effect, transition == "out") {
        (ShapeEffect::Appear, true) => {
            state.opacity = 0.0;
            state.visible = false;
        }
        (ShapeEffect::Appear, false) => {
            state.visible = true;
            state.opacity = 1.0;
        }
        (ShapeEffect::Fade, true) => {
            state.opacity = (1.0 - progress) as f32;
            state.visible = progress < 1.0;
        }
        (ShapeEffect::Fade, false) => {
            state.visible = true;
            state.opacity = progress as f32;
        }
        (ShapeEffect::Wipe(direction), exiting) => {
            state.visible = !exiting || progress < 1.0;
            state.opacity = 1.0;
            state.clip = Some(wipe_clip(direction, progress, exiting));
        }
    }
}

#[derive(Clone, Copy)]
enum ShapeEffect<'a> {
    Appear,
    Fade,
    Wipe(&'a str),
}

fn shape_effect(filter: &str) -> Option<ShapeEffect<'_>> {
    match filter.trim() {
        "appear" => Some(ShapeEffect::Appear),
        "fade" => Some(ShapeEffect::Fade),
        filter => filter
            .strip_prefix("wipe(")
            .and_then(|direction| direction.strip_suffix(')'))
            .filter(|direction| matches!(*direction, "left" | "right" | "up" | "down"))
            .map(ShapeEffect::Wipe),
    }
}

fn wipe_clip(direction: &str, progress: f64, exiting: bool) -> Rect {
    let progress = progress.clamp(0.0, 1.0);
    match (direction, exiting) {
        ("left", false) => Rect {
            x: 1.0 - progress,
            y: 0.0,
            width: progress,
            height: 1.0,
        },
        ("up", false) => Rect {
            x: 0.0,
            y: 1.0 - progress,
            width: 1.0,
            height: progress,
        },
        ("down", false) => Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: progress,
        },
        ("right", false) => Rect {
            x: 0.0,
            y: 0.0,
            width: progress,
            height: 1.0,
        },
        ("left", true) => Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0 - progress,
            height: 1.0,
        },
        ("up", true) => Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0 - progress,
        },
        ("down", true) => Rect {
            x: 0.0,
            y: progress,
            width: 1.0,
            height: 1.0 - progress,
        },
        _ => Rect {
            x: progress,
            y: 0.0,
            width: 1.0 - progress,
            height: 1.0,
        },
    }
}

fn evaluate_motion(
    node: &TimingMotionPath,
    start: u64,
    end: u64,
    context: &mut EvaluationContext<'_>,
) {
    let phase = phase(&node.common, start, end, context.position.elapsed_ms);
    if !phase.applies {
        return;
    }
    let Some(id) = target_shape(&node.target, context.state) else {
        return;
    };
    let Some(points) = node.path.as_deref().and_then(parse_motion_path) else {
        context.state.diagnostics.push(Diagnostic {
            message: "unsupported motion path".to_owned(),
        });
        return;
    };
    let (x, y) = point_on_polyline(&points, phase.progress);
    if !x.is_finite() || !y.is_finite() {
        context.state.diagnostics.push(Diagnostic {
            message: "non-finite motion path ignored".to_owned(),
        });
        return;
    }
    let origin = match node.origin.as_deref().unwrap_or("parent") {
        "parent" => MotionOrigin::Parent,
        "layout" => MotionOrigin::Layout,
        value => {
            context.state.diagnostics.push(Diagnostic {
                message: format!("unsupported motion path origin {value}"),
            });
            return;
        }
    };
    let state = shape_state(context.state, id);
    state.animation.motion = Some(EvaluatedMotion { x, y, origin });
    state.rebuild_public_transform();
}

fn parse_number(value: &str) -> Option<f64> {
    value
        .trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

fn normalize_fraction(value: f64) -> f64 {
    normalize_percentage(value).clamp(0.0, 1.0)
}

fn normalize_percentage(value: f64) -> f64 {
    if value.abs() > 10.0 {
        value / 100_000.0
    } else {
        value
    }
}

fn interpolate(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress
}

fn checked_interpolate(from: f64, to: f64, progress: f64) -> Option<f64> {
    let value = from + (to - from) * progress;
    value.is_finite().then_some(value)
}

fn parse_motion_path(path: &str) -> Option<Vec<(f64, f64)>> {
    #[derive(Clone, Copy)]
    enum Command {
        MoveAbsolute,
        LineAbsolute,
        MoveRelative,
        LineRelative,
    }

    let tokens = path
        .split(|character: char| character.is_ascii_whitespace() || character == ',')
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let mut index = 0;
    let mut command = None;
    let mut current = (0.0, 0.0);
    let mut points = Vec::new();
    while index < tokens.len() {
        command = match tokens[index] {
            "M" => Some(Command::MoveAbsolute),
            "L" => Some(Command::LineAbsolute),
            "m" => Some(Command::MoveRelative),
            "l" => Some(Command::LineRelative),
            "E" | "e" => break,
            _ => command,
        };
        if matches!(tokens[index], "M" | "L" | "m" | "l") {
            index += 1;
            continue;
        }
        let active = command?;
        let x = parse_number(tokens[index])?;
        let y = parse_number(tokens.get(index + 1)?)?;
        current = match active {
            Command::MoveAbsolute | Command::LineAbsolute => (x, y),
            Command::MoveRelative | Command::LineRelative => (current.0 + x, current.1 + y),
        };
        if !current.0.is_finite() || !current.1.is_finite() {
            return None;
        }
        points.push(current);
        command = Some(match active {
            Command::MoveAbsolute => Command::LineAbsolute,
            Command::MoveRelative => Command::LineRelative,
            command => command,
        });
        index += 2;
    }
    if points.len() < 2 {
        return None;
    }
    Some(points)
}

fn point_on_polyline(points: &[(f64, f64)], progress: f64) -> (f64, f64) {
    let lengths = points
        .windows(2)
        .map(|pair| (pair[1].0 - pair[0].0).hypot(pair[1].1 - pair[0].1))
        .collect::<Vec<_>>();
    let total = lengths.iter().sum::<f64>();
    if total <= f64::EPSILON {
        return points[0];
    }
    let mut remaining = total * progress.clamp(0.0, 1.0);
    for (index, length) in lengths.iter().copied().enumerate() {
        if remaining <= length || index + 1 == lengths.len() {
            let local = if length <= f64::EPSILON {
                0.0
            } else {
                remaining / length
            };
            return (
                interpolate(points[index].0, points[index + 1].0, local),
                interpolate(points[index].1, points[index + 1].1, local),
            );
        }
        remaining -= length;
    }
    *points
        .last()
        .expect("motion paths have at least two points")
}

fn scale_about(scale_x: f64, scale_y: f64, center_x: f64, center_y: f64) -> Transform {
    Transform {
        a: scale_x,
        d: scale_y,
        e: center_x * (1.0 - scale_x),
        f: center_y * (1.0 - scale_y),
        ..Transform::IDENTITY
    }
}

fn translation(x: f64, y: f64) -> Transform {
    Transform {
        e: x,
        f: y,
        ..Transform::IDENTITY
    }
}

fn transform_is_finite(transform: Transform) -> bool {
    [
        transform.a,
        transform.b,
        transform.c,
        transform.d,
        transform.e,
        transform.f,
    ]
    .into_iter()
    .all(f64::is_finite)
}

fn rect_is_finite(rect: Rect) -> bool {
    [rect.x, rect.y, rect.width, rect.height]
        .into_iter()
        .all(f64::is_finite)
}

fn evaluate_transition(
    transition: &CT_SlideTransition,
    position: TimelinePosition,
) -> Option<EvaluatedTransition> {
    let effect = transition.effect.clone()?;
    let duration_ms = transition.duration_ms.unwrap_or(match transition.speed {
        Some(TransitionSpeed::Slow) => 1_000,
        Some(TransitionSpeed::Fast) => 500,
        _ => 750,
    });
    let progress = if duration_ms == 0 {
        1.0
    } else {
        position.elapsed_ms as f64 / duration_ms as f64
    }
    .clamp(0.0, 1.0) as f32;
    Some(EvaluatedTransition {
        effect,
        progress,
        duration_ms,
        parameters: transition.effect_parameters.clone(),
        morph_option: transition
            .morph
            .as_ref()
            .and_then(|morph| morph.option.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::{TimelinePosition, evaluate_timeline};
    use oxml_layout::Point;
    use rpptx_oxml::timing::CT_Timing;

    const P_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";

    fn timing(nodes: &str) -> CT_Timing {
        CT_Timing::from_xml(
            format!(r#"<p:timing xmlns:p="{P_NS}"><p:tnLst>{nodes}</p:tnLst></p:timing>"#)
                .as_bytes(),
        )
        .unwrap()
    }

    #[test]
    fn explicit_timestamps_evaluate_sequences_parallel_groups_and_clicks_deterministically() {
        let timing = timing(
            r#"<p:seq><p:cTn id="1" nodeType="mainSeq"><p:childTnLst>
            <p:set><p:cBhvr><p:cTn id="2" dur="100" fill="hold" nodeType="clickEffect"/><p:tgtEl><p:spTgt spid="7"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="0"/></p:to></p:set>
            <p:anim from="0" to="100000"><p:cBhvr><p:cTn id="3" dur="200" fill="hold" nodeType="withEffect"/><p:tgtEl><p:spTgt spid="8"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            <p:set><p:cBhvr><p:cTn id="4" dur="100" fill="hold" nodeType="afterEffect"/><p:tgtEl><p:spTgt spid="9"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>
            <p:par><p:cTn id="5" dur="100"><p:childTnLst>
            <p:anim from="0" to="100000"><p:cBhvr><p:cTn id="6" dur="100"/><p:tgtEl><p:spTgt spid="10"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            <p:anim from="100000" to="0"><p:cBhvr><p:cTn id="7" dur="100"/><p:tgtEl><p:spTgt spid="11"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            </p:childTnLst></p:cTn></p:par>
            </p:childTnLst></p:cTn></p:seq>
            <p:set><p:cBhvr><p:cTn id="8" dur="1" fill="hold"><p:stCondLst><p:cond delay="100"/><p:cond delay="200"/></p:stCondLst></p:cTn><p:tgtEl><p:spTgt spid="12"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>"#,
        );

        let before_click = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 50,
                click_count: 0,
            },
        )
        .unwrap();
        let after_click = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 150,
                click_count: 1,
            },
        )
        .unwrap();
        let after_previous = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 250,
                click_count: 1,
            },
        )
        .unwrap();
        let parallel = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 350,
                click_count: 1,
            },
        )
        .unwrap();

        assert!(!before_click.shapes.contains_key(&7));
        assert_eq!(after_click.shapes[&7].opacity, 0.0);
        assert_eq!(after_click.shapes[&8].opacity, 0.75);
        assert!(!after_click.shapes.contains_key(&9));
        assert!(!after_click.shapes[&12].visible);
        assert!(!after_previous.shapes[&9].visible);
        assert_eq!(parallel.shapes[&10].opacity, 0.5);
        assert_eq!(parallel.shapes[&11].opacity, 0.5);
    }

    #[test]
    fn start_condition_alternatives_select_the_first_available_trigger() {
        let timing = timing(
            r#"<p:set><p:cBhvr><p:cTn id="1" dur="1" fill="hold"><p:stCondLst><p:cond delay="100"/><p:cond evt="onClick" delay="10"/></p:stCondLst></p:cTn><p:tgtEl><p:spTgt spid="7"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>"#,
        );

        let timed = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 150,
                click_count: 0,
            },
        )
        .unwrap();
        let clicked = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 50,
                click_count: 1,
            },
        )
        .unwrap();

        assert!(!timed.shapes[&7].visible);
        assert!(!clicked.shapes[&7].visible);
    }

    #[test]
    fn end_conditions_bound_intervals_and_indefinite_leaves_block_sequences() {
        let timing = timing(
            r#"<p:seq><p:cTn id="1"><p:childTnLst>
            <p:anim from="0" to="100000"><p:cBhvr><p:cTn id="2" dur="indefinite" fill="hold"><p:endCondLst><p:cond delay="200"/></p:endCondLst></p:cTn><p:tgtEl><p:spTgt spid="7"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            <p:set><p:cBhvr><p:cTn id="3" dur="1" fill="hold"/><p:tgtEl><p:spTgt spid="8"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>
            </p:childTnLst></p:cTn></p:seq>
            <p:seq><p:cTn id="4"><p:childTnLst>
            <p:set><p:cBhvr><p:cTn id="5" dur="indefinite" fill="hold"/><p:tgtEl><p:spTgt spid="9"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>
            <p:set><p:cBhvr><p:cTn id="6" dur="1" fill="hold"/><p:tgtEl><p:spTgt spid="10"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>
            </p:childTnLst></p:cTn></p:seq>
            <p:anim from="0" to="100000"><p:cBhvr><p:cTn id="7" dur="1000"><p:endCondLst><p:cond delay="100"><p:rtn val="all"/></p:cond></p:endCondLst></p:cTn><p:tgtEl><p:spTgt spid="11"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr></p:anim>"#,
        );

        let before_end = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 150,
                click_count: 0,
            },
        )
        .unwrap();
        let after_end = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 250,
                click_count: 0,
            },
        )
        .unwrap();

        assert_eq!(before_end.shapes[&7].opacity, 0.75);
        assert!(!before_end.shapes.contains_key(&8));
        assert!(!after_end.shapes[&8].visible);
        assert!(!after_end.shapes[&9].visible);
        assert!(!after_end.shapes.contains_key(&10));
        assert!(after_end.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("unsupported timing condition trigger")
        }));
    }

    #[test]
    fn container_end_boundaries_stop_parallel_and_sequence_children() {
        let timing = timing(
            r#"<p:par><p:cTn id="1" dur="200"><p:childTnLst>
            <p:anim from="0" to="100000"><p:cBhvr><p:cTn id="2" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="7"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            </p:childTnLst></p:cTn></p:par>
            <p:seq><p:cTn id="3" dur="200"><p:childTnLst>
            <p:anim from="0" to="100000"><p:cBhvr><p:cTn id="4" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="8"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            <p:set><p:cBhvr><p:cTn id="5" dur="1" fill="hold"/><p:tgtEl><p:spTgt spid="9"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>
            </p:childTnLst></p:cTn></p:seq>"#,
        );

        let active = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 150,
                click_count: 0,
            },
        )
        .unwrap();
        let ended = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 500,
                click_count: 0,
            },
        )
        .unwrap();

        assert_eq!(active.shapes[&7].opacity, 0.15);
        assert_eq!(active.shapes[&8].opacity, 0.15);
        assert!(!active.shapes.contains_key(&9));
        assert!(!ended.shapes.contains_key(&7));
        assert!(!ended.shapes.contains_key(&8));
        assert!(!ended.shapes.contains_key(&9));
    }

    #[test]
    fn entrance_exit_emphasis_and_motion_states_clamp_at_boundaries() {
        let timing = timing(
            r#"<p:par><p:cTn id="1"><p:childTnLst>
            <p:animEffect transition="in" filter="fade"><p:cBhvr><p:cTn id="2" dur="1000" fill="hold"><p:stCondLst><p:cond delay="100"/></p:stCondLst></p:cTn><p:tgtEl><p:spTgt spid="7"/></p:tgtEl></p:cBhvr></p:animEffect>
            <p:animMotion origin="layout" path="M 0 0 L .5 0 L .5 1 E"><p:cBhvr><p:cTn id="3" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="8"/></p:tgtEl></p:cBhvr></p:animMotion>
            <p:animEffect transition="out" filter="fade"><p:cBhvr><p:cTn id="4" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="9"/></p:tgtEl></p:cBhvr></p:animEffect>
            <p:animEffect transition="in" filter="appear"><p:cBhvr><p:cTn id="5" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="10"/></p:tgtEl></p:cBhvr></p:animEffect>
            <p:anim from="100000" to="200000"><p:cBhvr><p:cTn id="6" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="11"/></p:tgtEl><p:attrNameLst><p:attrName>scale</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            <p:anim from="0" to="5400000"><p:cBhvr><p:cTn id="7" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="11"/></p:tgtEl><p:attrNameLst><p:attrName>spin</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            <p:animEffect transition="in" filter="wipe(left)"><p:cBhvr><p:cTn id="8" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="12"/></p:tgtEl></p:cBhvr></p:animEffect>
            <p:set><p:cBhvr><p:cTn id="9" dur="100" fill="remove"/><p:tgtEl><p:spTgt spid="13"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>
            <p:animEffect transition="out" filter="wipe(down)"><p:cBhvr><p:cTn id="10" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="14"/></p:tgtEl></p:cBhvr></p:animEffect>
            <p:animEffect transition="out" filter="appear"><p:cBhvr><p:cTn id="11" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="15"/></p:tgtEl></p:cBhvr></p:animEffect>
            <p:anim from="0" to="NaN"><p:cBhvr><p:cTn id="12" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="16"/></p:tgtEl><p:attrNameLst><p:attrName>style.opacity</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            </p:childTnLst></p:cTn></p:par>"#,
        );
        let before_start = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 99,
                click_count: 0,
            },
        )
        .unwrap();
        let at_start = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 100,
                click_count: 0,
            },
        )
        .unwrap();
        let at_zero = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 0,
                click_count: 0,
            },
        )
        .unwrap();
        let at_end = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 1000,
                click_count: 0,
            },
        )
        .unwrap();
        let after_end = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 1101,
                click_count: 0,
            },
        )
        .unwrap();
        let delayed_end = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 1100,
                click_count: 0,
            },
        )
        .unwrap();

        assert!(!before_start.shapes[&7].visible);
        assert!(at_start.shapes[&7].visible);
        assert_eq!(at_start.shapes[&7].opacity, 0.0);
        assert_eq!(at_end.shapes[&7].opacity, 0.9);
        assert_eq!(delayed_end.shapes[&7].opacity, 1.0);
        assert_eq!(after_end.shapes[&7].opacity, 1.0);
        assert_eq!(at_end.shapes[&8].transform.e, 0.5);
        assert_eq!(at_end.shapes[&8].transform.f, 1.0);
        assert!(!at_end.shapes[&9].visible);
        assert_eq!(at_zero.shapes[&10].opacity, 1.0);
        assert!(!at_zero.shapes[&15].visible);
        let centre = at_end.shapes[&11].transform.apply(Point { x: 0.5, y: 0.5 });
        assert!((centre.x - 0.5).abs() < 1.0e-10);
        assert!((centre.y - 0.5).abs() < 1.0e-10);
        assert_eq!(at_start.shapes[&12].clip.unwrap().x, 0.9);
        assert_eq!(at_end.shapes[&12].clip.unwrap().width, 1.0);
        assert!(!at_end.shapes.contains_key(&13));
        assert_eq!(at_end.shapes[&14].clip.unwrap().height, 0.0);
        assert!(!at_start.shapes[&15].visible);
        assert!(!at_end.shapes.contains_key(&16));
        assert!(
            at_end
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("non-finite"))
        );
    }

    #[test]
    fn unsupported_timing_nodes_do_not_hide_supported_siblings() {
        let timing = timing(
            r#"<p:par><p:cTn id="1"><p:childTnLst><p:cmd type="call" cmd="x"/><p:set><p:cBhvr><p:cTn id="2" dur="1" fill="hold"/><p:tgtEl><p:spTgt spid="9"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set></p:childTnLst></p:cTn></p:par>"#,
        );
        let state = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 1,
                click_count: 0,
            },
        )
        .unwrap();

        assert!(!state.shapes[&9].visible);
        assert_eq!(state.diagnostics.len(), 1);
    }

    #[test]
    fn relative_motion_commands_accumulate_from_the_current_point() {
        let timing = timing(
            r#"<p:animMotion origin="parent" path="M 0 0 l .2 .1 l .2 .1 E"><p:cBhvr><p:cTn id="1" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="7"/></p:tgtEl></p:cBhvr></p:animMotion>"#,
        );
        let state = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 1000,
                click_count: 0,
            },
        )
        .unwrap();

        assert_eq!(state.shapes[&7].transform.e, 0.4);
        assert_eq!(state.shapes[&7].transform.f, 0.2);
    }

    #[test]
    fn unsupported_semantics_and_overflow_are_diagnostic_and_finite() {
        let timing = timing(
            r#"<p:anim from="0" to="100000"><p:cBhvr><p:cTn id="1" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="7"/></p:tgtEl><p:attrNameLst><p:attrName>ppt_x</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            <p:anim from="-1.7976931348623157e308" to="1.7976931348623157e308"><p:cBhvr><p:cTn id="2" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="8"/></p:tgtEl><p:attrNameLst><p:attrName>scale</p:attrName></p:attrNameLst></p:cBhvr></p:anim>
            <p:animEffect transition="none" filter="fade"><p:cBhvr><p:cTn id="3" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="9"/></p:tgtEl></p:cBhvr></p:animEffect>
            <p:set><p:cBhvr><p:cTn id="4" dur="1" fill="hold"><p:stCondLst><p:cond delay="0"><p:rtn val="all"/></p:cond></p:stCondLst></p:cTn><p:tgtEl><p:spTgt spid="10"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>
            <p:set><p:cBhvr><p:cTn id="5" dur="1" fill="hold"><p:stCondLst><p:cond delay="0"/></p:stCondLst></p:cTn><p:tgtEl><p:spTgt spid="11"/></p:tgtEl><p:attrNameLst><p:attrName>style.visibility</p:attrName></p:attrNameLst></p:cBhvr><p:to><p:strVal val="hidden"/></p:to></p:set>
            <p:animEffect transition="in" filter="producer-fade-token"><p:cBhvr><p:cTn id="6" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="12"/></p:tgtEl></p:cBhvr></p:animEffect>
            <p:animEffect transition="in" filter="wipe(diagonal)"><p:cBhvr><p:cTn id="7" dur="1000" fill="hold"/><p:tgtEl><p:spTgt spid="13"/></p:tgtEl></p:cBhvr></p:animEffect>"#,
        );
        let state = evaluate_timeline(
            Some(&timing),
            None,
            TimelinePosition {
                elapsed_ms: 1000,
                click_count: 0,
            },
        )
        .unwrap();

        assert!(!state.shapes.contains_key(&7));
        assert!(!state.shapes.contains_key(&8));
        assert!(!state.shapes.contains_key(&9));
        assert!(!state.shapes.contains_key(&10));
        assert!(!state.shapes[&11].visible);
        assert!(!state.shapes.contains_key(&12));
        assert!(!state.shapes.contains_key(&13));
        assert!(
            state
                .shapes
                .values()
                .all(super::EvaluatedShapeState::is_finite)
        );
        assert!(
            state
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("ppt_x"))
        );
        assert!(
            state
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("arithmetic"))
        );
        assert!(state.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("unsupported timing condition trigger")
        }));
        assert!(state.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("unsupported timing effect transition none")
        }));
        assert!(state.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("unsupported timing effect producer-fade-token")
        }));
        assert!(state.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("unsupported timing effect wipe(diagonal)")
        }));
    }
}

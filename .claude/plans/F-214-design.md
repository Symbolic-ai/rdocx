# F-214, Timeline evaluation and transition rendering

**Status**: approved
**Sprint**: S60
**Size**: L
**Depends on**: F-213

## Problem

Presentation timing and transitions do not affect rendering. The static
pipeline resolves each slide and lowers it directly to one page, and the
resolved model carries neither timing state nor stable shape-target identity.

F-214 must evaluate the typed F-213 model at explicit timestamps, apply the
supported entrance, exit, emphasis, and motion behavior, and compose ordinary
transitions and bounded morph output. Existing static rendering must remain
independent of timeline execution, including all 49 deterministic harness
entries.

## Spec reference

- ECMA-376 Part 1, PresentationML timing semantics, animation behaviors, and
  slide transitions.
- Microsoft Office PresentationML extensions for morph transitions.
- `docs/hld/02-scope-and-non-goals.md`, "Beyond v1" and "Superseded".
- `docs/hld/03-architecture.md`, "The dependency rule" and the frozen
  PresentationML model, resolver, and renderer seams.
- `docs/hld/06-presentationml-model.md`, "Preservation strategy".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap", "Why
  Group is the whole design", and "The renderer's input".
- `docs/hld/10-bindings-spec.md`, published native Rust API policy.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The deck corpus", and
  "The render fidelity gate".
- `docs/hld/14-development-backlog.md`, "F-214, Timeline evaluation and
  transition rendering".

## Approach

Consume the exact F-213 timing and transition values after F-213 is integrated
and verified. Shape targets use `p:cNvPr/@id`, never resolved vector indices or
facade paths.

Add a pure evaluator in `rpptx-layout`:

```rust
pub struct TimelinePosition {
    pub elapsed_ms: u64,
    pub click_count: u32,
}

pub struct EvaluatedShapeState {
    pub visible: bool,
    pub opacity: f32,
    pub transform: oxml_layout::Transform,
}

pub struct EvaluatedFrameState {
    pub shapes: HashMap<u32, EvaluatedShapeState>,
    pub transition: Option<EvaluatedTransition>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn evaluate_timeline(
    timing: Option<&CT_Timing>,
    transition: Option<&CT_SlideTransition>,
    position: TimelinePosition,
) -> Result<EvaluatedFrameState, ResolveError>;
```

Sequences and parallel groups calculate deterministic active intervals.
Click-effect nodes consume the explicit click ordinal. With-previous and
after-previous nodes derive from their containing sequence. Unsupported
shape-event triggers remain diagnostic and do not suppress supported siblings.
Finite progress clamps at exact start and end boundaries. The supported first
slice is appear, fade, and wipe entrance and exit, opacity, scale, and spin
emphasis, line and polyline motion paths, and once-only linear timing with hold
or remove fill.

Extend the resolver through an additive timestamped path. Preserve every
existing `resolve_slide*` method unchanged. During flattening, retain each
leaf's slide shape id and containing group ids in the timestamped path so a
group target affects every resolved descendant. Apply evaluated visibility,
opacity, and finite transforms before freezing a frame-specific
`ResolvedSlide`. Master and layout ids remain in separate identity scopes.

Add a timeline renderer that lowers the evaluated slide through the same
private shape and text path as static rendering. Compose cut, fade, wipe, push,
and zoom transitions from outgoing and incoming `PageFrame` groups. Morph pairs
only shapes with matching explicit `!!` names and compatible resolved geometry.
It interpolates finite bounds and transforms. Unmatched or incompatible shapes
crossfade and emit a stable diagnostic. The PDF and raster backends remain
timeline-agnostic.

Add a native deterministic facade entry point that returns the evaluated state
beside the page frame and diagnostics. Timestamps are slide-local. Transition
time starts at zero for the incoming slide, and the caller supplies the
outgoing slide when a two-slide effect needs it. Existing
`render_deterministic()` and `to_pdf_deterministic()` continue through the
unchanged static path.

No trait, generic, feature flag, crate, backend animation variant, runtime
oracle dependency, or binary fixture is added. The public change is additive
for the pre-1.0 `rpptx-layout`, `rpptx-render`, and `rpptx` crates.

## Rejected alternatives

- Executing raw `p:timing` in `rpptx-render` would reverse the model and
  resolver boundary.
- Making static rendering select timestamp zero would couple ordinary output
  to dynamic behavior and violate the sprint contract.
- Adding animation variants to `oxml-layout::PositionedElement` would make
  every backend understand time when existing group transforms, clips, and
  opacity already carry evaluated frames.
- Identifying targets by resolved vector index would break under group
  flattening and inherited shapes.
- Matching morph shapes by ordinary names or vector order would animate
  unrelated content. Explicit `!!` names provide a bounded opt-in.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `explicit_timestamps_evaluate_sequences_parallel_groups_and_clicks_deterministically` | Exact active intervals and boundary behavior for parallel, sequence, click, with-previous, and after-previous nodes. |
| unit | `entrance_exit_emphasis_and_motion_states_clamp_at_boundaries` | Visibility, opacity, finite transforms, path endpoints, hold, remove, and exact endpoint clamping. |
| unit | `group_targets_apply_to_every_resolved_descendant` | One group id affects every flattened descendant and no unrelated shape. |
| regression | `unsupported_timing_nodes_do_not_hide_supported_siblings` | Stable diagnostics and continued supported evaluation. |
| regression | `ordinary_static_rendering_does_not_execute_the_timeline` | Existing page, PDF, and raster output remain identical. |
| integration | `timeline_frames_render_through_the_existing_resolved_slide_boundary` | Facade, resolver, renderer, deterministic fonts, media, and text reuse the existing path. |
| differential | `pinned_timestamps_match_powerpoint_frame_oracle_within_declared_tolerances` | Exact oracle identity, timestamps, dimensions, geometric tolerance, pixel metric, and transition and morph cases. |

The exact backlog **test gate is differential**: "Pinned timestamps match the
PowerPoint frame oracle within the declared geometric and pixel tolerances."

The proposed oracle is Microsoft PowerPoint 16.104, Info.plist build
16.104.25121423, AppleScript build 1214. A source-built deck and a manifest of
SHA-256-pinned PowerPoint frame images are fetched into the ignored corpus,
never committed as binary fixtures. Rust frames use deterministic fonts and
150 dpi. Geometry permits at most 1 point error, and raster output requires
global luminance SSIM of at least 0.99. The harness rejects missing corpus when
`RDOCX_PPTX_CORPUS_REQUIRED` is set and records every intentional divergence.

Add tests to existing unit sections and
`crates/rpptx/tests/integration.rs`. Do not add a new integration binary.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Unit conversion and `Emu`: motion paths and shape geometry preserve the
  pinned truncating constructors. Add exact endpoint tests and declare the
  harness result.
- Layout: every frame and raster comparison uses deterministic font mode.
  Never record a baseline against system fonts.
- Public API of published crates: state the additive pre-1.0 semver impact.
  Run publish dry runs for `rpptx-layout`, `rpptx-render`, and `rpptx`, and
  assert every archive remains below 10 MiB.
- New module or file: obtain explicit approval for
  `crates/rpptx-layout/src/timeline.rs` and
  `crates/rpptx-render/src/timeline.rs`. No new trait, generic, crate, feature,
  or dependency is introduced.
- External oracle comparison: pin PowerPoint 16.104 and both build identities
  in the harness. Record the source artifact, exact timestamps, 150 dpi,
  1 point geometry tolerance, SSIM 0.99 threshold, and classification of every
  divergence.

## Hash harness

Expected unchanged. Timeline execution is additive and ordinary static
rendering stays byte-identical. Any change to the 49 deterministic entries is
unexplained and blocks integration.

## Implementation checklist

- [ ] Integrate and verify F-213, then reconcile exact type names and supported
  metadata against its completed plan.
- [ ] Add evaluator tests before implementation.
- [ ] Implement deterministic sequence, parallel, click, duration, and fill
  evaluation.
- [ ] Map slide and group shape ids to resolved descendants.
- [ ] Apply entrance, exit, emphasis, and motion state through the additive
  resolver path.
- [ ] Compose cut, fade, wipe, push, and zoom transitions.
- [ ] Implement explicit-name bounded morph correlation and crossfade fallback.
- [ ] Add the deterministic facade frame API without changing static methods.
- [ ] Add the pinned PowerPoint differential and static-output regression.
- [ ] Run focused `rpptx-layout`, `rpptx-render`, and `rpptx` checks plus every
  routed rider.

## Open questions

None. The two timeline modules, bounded effect and morph subset, slide-local
timestamp and click convention, fetched SHA-pinned PowerPoint frame oracle,
declared tolerances, and page-frame-plus-state return contract are approved.

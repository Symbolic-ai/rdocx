# F-216, Media poster and playback rendering

**Status**: completed
**Sprint**: S61
**Size**: M
**Depends on**: F-214, F-215

## Problem

The deterministic facade currently assembles only renderer-compatible image
relationships into `RenderInput.media` in `crates/rpptx/src/lib.rs`. Pictures
then resolve either to `ResolvedContent::Image` or a visible fallback in
`crates/rpptx-layout/src/context.rs`. No media-specific poster or playback
projection exists.

The completed timeline state covers shapes and transitions only. It cannot
expose audio or video playback state to F-227. F-216 must add poster and media
fallback behavior without decoding an unapproved codec or changing the static
rendering path.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Explicitly not in v1" and "Beyond
  v1".
- `docs/hld/03-architecture.md`, "Why these seams" and the timeline ownership
  boundary.
- `docs/hld/07-inheritance-and-resolution.md`, source-scoped media resolution
  and visible fallback behavior.
- `docs/hld/08-rendering-spec.md`, deterministic presentation entry points,
  timeline lowering, and renderer media admission.
- `docs/hld/10-bindings-spec.md`, "Native PowerPoint timing model".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-216, Media poster and playback
  rendering".

## Approach

Consume the exact media attachment, playback settings, timing command, poster,
and stable shape identity types delivered by F-215. Do not introduce a second
media model.

Package assembly admits the F-215 poster image relationship through the
existing scoped image path. Audio and video payload bytes do not enter
`RenderInput.media` and are never offered to image probing or a raster backend.
A valid local PNG or JPEG poster resolves through the existing
`ResolvedContent::Image` path. An absent, linked, malformed, or unsupported
poster freezes a deterministic labelled `Audio` or `Video` group inside the
media bounds and emits one stable diagnostic. Existing renderer lowering
already handles images and frozen groups, so no codec branch or new positioned
element variant is added.

Playback evaluation belongs in the existing `rpptx-layout::timeline` module.
A private adapter folds the F-215 automatic and click-triggered play, pause,
and stop operations in source order against `TimelinePosition`. Invalid or
unsupported triggers diagnose only their media object. Trim values use checked
millisecond arithmetic. Volume normalizes from the F-215 schema range. Looping
wraps only when a finite positive trim end or known duration defines an
interval.

Add concrete output values and one additive facade entry point:

```rust
pub enum MediaPlaybackPhase {
    Stopped,
    Playing,
    Paused,
}

pub struct EvaluatedMediaState {
    pub shape_id: u32,
    pub phase: MediaPlaybackPhase,
    pub source_position_ms: u64,
    pub volume: f32,
    pub looping: bool,
}

pub struct DeterministicMediaTimelineFrame {
    pub frame: DeterministicTimelineFrame,
    pub media: Vec<EvaluatedMediaState>,
}

pub enum MediaFallbackPolicy {
    PosterFrame,
    DeterministicPlaceholder,
    Fail,
}

impl Presentation {
    pub fn render_media_timeline_deterministic(
        &self,
        slide_index: usize,
        position: TimelinePosition,
        outgoing_slide_index: Option<usize>,
        fallback_policy: MediaFallbackPolicy,
    ) -> Result<DeterministicMediaTimelineFrame>;
}
```

The fallback policy is an explicit input to the media-aware facade entry point.
This keeps all three approved policies callable and leaves the existing static
and timeline entry points unchanged.

The nested result preserves source compatibility for
`render_timeline_deterministic`. A shared private implementation assembles and
evaluates once. F-227 is the existing second consumer of the combined page and
media result. No Python, WASM, CLI, trait, generic, feature, dependency,
module, or file is added.

This is an additive public API change to the pre-1.0 `rpptx-layout` and `rpptx`
crates. `rpptx-render` gains no public surface.

## Rejected alternatives

- Adding audio or video variants to `PositionedElement` would make every
  backend understand codecs and time.
- Putting playback evaluation in `rpptx-render` would reverse the model,
  resolver, and renderer boundary.
- Adding fields to existing public result structs would break downstream
  exhaustive construction.
- Calling separate public page and playback evaluators would assemble the
  package twice for every exported frame.
- Deriving posters by decoding video would violate the bounded-codec rule.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `media_playback_state_clamps_trim_volume_and_exact_boundaries` | Before-start, exact-start, pause, resume, exact-stop, trim, volume, and finite loop behavior. |
| unit | `click_triggered_media_uses_the_existing_timeline_click_count` | Media starts only at its declared click ordinal and unrelated shape state remains unchanged. |
| unit | `valid_media_posters_resolve_through_the_existing_image_content_path` | A local poster becomes the expected resolved image without admitting audio or video payloads. |
| regression | `unsupported_codecs_keep_the_poster_without_attempting_to_decode_the_payload` | Opaque payload bytes remain untouched while the poster renders and a stable diagnostic remains. |
| regression | `missing_media_posters_remain_visible_and_do_not_change_timeline_siblings` | A labelled fallback appears and unrelated shapes and media states are unchanged. |
| integration | `media_timeline_frames_return_the_page_and_synchronized_playback_state_once` | Facade, poster resolution, timeline position, diagnostics, and combined result share one assembly path. |
| golden | `static_poster_output_and_timestamped_playback_state_match_source_built_oracle_fixtures` | Exact decoded RGBA hashes at 150 dpi and exact normalized playback rows cover poster, fallback, link, codec, trigger, trim, volume, loop, pause, and stop cases. |

The exact backlog **test gate is golden**: "Static poster output and
timestamped playback state match the source-built oracle fixtures."

Construct package, poster PNG, and opaque media bytes in the existing
`crates/rpptx/tests/integration.rs` binary. Record only textual RGBA hashes and
normalized playback rows. Do not add a binary fixture or integration binary.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout, pagination, line breaking, and text shaping: every labelled fallback
  and golden baseline uses deterministic font mode at a fixed 150 dpi. Never
  record against system fonts.
- Public API of published crates: state the additive pre-1.0 impact for
  `rpptx-layout` and `rpptx`. Run their publish dry runs and assert every
  archive remains below 10 MiB.

## Hash harness

Expected unchanged, 49 of 49. The harness samples contain no presentation
media, and this story does not change Word output. Any delta is unexplained and
blocks integration.

## Implementation checklist

- [x] Complete and verify the F-215 dependency prefix.
- [x] Reconcile input type names and stable identity with completed F-215.
- [x] Add playback boundary and click-count tests to the existing timeline
  module.
- [x] Resolve valid posters and deterministic labelled fallbacks.
- [x] Keep audio and video payloads outside renderer image media.
- [x] Add checked playback-state evaluation without decoding codecs.
- [x] Add the combined deterministic facade result.
- [x] Add source-built integration, regression, and golden cases to the
  existing integration binary.
- [x] Prove existing static and timeline entry points remain unchanged.
- [x] Run focused `rpptx-layout`, `rpptx-render`, and `rpptx` checks, then
  every routed rider.

## Open questions

None. The additive nested result, F-216-owned fallback policy, unknown-duration
loop diagnostic, and deterministic labelled fallback are approved.

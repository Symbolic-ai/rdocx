# F-227, Animated GIF and video export

**Status**: approved
**Sprint**: S61
**Size**: L
**Depends on**: F-214, F-216

## Problem

The native facade renders one deterministic timeline state through
`Presentation::render_timeline_deterministic`, but it has no bounded frame
sampler or animated container encoder. F-227 must sample that path at a
declared rate, apply F-216 media state and fallbacks, encode animated GIF and
one named video format, and prove exact frame identity, timestamps, loops,
duration, and dimensions on Ubuntu and macOS.

The export must reuse prepared package and resolver state. Reassembling the
presentation for every frame would make video-sized output impractical and
would separate media events from the frame timestamp that produced them.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Beyond v1" and "Superseded".
- `docs/hld/03-architecture.md`, "The dependency rule" and the resolver and
  renderer seam.
- `docs/hld/08-rendering-spec.md`, timeline lowering and "The renderer's
  input".
- `docs/hld/10-bindings-spec.md`, "Native PowerPoint timing model".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", native timeline
  evidence, and "What CI runs".
- `docs/hld/14-development-backlog.md`, "F-227, Animated GIF and video
  export".

## Approach

Keep sampling and encoding in the `rpptx` facade. Do not add time-aware values
to `oxml-layout` or codec knowledge to `rpptx-layout` and `rpptx-render`.

Add a private `crates/rpptx/src/animation.rs` module and re-export its concrete
request and result values:

```rust
pub enum AnimationTransition {
    None,
    FromSlide(usize),
}

pub enum GifLoopBehavior {
    Once,
    Infinite,
    TotalPlays(std::num::NonZeroU16),
}

pub enum AnimationFormat {
    Gif { loop_behavior: GifLoopBehavior },
    MotionJpegAvi { quality: u8 },
}

pub struct AnimationSegment {
    pub slide_index: usize,
    pub duration_ms: u64,
    pub click_count: u32,
    pub transition: AnimationTransition,
}

pub struct AnimationExportOptions {
    pub frame_rate: u16,
    pub width_px: u32,
    pub height_px: u32,
    pub format: AnimationFormat,
    pub media_fallback: MediaFallbackPolicy,
}

pub struct DeterministicAnimation {
    pub bytes: Vec<u8>,
    pub frame_timestamps_ms: Vec<u64>,
    pub diagnostics: Vec<oxml_layout::Diagnostic>,
}

impl Presentation {
    pub fn export_animation_deterministic(
        &self,
        segments: &[AnimationSegment],
        options: AnimationExportOptions,
    ) -> Result<DeterministicAnimation>;
}
```

Reject empty segments, zero durations and frame rate, invalid dimensions,
invalid play count and JPEG quality, missing slides, arithmetic overflow, and
requests above fixed frame, pixel, and output-byte caps before rendering.

Each segment produces `ceil(duration_ms * frame_rate / 1000)` frames. Frame
`n` uses `floor(n * 1000 / frame_rate)` milliseconds and never samples beyond
the segment duration. Click count stays fixed within a segment. Callers express
click advances with later segments. Transition sources are explicit.

F-216 media events use the same timestamp and the selected fallback policy.
GIF delays use cumulative centisecond error distribution, giving 30 fps a
deterministic 3, 3, 4 cadence without duration drift. Motion JPEG in an AVI
container is the bounded video backend. It is deterministic, pure Rust, and
cross-platform, with explicit JPEG quality and no system codec or subprocess.

Stream one raster frame at a time into the encoder. Composite opaque output
over the rendered page background before GIF quantization or JPEG encoding.
Preserve exact requested dimensions. Reuse one prepared F-216 assembly across
the export instead of calling the public one-frame facade repeatedly.

Add `gif = "0.14.2"` to workspace dependencies. Reuse the existing pinned
`jpeg-encoder = "=0.6.0"` and `tiny-skia` workspace dependencies. Add the
encoders to the existing `rpptx` `render` feature. No new feature, trait,
generic, wrapper, builder, crate, integration binary, or binary asset is added.
The two format variants are the two concrete implementations that justify the
format enum today.

This is additive native Rust API for the pre-1.0 `rpptx` crate. No Python,
WASM, or CLI surface is added.

## Rejected alternatives

- H.264 or MP4 through a system FFmpeg process would depend on an unbounded
  external runtime and codec build.
- A one-implementer codec trait would violate the structural rules.
- Guessing duration or click advancement from producer metadata would make
  malformed or unsupported timing silently executable.
- Rebuilding package and resolver state once per frame would make ordinary
  video-sized exports impractical.
- A new animation feature has no separate named consumer. The existing render
  feature is the correct boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `sampling_uses_integer_millisecond_timestamps_without_crossing_segment_duration` | Exact frame count, floor timestamps, boundaries, click state, and overflow-safe arithmetic. |
| unit | `gif_delays_distribute_centisecond_error_without_duration_drift` | Deterministic 30 fps cadence, total duration, frame count, and loop metadata. |
| unit | `motion_jpeg_avi_indexes_every_frame_at_the_declared_rate_and_dimensions` | RIFF size, stream headers, dimensions, rate, length, frame chunks, index offsets, and deterministic JPEG payloads. |
| regression | `invalid_animation_requests_fail_before_rendering_or_allocating_output` | Invalid segments, rates, sizes, quality, slides, overflow, and caps fail closed. |
| regression | `animated_export_does_not_change_static_rendering` | Static PDF, raster, and all 49 hash-harness entries remain byte-identical. |
| integration | `animated_export_samples_transitions_clicks_and_media_fallbacks_in_order` | F-214 transitions and F-216 media events share timestamps and preserve siblings. |
| golden | `animated_gif_and_motion_jpeg_avi_match_the_reviewed_two_machine_manifest` | Frame hashes, timestamps, GIF loop metadata, AVI duration, dimensions, count, container hashes, and diagnostics match. |
| regression | `the_animated_export_manifest_rejects_one_frame_timestamp_loop_and_dimension_mutations` | Negative mutations prove every required golden field is sensitive. |

The exact backlog **test gate is golden**: "Frame hashes, timestamps, loop
behavior, and output dimensions match the reviewed manifest on two machines."

Use a source-built deck and an in-source textual manifest in the existing
`crates/rpptx/tests/integration.rs` binary. Run the exact golden in the Ubuntu
workspace suite and in the existing macOS presentation-fidelity job.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout, pagination, line breaking, and text shaping: every sampled baseline
  uses deterministic font mode. Run the two-machine golden with bundled fonts
  only.
- Crate dependency graph and cross-family uses: keep both encoders in the
  `rpptx` facade. Run `cargo tree -p rpptx -e normal` and confirm no shared
  crate acquires a format-crate dependency.
- Public API of a published crate: state the additive pre-1.0 impact. Run the
  `rpptx` publish dry run and assert its archive remains below 10 MiB.
- New module or file: explicit approval is required for
  `crates/rpptx/src/animation.rs`. It keeps scheduling, bounds, two encoders,
  and muxing out of the already large facade file.

The source-built golden is not an external oracle comparison. No bundled asset
is added.

## Hash harness

Expected unchanged. Animated output is opt-in and separate from
`scripts/hash_baseline.json`. Any static output delta is unexplained and blocks
integration.

## Implementation checklist

- [ ] Complete and verify F-215, then F-216, through dependency-prefix
  checkpoints.
- [ ] Reconcile final F-216 media and fallback types.
- [ ] Add the approved private animation module.
- [ ] Add request validation, hard bounds, and deterministic segment sampling.
- [ ] Reuse one prepared media and timeline assembly per export.
- [ ] Stream opaque deterministic raster frames into GIF and Motion JPEG AVI.
- [ ] Preserve ordered F-214 and F-216 diagnostics.
- [ ] Add unit, regression, integration, golden, and negative-mutation tests to
  existing targets.
- [ ] Run the exact golden on Ubuntu and macOS.
- [ ] Run focused `rpptx` checks, every rider, and the unchanged hash harness.

## Open questions

None. Motion JPEG AVI, explicit segments, the private animation module,
F-216-owned fallback policy, two-machine CI enforcement, native Rust scope, and
cumulative GIF timing are approved.

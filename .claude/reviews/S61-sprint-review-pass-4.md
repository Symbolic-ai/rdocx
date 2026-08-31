# S61 sprint review, pass 4

**Reviewed**: sprint/s61 at
`d754050c0ae7ca76c7ceea5fe21b9cede1907fdb` against
`7c7742616134ec34aebd6aef945f8682e70d19fd`, 52 files, 23,468 changed
lines, crates: oxml-media, oxml-opc, rpptx-layout, rpptx-oxml, rpptx-wasm,
rpptx
**Verdict**: 2 blocking, 1 should-fix, 0 nice-to-have
**Disposition**: both blocking findings and the should-fix finding are
fix-now. No finding is tracked-follow-up or human-action. Prior closed
dependency-prefix findings remain refuted by their cited remediation.

**Bound extension**: explicitly extended. Global pass 4 exceeds the configured
three passes only because earlier scheduled dependency-prefix boundaries
consumed passes 1 through 3. This is the scheduled final-closure boundary, not
a fourth pass at one review boundary. The recorded extension reason is
`scheduled dependency-prefix boundary`.

## Blocking

### B1, animation frames use fresh font ids with the static font table
`crates/rpptx/src/animation.rs:359`

**Disposition**: fix-now.

Both encoders rasterize every sampled page with
`prepared.assembly.layout.fonts`, which is the font table produced by the
initial static all-slide layout. The sampled page does not use that same font
manager. Timeline resolution constructs a fresh deterministic manager for each
sample at `crates/rpptx/src/lib.rs:2617`, and timeline lowering constructs a
second fresh manager at `crates/rpptx-render/src/timeline.rs:34`. Those managers
assign `FontId` values in their own slide-local resolution order, then their
font data is discarded when only `PageFrame` is returned.

This makes the final raster depend on an invalid identity assumption. A fresh
sample can assign one face to `FontId(0)` while the static all-slide table maps
that id to a face first encountered on another slide. A synthetic Audio or
Video label can also name a font absent from a presentation with no ordinary
static text. The raster backend draws text only when the supplied table
contains the run's exact id at `crates/oxml-pdf/src/raster.rs:352`, so the
failure is either wrong glyphs or silently missing text. The current golden
uses colour fields plus one Carlito fallback and does not vary font resolution
order across slides.

Make every sampled `PageFrame` and its `FontData` come from one coherent
prepared font identity. Add a regression with different first-used faces on
the outgoing and incoming slides plus a media-only deck whose placeholder font
has no static text consumer. The decoded GIF and AVI frames must retain the
correct text and labelled fallback.

### B2, the prepared export still clones the deck and reloads fonts per sample
`crates/rpptx-render/src/timeline.rs:27`

**Disposition**: fix-now.

The F-227 contract makes prepared resolver state load-bearing because rebuilding
per frame makes video-sized output impractical at
`.claude/plans/F-227-design.md:17`. The implementation prepares parsed package
resources once, but every incoming and outgoing sample calls the existing
timeline helper. That helper clones the complete `RenderInput`, including every
resolved slide and media map, then builds another deterministic `FontManager`.
Before that call, the facade has already built a separate deterministic manager
for sample resolution at `crates/rpptx/src/lib.rs:2617`.

`FontManager::new_deterministic` rebuilds its database by copying every bundled
font at `crates/oxml-layout/src/font.rs:664` and
`crates/oxml-layout/src/font.rs:1870`. A 10,000-frame valid request therefore
performs at least 20,000 font-database constructions and at least 10,000
whole-deck clones, with more work for two-page transitions. The existing
50-frame regression counts only calls to the outer `prepare_render_context` at
`crates/rpptx/src/lib.rs:5502`. It cannot observe either inner rebuild and so
does not prove the HLD claim that one resolver, font, and layout assembly is
retained at `docs/hld/08-rendering-spec.md:959`.

Refactor timeline lowering so animation owns reusable per-export resolver,
font, and layout state and lowers one selected slide without cloning the whole
deck. Extend instrumentation to count the actual inner font and render-input
preparations across incoming and outgoing samples.

## Should-fix

### S1, the completion ledger assigns a global fallback policy to each segment
`docs/sprints/AS_BUILT.md:10494`

**Disposition**: fix-now.

The completion record says each segment carries its own media fallback policy.
`AnimationSegment` carries slide index, duration, click count, and transition at
`crates/rpptx/src/animation.rs:126`. `media_fallback` belongs to the single
export-wide `AnimationExportOptions` at
`crates/rpptx/src/animation.rs:135`. The approved plan records the same split at
`.claude/plans/F-227-design.md:59`. Correct the AS_BUILT wording to say the
fallback policy applies to the export, not independently to each segment.

## Nice-to-have

None.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The M21 gate does not yet hold, as expected before F-218 through F-220, F-222,
F-223, and F-226 complete. Its exact contract is at
`docs/hld/14-development-backlog.md:1896`.

The S61 package and media-timeline gates remain supported. F-215 has exact
corpus round-trip evidence at `docs/sprints/AS_BUILT.md:10408`. F-216 has the
150 dpi poster, fallback, playback, diagnostic, and legacy-path evidence at
`docs/sprints/AS_BUILT.md:10463`, including the shared click-schedule
remediation from sprint review pass 3.

The F-227 exact manifest test passes at this reviewed HEAD and pins both GIF
and AVI identities at `crates/rpptx/tests/integration.rs:583`. The worker record
also reports the same constants on macOS and Linux arm64, while the exact locked
test is present in the Ubuntu and macOS jobs at `.github/workflows/ci.yml:193`
and `.github/workflows/ci.yml:413`. B1 shows that this fixture does not prove
correct animation pixels for mixed or dynamically introduced fonts. B2 shows
that the required prepared execution model does not hold. The S61 definition
of done at `docs/sprints/CURRENT_SPRINT.md:51` therefore does not hold at this
pass.

The sprint state records full verification at the exact reviewed HEAD with 49
of 49 harness entries unchanged at `.claude/scratch/S61-run.json:114`. Focused
review execution also passed the four animation integration regressions, the
exact F-216 golden, the exact GIF and AVI golden, the corrected rpptx-wasm
manifest contract, default and render-enabled WASM target checks, the rpptx
no-default library check, and the 49-entry hash harness. These passing tests do
not exercise B1 or observe the inner preparation work in B2.

## Not found

Interaction produced no finding beyond B1 and B2. F-227 consumes the completed
F-214 transition state and F-216 playback, fallback, and diagnostic path without
creating another media or timing model. The shared click schedule and
source-scoped diagnostic remediation from pass 3 remain present.

Duplication produced no separate finding. Media classification, package graph
mutation, timing projection, playback evaluation, animation sampling, and each
container writer retain one owner. The whole-deck clone in B2 is execution
duplication rather than a second source implementation.

Layering produced zero findings. `oxml-media` remains dependency-free. No
`oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency, and `gif`,
`jpeg-encoder`, and `tiny-skia` remain optional dependencies at the `rpptx`
facade edge.

Harness produced zero findings. All three plans and completion records declare
49 of 49 unchanged, the exact-HEAD sprint state agrees, the focused rerun
agrees, and no hash baseline file changed.

Dependencies produced zero findings. `gif` 0.14.2 has the named animated GIF
consumer, while the existing pinned JPEG encoder and tiny-skia have the MJPEG
and exact-raster consumers. No new crate or feature flag was added.

Native and WASM isolation produced zero findings. The default rpptx-wasm graph
contains none of the three codec dependencies. Its explicit render profile
forwards the existing `rpptx/render` feature, and no animation method is added
to the JavaScript wrapper. The d754050 remediation now asserts the complete
native render dependency set and its absence from the WASM manifest at
`crates/rpptx-wasm/src/lib.rs:462`. Both default and render-enabled WASM target
checks pass.

Surface produced zero findings. The additive pre-1.0 animation values and one
native entry point match the approved design at
`.claude/plans/F-227-design.md:39`. No Python, WASM, CLI, trait, generic,
builder, wrapper, crate, public module, integration binary, or binary asset was
added.

Sampling, caps, and containers produced zero findings. Integer ceiling frame
counts, floor timestamps, segment boundaries, cumulative GIF delays, loop
mapping, dimensions, frame and pixel caps, capped codec writes, in-place RIFF
patches, bounded index records, exact AVI structure, quality response, and
diagnostic order retain executable coverage. B1 and B2 concern the page and
prepared-state input to those otherwise bounded encoders.

Legacy compatibility produced zero findings. Static PDF and raster bytes remain
unchanged after export, and the existing static and timeline entry points retain
their exact diagnostics. Audio and video payload bytes remain outside renderer
media and both output encoders.

OOXML preservation produced zero findings. F-215 retains namespace-tolerant
reads, fixed-prefix schema order, direct relationship ownership, raw sibling
bytes, timing targets, duplicate-slide remapping, and atomic package mutation.
F-216 and F-227 add no parser or serializer path that weakens those guarantees.

HLD impact produced no finding outside B2. The sprint changes exactly the union
of the approved impact lists: 02, 03, 04, 06, 07, 08, 10, and 12. The package,
model, playback, fallback, API, CI, and container descriptions match current
behavior. B2 is the one contradicted prepared-state claim.

Delivery records produced no finding outside S1. All three F-IDs are done and
unowned in the current sprint and backlog, each has one tracker row and one
AS_BUILT entry, and the run state records consumed handoffs, integration
commits, completed features, and exact-HEAD full verification.

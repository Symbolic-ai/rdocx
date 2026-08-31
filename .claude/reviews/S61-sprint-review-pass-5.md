# S61 sprint review, pass 5

**Reviewed**: sprint/s61 at
`15b3579b95ca39eef5e373cb1b6ab29f78caf8c8` against
`7c7742616134ec34aebd6aef945f8682e70d19fd`, 55 files, 24,589 changed
lines, crates: oxml-media, oxml-opc, rpptx-layout, rpptx-oxml,
rpptx-render, rpptx-wasm, rpptx
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have
**Clean status**: not clean
**Disposition**: B1 and S1 are fix-now. No finding is tracked-follow-up or
human-action. Pass-4 B1, B2, and S1 are refuted by the cited remediation.

**Bound extension**: explicitly extended. Global pass 5 exceeds the configured
three passes only because earlier scheduled dependency-prefix boundaries
consumed passes. This is remediation pass 2 within the scheduled final-closure
boundary. The recorded extension reason is `scheduled dependency-prefix
boundary`.

## Blocking

### B1, final full verification is not recorded at the remediation HEAD
`.claude/scratch/S61-run.json:121`

**Disposition**: fix-now.

The last full verification in the sprint state is bound to
`d754050c0ae7ca76c7ceea5fe21b9cede1907fdb`, before the six-file remediation
commit reviewed here. The sprint contract requires full verification at
`docs/sprints/CURRENT_SPRINT.md:62`, and the run workflow requires the full gate
and its exact-HEAD record at `.claude/commands/run-sprint.md:244`. Focused
remediation checks and the unchanged 49-entry harness pass, but they cannot
stand in for the missing full workspace, documentation, packaging, and supply
chain gate. Run `/verify --full` plus the S61 risk riders at the reviewed HEAD
and record the passing result before another review or closure attempt.

## Should-fix

### S1, remediation exposes two unapproved rpptx-render entry points
`crates/rpptx-render/src/lib.rs:255`

**Disposition**: fix-now.

The remediation adds
`layout_presentation_with_font_manager_and_text_directions_mut` and
`layout_timeline_slide_with_font_manager` as ordinary public functions in the
published `rpptx-render` crate. The second function is at
`crates/rpptx-render/src/timeline.rs:38`. The approved F-216 contract explicitly
says `rpptx-render` gains no public surface at
`.claude/plans/F-216-design.md:109`, while F-227 approves additive public API
for the `rpptx` facade at `.claude/plans/F-227-design.md:116`. The F-227 public
API rider likewise names the `rpptx` archive at
`.claude/plans/F-227-design.md:167`.

This is not only an inventory mismatch. The timeline function accepts an
arbitrary caller-owned `FontManager` beside a `ResolvedTimelineSlide`, but the
correctness established by the pass-4 remediation depends on that exact manager
having assigned the slide's `FontId` values. The one-line documentation does
not state or enforce that identity precondition. Either keep these as an
explicitly internal cross-crate integration seam, or approve and document the
published API, its pre-1.0 impact, and its font-identity contract with a direct
public regression. Update the completion record if the published surface is
retained.

## Nice-to-have

None.

## Prior-pass dispositions

Pass-4 B1 is refuted. `PreparedRenderAssembly` now retains the deterministic
manager at `crates/rpptx/src/lib.rs:5443`. Incoming resolution, dynamic media
fallback, outgoing resolution, and both page-lowering calls use that same
manager at `crates/rpptx/src/lib.rs:2648`,
`crates/rpptx/src/lib.rs:2660`, `crates/rpptx/src/lib.rs:2700`, and
`crates/rpptx/src/lib.rs:2719`. Both encoders rasterize against the retained
manager's complete font table at `crates/rpptx/src/animation.rs:359` and
`crates/rpptx/src/animation.rs:413`. The focused identity test covers Caladea,
Liberation Sans, and the dynamically introduced Carlito `Video` label at
`crates/rpptx/src/animation.rs:857`. The GIF and AVI integration regression
repeats both slide outputs and distinguishes labelled fallback pixels at
`crates/rpptx/tests/integration.rs:141`.

Pass-4 B2 is refuted. Timeline lowering now consumes the selected evaluated
slide directly at `crates/rpptx-render/src/timeline.rs:51`, without cloning
`RenderInput`, and the animation path creates its deterministic manager once at
`crates/rpptx/src/lib.rs:5555`. The 50-frame regression counts one package,
resolver, media, layout, and font preparation and at most one active resolved
sample at `crates/rpptx/src/animation.rs:820`. Source inspection finds no
per-sample `RenderInput` clone or deterministic database construction in the
animation branch.

Pass-4 S1 is refuted. The completion record now assigns timeline position,
click count, and transition source to each segment, while assigning the one
fallback policy to the whole export at `docs/sprints/AS_BUILT.md:10492`.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The M21 gate does not yet hold, as expected before F-218 through F-220, F-222,
F-223, and F-226 complete. Its exact contract is at
`docs/hld/14-development-backlog.md:1896`.

The implemented S61 behavior gates are supported. F-215 retains its exact
package round-trip evidence at `docs/sprints/AS_BUILT.md:10408`. The F-216
poster, fallback, playback, diagnostic, and legacy-path golden remains recorded
at `docs/sprints/AS_BUILT.md:10463`. The F-227 completion record binds one
unchanged GIF and AVI identity to macOS and Linux arm64 at
`docs/sprints/AS_BUILT.md:10518`, and the exact locked test remains in both CI
jobs at `.github/workflows/ci.yml:193` and `.github/workflows/ci.yml:413`.

Focused pass-5 execution passed the two remediation unit regressions, the
mixed-font GIF and AVI integration regression, the exact GIF and AVI manifest,
the exact F-216 poster and playback golden, the exact legacy diagnostic
regression, all 97 `rpptx-render` tests, both rpptx-wasm target profiles, the
manifest-isolation regression, the rpptx no-default library check, formatting,
skill synchronization, and the unchanged 49-entry hash harness. B1 prevents
the full-verification clause from holding, while S1 keeps the bounded sprint
review unclean.

## Not found

Interaction produced zero findings beyond the refuted pass-4 B1 and B2.
F-227 uses one shared F-216 media-aware assembly for incoming and outgoing
slides, retains click state and ordered F-214, F-215, and F-216 diagnostics,
and rasterizes dynamic labels with the same font identity that shaped them.

Duplication produced zero findings. Media classification, package mutation,
timing evaluation, timeline lowering, sampling, and each container writer keep
one implementation owner. The remediation removes the whole-deck sample clone
rather than adding another rendering path.

Layering produced zero findings. `oxml-media` remains dependency-free. No
`oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency, and no codec edge
enters `rpptx-layout` or `rpptx-render`.

Harness produced zero findings. All plans and completion records declare 49 of
49 unchanged, no baseline file changed, and the focused pass-5 rerun reports
49 of 49 unchanged.

Gate produced no behavior finding beyond B1. Exact GIF and AVI constants remain
unchanged after the font remediation, and the 50-frame preparation regression
now observes the package, resolver, media, layout, font, and active-sample
boundaries that the HLD claims.

Docs produced no finding beyond S1. The pass-4 export-wide fallback wording is
corrected, the HLD accurately describes one retained deterministic assembly,
streaming, caps, diagnostics, and native codec ownership, and it does not claim
that the incomplete M21 representative-deck gate is closed.

Dependencies produced zero findings. The existing named GIF, Motion JPEG, and
raster consumers remain at the `rpptx` facade edge. No new dependency, crate,
feature, trait, generic, module, integration binary, or binary asset was added
by the remediation.

Native and WASM isolation produced zero findings. The d754050 manifest
regression checks the complete native render dependency set and its exclusion
from the default WASM graph at `crates/rpptx-wasm/src/lib.rs:462`. Default and
render-enabled `wasm32-unknown-unknown` checks pass. The JavaScript surface
still gains no animation method.

Surface compatibility produced no breaking-signature finding. The existing
owned-manager `rpptx-render` functions retain their signatures at
`crates/rpptx-render/src/lib.rs:231` and
`crates/rpptx-render/src/timeline.rs:23`, and the existing F-216 facade paths
remain source-compatible. S1 concerns the two additional exported seams.

Sampling, streaming, caps, and determinism produced zero findings. Checked
ceil and floor timestamp arithmetic, fixed segment click state, explicit
outgoing sources, frame and pixel caps, capped codec writes, in-place RIFF
patching, bounded index retention, immediate frame release, and cumulative GIF
delay distribution retain executable coverage.

Diagnostics and legacy compatibility produced zero findings. Package, timing,
media, and composition diagnostics retain their order, fallback replacement is
source scoped, and the existing static and one-frame timeline entry points
retain their exact page and diagnostic behavior. Static PDF and raster bytes
remain unchanged after export.

OOXML and payload preservation produced zero findings. F-215 still preserves
namespace-tolerant reads, fixed-prefix schema order, direct relationship
ownership, raw siblings, timing targets, remapped relationship ids, and
unsupported media bytes. F-216 and F-227 add no parser or serializer path, and
audio or video payloads never enter renderer media.

Delivery records produced no finding beyond B1 and S1. All three F-IDs are done
and unowned in the current sprint and backlog, each has one tracker row and one
AS_BUILT entry, and the feature handoffs remain consumed in sprint state.

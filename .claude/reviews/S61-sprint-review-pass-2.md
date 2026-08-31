# S61 sprint review, pass 2

**Reviewed**: sprint/s61 against 7c7742616134ec34aebd6aef945f8682e70d19fd,
41 files, 9,325 lines, crates: oxml-media, oxml-opc, rpptx-layout,
rpptx-oxml, rpptx
**Verdict**: 2 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, media commands and page effects disagree on the shared click ordinal
`crates/rpptx-layout/src/timeline.rs:440`

**Disposition**: fix-now.

The F-216 playback traversal schedules every non-audio and non-video node,
including `MediaCommand`, and advances its click ordinal for each click effect
at `crates/rpptx-layout/src/timeline.rs:564`. The F-214 page traversal instead
skips every audio, video, and media-command node before scheduling at
`crates/rpptx-layout/src/timeline.rs:699`. A click-triggered media command that
precedes a click-triggered shape effect is therefore click 1 for media, while
the page evaluator treats the later shape effect as click 1 too. The combined
frame reports both as active after one click even though their source order
requires the shape effect to wait for click 2.

The existing interaction test places the ordinary shape click before the media
click at `crates/rpptx-layout/src/timeline.rs:1597`, which is the ordering where
both traversals happen to agree. Make one shared schedule advance the ordinal
for media and page evaluation, then add the inverse source-order regression and
assert both the page state and media phase at clicks 0, 1, and 2.

### B2, the combined frame drops package and unmatched timing diagnostics
`crates/rpptx/src/lib.rs:5522`

**Disposition**: fix-now.

F-215 reports package-level limitations such as an unsupported content type on
`MediaInfo` at `crates/rpptx/src/lib.rs:2338`. F-216 does not consume those
diagnostics. `evaluate_slide_media` receives only the parsed slide, trim, and
timing model, so it returns evaluator diagnostics but cannot carry the F-215
unsupported-codec diagnostic into `DeterministicMediaTimelineFrame`. The codec
regression confirms the diagnostic only through a separate `media()` call at
`crates/rpptx/tests/integration.rs:8347`, then checks the rendered frame without
asserting any codec diagnostic.

The same assembly removes every generic retained-media timing diagnostic from
the incoming state at `crates/rpptx/src/lib.rs:5433`. It does so even for an
orphan command or a timing target that has no media picture and was therefore
not evaluated. The outgoing path removes the same diagnostics at
`crates/rpptx/src/lib.rs:5455` without evaluating or returning outgoing media
state at all. This can make unsupported embedded payloads, unmatched incoming
timing, and all outgoing media timing disappear from the only frame diagnostics
F-227 is told to preserve.

Merge the F-215 per-object diagnostics into the media-aware frame and suppress
the retained-timing marker only for a media node that was actually consumed.
Add regressions for an opaque codec, an unmatched incoming command, and an
outgoing media slide so every limitation remains source-scoped and observable.

## Should-fix

### S1, the HLD and completion ledger overstate the named golden cases
`docs/hld/12-testing-strategy.md:92`

**Disposition**: fix-now.

The HLD says the named golden constructs codec and unknown-duration cases at
`docs/hld/12-testing-strategy.md:94`, and the completion record says normalized
rows cover both at `docs/sprints/AS_BUILT.md:10470`. The named test begins at
`crates/rpptx/tests/integration.rs:8807` and ends after its linked-media case at
`crates/rpptx/tests/integration.rs:9003`. It contains supported poster and
fallback hashes, finite-loop playback rows, one click case, and one linked
case, but no opaque codec or unknown-duration loop row. Those cases live only
in separate tests at `crates/rpptx/tests/integration.rs:8347` and
`crates/rpptx-layout/src/timeline.rs:1632`.

Extend the named golden with the approved codec diagnostic and normalized
unknown-duration row, or narrow the HLD and completion record to describe the
actual split evidence. The approved golden contract at
`.claude/plans/F-216-design.md:134` still requires codec coverage, so fixing the
test is the contract-preserving choice.

## Nice-to-have

None.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The M21 gate does not yet hold. Its exact contract is at
`docs/hld/14-development-backlog.md:1896`, and F-227 remains pending at
`docs/sprints/CURRENT_SPRINT.md:40`. Later M21 stories still own SmartArt,
package variants, notes export, handout export, and the combined
representative-deck oracle. This is expected at the dependency-prefix boundary.

The completed F-215 package gate remains supported by the round-trip evidence
recorded in pass 1. The F-216 gate at
`docs/hld/14-development-backlog.md:1928` does not yet hold across the
integrated F-214 and F-215 boundary. The named 150 dpi golden has recorded
poster and labelled-fallback hashes plus normalized playback rows, but B1 shows
that its page and media state are not synchronized for the inverse click order.
B2 shows that the combined result can omit required diagnostics, and S1 limits
the scope that can be attributed to the named golden.

The sprint state records full verification at exact reviewed HEAD
73c3ce00362e0940ef3b2a2c70ae9843715ad9c1 at
`.claude/scratch/S61-run.json:76`, with 49 of 49 harness entries unchanged at
`.claude/scratch/S61-run.json:77`. Full verification cannot cover the missing
interaction cases identified here. This review inspected the executable tests
and exact-HEAD verification record. It did not rerun the broad suite.

## Not found

Interaction produced no finding beyond B1 and B2. F-216 consumes the F-215
shape id, picture-owned trim, timing model, poster relationship, and media kind
without creating a second XML model. Valid posters stay on the existing image
path, and payload bytes remain outside renderer media.

Duplication produced no separate finding beyond the divergent scheduling paths
in B1. Media classification remains singular in `oxml-media`, picture and
timing projection remain in `rpptx-oxml`, playback evaluation remains in
`rpptx-layout`, and package and render assembly remain in `rpptx`.

Layering produced zero findings. No Cargo manifest or lockfile changed. No
`oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency, and no media codec
knowledge moved into the renderer or shared output backends.

Harness produced zero findings. Both completed plans declare 49 of 49 expected
unchanged, both AS_BUILT entries report that result, and the exact-HEAD sprint
state agrees. No harness baseline file changed.

Docs produced no finding outside S1. The integrated HLD delta is exactly the
union of the F-215 impact list at `.claude/plans/F-215-design.md:197` and the
F-216 impact list at `.claude/plans/F-216-design.md:143`. The ownership,
layering, fallback, native API, and legacy-path prose otherwise matches the
integrated implementation.

Dependencies produced zero findings. No dependency, crate, feature flag, or
lockfile entry was added. F-227 is the named second consumer of the combined
media frame, as recorded at `.claude/plans/F-216-design.md:103`.

Surface produced zero findings. `MediaPlaybackPhase`, `EvaluatedMediaState`,
`MediaFallbackPolicy`, `DeterministicMediaTimelineFrame`, and the one facade
entry point match the approved additive pre-1.0 surface at
`.claude/plans/F-216-design.md:65`. All three fallback policies have current
call sites, including fail-closed coverage at
`crates/rpptx/tests/integration.rs:8417`.

Legacy compatibility produced zero findings. Diagnostic identity is enabled
only for the media-aware incoming or outgoing path at
`crates/rpptx/src/lib.rs:5389`. The exact static and existing timeline strings
and bytes are asserted by
`legacy_render_entry_points_keep_exact_unresolved_poster_diagnostics` at
`crates/rpptx/tests/integration.rs:8543`.

Source-scoped fallback identity produced zero findings outside B2. The private
tag contains producing slide, layout, or master scope plus local shape id, and
the same-id inherited regression at `crates/rpptx/tests/integration.rs:8614`
proves that fallback replacement leaves the inherited diagnostic intact.

Delivery ledgers produced zero findings. F-215 and F-216 are done and unowned
at `docs/sprints/CURRENT_SPRINT.md:38`, both are done in the backlog at
`docs/sprints/BACKLOG.md:414` and `docs/sprints/BACKLOG.md:415`, and both have
matching AS_BUILT and sprint tracker entries. The run state records both
consumed handoffs and completed integration commits at
`.claude/scratch/S61-run.json:3` and `.claude/scratch/S61-run.json:16`.

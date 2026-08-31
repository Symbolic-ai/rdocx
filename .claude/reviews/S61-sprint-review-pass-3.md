# S61 sprint review, pass 3

**Reviewed**: sprint/s61 at
e68d4befe1cdbd0d6404c2619ef13032147e3e22 against
7c7742616134ec34aebd6aef945f8682e70d19fd, 42 files, 19,924 lines,
crates: oxml-media, oxml-opc, rpptx-layout, rpptx-oxml, rpptx
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Disposition**: no findings require fix-now, tracked-follow-up, or
human-action. All three pass-2 findings are refuted by the remediation.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Prior finding disposition

### B1, shared click ordinals

**Disposition**: refuted.

`ClickSchedule` constructs one source-order map from the exact
`CommonTimeNode` addresses at `crates/rpptx-layout/src/timeline.rs:665`. Both
page evaluation and media evaluation construct that same schedule from the
same `CT_Timing` and consult it by exact node identity at
`crates/rpptx-layout/src/timeline.rs:249` and
`crates/rpptx-layout/src/timeline.rs:320`. The collector counts click-bearing
ordinary effects, containers, and media commands in one traversal at
`crates/rpptx-layout/src/timeline.rs:685`, while deliberately excluding the
audio and video carrier nodes that neither evaluator schedules as click
effects.

The original shape-before-media order now asserts synchronized page visibility
and media phase at clicks 0, 1, and 2 at
`crates/rpptx-layout/src/timeline.rs:1630`. The inverse media-before-shape order
does the same at `crates/rpptx-layout/src/timeline.rs:1667`. The expected rows
prove that exactly one side advances at click 1 and the later source node
advances at click 2 for each order.

### B2, source-scoped diagnostic flow

**Disposition**: refuted.

The public inspection path and render assembly now reuse one
`media_infos_for_slide` implementation at `crates/rpptx/src/lib.rs:2273` and
`crates/rpptx/src/lib.rs:2754`. The incoming media-aware path obtains those
package diagnostics for only the incoming slide, preserves media and diagnostic
source order, then appends evaluator diagnostics at
`crates/rpptx/src/lib.rs:5441`. The mapping is exhaustive over the F-215
diagnostic variants and includes the media shape id in every message at
`crates/rpptx/src/lib.rs:5565`.

Incoming retained-timing markers are no longer removed by message alone.
Suppression first derives the set of media shape ids whose states were
evaluated, then walks the exact timing tree in diagnostic source order and
suppresses only markers targeting those ids at
`crates/rpptx/src/lib.rs:5595`. Markers without a matching suppression entry
remain. The orphan incoming command regression preserves its marker at
`crates/rpptx/tests/integration.rs:8385`, and the outgoing-media regression
preserves both outgoing timing markers at
`crates/rpptx/tests/integration.rs:8429`. When incoming and outgoing refer to
the same slide, assembly clones the outgoing state before incoming suppression
at `crates/rpptx/src/lib.rs:5435`, so it does not silently convert the outgoing
diagnostic view into the incoming one.

The opaque-codec regression now requires the combined frame itself to carry the
source-scoped F-215 content-type diagnostic at
`crates/rpptx/tests/integration.rs:8347`.

### S1, named golden coverage

**Disposition**: refuted.

The named 150 dpi golden begins at
`crates/rpptx/tests/integration.rs:8890`. It retains the existing poster hash
at `crates/rpptx/tests/integration.rs:8920` and the existing Audio and Video
labelled-fallback hashes at `crates/rpptx/tests/integration.rs:8954`. The same
named test now directly asserts the opaque-codec frame diagnostic at
`crates/rpptx/tests/integration.rs:8962` and the normalized unknown-duration
loop state and diagnostic row at `crates/rpptx/tests/integration.rs:8981`.
Existing normalized finite-loop, click, and linked-media evidence continues
after those additions at `crates/rpptx/tests/integration.rs:9034`.

The executable test now matches the HLD claim at
`docs/hld/12-testing-strategy.md:92` and the completion ledger claim at
`docs/sprints/AS_BUILT.md:10463`.

## Milestone gate

The M21 gate is: "one representative modern deck round-trips its comments,
sections, SmartArt, media, animation timeline, signatures, and package variant
without repair. Its static frames, animated export, notes, and handouts match
the pinned PowerPoint oracle at their declared fidelity boundaries."

The M21 gate does not yet hold. Its exact contract is at
`docs/hld/14-development-backlog.md:1896`, and F-227 remains pending at
`docs/sprints/CURRENT_SPRINT.md:40`. Later M21 stories still own SmartArt,
package variants, notes export, handout export, and the combined
representative-deck oracle. This remains expected at the completed F-215 and
F-216 dependency-prefix boundary.

The completed F-215 package gate remains supported by the exact corpus
round-trip evidence recorded at `docs/sprints/AS_BUILT.md:10408`. The F-216
gate at `docs/hld/14-development-backlog.md:1928` now holds for this boundary.
The named golden pins exact 150 dpi poster and fallback pixels, playback rows,
opaque-codec diagnostics, and unknown-duration behavior. The two click-order
regressions additionally prove the integrated F-214 page state and F-216 media
state use one ordinal schedule.

The sprint state records full integrated verification before remediation at
73c3ce00362e0940ef3b2a2c70ae9843715ad9c1 with 49 of 49 harness entries
unchanged at `.claude/scratch/S61-run.json:83`. The exact-HEAD remediation
commit records the rpptx-layout and rpptx integration tests, named 150 dpi
golden, publication riders, and the same unchanged harness result. This review
inspected the executable remediation and recorded verification evidence. It
did not rerun the broad suite.

## Not found

Interaction produced zero findings. F-216 consumes the F-215 shape id,
picture-owned trim, timing model, poster relationship, package diagnostics,
and media kind through one assembly path. B1 and B2 account for the pass-2
interaction gaps and are refuted above.

Duplication produced zero findings. The remediation extracted the existing
media inspection implementation for reuse at
`crates/rpptx/src/lib.rs:2754`, and page and media evaluation share one click
schedule at `crates/rpptx-layout/src/timeline.rs:665`. Media classification,
PresentationML projection, playback evaluation, and facade assembly retain
their existing single owners.

Layering produced zero findings. No Cargo manifest or lockfile changed. No
`oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency, and audio or video
payloads still remain outside renderer image admission as required at
`docs/hld/08-rendering-spec.md:1108`.

Harness produced zero findings. Both completed plans declare 49 of 49 expected
unchanged at `.claude/plans/F-215-design.md:220` and
`.claude/plans/F-216-design.md:161`. Both AS_BUILT entries record that result,
and no harness baseline file changed in the sprint diff.

Gate produced zero completed-boundary findings. The F-215 round-trip and F-216
golden gates have executable evidence. The broader S61 definition of done and
M21 gate remain open because F-227 and later milestone work are still pending,
rather than being inferred from the completed prefix.

Docs produced zero findings. The HLD files changed by F-215 and F-216 are the
exact union of their impact lists at `.claude/plans/F-215-design.md:197` and
`.claude/plans/F-216-design.md:143`. Their ownership, layering, fallback,
diagnostic, native API, legacy-path, and named-golden prose matches the
integrated implementation after S1 remediation.

Dependencies produced zero findings. No dependency, crate, feature flag, or
lockfile entry was added. F-227 remains the named consumer of the combined
media frame at `.claude/plans/F-216-design.md:103`.

Surface produced zero findings. `MediaPlaybackPhase`, `EvaluatedMediaState`,
`MediaFallbackPolicy`, `DeterministicMediaTimelineFrame`, and the one facade
entry point match the approved additive pre-1.0 surface at
`.claude/plans/F-216-design.md:60`. All three fallback policies have current
call sites, including fail-closed coverage at
`crates/rpptx/tests/integration.rs:8502`.

Legacy compatibility produced zero findings. The media-aware method remains an
additive facade entry point at `crates/rpptx/src/lib.rs:904`, and the exact
static and existing timeline diagnostic strings and bytes remain asserted at
`crates/rpptx/tests/integration.rs:8627`.

Source-scoped fallback identity produced zero findings. Diagnostic identity is
enabled only for requested incoming or outgoing media-aware slides at
`crates/rpptx/src/lib.rs:5391`, and the same-id inherited regression remains at
`crates/rpptx/tests/integration.rs:8698`.

Delivery ledgers produced zero findings. F-215 and F-216 are done and unowned
at `docs/sprints/CURRENT_SPRINT.md:38`, both are done in the backlog at
`docs/sprints/BACKLOG.md:414`, and both have matching AS_BUILT and sprint
tracker entries. F-227 remains pending, and the M21 and full-sprint gates remain
open rather than being overstated.

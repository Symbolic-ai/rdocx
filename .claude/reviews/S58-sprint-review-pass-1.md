# S58 sprint review, pass 1

**Reviewed**: `sprint/s58` against
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 32 files, 2,615 changed lines,
crates: `rdocx-layout`, `rdocx`
**Verdict**: 0 blocking, 1 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

### S1, F-202 delivery record overstates its HLD impact

`docs/sprints/AS_BUILT.md:9425` says F-202 touched
`docs/hld/03-architecture.md` and `docs/hld/14-development-backlog.md` in
addition to three mechanism and test documents. The approved F-202 impact list
contains only `08-rendering-spec.md`, `12-testing-strategy.md`, and
`15-build-and-toolchain.md` at `.claude/plans/F-202-design.md:80`. The reviewed
sprint has no architecture-document delta, and its backlog delta belongs to
the later multilingual, release, workflow, and reporter-story planning. This
makes the feature-local delivery record disagree with both its plan and its
implementation. Change the F-202 entry to list only the three approved HLD
files.

## Nice-to-have

None.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

That end-of-milestone gate does not yet hold at this partial S58 checkpoint.
The shared substrate and hyphenation work remain in progress, while complex
shaping and bidirectional layout remain pending at
`docs/sprints/CURRENT_SPRINT.md:36`. This is not a checkpoint blocker because
the reviewed HEAD contains only completed F-202 and F-X061 implementation work
plus sprint design and ledger changes. It does not claim completion of the
remaining milestone stories.

The applicable checkpoint gates hold. The exact F-202 engine regression at
`crates/rdocx-layout/src/engine.rs:8806` and public facade regression at
`crates/rdocx/tests/regression_test.rs:215` each passed and prove bounded warm
page work for a thousand-page document. Four focused F-X061 workflow tests,
including ordinary dependency chains, release checkpoints, and resume metadata
preservation at `scripts/test_sprint_workflow.py:4173`, passed. The integrated
delivery record reports the dependency-prefix full workspace gate at
`docs/sprints/AS_BUILT.md:9432` and the independent deterministic hash check
reproduces 49 of 49 unchanged entries.

## Not found

- **Interaction**: F-202 changes retained Word pagination while F-X061 changes
  sprint workflow state and command ordering. No production path or mutable
  state is shared between them. Their integrated focused regressions pass at
  the reviewed checkpoint.
- **Duplication**: F-202 uses the existing restart cache and adds test-only
  invocation accounting in the existing engine. F-X061 extends the existing
  run-sprint command and state driver. Neither feature adds a parallel helper,
  module, or state authority.
- **Layering**: no Cargo manifest, lockfile, or `oxml-*` source changed. No
  reverse format dependency entered the workspace graph.
- **Harness**: both completed feature records declare an unchanged 49-entry
  harness at `docs/sprints/AS_BUILT.md:9436` and
  `docs/sprints/AS_BUILT.md:9475`. The independent check reproduces that
  result.
- **Gate**: the 1,024-entry bound remains subordinate to the existing byte
  ceilings at `crates/rdocx-layout/src/engine.rs:740`, and the F-X061 command
  requires dependency-prefix review and verification before a dependent wave
  at `.claude/commands/run-sprint.md:127`. Focused mutation regressions for
  both contracts pass.
- **Docs**: apart from S1, F-202 updates its three planned HLD files and F-X061
  updates its three planned HLD files. The resulting pagination limits,
  dependency checkpoint sequence, and resume behavior agree with the reviewed
  implementations.
- **Deps**: no package dependency or lock record changed.
- **Surface**: no production public API was added. F-202's counter is private
  and test-only, and F-X061 changes repository workflow tooling rather than a
  published crate surface.

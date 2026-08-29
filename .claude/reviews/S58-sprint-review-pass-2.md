# S58 sprint review, pass 2

**Reviewed**: `sprint/s58` against
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 33 files, 2,697 changed lines,
crates: `rdocx-layout`, `rdocx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

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
the reviewed implementation delta contains the completed F-202 and F-X061
work. It does not claim completion of the remaining milestone stories.

The applicable checkpoint gates hold at
`043e26bd6f5d02e76e06dedbd8cc8f322b438c84`. The exact F-202 engine regression
at `crates/rdocx-layout/src/engine.rs:8806` and facade regression at
`crates/rdocx/tests/regression_test.rs:215` each pass. Four focused F-X061
workflow tests pass, covering ordinary dependency chains, release checkpoints,
and resume metadata preservation beginning at
`scripts/test_sprint_workflow.py:4173`. The deterministic hash check also
reports all 49 entries unchanged.

## Not found

- **Interaction**: F-202 changes retained Word pagination while F-X061 changes
  sprint workflow state and command ordering. No production path or mutable
  state is shared between them, and their integrated focused regressions pass.
- **Duplication**: F-202 uses the existing restart cache and adds test-only
  invocation accounting in the existing engine. F-X061 extends the existing
  command and state driver. Neither adds a parallel helper, module, or state
  authority.
- **Layering**: no Cargo manifest, lockfile, or `oxml-*` source changed. No
  reverse format dependency entered the workspace graph.
- **Harness**: both completed feature records declare an unchanged 49-entry
  harness at `docs/sprints/AS_BUILT.md:9434` and
  `docs/sprints/AS_BUILT.md:9473`. The independent current-HEAD check
  reproduces that result.
- **Gate**: the 1,024-entry bound remains subordinate to the existing byte
  ceilings at `crates/rdocx-layout/src/engine.rs:740`, and the F-X061 command
  requires dependency-prefix review and verification before a dependent wave
  at `.claude/commands/run-sprint.md:127`. Focused regressions for both
  contracts pass.
- **Docs**: pass-1 S1 is remediated. The F-202 delivery record now lists
  `08-rendering-spec.md`, `12-testing-strategy.md`, and
  `15-build-and-toolchain.md` at `docs/sprints/AS_BUILT.md:9425`, exactly
  matching the approved impact list at `.claude/plans/F-202-design.md:80`.
  F-X061 also updates exactly its three planned HLD files. The resulting limits
  and workflow behavior agree with the implementations.
- **Deps**: no package dependency or lock record changed.
- **Surface**: no production public API was added. F-202's counter is private
  and test-only, and F-X061 changes repository workflow tooling rather than a
  published crate surface.

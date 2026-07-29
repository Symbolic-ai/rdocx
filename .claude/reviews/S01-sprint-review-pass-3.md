# S01 sprint review, pass 3

**Reviewed**: `sprint/s01` at `a3c80bf` against
`7646bcc9f56ecdb0ef65efa8c7503ba427312004`, 139 files, 11,241 changed
lines, crates: `rdocx-layout`, `rdocx-pdf`, `rdocx`
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, Close preflight does not bind verification or review evidence to HEAD

`scripts/sprint_workflow.py:344`

`record-review` stores only the pass number and finding counts, and
`record-verification` stores only scope, outcome, and harness text at
`scripts/sprint_workflow.py:367`. Close preflight accepts those unversioned
records at `scripts/sprint_workflow.py:449` without checking the commit they
covered. This review observed `close-preflight S01: ok, ready to close` at
`a3c80bf` even though the latest recorded clean review covered `438289e` and
the merge added 601 lines of product code afterward. The command can therefore
authorize closure after unreviewed or unverified code lands. A fix must record
the reviewed and verified commit IDs, compare both with current HEAD, and
refuse stale evidence.

## Should-fix

### S1, The pre-churn release story still targets an obsolete version

`docs/sprints/BACKLOG.md:54`

F-012 remains “Tag v0.2.1”, and the milestone contract repeats that version at
`docs/hld/14-development-backlog.md:54`, but the reconciled repository is
already version 0.3.0 at `Cargo.toml:15` and main carries the v0.3.0 release.
`docs/sprints/CURRENT_SPRINT.md:8` also says the release remains F-012 in S02.
Before S02 starts, replan F-012 and the M1 gate around the actual published
state so the story does not ask for a lower-version release after v0.3.0.

## Nice-to-have

None.

## Reconciled-main interactions

- The merged `Document` additions preserve F-005’s greatest-suffix scan at
  `crates/rdocx/src/document.rs:137` and collision-safe allocator at
  `crates/rdocx/src/document.rs:592`. This review observed all 16 rdocx
  regression tests pass, including all four image-allocation cases.
- Every workspace package and internal dependency is now version 0.3.0 at
  `Cargo.toml:15` and `Cargo.toml:27`. No mixed internal version requirement was
  found.
- The run-content and table preservation changes do not alter the seven sample
  states covered by F-003. This review reran
  `python3 scripts/hash_harness.py --check` and observed all 28 entries match.
- All six completed F-IDs return no durable-record problems through
  `scripts/sprint_workflow.py:174`. Their current-sprint rows, design plans,
  AS_BUILT entries, and tracker rows remain present after reconciliation.

## Milestone gate

The written M1 gate is: workspace tests are green, the hash baseline reproduces
on a second machine, and `v0.2.1` is tagged.

The post-merge full gate was observed as passing, and this review independently
observed the focused rdocx regression suite and the 28-entry hash check pass.
Pass 2 recorded the clean Linux reproduction under
`rust:1.97.1-bookworm` at `.claude/reviews/S01-sprint-review-pass-2.md:29`.

The gate does not hold literally because its v0.2.1 release condition is stale,
which is S1. S01 does not close M1, but the condition must be reconciled before
F-012 is executed.

## Not found

- `interaction`: no product defect between F-001 through F-006 and the merged
  main changes.
- `duplication`: no duplicate implementation introduced by reconciliation.
- `layering`: no forbidden dependency direction was added.
- `harness`: the 28-entry baseline and its manifest digest are unchanged.
- `docs`: apart from S1, the S01 HLD updates and delivery records remain
  consistent with the integrated feature code.
- `deps`: no new external dependency entered through the reconciliation.
- `surface`: the merged public APIs came from main and do not conflict with the
  deterministic rendering surface added by S01.

# S01 sprint review, pass 1

**Reviewed**: `sprint/s01` against `4cc77e1ada581046d90c5c482cee0d20d276a145`,
135 files, 10,880 changed lines, crates: `rdocx-layout`, `rdocx-pdf`, `rdocx`
**Verdict**: 4 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, The hash harness is neither a PR gate nor proven on a second machine

`.github/workflows/ci.yml:15`

The sprint contract requires the recorded baseline to reproduce byte for byte on
a second machine at `docs/sprints/CURRENT_SPRINT.md:56`, and the testing strategy
requires every M1 through M6 PR to gate on the harness at
`docs/hld/12-testing-strategy.md:49`. The CI workflow defines test, clippy, fmt,
doc, MSRV, and supply-chain jobs, but no job runs
`python3 scripts/hash_harness.py --check`. The durable evidence records a local
28-entry pass and a local injected-writer failure only. There is no observed
second-machine result. This leaves the sprint definition of done unproved and
allows later PRs to bypass the safety net. A fix must add the harness to the PR
gate and record an observed pass on a machine distinct from the baseline host.

### B2, The canonical full verification procedure is not executable in S01

`.claude/commands/verify.md:15`

The literal clippy command omits the `--` before `-D warnings` and exits with
`unexpected argument '-D'`. The no-default command at line 42 names
`oxml-layout`, which is not a workspace member, and the wasm command at line 46
names `rpptx-wasm`, which does not exist. `Cargo.toml:3` shows that the current
workspace has `rdocx-layout` and `rdocx-wasm` instead. Step 10 at line 52 also
requires a crate-size assertion without giving an executable command. The
recorded verification substituted current crates and manually inspected package
sizes, so it did not execute the canonical full gate as written. A fix must make
every current-stage command literal and executable, including the clippy
separator and an explicit package-size check, while preserving the intended
future checks when those crates land.

### B3, Run-sprint requires an integration phase that the state authority rejects

`.claude/commands/run-sprint.md:129`

The orchestrator must set phase `integration` before integrating the first
worker, but `scripts/sprint_workflow.py:69` omits that value from `PHASES` and
`cmd_set_phase` rejects it as unknown. This exact workflow therefore cannot
follow its own required transition. A fix must add the integration phase to the
state model and cover the declared run-sprint phase sequence with a state-machine
test.

### B4, Close preflight does not validate the delivery ledgers it claims to gate

`scripts/sprint_workflow.py:410`

The comment says the trackers must agree with run state, but the implementation
checks only each completed F-ID's status in `BACKLOG.md`. It never checks
`CURRENT_SPRINT.md`, design-plan completion, AS_BUILT coverage, review coverage,
or the `SPRINT_TRACKER.md` row that `/run-sprint` says preflight must reconcile.
A missing completion entry or a stale current-sprint row can therefore pass
`close-preflight`. A fix must validate the full mechanically checkable delivery
record, directly or through an implemented consistency check, before reporting
the sprint ready to close.

## Should-fix

### S1, The active sprint goal claims a release that belongs to S02

`docs/sprints/CURRENT_SPRINT.md:5`

The goal says the three defects are fixed and released. The canonical sprint
plan only promises that they are fixed at `docs/sprints/SPRINT_PLAN.md:21`, and
the release story F-012 remains pending in S02 at
`docs/sprints/BACKLOG.md:54`. Correct the S01 goal to match the roadmap so closing
this sprint does not assert a release that did not occur.

## Nice-to-have

None.

## Milestone gate

The M1 gate is: workspace tests are green, the hash baseline reproduces on a
second machine, and `v0.2.1` is tagged.

The workspace portion holds. The integrated verification records the full
workspace test as passing, and this review observed the focused deterministic
font, JPEG marker, and image-counter regression tests pass. The baseline also
matches all 28 entries locally, and the F-003 evidence records the deliberate
writer mutation being detected.

The gate does not yet hold. No observed second-machine reproduction is recorded,
which is B1. The `v0.2.1` tag is assigned to pending F-012 in S02, so S01 is not
the milestone-closing sprint. That later tag is expected, but S01's own
second-machine definition of done remains unmet now.

## Not found

- `interaction`: no product-code interaction defect among F-001 through F-006.
- `duplication`: no duplicate helper or competing implementation introduced.
- `layering`: no new `oxml-*` dependency edge, and no forbidden direction.
- `harness delta`: the initial 28-entry baseline is isolated in F-003, its
  reason and SHA-256 match AS_BUILT, and later feature commits do not alter it.
- `deps`: no new external dependency was added.
- `surface`: the additive deterministic APIs are required by F-001 and consumed
  by F-003. No unrequested public surface was found.
- `licensing`: Caladea carries the Apache-2.0 text and notice, and the
  family-coverage test passes.

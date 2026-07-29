# S01 sprint review, pass 2

**Reviewed**: `sprint/s01` at `438289e` against
`4cc77e1ada581046d90c5c482cee0d20d276a145`, 138 files, 11,161 changed
lines, crates: `rdocx-layout`, `rdocx-pdf`, `rdocx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Pass 1 resolution

- **B1 resolved.** The pull-request workflow now runs
  `python3 scripts/hash_harness.py --check` at `.github/workflows/ci.yml:25`.
  This review streamed a clean `git archive HEAD` into
  `rust:1.97.1-bookworm` and observed `hash_harness: 28 entries match`. The
  same check also matched all 28 entries on Darwin.
- **B2 resolved.** The canonical lint command has the argument separator at
  `.claude/commands/verify.md:15`. The current no-default package and wasm
  package are named at `.claude/commands/verify.md:42` and
  `.claude/commands/verify.md:47`, with the future rename and package addition
  assigned to their F-IDs. The executable archive-size assertion is at
  `.claude/commands/verify.md:53`. This review observed the literal clippy,
  workspace test, no-default, wasm, docs, publish dry-run, size, and
  supply-chain checks pass.
- **B3 resolved.** `integration` is an accepted phase at
  `scripts/sprint_workflow.py:77`, and the declared phase sequence is exercised
  at `scripts/test_sprint_workflow.py:28`. This review observed both workflow
  unit tests pass.
- **B4 resolved.** Close preflight now reconciles `CURRENT_SPRINT.md`, completed
  design plans, `AS_BUILT.md`, and `SPRINT_TRACKER.md` through
  `scripts/sprint_workflow.py:174`, then checks the latest sprint-review file at
  `scripts/sprint_workflow.py:472`. All six completed F-IDs currently return no
  durable-record problems. The current preflight correctly refuses while pass
  1 remains the latest recorded review, so a stale blocking review cannot be
  bypassed.
- **S1 resolved.** The sprint goal now assigns release to F-012 in S02 at
  `docs/sprints/CURRENT_SPRINT.md:8`.

## Milestone gate

The M1 gate is: workspace tests are green, the hash baseline reproduces on a
second machine, and `v0.2.1` is tagged.

The workspace and baseline portions hold. This review observed the full
workspace test pass and observed all 28 baseline entries reproduce from a clean
committed-tree archive on Linux under `rust:1.97.1-bookworm`. It also observed
the focused no-default and wasm checks pass. The deliberate writer mutation is
recorded with its seven changed `document.xml` digests at
`docs/sprints/AS_BUILT.md:129`.

The milestone gate as a whole does not yet hold because `v0.2.1` is not tagged.
That release remains assigned to F-012 in S02 at
`docs/sprints/BACKLOG.md:54`, so it is not a closure condition for S01.

## Not found

- `interaction`: no product-code interaction defect among F-001 through F-006.
- `duplication`: no duplicate helper or competing implementation introduced.
- `layering`: no new forbidden dependency edge from an `oxml-*` crate.
- `harness`: the single 28-entry initial delta remains isolated to F-003, its
  manifest digest matches `docs/sprints/AS_BUILT.md:134`, and Darwin and Linux
  both reproduce it.
- `gate`: S01's definition of done has executable evidence for workspace tests,
  cross-platform baseline reproduction, mutation detection, and bundled-font
  licence coverage.
- `docs`: the F-001 through F-006 HLD impact lists are reflected in the current
  specification, and the incorrect S01 release claim is gone.
- `deps`: no new external dependency was added.
- `surface`: the deterministic public APIs are required by F-001 and consumed
  by F-003. No unrequested public surface was found.

# S33 sprint review, pass 3

**Reviewed**: `sprint/s33` at `eca3ed3` against merge base `9c2381b`, 58
files and 6,990 changed lines, with 6,848 additions and 142 deletions. Crates:
new `oxml-py-support`, new `rdocx-py`, and published `rdocx` facade changes.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Run-sprint disposition**: 0 fix-now, 0 tracked-follow-up, 0 human-action,
0 refuted findings

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Earlier findings

### S1, remains resolved

The durable dependency records still agree. F-131 names F-130 and F-132 at
`.claude/plans/F-131-design.md:6`, and F-132 names F-129 and F-130 at
`.claude/plans/F-132-design.md:6`. The sprint order at
`docs/sprints/CURRENT_SPRINT.md:40` matches the backlog dependencies at
`docs/hld/14-development-backlog.md:1015` and
`docs/hld/14-development-backlog.md:1024`.

### S2, remains resolved

Index normalization, Python `Length` construction, and enum construction still
have one crate-local implementation each at `crates/rdocx-py/src/lib.rs:24`,
`crates/rdocx-py/src/lib.rs:36`, and `crates/rdocx-py/src/lib.rs:43`. No second
definition has returned in paragraph, run, table, or formatting code.

## Close-preflight owner semantics

Completed durable rows now accept both the current blank owner cell and the
legacy dash, while rejecting a named owner at
`scripts/sprint_workflow.py:209`. Completed run-state entries independently
require a cleared owner at `scripts/sprint_workflow.py:238`, and
`cmd_close_preflight` applies that check at `scripts/sprint_workflow.py:520`.
Carried entries are intentionally outside that completed-only check, so a
retained carried worker remains valid.

The regression covers dash, blank, and named durable owners at
`scripts/test_sprint_workflow.py:675`, completed and carried run-state owners at
`scripts/test_sprint_workflow.py:708`, and the actual close-preflight hook at
`scripts/test_sprint_workflow.py:722`. Deleting the hook makes the command-level
test fail. All five canonical run-state owners are cleared at
`.claude/scratch/S33-run.json:9`, and the five completed CURRENT rows have blank
owner cells at `docs/sprints/CURRENT_SPRINT.md:32`.

The live preflight rejected only because its latest recorded review and full
verification cover the preceding review commit, as shown at
`.claude/scratch/S33-run.json:80` and `.claude/scratch/S33-run.json:97`. It
reported no owner, handoff, backlog, plan, tracker, or AS_BUILT inconsistency.
Recording this pass and a full verification at the resulting final HEAD remains
the normal closure step, not a finding in the reviewed implementation.

## Milestone gate

The M13 end gate remains: "wheels install and pass the parity suites on every
target platform" at `docs/hld/14-development-backlog.md:994`.

The milestone is not yet complete. The ledger records five of eighteen M13
stories done and thirteen pending at `docs/sprints/BACKLOG.md:31`. Type stubs,
broad python-docx parity, rpptx bindings, platform wheels, and the PR Python job
remain scheduled at `docs/hld/14-development-backlog.md:1032` through
`docs/hld/14-development-backlog.md:1055`.

The S33 slice retains concrete passing evidence:

- The three owner regressions passed, and the full sprint-workflow suite passed
  all 24 tests.
- `cargo test -p oxml-py-support` passed five tests, including stale revision
  reporting and canonical Length conversion.
- `cargo test -p rdocx-py --lib` passed the exact public layout-error mapping
  regression at `crates/rdocx-py/src/lib.rs:103`.
- `cargo test -p rdocx --test integration_test` passed all 76 facade tests.
- The pass-2 wheel rebuilt from the integrated binding code passed all 31
  installed-package tests. The commits after that evidence change only review
  and workflow files.
- `python3 scripts/hash_harness.py --check` reported all 28 entries matching,
  consistent with the five declarations at `docs/sprints/AS_BUILT.md:4734`,
  `docs/sprints/AS_BUILT.md:4775`, `docs/sprints/AS_BUILT.md:4815`,
  `docs/sprints/AS_BUILT.md:4855`, and `docs/sprints/AS_BUILT.md:4892`.
- Formatting, prose, generated-skill sync, and diff whitespace checks passed.

## Ledger and status consistency

All five sprint rows remain done and unowned at
`docs/sprints/CURRENT_SPRINT.md:30`. The same five rows are done at
`docs/sprints/BACKLOG.md:263`, have one tracker row each at
`docs/sprints/SPRINT_TRACKER.md:192`, and have durable AS_BUILT entries beginning
at `docs/sprints/AS_BUILT.md:4700`. The M13 count remains 18 total, 5 done, 0 in
progress, and 13 pending. The repository total remains 159, 142 done, 0 in
progress, and 17 pending at `docs/sprints/BACKLOG.md:33`.

## Not found

- **Interaction**: the owner-preflight correction does not alter feature state,
  worker cleanup, binding behavior, or carried-worker semantics.
- **Duplication**: S2 remains resolved, and the small owner validation has one
  implementation reused by close-preflight.
- **Layering**: no crate dependency changed after pass 2. The production
  binding edges remain at `crates/rdocx-py/Cargo.toml:28`, and the concrete
  `oxml-layout` test edge remains dev-only at
  `crates/rdocx-py/Cargo.toml:34`.
- **Harness**: no baseline changed, and the fresh 28-entry check agrees with
  every S33 AS_BUILT declaration.
- **Docs**: no new code contradicts the path, facade, Python, threading,
  packaging, release, dependency, or workflow contracts.
- **Dependencies**: the preflight fix adds only standard-library test imports.
  PyO3 remains exactly 0.29.0 with `abi3-py39` at `Cargo.toml:103` and one
  coherent lockfile family at `Cargo.lock:551`.
- **Surface**: the new validation helpers are script-local. No Rust or Python
  package API changed after pass 2.
- **Artifacts**: no sample output, package archive, cache, handoff, progress
  note, or unrelated file entered the sprint diff.

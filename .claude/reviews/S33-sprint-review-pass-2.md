# S33 sprint review, pass 2

**Reviewed**: `sprint/s33` at `d42beab` against merge base `9c2381b`, 56
files and 6,779 changed lines, with 6,638 additions and 141 deletions. Crates:
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

### S1, resolved

The completed records now state the integrated dependency order consistently.
F-131 names F-130 and F-132 at `.claude/plans/F-131-design.md:6`, and F-132
names F-129 and F-130 at `.claude/plans/F-132-design.md:6`. The sprint sequence
now states the same order at `docs/sprints/CURRENT_SPRINT.md:40`, matching the
canonical backlog dependencies at `docs/hld/14-development-backlog.md:1015`
and `docs/hld/14-development-backlog.md:1024`.

### S2, resolved

Index normalization, Python `Length` construction, and enum construction each
have one crate-local implementation at `crates/rdocx-py/src/lib.rs:24`,
`crates/rdocx-py/src/lib.rs:36`, and `crates/rdocx-py/src/lib.rs:43`.
Paragraph and run indexing retain their exact collection labels at
`crates/rdocx-py/src/paragraph.rs:272` and `crates/rdocx-py/src/run.rs:184`.
Tables, rows, cells, and nested paragraphs use the same implementation while
retaining their labels at `crates/rdocx-py/src/table.rs:117`,
`crates/rdocx-py/src/table.rs:351`, `crates/rdocx-py/src/table.rs:489`, and
`crates/rdocx-py/src/table.rs:746`. Formatting and table value conversions also
use the shared helpers at `crates/rdocx-py/src/formatting.rs:193` and
`crates/rdocx-py/src/table.rs:254`. A wheel rebuilt from this HEAD passed all
31 installed-package tests, including negative indexing, lazy collections,
units, enums, tri-state formatting, stale handles, rendering, and concurrency.

## Milestone gate

The M13 gate remains: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`.

That end-of-milestone gate is not yet due and S33 does not claim M13 complete.
The ledger records five of eighteen M13 stories done and thirteen pending at
`docs/sprints/BACKLOG.md:31`. The missing type stubs, broad python-docx parity,
rpptx binding, platform wheel, and PR job work remains explicitly scheduled at
`docs/hld/14-development-backlog.md:1032` through
`docs/hld/14-development-backlog.md:1055`.

The completed S33 slice has concrete passing evidence:

- A fresh wheel built as `cp39-abi3`, installed into a clean Python 3.12
  environment, and passed all 31 tests under `crates/rdocx-py/tests/`.
- `cargo test -p rdocx-py --lib` passed the exact concrete layout-error mapping
  test at `crates/rdocx-py/src/lib.rs:103`.
- `cargo test -p rdocx --test integration_test` passed all 76 facade tests,
  including the total accessor and compatibility regressions named in the
  durable evidence at `docs/sprints/AS_BUILT.md:4809`.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm` passed.
- `python3 scripts/hash_harness.py --check` reported all 28 entries matching.
  This agrees with the five S33 records at `docs/sprints/AS_BUILT.md:4734`,
  `docs/sprints/AS_BUILT.md:4775`, `docs/sprints/AS_BUILT.md:4815`,
  `docs/sprints/AS_BUILT.md:4855`, and `docs/sprints/AS_BUILT.md:4892`.
- Formatting, focused clippy, prose, generated-skill sync, sprint-workflow
  tests, metadata, and diff whitespace checks passed. The full verification is
  recorded as passed with unchanged harness at
  `.claude/scratch/S33-run.json:83`. The later review remediation has the
  focused wheel, facade, and WASM checks described above.

## Ledger and status consistency

All five sprint rows are done and unowned at
`docs/sprints/CURRENT_SPRINT.md:30`. The backlog has the same five done rows at
`docs/sprints/BACKLOG.md:263`, and the tracker has exactly one S33 row for each
at `docs/sprints/SPRINT_TRACKER.md:192`. Durable AS_BUILT entries begin at
`docs/sprints/AS_BUILT.md:4700`. The M13 and repository totals remain
mechanically consistent at `docs/sprints/BACKLOG.md:31` and
`docs/sprints/BACKLOG.md:33`.

## Not found

- **Interaction**: no stale-handle, tri-state, exception, rendering, or facade
  conflict appeared when the five features and the remediation ran together.
- **Duplication**: S2 is resolved, and no second source of user-visible index,
  unit, or enum conversion behavior remains.
- **Layering**: `oxml-py-support` still points only inward. The production
  binding dependencies remain at `crates/rdocx-py/Cargo.toml:28`, and the
  concrete `oxml-layout` test dependency remains dev-only at
  `crates/rdocx-py/Cargo.toml:34`.
- **Harness**: no baseline changed, and the fresh 28-entry check agrees with
  all five AS_BUILT declarations.
- **Docs**: the integrated path, facade, Python, threading, packaging, release,
  and dependency-order descriptions agree after S1.
- **Dependencies**: each new direct dependency has a current consumer. PyO3 is
  exactly 0.29.0 with `abi3-py39` at `Cargo.toml:103`, and the lock contains one
  coherent 0.29.0 family at `Cargo.lock:551`. `cargo audit` found no
  vulnerability and only the existing allowed `ttf-parser` maintenance
  warning.
- **Surface**: no public API outside the five approved plans was found. The
  helper consolidation remains crate-local and adds no trait, generic, module,
  or file.
- **Artifacts**: no sample output, package archive, cache, handoff, progress
  note, or unrelated file is present in the sprint diff.

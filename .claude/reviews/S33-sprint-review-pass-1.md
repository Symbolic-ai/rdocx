# S33 sprint review, pass 1

**Reviewed**: `sprint/s33` at `ab0ad89` against merge base `9c2381b`, 55
files and 6,686 changed lines, with 6,545 additions and 141 deletions. Crates:
new `oxml-py-support`, new `rdocx-py`, and published `rdocx` facade changes.
**Verdict**: 0 blocking, 2 should-fix, 0 nice-to-have
**Run-sprint disposition**: 2 fix-now, 0 tracked-follow-up, 0 human-action,
0 refuted findings

## Blocking

None.

## Should-fix

### S1, The completed dependency records disagree with the integrated order

`docs/sprints/CURRENT_SPRINT.md:40`
`.claude/plans/F-131-design.md:6`
`.claude/plans/F-132-design.md:6`

**Classification**: fix-now.

The canonical backlog now records F-131 after both F-130 and F-132, and F-132
after both F-129 and F-130 at `docs/hld/14-development-backlog.md:1015` and
`docs/hld/14-development-backlog.md:1024`. The implemented plans also explain
those real prerequisites in prose at `.claude/plans/F-131-design.md:35` and
`.claude/plans/F-132-design.md:27`. Their completed `Depends on` headers still
omit those prerequisites, while the sprint sequencing note still says F-132 can
follow F-129 independently and names only F-130 for F-131. This leaves three
different answers in the durable delivery contract. Normalize the two plan
headers and the sprint sequencing note to the dependency order already approved
and implemented.

### S2, Binding helpers have multiple sources of truth

`crates/rdocx-py/src/paragraph.rs:56`
`crates/rdocx-py/src/run.rs:20`
`crates/rdocx-py/src/table.rs:11`

**Classification**: fix-now.

The same negative-index normalization is implemented independently in three
new modules. The duplication continues for Python `Length` construction at
`crates/rdocx-py/src/formatting.rs:10` and
`crates/rdocx-py/src/table.rs:103`, and for enum construction at
`crates/rdocx-py/src/formatting.rs:17` and
`crates/rdocx-py/src/table.rs:110`. These helpers define user-visible index and
conversion behavior, so later fixes can drift between paragraphs, runs, tables,
and formatting. Consolidate each behavior into one existing module and reuse
it. This needs no new trait, generic, module, or file.

## Nice-to-have

None.

## Milestone gate

The M13 gate is: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`.

That end-of-milestone gate does not yet hold, and S33 does not claim that M13 is
complete. The backlog records five of eighteen M13 stories done and thirteen
pending at `docs/sprints/BACKLOG.md:31`. Type stubs, the python-docx parity
suite, rpptx bindings, wheel construction, and the PR Python job remain named
work at `docs/hld/14-development-backlog.md:1032` through
`docs/hld/14-development-backlog.md:1055`. Treating the unfinished milestone
gate as an S33 closure defect is therefore refuted by the scheduled backlog.

The S33 slice gate is supported by concrete evidence:

- `cargo test -p oxml-py-support` passed all five tests, including
  `stale_path_reports_both_revisions`.
- The freshly installed abi3 wheel passed all 31 tests under
  `crates/rdocx-py/tests/`. This includes stale handles, lazy indexing and
  iteration, tri-state reopen checks, exact enums and units, concrete exception
  mapping, all four GIL-progress gates, exact Poppler 26.01.0 checks, complete
  PDF semantics, and the four-document concurrency median at
  `crates/rdocx-py/tests/test_rendering_threads.py:263`.
- The total facade regressions passed at
  `crates/rdocx/tests/integration_test.rs:209` and
  `crates/rdocx/tests/integration_test.rs:1880`.
- `cargo test -p oxml-layout --no-default-features` and
  `cargo check --target wasm32-unknown-unknown -p rdocx-wasm` passed.
- `python3 scripts/hash_harness.py --check` reported all 28 entries matching,
  consistent with every S33 AS_BUILT entry, including
  `docs/sprints/AS_BUILT.md:4734`, `docs/sprints/AS_BUILT.md:4775`,
  `docs/sprints/AS_BUILT.md:4815`, `docs/sprints/AS_BUILT.md:4855`, and
  `docs/sprints/AS_BUILT.md:4892`.
- The recorded full verification is passed at the PyO3 remediation commit in
  `.claude/scratch/S33-run.json:75`. The only later commit changes the four
  sprint ledgers. Independent checks at HEAD passed formatting, clippy, prose,
  generated-skill sync, diff whitespace, RustSec audit, and `cargo deny check`.

## Ledger and status consistency

All five sprint rows are done and unowned at
`docs/sprints/CURRENT_SPRINT.md:30`. The same five rows are done in the backlog
at `docs/sprints/BACKLOG.md:263`, have one tracker row each at
`docs/sprints/SPRINT_TRACKER.md:192`, and have durable AS_BUILT entries beginning
at `docs/sprints/AS_BUILT.md:4700`. The M13 count is 18 total, 5 done, 0 in
progress, and 13 pending. The repository total is 159, 142 done, 0 in progress,
and 17 pending at `docs/sprints/BACKLOG.md:33`. All five plans are completed.

## Not found

- **Interaction**: no stale-handle, tri-state, exception-mapping, rendering, or
  facade conflict appeared when the five features ran together.
- **Layering**: `oxml-py-support` depends only on `oxml-core`, `smallvec`, and
  `thiserror` at `crates/oxml-py-support/Cargo.toml:17`. The production binding
  edges point inward at `crates/rdocx-py/Cargo.toml:28`. The concrete
  `oxml-layout` error constructor remains dev-only at
  `crates/rdocx-py/Cargo.toml:34`.
- **Harness**: no baseline changed. All declarations and AS_BUILT records say
  unchanged, and the independent 28-entry check agrees.
- **Docs**: the HLD impact union is exactly HLD 03, 10, 14, and 15. No
  implementation contradicted the current path, facade, Python surface,
  threading, packaging, or release intent beyond S1's delivery-record mismatch.
- **Dependencies**: every new direct dependency has a current consumer. PyO3
  remains exact 0.29.0 with `abi3-py39` at `Cargo.toml:103`, and the lock holds
  one coherent 0.29.0 family at `Cargo.lock:551`. RustSec reported no
  vulnerability, with only the existing allowed `ttf-parser` maintenance
  warning.
- **Surface**: the additive published facade methods are the direct total,
  tri-state, and clear-capable accessors required by the F-130 and F-131 plans
  at `.claude/plans/F-130-design.md:46` and
  `.claude/plans/F-131-design.md:28`. No unrequested public API was found.
- **Artifacts**: no generated sample, package archive, cache, handoff, progress
  note, or unrelated file entered the sprint diff.

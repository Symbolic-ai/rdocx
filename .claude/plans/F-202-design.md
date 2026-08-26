# F-202, Incremental layout

**Status**: completed
**Sprint**: S58
**Size**: L
**Depends on**: F-201

## Problem

The facade invalidates each completed result after mutation while retaining
reusable layout engines at `crates/rdocx/src/document.rs:1844`. The engine
already supports safe incremental pagination, including exact prefix restart
and unchanged-tail attachment at `crates/rdocx-layout/src/engine.rs:1278` and
`crates/rdocx-layout/src/engine.rs:1343`.

The remaining scale defect is the restart partition. It allows only 32 page or
checkpoint slots at `crates/rdocx-layout/src/engine.rs:730` and rejects the
complete restart record above that limit at
`crates/rdocx-layout/src/engine.rs:1610`. The regression at
`crates/rdocx-layout/src/engine.rs:8661` proves that a 33-page document drops
the optimisation, so a thousand-page editing session still performs full
pagination. F-201 already supplies the source-built thousand-page fixture and
performance contract at `crates/rdocx/tests/regression_test.rs:146`.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade and retained-engine boundary".
- `docs/hld/08-rendering-spec.md`, "Performance".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and the thousand-page regression.
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and the large-document CI gate.
- `docs/hld/14-development-backlog.md`, "F-202, Incremental layout".

## Approach

Raise the bounded restart page and checkpoint partition from 32 to 1,024
entries, and raise the aggregate entry ceiling from 4,224 to 5,216. Keep the
restart byte ceiling at 8 MiB and the aggregate retained-byte ceiling at 64
MiB. Preserve complete typed body equality, safe empty-page checkpoints, exact
tail context and suffix equality, and full-pagination fallback for unsupported
or oversized state.

Add private test-only page-layout invocation accounting around the paginator
branches at `crates/rdocx-layout/src/engine.rs:1343` and
`crates/rdocx-layout/src/engine.rs:1422`. Exercise the public deterministic
reusable path through `Document::layout_with_fonts_and_bundled_fallback`, which
retains a bundled-font engine across edits at
`crates/rdocx/src/document.rs:4738`.

Build 1,000 one-page paragraphs with the F-201 `page_break_before` fixture
pattern, prime layout, mutate paragraph 500 through the facade, and lay out
again. Assert exact warm-versus-fresh equality, at most two pagination
invocations, at least 998 page-frame `Arc` identities retained, 999 paragraph
cache hits, one paragraph rebuild, and safe fallback for a 1,025-page record.

## Rejected alternatives

- Add another incremental cache layer. The existing engine already implements
  exact restart and unchanged-tail attachment.
- Make the restart cache unbounded. That violates the retained-memory contract.
- Raise the 8 MiB byte ceiling before measurement. The entry cap is the known
  blocker.
- Expose public layout statistics. The story does not request new published
  surface.
- Gate only on elapsed time. Timing is machine-sensitive and does not prove
  bounded work.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `editing_one_paragraph_of_a_thousand_page_document_rebuilds_at_most_two_pages` | Public deterministic bundled-fallback layout keeps 1,000 pages, changes at most two page `Arc`s, and equals a fresh result |
| unit | `thousand_page_restart_records_at_most_two_page_layout_invocations` | Private instrumentation counts at most two warm page-layout invocations, with 999 paragraph hits and one build |
| unit | `substituted_page_reuse_is_bounded_and_complete_equal` | 1,024 slots remain accepted within the byte limit and 1,025 slots fall back |
| regression | `a_thousand_page_document_paginates_and_renders_within_the_declared_limits` | The F-201 64 MiB, 250 pages per second layout limit and PDF limits remain green |

The backlog test gate is **regression**: editing one paragraph of a
thousand-page document re-lays out a bounded number of pages, asserted by
counting layout invocations.

## HLD impact

- `docs/hld/08-rendering-spec.md`, "Performance".
- `docs/hld/12-testing-strategy.md`, the incremental-layout regression.
- `docs/hld/15-build-and-toolchain.md`, the reusable Word engine cache summary and restart slot ceiling.

## Risk routing

- Layout and pagination. Re-read `docs/hld/08-rendering-spec.md`, use bundled
  deterministic fonts for every structural and render comparison, require
  exact warm-versus-fresh equality, do not re-record a baseline, and run
  `python3 scripts/hash_harness.py --check`.

## Hash harness

Expected unchanged at 49 of 49. This changes retained work and invocation count
only. Any output delta blocks integration and must not be absorbed into F-198's
declared hyphenation delta.

## Implementation checklist

- [x] Raise restart slots to 1,024 and aggregate slots to 5,216.
- [x] Keep restart bytes at 8 MiB and aggregate bytes at 64 MiB.
- [x] Add private test-only page-layout invocation accounting.
- [x] Add the exact thousand-page engine regression.
- [x] Add the public deterministic bundled-fallback facade regression.
- [x] Update the oversized restart fallback regression for 1,025 pages.
- [x] Prove warm and fresh complete equality and retained `Arc` identity.
- [x] Re-run the unchanged F-201 release performance gate.
- [x] Run scoped checks, full verification, and the unchanged hash harness.
- [x] Update exactly the three listed HLD files.

## Open questions

None. The restart slot ceiling becomes exactly 1,024, the 8 MiB byte ceiling
remains, and larger records use the existing safe full-pagination fallback.

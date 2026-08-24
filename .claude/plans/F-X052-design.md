# F-X052, Restore interactive relayout performance

**Status**: approved
**Sprint**: S55
**Size**: L
**Depends on**: F-X039, F-X040, F-X043, F-X044, F-X045, F-X046, F-173,
F-X048, F-X051

## Problem

The hardened reusable engine produces correct warm output, but its hot path
still performs work proportional to retained payload size. Every layout clones
the complete non-body context before checking compatibility
(`crates/rdocx-layout/src/engine.rs:459` and
`crates/rdocx-layout/src/engine.rs:778`). Paragraph lookup writes each complete
typed paragraph through `Debug` (`crates/rdocx-layout/src/engine.rs:2917`), and
restart identity allocates one `Debug` string for every body item
(`crates/rdocx-layout/src/engine.rs:1028`).

Restart pagination deep-clones unchanged prefix and tail page frames, wraps the
copies in `Arc`, recursively compares them, then reattaches the retained values
(`crates/rdocx-layout/src/engine.rs:1104`,
`crates/rdocx-layout/src/engine.rs:1148`, and
`crates/rdocx-layout/src/engine.rs:1190`). Cache-safe tables also disable the
restart path, and a warm table lookup clones its complete key before knowing
whether it hit. These costs reproduce the two to four times slowdown reported
in Issue 46 even though note operations have recovered.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams" and the exact private
  retained-engine boundary.
- `docs/hld/08-rendering-spec.md`, "Performance".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and "WASM".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and the editor-scale and
  restart-pagination regressions.
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".
- `docs/hld/14-development-backlog.md`, "F-X052, Restore interactive relayout
  performance".
- GitHub Issue 46 and the source-built workload at
  `emptinessform/rdoc@fde271547d5e497b6c25adbf88b6992c0c3df0b8`.

## Approach

Add a borrowed `ReusableEngineContext::matches_input` comparison that checks
every retained-work input with exact typed equality and compares section
properties through an iterator. Build a new owned context only after a
successful layout whose previous context did not match. Use the same borrowed
comparison for checked normal and bundled-fallback transfer, so a body-only
receiver does not clone its complete styles, stories, media, notes, theme,
fonts, or aliases before taking the compatible engine.

Replace paragraph `Debug` hashing with a cheap stable borrowed fingerprint over
safe paragraph text and structural discriminants. Keep the complete `CT_P`
equality check authoritative. Give table cache entries the same prefilter and
borrowed exact hit check, allocate an owned table key only on a miss, retain
FIFO order on a hit, and replace `Debug`-based retained-byte estimation with
recursive capacity accounting.

Represent restart body identity with one same-file concrete enum:

```rust
enum RestartBodyEntry {
    Paragraph { fingerprint: u64, value: Arc<CT_P>, bytes: usize },
    Table { fingerprint: u64, value: Arc<CT_Tbl>, bytes: usize },
}
```

Fingerprints only select comparison candidates. Exact borrowed typed equality
finds the first changed item and common suffix. Publishing a new restart record
reuses each equal entry's `Arc` and clones only inserted or changed typed
values. Unsupported body variants remain unrepresentable. Charge the entry
vector and exact retained payload to the existing 32-entry and 2 MiB restart
partition, with shared values counted once.

Carry `Arc<PageFrame>` through the eligible recorded-pagination branch. Splice
retained prefix and tail values directly and allocate `Arc` only for newly
paginated pages. Exact body, context, checkpoint, page number, font trace, and
field-substitution identities remain the authority for reuse. Page mutation
uses `Arc::make_mut` only for a genuinely changed page. The semantic
`MarkedContent` tree and recursive walkers remain unchanged.

Allow cache-safe traversal-independent tables to participate in restart body
identity and tail reuse. Never checkpoint inside a table. Reject any table
whose recursive cell content contains fields, notes, drawings, relationships,
controls, revisions, keeps, or another state the checkpoint does not model.
The next safe empty-page boundary after a table may be a checkpoint, which
allows the mixed Issue 46 workload to restart before an edit and attach an
exact unchanged suffix after later safe tables.

Add test-only counters for owned-context construction, body debug work,
retained-page deep copies, paragraph and table hits and builds, and rebuilt
page range. They do not enter release builds or public API. Optimize source
registry lookup only if the pinned A/B measurements still exceed the budget
after these named causes are removed.

## Rejected alternatives

- Making fingerprints authoritative would allow collisions to publish stale
  layout and violate the exact-context contract.
- Copying the reference implementation's unbounded caches would trade the
  measured regression for unbounded retained memory.
- Dropping `MarkedContent` would reverse the accessible PDF structure shipped
  in v0.9.0.
- Disabling restart pagination for all tables makes the reported 14-table
  workload incapable of reaching the intended path.
- Adding a public engine or unchecked transfer API would make stale retained
  state representable outside the facade.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `paragraph_and_table_fingerprint_collisions_require_typed_equality` | Forced equal prefilters cannot serve unequal typed paragraphs or tables. |
| unit | `body_only_layout_and_transfer_do_not_rebuild_owned_context` | Warm body edits and compatible checked transfer perform zero owned context builds, while every real context change still rejects without consuming either engine. |
| regression | `mixed_editor_relayout_reuses_every_safe_unchanged_block_and_page` | A 700-paragraph, 14-table deterministic document rebuilds one edited paragraph, hits 699 paragraphs and 14 tables, performs no body `Debug` work or retained-page deep copy, reuses prefix and tail page `Arc`s, and stays within all cache partitions. |
| regression | `mixed_editor_table_mutation_rebuilds_only_the_changed_table` | One changed table builds once, 13 tables and every safe unchanged paragraph hit, restart range stays bounded, and warm output equals fresh output. |
| regression | `restored_body_transfer_is_exact_for_normal_and_bundled_fallback_engines` | Normal, bundled-fallback, and alias-aware engines transfer across body-only restore, reject each context difference, and preserve both documents on rejection. |
| regression | `warm_mixed_editor_output_equals_fresh_output` | Pages, structure, fonts, diagnostics, provenance, numbering, notes, fields, outlines, PDF, and raster output are exactly equal under deterministic fonts. |
| benchmark | pinned Issue 46 interleaved release A/B | Native and bundled-fallback load, typing, checked undo, and table mutation are each no more than 1.25 times the pinned reference. |

The **test gate** is regression. Instrumented tests prove a one-paragraph edit
does no whole-document debug serialization or deep copy of unchanged prefix
and tail pages, reports cache hits for every unchanged safe block, rebuilds
only the affected restart region, and accepts an exactly compatible
restored-body transfer. Warm output remains exactly equal to a fresh engine in
pages, structure, fonts, diagnostics, provenance, numbering, notes, fields,
and outlines. Retained and pending memory remain bounded, both WASM targets
pass, and interleaved release measurements for load, typing, undo, and table
mutation are no more than 1.25 times the reference on the same machine and
workload.

The benchmark pins the workload to
`emptinessform/rdoc@fde271547d5e497b6c25adbf88b6992c0c3df0b8` and the reference
implementation to
`emptinessform/rdocx@29a4a5ceade1532919aab8ad79821fa4cd4f24f5`. It uses real
dependency edits because Cargo patches do not replace revision-pinned Git
dependencies. Wall-clock ratios are reviewed completion evidence rather than
a timing assertion in the ordinary unit suite.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Layout, pagination, line breaking, or text shaping**. Read
  `docs/hld/08-rendering-spec.md`. Use deterministic caller fonts for every
  structural or render comparison, require exact warm and fresh equality, and
  keep all 49 hash entries unchanged.
- **An external oracle comparison**. Follow
  `.claude/skills/differential-testing.md`. Pin both workload and reference
  commits as stated above, run release builds interleaved on one machine, and
  record the four native and bundled-fallback operation ratios.
- **WASM or PyO3 bindings**. The binding surface stays unchanged, but the
  shared layout path is compiled by both WASM crates. Run both wasm32 target
  checks and retain both Python binding exclusions in the workspace suite.

## Hash harness

Expected unchanged, 49 of 49. The change removes redundant work while keeping
the exact layout result and deterministic font path. Any delta blocks
integration and the baseline is not re-recorded.

## Implementation checklist

- [ ] Compare complete retained context through borrowed exact values and
      allocate owned context only after a successful real change.
- [ ] Replace paragraph and table `Debug` fingerprints with cheap stable
      prefilters while retaining exact typed equality.
- [ ] Avoid table-key clones on hits and account retained table bytes without
      formatting the table.
- [ ] Retain exact paragraph and table restart identities behind bounded
      shared ownership and clone only changed entries.
- [ ] Carry retained page `Arc`s through prefix and tail pagination without a
      deep-copy and reattachment cycle.
- [ ] Admit only cache-safe tables to restart participation and never
      checkpoint inside a table.
- [ ] Add collision, instrumentation, mixed-edit, table-mutation, transfer,
      equality, transaction, and memory-bound regressions.
- [ ] Run scoped layout and facade tests, no-default layout, both WASM checks,
      full verification, and the unchanged hash harness.
- [ ] Run and record the pinned interleaved release A/B gate.

## Open questions

None.

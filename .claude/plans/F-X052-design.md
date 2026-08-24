# F-X052, Restore interactive relayout performance

**Status**: completed
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

Shape each parent `TextSegment` once when deriving character spacing, then
reuse that spacing while the unchanged UAX subsegment loop reshapes each exact
byte slice. Search the unchanged 2,048-entry and 16 MiB shaping memo from
newest to oldest without changing its FIFO eviction order on a hit. This
removes repeated whole-segment shaping while preserving the exact glyph ids,
advances, Unicode-scalar source ranges, and line-break ownership.
Store an accounted 64-bit fingerprint beside each shaping entry to reject
nonmatching candidates before the complete font, size, and text comparison.
The complete key remains authoritative, so a fingerprint collision remains a
miss. The fingerprint adds eight bytes to each charged entry without changing
either bound.

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
vector and exact retained payload to the existing 32-entry restart partition,
with shared values counted once. Rebalance the unchanged 64 MiB aggregate
ceiling so paragraph blocks receive 50 MiB and restart state receives 8 MiB.
The original 2 MiB restart proposal cannot retain the required mixed fixture.
Its exact deterministic 22-page record measured 4,757,572 retained bytes.
Table and header or footer partitions remain 2 MiB and 4 MiB respectively.

Store cacheable laid-out paragraph and table payloads in `Arc` and share those
immutable blocks between the active layout transaction and the bounded cache.
Keep `ParagraphBlock`, `TableBlock`, `LayoutBlock`, and `Section` unchanged.
The engine and paginator use one private concrete shared-block representation
whose side overlay holds result-local provenance nodes and the paragraph
structure ids consumed during emission. The public `LayoutBlock` and private
shared block are the two current implementations of the same private
paginator input trait. Structure construction writes the overlay instead of
mutating cached blocks, and paginator emission applies it while preserving the
existing recursive `MarkedContent` tree. Cache misses allocate each shared
payload once. Cache hits clone only the `Arc`. Staged publication remains a
whole-layout transaction, and retained-byte accounting continues to charge
the complete allocation once through the cache entry.

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
- Moving shared ownership into the public block model would impose an API and
  migration cost on every external low-level consumer. The private overlay
  provides the same ownership without changing public types.
- Disabling restart pagination for all tables makes the reported 14-table
  workload incapable of reaching the intended path.
- Adding a public engine or unchecked transfer API would make stale retained
  state representable outside the facade.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `paragraph_and_table_fingerprint_collisions_require_typed_equality` | Forced equal prefilters cannot serve unequal typed paragraphs or tables. |
| unit | `body_only_layout_and_transfer_do_not_rebuild_owned_context` | Warm body edits and compatible checked transfer perform zero owned context builds, while every real context change still rejects without consuming either engine. |
| unit | `shaping_memo_hits_preserve_fifo_order` | Newest matching exact shaping work wins without refreshing the bounded FIFO queue. |
| unit | `shaping_memo_fingerprint_collision_requires_exact_key_equality` | A forced shaping fingerprint collision remains a miss until the complete font, size, and text key matches. |
| unit | `ligature_runs_reshape_each_break_chunk_without_duplicate_glyphs` | Parent spacing reuse preserves exact subsegment glyph ownership and scalar ranges. |
| regression | `mixed_editor_relayout_reuses_every_safe_unchanged_block_and_page` | A 700-paragraph, 14-table deterministic document rebuilds one edited paragraph, hits 699 paragraphs and 14 tables, performs no body `Debug` work or retained-page deep copy, reuses prefix and tail page `Arc`s, and stays within all cache partitions. |
| regression | `mixed_editor_table_mutation_rebuilds_only_the_changed_table` | One changed table builds once, 13 tables and every safe unchanged paragraph hit, restart range stays bounded, and warm output equals fresh output. |
| regression | `shared_cached_blocks_keep_result_local_semantics_exact` | Warm and fresh pages, structure, provenance, and nested table source paths are exactly equal while active cacheable blocks share their immutable cache payloads. |
| regression | `cached_heading_keeps_result_local_provenance` | A cached heading converted to result-local owned structure keeps its exact current body source path before and after insertion. |
| regression | `overflowed_table_font_trace_keeps_result_local_provenance` | A real 4,100-run table trace overflow bypasses cache publication without exposing the cache source sentinel. |
| unit | `restart_body_accounting_charges_cache_safe_property_payloads` | Paragraph, table, row, and cell property allocations are charged, and an over-limit safe property payload cannot enter restart retention. |
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

The first clean post-shaping measurement passed all four cold pairs at 0.84,
1.16, 1.06, and 1.13 times the reference, but warm pairs were 1.33, 1.38,
1.25, and 1.45 times the reference. Profiling the exact 700-paragraph and
14-table payload found 714 cacheable active block clones. Cloning and dropping
those retained blocks 200 times took 2.352 ms per iteration. This measured
warm residual justifies the private shared-block representation. It does not
justify a larger cache. In the final clean native run, the four cold ratios
were 0.83, 1.04, 1.00, and 1.04. Warm mean ratios were 0.82, 0.87, 0.83, and
0.92. Every pair produced 58 pages in both builds and met the 1.25 gate, so no
further restart-bound revision is justified.

The remaining bundled-fallback cold profile observed 38,045 exact shaping
calls across 827 parent text segments. Of those calls, 36,492 were memo hits
and 1,553 were misses. The newest-first exact scan consumed 25.3 ms and the
miss shaping and insertion work consumed 21.6 ms within 67.7 ms of block
construction. The collision-safe shaping fingerprint prefilter reduced the
scan to 11.0 ms and block construction to 51.6 ms without changing miss work,
scalar mapping, or shaping output.

The final clean bundled-fallback run used the same 58-page source-built
document, two caller fonts, and twelve aliases. Its four cold ratios were 1.16,
1.08, 1.10, and 1.12. Typing ratios were 1.03, 0.95, 1.12, and 1.19. Checked
undo-transfer ratios were 1.03, 1.03, 1.04, and 1.15. Table-mutation ratios
were 1.09, 0.97, 1.02, and 1.11. Every operation and pair met the 1.25 gate.

After microscope remediation, the rebuilt wrapper repeated all four bundled
pairs. Cold ratios were 1.02, 1.11, 1.11, and 1.06. Typing ratios were 1.10,
1.15, 1.10, and 0.97. Checked undo-transfer ratios were 1.11, 1.22, 1.14, and
1.07. Table-mutation ratios were 1.03, 1.07, 1.14, and 1.09. Every run produced
58 pages and every ratio remained within 1.25.

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

- [x] Compare complete retained context through borrowed exact values and
      allocate owned context only after a successful real change.
- [x] Shape each parent segment once for spacing and search exact shaping memo
      hits newest to oldest through a collision-safe fingerprint prefilter
      without refreshing FIFO order.
- [x] Replace paragraph and table `Debug` fingerprints with cheap stable
      prefilters while retaining exact typed equality.
- [x] Avoid table-key clones on hits and account retained table bytes without
      formatting the table.
- [x] Share immutable cacheable paragraph and table blocks between active
      transactions and cache entries while keeping public block APIs unchanged.
- [x] Apply result-local provenance and semantic structure through the private
      paginator overlay without changing scalar ranges or `MarkedContent`.
- [x] Retain exact paragraph and table restart identities behind bounded
      shared ownership and clone only changed entries.
- [x] Carry retained page `Arc`s through prefix and tail pagination without a
      deep-copy and reattachment cycle.
- [x] Admit only cache-safe tables to restart participation and never
      checkpoint inside a table.
- [x] Add collision, instrumentation, mixed-edit, table-mutation, transfer,
      equality, transaction, and memory-bound regressions.
- [x] Run scoped layout and facade tests, no-default layout, both WASM checks,
      full verification, and the unchanged hash harness.
- [x] Run and record the pinned interleaved release A/B gate.

## Open questions

None.

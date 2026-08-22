# F-X040, Restart pagination and cache table blocks

**Status**: approved
**Sprint**: S52
**Size**: L
**Depends on**: F-X038, F-X039

## Problem

The reusable engine caches safe paragraphs at
`crates/rdocx-layout/src/engine.rs:393`, but `layout_transaction` rebuilds the
complete section list and calls full `paginate_sections` for every edit at
`crates/rdocx-layout/src/engine.rs:560`. Tables always run through
`layout_table_with_provenance` and have no equivalent bounded cache.

The existing paragraph safety test at
`crates/rdocx-layout/src/engine.rs:916` excludes paragraphs containing their
own generated note markers, but a cached later paragraph can still retain a
marker number made stale by insertion of an earlier footnote reference. Warm
layout must first become correct for that case before pagination tails or table
blocks can be reused.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Performance", "Word revision views", and
  "Word bookmark field pagination".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

Extend the private engine context with traversal state that affects numbering,
notes, fields, outlines, diagnostics, and font identity. Invalidate paragraph
reuse after a changed prefix can alter a later generated marker. Add a table
cache beside the existing paragraph cache using the same transactional pending
and published queues, byte ceilings, diagnostic replay, font trace replay, and
source rebind discipline. Tables with numbering, note references, fields, or
unrepresented traversal state bypass reuse.

Record restart checkpoints while laying out a single safe section. A
checkpoint stores the next body block index, complete paginator state,
traversal state, accumulated outlines and diagnostics, font trace boundary,
and the shared page prefix. It is eligible only at a page boundary with no
split paragraph, table row, footnote continuation, float, keep constraint, or
other carried state. On a warm edit, restart from the last eligible checkpoint
before the first changed block. Attach a shared old tail only when the new
boundary state and complete environment identity equal the recorded state.

Multi-section documents and documents with floating drawings initially fall
back to the full path. Failed layout publishes no checkpoints, table entries,
or tail. Bound the combined retained restart and block state by explicit entry
and byte ceilings in the existing engine.

## Rejected alternatives

- Restarting from an arbitrary paragraph cannot reproduce split blocks, notes,
  floats, keep constraints, or numbering state.
- Comparing page pixels before attaching a tail would miss diagnostics,
  provenance, fields, outlines, and invisible state.
- Caching every table would reuse traversal-sensitive numbering and note
  markers incorrectly.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `earlier_note_insertion_invalidates_later_cached_markers` | Warm and cold marker numbers, note placement, provenance, and diagnostics match after prefix insertion and deletion. |
| regression | `warm_restart_rebuilds_only_the_bounded_changed_region` | Start, middle, tail, and exact page-boundary edits reuse a safe prefix and matching tail while reporting the expected rebuilt page range. |
| regression | `unsafe_pagination_state_falls_back_to_full_layout` | Multiple sections, floats, split paragraphs, split tables, note continuations, keep constraints, and mismatched boundary state never splice a tail. |
| regression | `safe_tables_reuse_transactionally_and_with_bounds` | Safe table blocks hit, traversal-sensitive tables bypass, late failure publishes nothing, diagnostics and font traces replay, and byte and entry ceilings hold. |
| regression | `warm_and_cold_outputs_are_complete_equals` | Pages, fonts, diagnostics, provenance, numbering, notes, fields, outlines, and rendered bytes are identical for every edit case. |

The test gate is **regression**. Warm pagination after edits at the start,
middle, tail, and a page boundary equals a fresh engine in pages, fonts,
diagnostics, provenance, numbering, notes, fields, and outlines while
rebuilding only the bounded affected page range. Insertions, deletions, style
and numbering edits, footnote-marker renumbering, multi-section fallback,
floating-drawing fallback, failed layouts, and table cache bounds all have
explicit cold-versus-warm evidence. The hash harness remains unchanged.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout and pagination: re-read `docs/hld/08-rendering-spec.md`. Run all
  restart and backend evidence in deterministic font mode, and require the
  full hash harness to remain byte-identical.

## Hash harness

Expected to be unchanged. Warm reuse must be observationally identical to a
fresh layout.

## Implementation checklist

- [ ] Fix footnote-marker invalidation before adding new reuse.
- [ ] Add transactional bounded safe-table caching.
- [ ] Capture only complete safe pagination checkpoints.
- [ ] Restart before the first changed block and validate exact boundary state.
- [ ] Share a tail only after complete environment and result-state equality.
- [ ] Add edit-range, fallback, failure, bounds, and complete warm-cold tests.
- [ ] Run focused layout, deterministic backend, and hash checks.

## Open questions

None. The backlog explicitly bounds the first implementation to safe
single-section, no-float cases and requires full-layout fallback otherwise.

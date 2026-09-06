# F-X084, Narrow note-part paragraph cache invalidation

**Status**: approved
**Sprint**: S70
**Size**: M
**Depends on**: F-X072, F-X083

## Problem

The retained engine context owns exact footnote and endnote parts, and its one
complete equality result currently controls paragraph-cache reads for the whole
transaction (`crates/rdocx-layout/src/engine.rs:895`,
`crates/rdocx-layout/src/engine.rs:1007`, and
`crates/rdocx-layout/src/engine.rs:1492`). A change to one note therefore
prevents an ordinary body paragraph from reaching the cache hit path even when
that paragraph contains no note reference (`crates/rdocx-layout/src/engine.rs:2350`).
The current regression records 699 hits and 1,401 builds after a note-only edit
to a 700-paragraph input (`crates/rdocx-layout/src/engine.rs:10313`).

This is broader than the HLD contract, which requires a changed note part to
invalidate required entries while preserving exact restart, note-page, header,
and footer safety (`docs/hld/08-rendering-spec.md:727`).

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Performance", retained paragraph context,
  note references, transactional publication, and restart safety.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", editor-scale cache and
  note-part equality regressions.
- `docs/hld/14-development-backlog.md`, "F-X084, Narrow note-part paragraph
  cache invalidation".
- GitHub Issue 69 and offered commit `4777a74167495a5116289e1f905dfd9ad4dbe807`.

## Approach

Split retained-context comparison into base, note-part, and full matches. Use
the base match for body paragraph-cache reads, then allow a cached entry only
when note parts match or that paragraph carries no note reference. Keep restart,
header, footer, note-page, and engine-transfer decisions on the full match.

After a successful note-only transaction, retain published paragraph entries
without note references, evict entries that carry references to changed note
parts, publish rebuilt referencing entries, and refresh the retained context to
the current notes. Preserve whole-layout transactional publication on error.
Review the offered patch as input, but do not adopt its existing post-success
clear behavior, which would make the improvement last only one transaction.

Keep all changes private to the existing engine file and inline test module.
Add no public API, dependency, feature, crate, module, file, trait, or generic.

## Rejected alternatives

- Remove notes from all context equality. Restart and note-page reuse could
  become stale.
- Reuse reference-bearing paragraphs after a note change. Their placement can
  depend on changed note content.
- Clear ordinary entries after serving one warm transaction. The next layout
  would lose valid reusable work.
- Add a public cache control. The existing context already owns the boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `note_part_changes_invalidate_only_referencing_paragraphs` | Footnote text, insertion, deletion, and matching endnote changes retain at least 698 of 700 ordinary hits, rebuild at most two paragraphs, and equal fresh deterministic output and source paths. |
| regression | `note_only_invalidation_preserves_unreferenced_entries_for_next_layout` | A third layout after a note edit still retains 699 ordinary hits and performs one build. |
| regression | extended `changed_note_reference_or_note_part_invalidates_required_cache_entry` | Both note streams miss only required entries with exact counts. |
| regression | `changed_related_story_context_invalidates_restart_state` | Note changes still force full restart pagination and never reuse stale note pages. |
| regression | existing transaction and cache-bound matrix | Late failure publishes no partial cache state and all limits remain enforced. |

The **test gate is regression**. Warm and fresh layout stay element-for-element
equal after note text, insertion, and deletion changes, at least 698 of 700
unaffected paragraphs hit the cache, and deterministic output hashes remain
unchanged.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Use deterministic
  bundled fonts for every baseline, require exact warm-to-fresh equality, and
  treat any hash delta as blocking.

## Hash harness

Expected unchanged across all 49 entries. The story narrows warm cache
selection without changing cold output.

## Implementation checklist

- [ ] Add failing note text, insertion, deletion, and third-layout regressions.
- [ ] Split base, note, and full retained-context predicates.
- [ ] Gate each paragraph hit on its own note-reference state.
- [ ] Preserve and evict cache entries transactionally after note-only changes.
- [ ] Keep restart and every related-story cache on exact full context.
- [ ] Run `cargo check -p rdocx-layout --all-targets` and the crate tests.
- [ ] Run full verification, deterministic hash checks, and routed riders.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The backlog defines the conservative boundary, and the offered patch is
review input rather than an implementation authority.

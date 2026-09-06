# F-X086, Provenance-safe restart after body-length changes

**Status**: approved
**Sprint**: S70
**Size**: L
**Depends on**: F-X075, F-X083

## Problem

Word body source ids are allocated from body-index paths, while reusable
restart currently rejects any provenance layout whose current and retained
body lengths differ (`crates/rdocx-layout/src/engine.rs:143` and
`crates/rdocx-layout/src/engine.rs:1834`). The first-change, suffix, and safe
prefix checkpoint logic therefore never runs after a sourced insertion or
deletion. Non-provenance insert and delete coverage already proves bounded
restart, but the sourced counterpart only proves output equality.

Retained tail pages are attached as existing `Arc<PageFrame>` values
(`crates/rdocx-layout/src/engine.rs:1985`). Reusing that tail after a body index
shift would retain stale `SourceSpan` paths. The HLD requires warm and cold
source provenance to stay exact (`docs/hld/08-rendering-spec.md:782`).

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Performance", safe prefix checkpoints,
  exact suffix reuse, and result-local Word source provenance.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", sourced editor operation
  and warm-to-fresh provenance regressions.
- `docs/hld/14-development-backlog.md`, "F-X086, Provenance-safe restart after
  body-length changes".
- GitHub Issue 69 and offered commits
  `9e48bc86876c294b8daa314e577e84b6fcd7ac97` and
  `c8315b92857c951146fc866cd044b214194a09a8`.

## Approach

Remove only the equal-body-length condition from reusable prefix eligibility.
Keep exact context, provenance mode, font trace, note-reference sequence, and
all existing unsafe-state gates. Keep the whole-body unchanged test length
aware. Find the first changed block and restart from the last complete safe
checkpoint before it.

Permit retained tail attachment only when provenance is absent or the current
and cached body lengths are equal. With provenance and a length delta, reuse the
safe prefix, paginate through the document end to rebuild shifted spans, and
publish a new exact restart record. Preserve the existing non-provenance suffix
optimization.

Review both offered patches as one mechanism. The first removes the current
whole-record veto and adds sourced coverage. The second adds the provenance
tail guard. Do not cherry-pick either blindly. Keep the change private to the
existing engine file and inline tests.

## Rejected alternatives

- Remove the veto while retaining a sourced tail. Shifted source paths would be
  stale.
- Force full pagination on every body-length change. That is the reported
  performance defect.
- Rewrite source ids inside shared retained page frames. It mutates shared data
  and would require recursive provenance surgery.
- Disable tail reuse for non-provenance layouts. Their existing optimization is
  safe and independent.
- Redesign `SourceRegistry` identity. That exceeds this story's boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `sourced_insert_and_delete_restart_instead_of_repaginating` | Insert and delete near block 640 of 700 paginate at most three pages, reuse safe prefix pages, and equal fresh output and source paths. |
| regression | `sourced_length_change_never_reuses_shifted_tail_pages` | No old tail page identity survives a sourced length delta, and all rebuilt source paths equal fresh layout. |
| regression | `sourced_enter_merge_and_selection_delete_restart_from_safe_prefix` | Enter, adjacent merge, and multi-block selection delete retain bounded prefix restart and exact provenance. |
| regression | existing non-provenance insert, delete, and undo matrix | Safe suffix attachment remains active without provenance. |
| regression | existing unsafe restart and note-sequence matrix | Fields, drawings, multiple sections, dirty notes, and other unsafe inputs still use full pagination. |

The **test gate is regression**. A sourced insert and delete near block 640 of
700 recomputes at most three pages, matches a fresh layout element for element
with equal source paths, and fails if full pagination returns or stale tail
provenance is reused.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Use deterministic
  bundled fonts, require exact warm-to-fresh layout and provenance equality,
  and treat any hash delta as blocking.

## Hash harness

Expected unchanged across all 49 entries. The story changes warm restart work
only, while cold deterministic output remains identical.

## Implementation checklist

- [ ] Add discriminating sourced operation and shifted-tail regressions.
- [ ] Relax only reusable prefix eligibility after body-length changes.
- [ ] Gate tail attachment on provenance absence or equal body length.
- [ ] Preserve exact context, note, font, capacity, and unsafe-state gates.
- [ ] Prove prefix page identity reuse and the absence of shifted tail reuse.
- [ ] Preserve non-provenance suffix optimization.
- [ ] Run scoped checks, full verification, hash checks, and routed riders.
- [ ] Update exactly the listed HLD files.

## Open questions

None after F-X083 corrects the external-input sentence to name the complete
two-commit mechanism.

# F-248, Style-linked numbering, counters, TOC, and REF

**Status**: approved
**Sprint**: S71
**Size**: L
**Depends on**: F-246, F-247

## Problem

Styles and numbering are currently mutated through separate public operations,
so a caller cannot atomically establish both sides of a style-linked numbering
invariant (`crates/rdocx/src/document.rs:4055` and
`crates/rdocx/src/document.rs:4098`). The layout counter state handles basic
list levels but not the full instance, restart, section, table, suppression,
TOC, and numbering-aware REF behavior (`crates/rdocx-layout/src/style_resolver.rs:48`).

Capability row `DOCX-013` is unsupported for creation and mutation
(`docs/hld/02-scope-and-non-goals.md:220`). The public corpus gate needs one
transaction that cannot publish a one-sided link and deterministic visible
counters across all required consumers.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, capability row `DOCX-013`.
- `docs/hld/03-architecture.md`, atomic cross-part invariants.
- `docs/hld/04-opc-and-packaging.md`, numbering and style ownership.
- `docs/hld/08-rendering-spec.md`, "The renderer's input" and Word bookmark
  field pagination.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, private authoring and Word fidelity gates.
- `docs/hld/13-risks-and-open-questions.md`, R4, R5, and R13.
- `docs/hld/14-development-backlog.md`, "F-248, Style-linked numbering,
  counters, TOC, and REF".

## Approach

Add `Document::link_style_to_numbering(style_id, num_id, level) -> Result<()>`
and a symmetric unlink operation. Clone the complete document, validate the
paragraph style and numbering level, set the level's paragraph-style link and
the style's numbering properties, validate both graphs, then publish once.

Extend `NumberingState` to key counters by concrete instance and level. A new
`numId` starts an independent sequence and reuse of the same `numId` continues
it. Apply
start values, start overrides, level overrides, legal numbering, level restart,
explicit continuation, section boundaries, and `numId` zero suppression.
Carry the same state through body and table-cell traversal in document order.

Use the resolved visible marker for numbered TOC entry text and numbering-aware
REF switches while preserving the existing bookmark and field pagination
pipeline. Rebuild from document input on every cold deterministic layout so no
process or host state enters allocation or counters.

## Rejected alternatives

- Expose two caller-ordered mutations. A failure between them leaves a broken
  cross-part invariant.
- Compute TOC and REF numbering with separate counters. They would diverge from
  body layout on restarts and tables.
- Reset all counters at every section. Word behavior depends on the definition
  and instance controls.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `three_level_style_linked_numbering_matches_pinned_word` | Body, table, TOC, and REF visible values match the sanitized Word oracle across independent instances, continuation, and restarts. |
| regression | `style_numbering_link_is_atomic_in_both_directions` | Invalid style, level, or graph state leaves both parts byte-identical. |
| regression | `num_id_zero_suppresses_only_the_selected_paragraph` | Suppression emits no marker and does not corrupt later counters. |
| regression | `numbering_state_crosses_tables_and_sections_in_document_order` | Table-cell and section placement follow Word continuation and restart rules. |
| round-trip | `style_linked_numbering_survives_reopen_and_rebuild` | Both sides of the link and all cached field results rebuild deterministically. |

The **test gate is differential**. Three-level numbered headings inside and
outside tables match Word in body text, TOC entries, cross-references, and
restart behavior.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Use deterministic
  fonts and compare cold source-built body, TOC, and REF output.
- **Any parser or serializer**. Preserve schema order, prefix aliases, and
  unmodeled style, numbering, and field subtrees byte for byte.
- **Public API of a published crate**. State additive pre-1.0 impact and run the
  verified workspace package dry-run with archive-size assertions.
- **External oracle comparison**. Pin the Word object and render oracles,
  compare normalized trees plus a stated raster threshold, and record every
  intentional divergence.

## Hash harness

Expected unchanged for existing fixtures. The new transaction and counter
paths are opt-in, and existing field output must remain stable.

## Implementation checklist

- [ ] Add atomic link and unlink facade operations.
- [ ] Validate both style and numbering graph sides before publish.
- [ ] Complete instance, continuation, restart, and suppression counters.
- [ ] Carry counter state through table cells and sections in document order.
- [ ] Feed resolved markers into numbered TOC and REF field materialization.
- [ ] Add deterministic differential, round-trip, and failure coverage.
- [ ] Run the full gate and every routed rider.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The user approved independent new instances, continuation through reuse
of one `numId`, and atomic rejection of an existing style or level link
conflict.

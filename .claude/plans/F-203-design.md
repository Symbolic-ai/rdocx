# F-203, Reader compatibility corrections

**Status**: approved
**Sprint**: S49
**Size**: M
**Depends on**: none

## Problem

The merged table-reader change recognizes `CT_TcPr` children by local name
alone. A foreign `<ext:tcW>` is therefore parsed as the typed Word cell-width
element and dropped when the table is written. The numbering writer also emits
raw XML at boundary 5 after `w:suff`, although that boundary represents the
slot before the suffix in the `CT_Lvl` schema sequence.

Both failures violate the reader's preservation contract for producer XML.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-203, Reader compatibility
  corrections".
- `docs/hld/03-architecture.md`, "Crate-level conventions", unmodelled
  subtree preservation.
- `docs/hld/04-opc-and-packaging.md`, "Facade conventions" and the package
  preservation rules.

## Approach

Thread the in-scope WordprocessingML namespace bindings from `CT_Tc` into
`CT_TcPr`. Recognize table-cell property elements and their attributes only
when both local name and WordprocessingML binding match. Preserve every other
property child in a schema-slot sidecar, then emit it unchanged at the same
slot during serialization.

Move boundary 5 emission before the typed `w:suff` element. The level parser
already assigns `isLgl` to that boundary, so the writer change restores the
reader's existing model without changing public types.

## Rejected alternatives

- Keep local-name matching and skip only foreign `tcW`. Every property has the
  same namespace-identity requirement, and a one-name exception would repeat
  the bug.
- Drop foreign properties while avoiding the typed projection. This still
  violates verbatim preservation on save.
- Renumber the level raw boundaries. The parser already assigns the schema
  slots correctly. Only the writer loop is misplaced.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `foreign_cell_width_remains_raw_and_unmodelled` | A foreign same-local-name `tcW` child neither sets typed width nor changes its bytes or schema slot on write |
| regression | `aliased_cell_width_uses_in_scope_word_bindings` | A WordprocessingML alias and matching alias attributes populate the typed width projection |
| round-trip | `level_raw_is_lgl_stays_before_suffix` | A preserved raw `isLgl` child remains byte-identical and before typed `suff` after parse and write |

The **test gate**, from the backlog, is regression. Foreign `tcW` XML remains
unmodelled and byte-identical, and an `isLgl` raw child stays before `suff`
after parse and write.

Fixtures stay in the existing `table.rs` and `numbering.rs` unit modules.

## HLD impact

None. The architecture already requires namespace-aware readers and verbatim
unmodelled-subtree preservation. This corrects the implementation to match it.

## Risk routing

- Parser or serialiser. Read HLD 04 and HLD 06. Add alias,
  foreign-collision, schema-order, and byte-preservation coverage.
- Public API of a published crate. `CT_TcPr` gains a hidden raw-preservation
  sidecar that is required to retain producer XML. Run the package dry-run and
  archive-size assertion during full verification.

## Hash harness

Expected unchanged. The affected XML stays the serialization source and no
hash-harness fixture changes a typed property.

## Implementation checklist

- [ ] Record in-scope WordprocessingML bindings while parsing `CT_TcPr`.
- [ ] Restrict typed cell-property and attribute parsing to those bindings.
- [ ] Preserve non-Word and unsupported cell-property XML in schema slots.
- [ ] Emit boundary 5 raw level XML before typed `w:suff`.
- [ ] Add the regression and byte-preservation coverage in the test plan.
- [ ] Run the changed-crate, package, and hash-harness checks.

## Open questions

None. The existing schema slots and namespace helpers specify the intended
reader behavior.

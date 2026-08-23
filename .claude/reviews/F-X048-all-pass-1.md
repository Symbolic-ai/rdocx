# F-X048, all, pass 1

**Reviewed**: working-tree diff against exact claim base
`fa3dacad97a58de7faf317eedc294f25bf95dfd9`, 15 files and 2,096 changed
lines, with 1,855 additions and 241 deletions
**Verdict**: 7 defects, 0 smells, 0 nitpicks

## Defects

### D1, exact-height rows pin geometry but do not clip their content

`crates/rdocx-layout/src/paginator.rs:2442`

The row resolver pins an exact row to its declared height, but that fact is not
retained on `TableRow` or `TableCell`. The renderer then emits every paragraph
and nested-table block without a cell clip. Text taller than an exact row
therefore paints through the row boundary instead of being clipped. The
focused row-height test checks only the numeric height, and the dense-form
golden does not put overflowing content in an exact row, so both remain green
when clipping is absent.

### D2, a vertical merge loses its terminal bottom border

`crates/rdocx-layout/src/paginator.rs:2615`

The restart cell is painted with `merged_height`, which already reaches the
bottom of the complete merge span. `merge_with_below` nevertheless suppresses
the bottom stroke at that terminal coordinate, while every continuation cell
is skipped and cannot emit it later. A merge that should suppress only its
crossing interior edges therefore has no bottom border. The restart also keeps
the first and last row flags of its physical start row, so an outer bottom edge
ending on the table's last row cannot select the table-bottom border or the
outer-edge `nil` fallback.

### D3, a merge-only minimum row charges content to the restart row

`crates/rdocx-layout/src/table.rs:364`

When a row contains only vertical-merge restart cells, `max_cell_height` is
zero and `fallback_height` restores the restart content height as that first
row's minimum. The later span pass consequently sees enough total height and
does not grow the final eligible non-exact row. This contradicts the approved
geometry rule that non-merged cells establish ordinary minima and merge
content grows the final eligible row in its span. The regression covers an
exact restart row, so it does not exercise this non-exact merge-only case.

### D4, conditional table-style selection omits shading and `cnfStyle`

`crates/rdocx-layout/src/table.rs:581`

`ResolvedTableCellStyle` carries only paragraph properties and borders, and
cell shading is later read only from direct `tcPr`. Conditional `tcPr/shd` and
`tblPr/shd` projections are therefore ignored. Region selection also derives
only from `tblLook` and row or column coordinates, without consulting the
already modeled row and cell `cnfStyle` values. The dense-form fixture contains
a first-row conditional fill at
`crates/rdocx/tests/regression_test.rs:5435`, but its golden assertions never
require that fill to appear, so the checksum currently locks an output missing
one of the fixture's declared concerns.

### D5, the 7 point paragraph mark does not control empty-line metrics

`crates/rdocx-layout/src/engine.rs:3921`

The new zero-width carrier correctly resolves the paragraph mark at 7 points,
then the compatibility block unconditionally overwrites that line's ascent,
descent, gap, and height with the legacy empty line. The segment still reports
`font_size == 7.0`, which is all the focused regression asserts, but the empty
cell retains the legacy 12 point line height instead of the mark's metrics.
This leaves the reported dense-form height defect unfixed while its test
passes.

### D6, character-relative cell anchors discard the paragraph indent

`crates/rdocx-layout/src/paginator.rs:2523`

Cell placement correctly selects the cell geometry for both `column` and
`character`, but it always passes a zero indent to `resolve_anchor_h`.
Character-relative anchors are defined by the start of the paragraph text, so
an indented cell paragraph is shifted left by its full indent. The focused test
uses only `relativeFrom="column"` and cannot detect the character case.

### D7, typed conditional-style mutations are never serialized

`crates/rdocx-oxml/src/styles.rs:258`

`CT_TblStylePr` publicly exposes typed paragraph, table, and cell projections,
but serialization always writes `raw_xml` and never compares or rebuilds it
from those projections. A caller can mutate a conditional border, shading, or
paragraph property successfully in memory, then save and receive the original
value. The round-trip test mutates only the base `table_properties` field, so
it does not exercise the conditional projections that the feature adds.

## Smells

None.

## Nitpicks

None.

## Verification limitation

The external Microsoft Word 16.104 build 16.104.25121423 observation required
by the design plan was not performed because that application is not installed
on this host, as recorded at `.claude/scratch/F-X048-progress.md:52`. No
headless or self-produced render was treated as Word evidence.

## Not found

- **Panics**: no new untrusted-input panic was found in the reviewed paths.
- **Structure**: no unjustified trait, generic, crate, module, file, wrapper, or
  feature flag was introduced. `CellBlock` has the two present paragraph and
  table variants required by the source-order model.
- **Recursive cache and provenance**: no additional defect was found in cache
  safety recursion, retained-byte recursion, transactional publication, or
  nested source rebinding.
- **Tagged PDF semantics**: no additional defect was found in recursive table
  structure ownership or F-173 marked-content assignment.
- **HLD scope**: only the four HLD files listed by the revised design plan were
  changed.
- **API, packaging, WASM, and hash scope**: no additional defect was found in
  the native-only facade addition, declared low-level source breaks, recorded
  package evidence, both WASM checks, or the exact two-entry 49-item harness
  revision.

# F-102, correctness, pass 1

**Reviewed**: uncommitted F-102 working diff, 10 files, 2,725 changed lines
**Verdict**: 4 defects, 1 smell, 0 nitpicks

## Defects

### D1, table styles overwrite explicit run formatting
`crates/rpptx-layout/src/context.rs:1466`

Cell text first resolves presentation defaults, master defaults, paragraph
properties, and explicit run properties. `apply_table_text_style` then writes
the table style's bold, italic, colour, and font onto every resolved run at
`crates/rpptx-layout/src/context.rs:1556`. A cell run with an explicit colour or
`b="0"` therefore loses its direct formatting to the lower-precedence table
style. Table style text properties must enter the cascade before paragraph and
run properties.

### D2, right-to-left cells take the wrong widths
`crates/rpptx-render/src/lib.rs:370`

The right-to-left branch converts the logical column to a reversed index, then
uses that reversed index to read offsets built from the unreversed width list.
With widths `[10, 20]`, logical column zero moves to the right but receives
width 20 rather than its own width 10. The cell rectangle and its border
segments are both wrong whenever right-to-left columns have unequal widths.

### D3, merged outer borders come only from the origin cell
`crates/rpptx-render/src/lib.rs:355`

Merge continuations are skipped before they can contribute borders. The origin
cell's right and bottom borders are then moved to the far edge of its complete
span at `crates/rpptx-render/src/lib.rs:422`. If the last covered column or row
has an edge or corner style, that continuation owns the actual outer border,
but the renderer emits the origin's inside border instead. The existing merged
gate uses identical borders everywhere and cannot distinguish these sources.

### D4, inside borders are shadowed by every outer border
`crates/rpptx-layout/src/context.rs:1688`

Each edge uses `outer.or(inside)`, so a whole-table style that defines both
forms applies the outer left, right, top, and bottom strokes to every cell.
`insideH` and `insideV` are reached only when the corresponding outer form is
absent. Real producer styles define all six forms, which means their distinct
inside strokes never resolve. Exterior edges must use the outer forms while
interior boundaries use the inside forms, with later region overlays still
able to replace them.

## Smells

### S1, advertised direct table-border aliases never parse as lines
`crates/oxml-drawing/src/table.rs:1878`

The dispatch accepts `lnL`, `lnR`, `lnT`, and `lnB`, but normalizes each name to
`left`, `right`, `top`, or `bottom` before calling `capture_border`.
`capture_border` consequently treats the direct line as a wrapper, marks it
unsupported, and preserves it raw. Either remove aliases that are outside the
accepted contract or pass their real names and test the supported path.

## Nitpicks

None.

## Not found

- `contract`: no work outside the approved parser, resolver, renderer, tests,
  plan, and five HLD files.
- `panics`: no production panic or unchecked index was found on malformed table
  dimensions or spans.
- `ooxml`: modelled writes use fixed prefixes and ordered boundaries, while
  unsupported subtrees remain captured.
- `structure`: no new source file, module, trait, generic parameter, crate, or
  forwarding wrapper was added.
- `tests`: the six named gates and corpus round-trip exist, but the defects
  above need distinguishing regressions before the test aspect is clean.

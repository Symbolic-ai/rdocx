# F-102, correctness, pass 2

**Reviewed**: remediated uncommitted F-102 working diff, 11 files, 2,897 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, corner regions ignore the table option flags
`crates/rpptx-layout/src/context.rs:1408`

The four corner regions apply solely from row and column position. A north-west
corner therefore overrides a cell even when first-row or first-column styling
is disabled, and the other corners behave the same way. Corner formatting is
the intersection of the corresponding enabled row and column regions. Each
corner must require both option flags before it enters the cascade. The current
precedence test enables only `firstRow` but still expects `nwCell`, so it locks
in the defect.

### D2, unsupported table fill forms disappear without diagnostics
`crates/rpptx-layout/src/context.rs:1438`

`concrete_fill` returns a specific unsupported category for path gradients,
patterns, picture fills, and malformed concrete fills, but table fill
resolution discards the category with `and_then`. Table border lowering also
discards the same category at `crates/rpptx-layout/src/context.rs:942`. The
result is a missing cell fill or border with no stable diagnostic, contrary to
the approved preservation and diagnosis contract. A regression must cover both
a cell fill and a border fill that the neutral paint model cannot render.

## Smells

None.

## Nitpicks

None.

## Not found

- `pass-1 D1`: table text style properties now enter the inherited character
  cascade before explicit paragraph and run properties.
- `pass-1 D2`: right-to-left offsets now use reversed physical widths and a
  distinguishing unequal-width raster passes.
- `pass-1 D3`: merged boundaries source the far covered cell's edge and the
  distinguishing continuation-border regression passes.
- `pass-1 D4`: whole-table outer edges and inside edges resolve separately.
- `pass-1 S1`: direct edge aliases retain their names through parsing and have
  a distinguishing parser regression.
- `panics`: remediated span, row, column, and border lookup paths remain bounded.
- `ooxml`: no preservation, prefix, or child-order regression was introduced.
- `structure`: the private four-field table position value reduces repeated
  argument groups and no new trait, generic, module, file, or crate was added.

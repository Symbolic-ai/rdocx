# F-031, all, pass 1

**Reviewed**: working tree against `f0aecfb`, 3 files, 284 added lines and 5 deleted lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the composition gate does not exercise off-diagonal coefficients
`crates/oxml-layout/src/transform.rs:161`

Both operands in the PDF composition test have zero `b` and `c` coefficients.
The identity test also cannot distinguish incorrect cross terms. An
implementation that drops or swaps the off-diagonal products in `then` would
therefore pass every test while composing rotations and skews incorrectly.
The gate needs a hand-computed composition with nonzero off-diagonal
coefficients while retaining the declared self-first order assertion.

### D2, the rectangle test cannot detect a two-corner bounds implementation
`crates/oxml-layout/src/transform.rs:203`

The test uses an exact negative quarter turn. Under that transform, the
axis-aligned bounds of the transformed top-left and bottom-right corners also
contain the other two corners. A wrong implementation that transforms only
those two opposing corners would pass the expected bounds and all four
containment assertions. Use a non-quarter-turn rotation or skew with
hand-computed extrema so the test proves that all four corners contribute.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: `then` implements `next * self`, so `self` is applied first and
  `next` second. `rotate_about` preserves an arbitrary pivot and uses the PDF
  coefficient convention. Rectangle bounds handle negative rotation and
  negative width or height by taking extrema across all four computed corners.
- Contract: the exact six public coefficients and the six planned operations
  are present. The type is exported from the crate root, and no dependency or
  released crate changed.
- Panics: no production panic path was found. The only indexing selects from a
  fixed four-element local array.
- OOXML: not applicable. This story adds no parser, serializer, namespace,
  schema-order, or raw-subtree behavior.
- Tests: the five scoped transform tests pass. Pivot preservation, rotation
  sign, exact identity, and self-first translation and scale order are covered.
  The two defects above prevent the suite from proving general affine
  composition and four-corner bounds.
- Structure: the authorized concrete module adds no trait, generic parameter,
  wrapper, feature flag, dependency, or speculative geometry operation.

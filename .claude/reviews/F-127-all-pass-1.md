# F-127, all, pass 1

**Reviewed**: working diff from claim base `b1a4abd`, 4 files, 626 changed
lines, with 521 additions and 105 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, Filled series discard resolved transparency and alpha
`crates/rpptx-chart/src/lib.rs:3517`

`filled_and_stroked_path` replaces the resolved colour alpha with `0.55`.
An area or filled-radar series whose direct paint is `a:noFill` therefore gets
an opaque-enough black fill instead of remaining transparent. The same path
also turns any partial alpha produced by the DrawingML transform stack into
`0.55`, so it does not pass the resolved series colour through exactly. The
existing 55 percent area policy needs to preserve transparent zero and compose
with resolved alpha rather than overwrite it. The direct-paint tests exercise
only bars, so neither trigger is covered.

### D2, Unused theme slots can block a direct solid series colour
`crates/rpptx-chart/src/lib.rs:1849`

`resolve_series_colours` builds and validates a lookup for all twelve theme
slots before it examines whether a series has a direct colour. The strict
collection at `crates/rpptx-chart/src/lib.rs:1942` rejects every preset or
scheme-valued theme slot, including an unused hyperlink slot. A chart whose
series has a direct `a:srgbClr` therefore fails before direct-paint precedence
can apply when any unrelated theme slot uses one of those valid modelled colour
choices. Direct colours must not depend on unused theme entries. An unstyled
series needs only the concrete slot selected by its mapped accent.

## Smells

None.

## Nitpicks

None.

## Not found

- Panics: no new reachable panic, unchecked slice, or untrusted arithmetic
  issue was found.
- OOXML: no schema-order, namespace, whitespace, or raw-preservation issue was
  found in the reviewed diff.
- Structure: no unjustified trait, generic, wrapper, module, file, or dynamic
  dispatch was introduced.
- Tests: the focused `series_` subset passed 12 of 12. Apart from the missing
  defect triggers described above, no additional test-gate issue was found.

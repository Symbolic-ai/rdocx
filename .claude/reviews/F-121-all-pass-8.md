# F-121, all, pass 8

**Reviewed**: working diff from claim base `7e2794b`, 1 source file and 1,838 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 7 remediation

- Pass 7 D1 is fixed. Repeated-run boundary zero is emitted before any current
  series or axis identifier at `crates/rpptx-chart/src/lib.rs:4969`.
- Internal and trailing original boundaries are reconciled independently from
  boundary zero at `crates/rpptx-chart/src/lib.rs:4972`. Each raw node is
  emitted once at the next surviving original item or at the trailing
  boundary.
- A newly inserted bar series remains after preserved `c:varyColors` at
  `crates/rpptx-chart/src/lib.rs:8097`.
- Swapped bar axis identifiers remain after preserved `c:serLines`, and the
  between-axis anchor follows its next surviving original identifier at
  `crates/rpptx-chart/src/lib.rs:8124`.
- The focused public-mutation, schema-order, and P6 first-pixel tests passed in
  this pass.

## Prior remediation rechecked

- Series identity edits, reorders, and colliding keys retain their preserved
  raw origins through the matching sequence at
  `crates/rpptx-chart/src/lib.rs:4887`.
- Axis identifier reorder keeps matched lexical markup through the inverse
  mapping at `crates/rpptx-chart/src/lib.rs:4744`.
- Parsed bar and line families cannot be replaced while family-specific raw
  metadata remains at `crates/rpptx-chart/src/lib.rs:4657`.
- The P6 helper consumes exactly one header delimiter at
  `crates/rpptx-chart/src/lib.rs:8380`, and the regression retains whitespace
  bytes as the first raster sample at `crates/rpptx-chart/src/lib.rs:7888`.

## Not found

- Correctness: no wrong enum mapping, default, range check, boolean handling,
  axis resolution, reciprocal-axis validation, or repeated-boundary defect was
  found.
- Contract: supported single-family plot areas own one typed plot and their
  axes, unsupported and combination choices remain opaque, and no F-125
  native geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted ChartML input was found.
- OOXML: no namespace-alias, fixed-prefix, modelled-child sequence,
  repeated-child reconciliation, unsupported-plot preservation, extension
  preservation, or unknown-attribute defect was found.
- Tests: malformed supported plots, duplicate modelled children, unresolved
  axes, series insertion and identity collision, axis-id reordering, family
  replacement, exact corpus coverage, P6 whitespace-valued samples, and the
  zero-MAE viewer comparison are exercised.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was
  found.

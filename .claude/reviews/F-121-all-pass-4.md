# F-121, all, pass 4

**Reviewed**: working diff from claim base `7e2794b`, 1 source file and 1,786 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, an identity edit that collides with another series swaps raw origins

`crates/rpptx-chart/src/lib.rs:4890`

Series reconciliation performs every exact key match before its positional
fallback. If a parsed two-series plot changes the first series `index` and
`order` to the second series values, the original second series matches the
current first item. The positional fallback then assigns the original first
series to the current second item. Plot-level raw content anchored before the
original first series, such as `c:varyColors`, consequently moves after the
current first series and violates the ChartML sequence. This mutation is not
rejected because plot validation does not require unique series keys. Prefer
same-position exact matches before global exact matches, then use the existing
positional fallback, or retain a stable private origin on each parsed series.

## Smells

None.

## Nitpicks

None.

## Pass 3 remediation

- Pass 3 D1 is fixed for ordinary identity edits and reorders. Repeated series
  use exact matching plus a positional fallback, and the one-series public
  mutation case retains `c:varyColors` before the series run.
- Pass 3 D2 is fixed. Plot axis identifiers reconcile by value, preserve their
  matched scalar attributes, and move raw boundaries with the next surviving
  original identifier when the two identifiers are swapped.
- Pass 3 D3 is fixed. Serialization rejects replacing a parsed Bar plot with a
  Line plot, or a parsed Line plot with a Bar plot, while family-specific
  preservation metadata remains attached.
- `cargo test -p rpptx-chart` passed all 33 tests with the required 50-deck
  corpus and the pinned LibreOffice and Poppler viewer gates.

## Not found

- Correctness beyond D1: no wrong enum mapping, range check, default, boolean
  handling, plot-axis resolution, or reciprocal-axis validation defect was
  found.
- Contract beyond D1: supported single-family plot areas own one typed plot and
  their axes, opaque and combination choices remain preserved, and no F-125
  native geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted ChartML input was found.
- OOXML beyond D1: no namespace-alias, fixed-prefix, unchanged repeated-child
  sequence, unsupported-plot preservation, extension preservation, or
  unknown-attribute defect was found.
- Tests beyond D1: malformed supported plots, duplicate modelled children,
  unresolved axes, ordinary public mutation, axis reordering, family
  replacement, exact corpus coverage, and the zero-MAE viewer gate are
  exercised. The colliding two-series identity mutation above is not covered.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

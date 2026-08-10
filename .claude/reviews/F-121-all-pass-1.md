# F-121, all, pass 1

**Reviewed**: working diff from claim base `7e2794b`, 1 file and 1,478 changed lines
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, malformed supported plot areas can bypass typed validation

`crates/rpptx-chart/src/lib.rs:4000`

Typed promotion is conditioned on the plot area already containing exactly two
axis roots. A single supported `barChart` or `lineChart` with one missing axis,
or with an extra axis, therefore takes the opaque return at line 4004 instead
of reaching `validate_typed`. The approved contract preserves unsupported and
combination plots, but requires malformed supported plots and invalid axis
references to return errors. Select typed mode from the supported single-family
plot choice, then let the parsed axis graph and plot references decide validity.

### D2, the viewer gate accepts non-pixel-equivalent renders

`crates/rpptx-chart/src/lib.rs:6083`

The approved gate requires pixel-equivalent original and candidate chart pages,
and the recorded run actually produced normalized RGB MAE `0.00000000` with
identical render hashes for both decks. The asserted threshold is `0.001`, so a
candidate may change a material number of pixels and still pass, especially
because the metric is diluted over the full slide. The differential-testing
rules also require a reason for any nonzero tolerance, and none is stated.
Require zero normalized MAE or identical decoded pixel buffers for this gate.

### D3, the malformed-input test omits two promised cases

`crates/rpptx-chart/src/lib.rs:7739`

The negative matrix has no duplicate modelled child, such as a second
`c:grouping`, `c:gapWidth`, `c:dLbls`, `c:marker`, or `c:smooth`. It also has no
otherwise-valid plot whose distinct axis references fail to resolve. The cases
using ids `1` and `2` all contain an earlier independent error, so they do not
prove missing-axis rejection. Add isolated cases for duplicate children and
unresolved axis references, including the missing-axis shape that exposes D1.

### D4, corpus counts do not prove corpus plots were promoted

`crates/rpptx-chart/src/lib.rs:7846`

The corpus gate increments the same counters for typed plots and opaque raw
plot roots. It would still report exactly 12 bar and 3 line plots if every
nonrepresentative single-family corpus plot silently remained opaque. The
corpus contains one intentional bar-and-line combination, so the gate should
track typed single-family plots separately from the preserved combination and
assert both populations before the structural reparse comparison.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness beyond D1: no wrong enum mapping, range check, or reciprocal-axis
  validation defect was found.
- Contract beyond D1, D2, D3, and D4: no native F-125 geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted XML was found.
- OOXML: no fixed-prefix, supported-child sequence, namespace-alias, or raw
  subtree preservation defect was found beyond the malformed typed boundary in
  D1.
- Tests: the focused required-corpus run passed 4 of 4 tests. It verified the
  pinned 50-deck corpus, observed 12 bar and 3 line plots, pinned LibreOffice
  26.2.5.2 and Poppler 26.01.0, and observed zero MAE for both SHA-bound
  representative candidates.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

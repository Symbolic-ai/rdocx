# F-121, all, pass 5

**Reviewed**: working diff from claim base `7e2794b`, 1 source file and 1,807 changed lines
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, the binary PPM reader can consume valid pixel bytes as header whitespace

`crates/rpptx-chart/src/lib.rs:8351`

After reading the P6 maximum-value token, the parser skips every consecutive
ASCII whitespace byte before taking the pixel slice. P6 has one required
whitespace separator followed immediately by arbitrary binary samples, and a
first sample byte such as `0x20` or `0x0a` is valid pixel data. Such a render
loses one or more samples here and then panics at the pixel-length assertion,
making the viewer gate depend on the top-left pixel value. Consume the single
header separator without interpreting subsequent binary bytes as whitespace,
and cover a valid P6 buffer whose first sample is whitespace-valued.

## Nitpicks

None.

## Pass 4 remediation

- Pass 4 D1 is fixed. Series reconciliation first reserves same-position exact
  identities at `crates/rpptx-chart/src/lib.rs:4890`, then performs global
  exact matches and positional fallback. The two-series colliding-key mutation
  retains plot-level content before the first series.
- Ordinary identity edits and reorders continue through exact matching and
  positional fallback without moving repeated-series raw boundaries.
- Plot axis identifiers reconcile by value and retain matched scalar markup
  and next-surviving raw boundaries when reordered.
- Serialization rejects replacing a parsed Bar plot with Line, or Line with
  Bar, while family-specific preservation metadata remains attached.
- The focused public mutation and schema-order tests both passed during this
  pass.

## Not found

- Correctness: no wrong enum mapping, range check, boolean handling,
  plot-axis resolution, reciprocal-axis validation, or repeated-item match was
  found.
- Contract: supported single-family plot areas own one typed plot and their
  axes, unsupported and combination choices remain opaque, and no F-125 native
  geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted ChartML input was found.
- OOXML: no namespace-alias, fixed-prefix, modelled-child sequence,
  repeated-child reconciliation, unsupported-plot preservation, extension
  preservation, or unknown-attribute defect was found.
- Tests beyond S1: malformed supported plots, duplicate modelled children,
  unresolved axes, colliding series identities, ordinary edits and reorders,
  axis reordering, family replacement, exact corpus coverage, and the zero-MAE
  viewer comparison are exercised.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

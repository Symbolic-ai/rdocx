# F-119, all, pass 8

**Reviewed**: the remediated F-119 working diff from `87b5d92`, 3 tracked
files, 2,169 insertions and 60 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, public optional-field edits can move preserved series children before their schema slot

`crates/rpptx-chart/src/lib.rs:657`

An unsupported series child is anchored only at the last modelled child that
was present during parsing. A valid line series with `c:marker` after
`c:order`, but without optional `c:tx` or `c:spPr`, therefore stores the marker
at raw boundary 2. If a caller later assigns the public `Series::name` or
`Series::sp_pr` field, the writer emits boundary 2 at
`crates/rpptx-chart/src/lib.rs:487` before the new `c:tx` and `c:spPr`. The
result puts `c:marker` before children that precede it in the series sequence.
Markers, data points, labels, trendlines, error bars, and series extensions
need stable schema slots that do not depend on which optional modelled fields
were present in the input.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 7 structural defect is fixed. `emit_cache_point_raw` is concrete over
`Writer<Vec<u8>>`, and F-119 adds no unjustified generic, trait, wrapper,
module, file, or crate. The pass 6 opaque-wrapper and cache-tail remediations
remain intact. The earlier namespace, duplicate-wrapper, sparse-cache,
foreign-shape, cache-state, preservation, malformed-input, and design-contract
remediations also remain intact.

No additional correctness, contract, panic, OOXML namespace, cache
consistency, test-gate, public-surface, or structural findings were found.

# F-122, all, pass 2

**Reviewed**: working diff from claim base `ff1e9c4`, 2 implementation files
and 1,535 changed lines, comprising 1,483 additions and 52 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, routed raw children leave following comments at a stale boundary

`crates/rpptx-chart/src/lib.rs:4936`

The pass 1 repair routes `c:dropLines` and `c:extLst` to schema-defined raw
slots, but it does not advance `boundary` to `raw_boundary`. The next comment,
processing instruction, or whitespace event is therefore stored at the old
boundary by `crates/rpptx-chart/src/lib.rs:4677`. Parse a pie plot with one
series, no labels or angle, a trailing `c:extLst`, and a comment after the
extension. The extension is routed to the final slot while the comment stays
at the post-series slot, so an unchanged write moves the comment before the
extension. If labels are then inserted, the comment also moves before those
labels. An area plot with no labels and a `c:dropLines` child has the same
failure for a following comment. Advancing the current boundary after routing
a known raw child is required to retain the original raw-node order.

### D2, opaque bubble-size wrappers bypass plot validation

`crates/rpptx-chart/src/lib.rs:2348`

The pass 1 repair rejects only `Series::bubble_size.is_some()`. When a
`c:bubbleSize` wrapper contains a preserved choice rather than the typed
`c:numRef`, series parsing sets private `opaque_bubble_size`, leaves the public
field as `None`, and retains the wrapper at
`crates/rpptx-chart/src/lib.rs:740`. A pie, doughnut, area, scatter, or radar
plot containing that series therefore passes validation and writes the
bubble-only wrapper into a non-bubble series type. Bubble plots remain opaque,
so plot validation must reject both typed and opaque bubble-size state.

## Smells

None.

## Nitpicks

None.

## Pass 1 remediation

- D1 is partially fixed. Family `varyColors`, `dropLines`, and `extLst` nodes
  now receive schema-defined raw slots at
  `crates/rpptx-chart/src/lib.rs:4924`, and the label-insertion regression
  passes. The stale boundary defect above remains for raw events that follow a
  routed node.
- D2 is fixed. Standalone scatter series now select their preserved wrapper
  mode at `crates/rpptx-chart/src/lib.rs:451`, and the direct round-trip
  assertions pass.
- D3 is partially fixed. Public typed bubble-size state is rejected at
  `crates/rpptx-chart/src/lib.rs:2348`, but the opaque representation described
  above bypasses the check.
- D4 is fixed. The malformed-input matrix now covers empty series, missing x
  and y caches, duplicate properties and labels, too many axis references, and
  an unresolved plot-area axis.

The three focused tests selected by the `remaining` filter passed in this
review, including the pinned corpus and viewer gate. The implementing session
also reports the full required 40-test suite, formatting, and strict workspace
Clippy as green.

## Not found

- Correctness beyond D1 and D2: no wrong enum mapping, default, range check,
  scatter-cache mapping, axis resolution, or reciprocal-axis defect was found.
- Contract beyond D1 and D2: the five requested variants remain within the
  approved boundary, unsupported and combination plots remain opaque, and no
  F-125 native geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted ChartML input was found.
- OOXML beyond D1 and D2: fixed prefixes, modelled-child order, unchanged
  repeated-item reconciliation, unknown attributes, unsupported families, and
  combination choices produced no additional finding.
- Tests beyond the two cases above: the approved gate, ordering, scatter,
  malformed-value, preservation, corpus, and viewer paths are exercised.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

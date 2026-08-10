# F-121, all, pass 3

**Reviewed**: working diff from claim base `7e2794b`, 1 source file and 1,606 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, editing a series identity moves preserved plot content
`crates/rpptx-chart/src/lib.rs:4832`

Series reconciliation identifies an original item only by its public, mutable
`index` and `order` fields. If a caller changes either field on a parsed series,
that series becomes an insertion and its original entry becomes a deletion. The
next-surviving-item rule at line 4858 then moves raw content from before the
edited series to the next unchanged series, or to the trailing boundary. For
example, changing the first series index in the bar fixture moves the preserved
`c:varyColors` from before the series run to after the edited first series,
which violates the ChartML sequence. Reconciliation needs stable origin
identity or the positional fallback used by the existing mutable collection
models.

### D2, plot axis identifier edits do not reconcile per-item markup
`crates/rpptx-chart/src/lib.rs:4730`

The two public plot axis identifiers are written with markup selected only by
the current array index, and raw boundaries after each identifier are also
emitted at fixed original positions. Swapping the two identifiers therefore
leaves each identifier's unknown attributes at the old position and leaves raw
content that was anchored to the next original identifier between the new
pair. The line branch repeats the same behavior at line 4814. This does not
follow the repository's next-surviving-original-item reconciliation rule for a
mutable repeated collection. The new test at line 7821 proves unchanged
round-trip ordering only. It does not mutate or reorder either repeated
collection.

### D3, replacing a parsed plot variant reuses incompatible raw-slot metadata
`crates/rpptx-chart/src/lib.rs:4110`

`plots_mut` exposes a mutable slice of the public `Plot` enum, so a caller can
replace a parsed `Bar` value with a valid `Line` value. Serialization still
passes the bar `PlotMarkup` at line 4059 into the line writer. Bar and line raw
boundaries use different offsets and trailing layouts. A preserved
`c:varyColors` at the bar boundary before the series run is consequently read
as a line repeated-series boundary and can move after the series. Bar-only raw
children such as `c:serLines` can likewise be emitted in a line axis slot. The
writer must either reject relabelling a parsed plot family, as `Axis` already
does for its parsed kind, or translate preservation state without
reinterpreting the old schema slots.

## Smells

None.

## Nitpicks

None.

## Pass 2 remediation

- Pass 2 D1 is fixed for unchanged collections. Each parsed series and plot
  axis identifier advances its own raw boundary at
  `crates/rpptx-chart/src/lib.rs:4413` and
  `crates/rpptx-chart/src/lib.rs:4447`, with equivalent line handling at lines
  4611 and 4631.
- The targeted ordering test at `crates/rpptx-chart/src/lib.rs:7821` passes and
  retains comments, processing instructions, and whitespace between two
  unchanged series and between two unchanged plot axis identifiers.
- The public mutation test at `crates/rpptx-chart/src/lib.rs:7920` passes, but
  it changes only a cached value, gap width, and overlap. It does not exercise
  the mutable series identity, collection order, axis identifier order, or plot
  variant cases above.

## Not found

- Correctness beyond D1, D2, and D3: no wrong enum mapping, range check,
  boolean handling, axis resolution, or reciprocal-axis validation defect was
  found.
- Contract beyond D1, D2, and D3: supported single-family plots own their
  series and axis references, unsupported and combination choices remain
  opaque, and no F-125 native geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted ChartML input was found.
- OOXML beyond D1, D2, and D3: no namespace-alias, fixed-prefix,
  unchanged-collection sequence, unsupported-plot preservation, extension
  preservation, or unknown-root-attribute defect was found.
- Tests beyond the mutation gaps cited above: malformed supported plots,
  duplicate modelled children, unresolved axes, exact corpus coverage, and the
  zero-MAE viewer gate are exercised. The two focused local tests completed
  successfully.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

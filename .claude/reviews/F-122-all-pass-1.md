# F-122, all, pass 1

**Reviewed**: working diff from claim base `ff1e9c4`, 2 files and 1,504 changed
lines, comprising 1,414 additions and 90 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, inserting a missing typed child can move preserved content before it

`crates/rpptx-chart/src/lib.rs:4917`

Every unmodelled family child is stored at the boundary reached by the typed
children that happened to precede it. The writer emits that raw boundary at
`crates/rpptx-chart/src/lib.rs:5679` before it writes an optional `c:dLbls` at
line 5701. Parse a pie plot that has one series and a trailing `c:extLst`, but
no `c:dLbls`, then set the public `data_labels` field. The extension remains at
the post-series boundary and serialises before the newly inserted labels, even
though `c:extLst` must be last. An area plot with `c:dropLines` and no labels
has the same failure. This contradicts the fixed ChartML child sequence and the
mutation coverage claimed at `docs/hld/09-charts-spec.md:288`. Family-specific
opaque children need schema-defined boundaries that remain valid when optional
typed fields are inserted or removed.

### D2, standalone scatter series serialization changes x/y wrappers

`crates/rpptx-chart/src/lib.rs:451`

The public `Series::to_xml()` always calls `to_xml_for_plot(false)`. A series
parsed from `c:xVal` plus `c:yVal` records `uses_scatter_wrappers`, but this
public round-trip writes the same caches as `c:cat` plus `c:val`. The plot-level
scatter writer passes `true` and avoids the loss, so the current test does not
expose it. The approved contract says private markup remembers the original
wrapper names. `Series::from_xml(scatter_xml)?.to_xml()` must preserve those
names rather than silently changing the series kind.

### D3, supported plots can emit bubble-only series content

`crates/rpptx-chart/src/lib.rs:5688`

All five new plot variants serialize through the common series writer, which
emits the public `bubble_size` field as `c:bubbleSize` at
`crates/rpptx-chart/src/lib.rs:572`. None of the new plot validation branches
rejects that field. A caller can therefore set `Series::bubble_size`, construct
a pie, doughnut, area, scatter, or radar plot, and receive XML containing a
bubble-series-only child in a non-bubble series type. Bubble plots are outside
the typed F-122 boundary and remain opaque. The supported plot writers must
reject bubble-only payload rather than produce a schema-invalid plot.

### D4, the required malformed-input matrix is incomplete

`crates/rpptx-chart/src/lib.rs:7253`

The approved negative test requires bad series, missing scatter x or y caches,
duplicate modelled children, and invalid axis references in addition to the
range and enum cases. The implemented case list covers bad ranges and enums,
one missing axis, an axis on a pie plot, a category series in scatter, and one
mixed-wrapper form. It has no empty-series case, no missing x-cache or y-cache
case, and no duplicate grouping, style, angle, hole-size, labels, or axis-id
case. The parser currently uses shared helpers for several of these paths, but
the story contract requires direct non-panicking regressions for the new
families.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness beyond D1 through D3: no wrong enum mapping, default, numeric
  range, axis-resolution, or reciprocal-axis defect was found.
- Contract beyond D1 through D4: all five requested plot variants are present,
  unsupported and combination choices remain opaque, and no F-125 native
  geometry scope was taken.
- Panics: no production panic, unchecked index, slice, or arithmetic overflow
  on untrusted ChartML input was found.
- OOXML beyond D1 through D3: namespace aliases, fixed prefixes, unchanged
  fixture ordering, unknown attributes, comments, whitespace, unsupported
  families, and combination plots are preserved.
- Tests beyond D4: all 40 focused library tests passed, including the pinned
  50-deck corpus and the LibreOffice 26.2.5.2 plus Poppler 26.01.0 viewer gates.
  Scoped Clippy also passed with warnings denied.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

# F-121, all, pass 7

**Reviewed**: working diff from claim base `7e2794b`, 1 source file and 1,823 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, repeated-run reconciliation can move schema-leading content after an item

`crates/rpptx-chart/src/lib.rs:4969`

The effective-boundary calculation applies next-surviving-original-item
reconciliation to original boundary zero. That boundary is also the schema
slot before the complete repeated run. For a parsed bar plot, it contains
`c:varyColors` before the series run and `c:serLines` before the axis-id run.
If a caller inserts a new series before the first original series, boundary
zero resolves to the first original series at current index one, so the new
`c:ser` writes before `c:varyColors`. The already tested axis-id swap has the
same problem. Original axis zero moves to current index one, so `c:serLines`
writes after the first current `c:axId`. Both outputs violate the ChartML
sequence even though the public mutation is otherwise valid. Emit repeated-run
boundary zero before every current item, then apply next-surviving
reconciliation only to internal and trailing original boundaries. Add
assertions for a series insertion or reorder and for `c:serLines` remaining
before both swapped axis identifiers.

## Smells

None.

## Nitpicks

None.

## Pass 6 remediation

- Pass 6 S1 is fixed. The P6 helper consumes exactly one delimiter byte at
  `crates/rpptx-chart/src/lib.rs:8369`.
- The regression at `crates/rpptx-chart/src/lib.rs:7886` preserves space, line
  feed, carriage return, and tab as first raster samples after an LF delimiter.
- The final regression uses a carriage-return delimiter followed by a line-feed
  first sample, so the ambiguous byte pair no longer loses raster data.
- Prior series identity collision, axis-id markup reconciliation, parsed-family
  replacement, malformed typed-boundary, exact corpus-count, and zero-MAE
  remediations remain present.

## Not found

- Correctness beyond D1: no wrong enum mapping, range check, default, boolean
  handling, plot-axis resolution, reciprocal-axis validation, or P6 delimiter
  defect was found.
- Contract beyond D1: supported single-family plot areas own one typed plot and
  their axes, opaque and combination choices remain preserved, and no F-125
  native geometry scope was taken.
- Panics in production: no production panic, unchecked index, slice, or
  arithmetic overflow on untrusted ChartML input was found.
- OOXML beyond D1: no namespace-alias, fixed-prefix, unsupported-plot
  preservation, extension preservation, or unknown-attribute defect was found.
- Tests beyond D1: malformed supported plots, duplicate modelled children,
  unresolved axes, colliding series identities, axis-id reordering, family
  replacement, exact corpus coverage, the P6 whitespace-sample cases, and the
  zero-MAE viewer comparison are exercised.
- Structure: no new crate, file, module, dependency, trait, generic parameter,
  feature flag, forwarding wrapper, or unnecessary dynamic dispatch was found.

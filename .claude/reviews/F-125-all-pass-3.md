# F-125, all, pass 3

**Reviewed**: uncommitted working diff, 6 files and 2,078 changed lines,
including 2,067 additions and 11 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Prior pass-2 D1 remediation: `c:dispBlanksAs` now reaches line, area, and
  scatter lowering at `crates/rpptx-chart/src/lib.rs:126`. Gap splits paths at
  nonadjacent logical indexes at `crates/rpptx-chart/src/lib.rs:703`. Zero uses
  a bounded set of endpoints and neighbours at
  `crates/rpptx-chart/src/lib.rs:666`, while Span retains one path. Markers stay
  limited to present line points at `crates/rpptx-chart/src/lib.rs:370` and
  matched scatter points at `crates/rpptx-chart/src/lib.rs:462`. Sparse line,
  area, and scatter tests assert exact Gap, Zero, and Span paths at
  `crates/rpptx-chart/src/lib.rs:8754`.
- Prior pass-2 D2 remediation: aggregate totals fail with contextual errors at
  `crates/rpptx-chart/src/lib.rs:840`. Domain normalisation scales before
  subtraction at `crates/rpptx-chart/src/lib.rs:911`, coordinate mapping checks
  its result at `crates/rpptx-chart/src/lib.rs:931`, and the final path walk
  rejects every nonfinite point at `crates/rpptx-chart/src/lib.rs:194`.
  Opposite finite extremes and overflow in stacked, percentage, pie, and
  scatter inputs are exercised at `crates/rpptx-chart/src/lib.rs:8985`.
- Prior pass-2 D3 remediation: scatter x caches receive explicit finite-value
  validation at `crates/rpptx-chart/src/lib.rs:457`. The negative test mutates
  the public x cache to NaN, positive infinity, and negative infinity at
  `crates/rpptx-chart/src/lib.rs:8970`, and requires an x-value context in each
  error.
- Correctness: adjacent sparse cases were traced for leading, trailing,
  consecutive, and cross-series blanks. Logical cache counts and preserved
  indexes are recovered at `crates/rpptx-chart/src/lib.rs:722`, scatter caches
  pair through logical-index maps at `crates/rpptx-chart/src/lib.rs:458`, and
  no additional wrong geometry or numeric edge was found.
- Contract: geometry remains limited to plot paths and markers. Axes, labels,
  final colours, relationship routing, and fallbacks remain with F-126 through
  F-128 as recorded at `.claude/plans/F-125-design.md:17`,
  `docs/hld/09-charts-spec.md:448`, and `docs/hld/09-charts-spec.md:450`.
- Panics: no reachable production panic was found. The indexed layer and marker
  accesses follow vectors built from the same series iteration at
  `crates/rpptx-chart/src/lib.rs:357` and
  `crates/rpptx-chart/src/lib.rs:506`. The `top[0]` access at
  `crates/rpptx-chart/src/lib.rs:423` follows a nonempty range returned only for
  a nonempty index slice.
- OOXML: no parser or serializer behaviour changed. Geometry reads preserved
  private cache layout without changing it at
  `crates/rpptx-chart/src/lib.rs:724`, so no namespace, schema-order, or raw
  preservation issue was found.
- Tests: the exact sparse-policy, mutated-x, finite-extreme, aggregate-error,
  family-coordinate, deterministic, and raster gate tests cover the approved
  contract. The full `rpptx-chart` library suite passed 48 tests with no
  failures.
- Structure: no new trait, generic, feature, crate, module, or source file was
  introduced. The normal dependency added at
  `crates/rpptx-chart/Cargo.toml:17` points from format-specific
  `rpptx-chart` to format-neutral `oxml-layout`, matching the documented edge
  at `docs/hld/03-architecture.md:87`.

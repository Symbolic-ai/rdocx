# F-125, all, pass 2

**Reviewed**: uncommitted working diff, 6 files and 1,637 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, sparse blanks are always spanned regardless of chart policy

`crates/rpptx-chart/src/lib.rs:126`
`crates/rpptx-chart/src/lib.rs:296`
`crates/rpptx-chart/src/lib.rs:6999`

The public chart model carries `disp_blanks_as`, but `render_geometry` passes
only the plot to the family renderer. Line geometry then puts every present
sparse point into one polyline. For a cache with declared indexes 0 through 2
and points only at indexes 0 and 2, the default `Gap` policy must leave a break,
`Zero` must place the missing value on the baseline, and `Span` may connect the
two present points. The implementation always connects them, so it renders all
three policies as `Span`. Area geometry and sparse scatter lines have the same
loss of blank-display semantics.

### D2, finite extreme values can produce nonfinite path coordinates

`crates/rpptx-chart/src/lib.rs:528`
`crates/rpptx-chart/src/lib.rs:662`
`crates/rpptx-chart/src/lib.rs:704`

Input validation proves only that each cached value is individually finite.
The scale mapping subtracts the unscaled domain endpoints. A line series with
`-f64::MAX` and `f64::MAX` therefore has a finite accepted cache, but
`domain.max - domain.min` overflows to infinity and the maximum endpoint maps
through infinity divided by infinity to `NaN`. `render_geometry` returns an
apparently successful path containing a nonfinite point instead of finite
geometry or a contextual error. Stacked totals and pie totals have the same
unchecked aggregate-overflow class.

### D3, mutable scatter x caches bypass finite-value validation

`crates/rpptx-chart/src/lib.rs:364`
`crates/rpptx-chart/src/lib.rs:375`
`crates/rpptx-chart/src/lib.rs:523`

Scatter rendering validates each series' y cache but never validates the
numeric category cache used for x coordinates. Because `NumericData::values`
and the typed plots are publicly mutable, a caller can replace a parsed or
authored x value with `NaN` or infinity before calling `render_geometry`. The
function accepts that current model state, builds its x domain from the
nonfinite value, and can return path or marker coordinates containing `NaN`.
The result contract requires contextual rejection rather than invalid
backend-neutral geometry.

## Smells

None.

## Nitpicks

None.

## Not found

- Prior D1 remediation: declared point counts and sparse indexes now reach
  category slots, and scatter x and y values pair only at matching indexes.
- Prior D2 remediation: the family tests now assert exact bar rectangles,
  ordered line, scatter, and radar points, pie and doughnut angles and radii,
  and area baselines. The scoped bar raster, sparse-cache, and negative tests
  pass.
- Prior S1 remediation: the plan and chart HLD consistently assign chart
  relationship resolution, supported routing, and preserved fallback routing
  to F-128.
- Panics: no reachable production `unwrap`, `expect`, slice, index, or integer
  arithmetic panic was found. The guarded `top[0]` access and fixed nonempty
  palette index are safe.
- OOXML: no parser or serializer behavior changed, and no namespace,
  preservation, or schema child-order defect was found.
- Structure: no new trait, generic, wrapper, feature, crate, module, or source
  file was introduced. The new dependency follows the approved
  format-specific to format-neutral direction.
- Contract beyond the defects above: geometry remains within F-125 scope, the
  placeholder palette leaves final colour resolution to F-127, and the diff
  updates exactly the two approved HLD files.
- Tests beyond D1: the raster gate would fail if bar paths or their computed
  positions regressed, and the remaining exact family assertions prove their
  stated calculations.
- Smells: none found.
- Nitpicks: none found.

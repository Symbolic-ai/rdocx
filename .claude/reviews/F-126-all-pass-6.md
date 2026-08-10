# F-126, all aspects, pass 6

**Reviewed**: the complete working-tree diff against `HEAD`, 3 files and 9,682
changed lines, comprising 6,695 additions and 2,987 deletions. Pass 1 through
pass 5 and both pass-5 remediations were checked. `cargo test -p rpptx-chart`
passed all 73 unit tests and 0 doc tests. Focused clippy, formatting, diff,
prose, adapter-sync, dependency-tree, and hash-harness checks also passed.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the zero-radius radar direction gate covers only vertical spokes
`crates/rpptx-chart/src/lib.rs:12078`

The test gives the radar series two zero values, so its two category spokes are
only north and south. Both outside-end assertions therefore pass if the
degenerate-anchor fallback is hard-coded to vertical directions instead of
preserving an arbitrary category spoke. The approved test says zero-radius
radar anchors preserve their spoke direction, but no horizontal or oblique
zero-radius case proves that behavior. A four-category zero-radius radar, or an
equivalent nonvertical case, must assert the corresponding outside-end origin.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-5 D1 production behavior: degenerate exact-bound columns and horizontal
  bars keep their original value-axis vector under normal and reversed
  orientation. Zero-radius radar anchors derive their vector from the oriented
  category spoke. Inside positions stay at the collapsed point and outside-end
  follows the retained vector. D1 records the remaining test-gate weakness.
- Pass-5 D2: high radar category-label origins are clamped to the chart label
  space. The focused coverage exercises the tight top margin in a standard
  chart and the tight right margin in a narrow chart. Low, next-to-axis, and
  hidden positions remain distinct.
- Correctness and contract: no recurrence was found in explicit bounds,
  negative and mixed radar domains, clipped bar anchors, category-count
  limits, effective number formats, sparse-cache joins, percentage totals, or
  point-level label projection.
- Panics: no reachable production indexing, slicing, `unwrap`, or `expect`
  panic was found in the reviewed rendering and projection paths.
- OOXML: namespace aliases, malformed and duplicate modelled point overrides,
  schema order, foreign lookalikes, and exact raw-subtree preservation remain
  covered without adding a second serialization source.
- Structure: no new file, module, trait, generic parameter, dynamic dispatch,
  forwarding wrapper, feature, or dependency was introduced. The one public
  labelled rendering entry point matches the approved concrete contract.
- Deterministic shaping and z-order: all text uses the caller's `FontManager`,
  and output remains ordered as gridlines, clipped plot, axis lines, ticks,
  legend swatches, then text.

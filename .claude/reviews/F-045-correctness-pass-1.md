# F-045, correctness, pass 1

**Reviewed**: uncommitted worker diff, 5 files, 905 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, raster gradients do not normalize stop order or repeated offsets

`crates/oxml-pdf/src/raster.rs:424`

`gradient_stops` forwards declaration order directly to tiny-skia. Tiny-skia
makes offsets monotonic in that order rather than sorting them, so an unsorted
gradient or repeated offset does not follow the F-043 paint contract. The
raster path must clamp and sort offsets, then keep the last stop at a repeated
offset before constructing the tiny-skia shader.

## Smells

None.

## Nitpicks

None.

## Not found

- Contract: the implementation stays within the approved path, group,
  gradient, dash, and page-background scope.
- Panics: no input-reachable panic, unchecked indexing, or unsafe arithmetic
  was introduced.
- OOXML: no parser, schema order, namespace, whitespace, or preserved subtree
  behavior changed.
- Tests: recursive transforms, intersecting clips, group opacity, path fill
  rules, solid and gradient paint, dashes, and backgrounds have direct sampled
  coverage. Stop normalization is the one missing contract case recorded
  above.
- Structure: the implementation remains private in the existing raster module
  with no new trait, generic, public type, module, feature, or dependency.

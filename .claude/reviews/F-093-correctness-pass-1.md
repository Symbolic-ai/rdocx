# F-093, correctness, pass 1

**Reviewed**: working-tree diff from `5d3546e`, 5 files, 491 insertions and 7 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: slide bounds, page order, local path geometry, fallback paint,
  fill and stroke copying, and gradient transform composition match the approved
  contract.
- Contract: `crates/rpptx-render/src/lib.rs:142` implements the two specified
  entry points without inspecting PresentationML or DrawingML model types.
- Panics: production lowering uses checked slide access at
  `crates/rpptx-render/src/lib.rs:159` and adds no panic path for caller input.
- OOXML: no parser, serializer, namespace, schema-order, or raw-subtree code is
  changed.
- Tests: the named gate at `crates/rpptx-render/src/lib.rs:317` crosses the
  public lowering boundary and shared raster backend, while
  `crates/oxml-pdf/src/raster.rs:986` isolates the double-transform regression.
- Structure: the diff adds no trait, generic parameter, wrapper, feature flag,
  crate, module, or source file. Backend dependencies remain test-only at
  `crates/rpptx-render/Cargo.toml:21`.

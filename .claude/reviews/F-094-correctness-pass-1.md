# F-094, correctness, pass 1

**Reviewed**: working-tree diff from `6fff518`, 2 files, 226 insertions and 8 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: `crates/rpptx-render/src/lib.rs:210` composes local rotation,
  centre flips, bounds translation, and the accumulated parent transform in
  the required order.
- Contract: `crates/rpptx-render/src/lib.rs:201` retains geometry, fill, and
  outline under the existing single group boundary.
- Panics: production transform lowering at
  `crates/rpptx-render/src/lib.rs:210` adds no indexing, unwrap, or caller-input
  panic path.
- OOXML: no parser, serialiser, namespace, schema-order, or raw-subtree code is
  changed.
- Tests: the gate at `crates/rpptx-render/src/lib.rs:356` computes expected
  corners independently, the combined-order regression at
  `crates/rpptx-render/src/lib.rs:444` distinguishes child-before-parent
  composition, and the deterministic raster check at
  `crates/rpptx-render/src/lib.rs:481` covers rotated fill and outline.
- Structure: the diff adds one private helper in the existing renderer source
  and adds no trait, generic parameter, wrapper, feature flag, crate, module,
  source file, or dependency.

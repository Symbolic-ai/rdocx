# F-X086, correctness, pass 1

**Reviewed**: the working diff from
`267efde4427a1860d0faea4f32845d8f86b88b36`, 1 file and 228 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the equal-body-length requirement is removed only from reusable
  prefix eligibility. The whole-body unchanged check remains length aware.
- Contract: retained sourced tails remain gated on equal body length, while
  source-free suffix attachment keeps its existing path.
- Panics: no new input-derived indexing, slicing, unchecked arithmetic, or
  panic path was introduced.
- OOXML: the change does not parse, serialize, reorder, or discard XML.
- Tests: the three sourced regressions fail against the prior implementation
  through the 22-page full-pagination path, then prove bounded restart, exact
  fresh output and source paths, contiguous prefix identity reuse, and no
  shifted retained tail identity.
- Structure: all implementation and test work stays in the existing engine
  file, with no new trait, generic parameter, wrapper, feature flag, module,
  crate, or dependency.
- Existing gates: the full 255-test `rdocx-layout` suite and 49-entry hash
  harness pass, including non-provenance suffix, note-boundary, and unsafe-state
  coverage.

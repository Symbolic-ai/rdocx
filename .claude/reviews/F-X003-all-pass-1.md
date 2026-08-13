# F-X003, all, pass 1

**Reviewed**: working-tree deletion of
`crates/rdocx/examples/generate_samples.rs`, 1 file and 783 deleted lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the deleted example is not invoked by either harness, while the
  unchanged canonical generator still enumerates all seven required samples.
- Contract: the diff is exactly the approved deletion and contains no sample
  behavior, baseline, harness, HLD, or tracker change.
- Panics: deleting the unused executable introduces no new panic surface.
- OOXML: no parser, serializer, schema ordering, namespace, whitespace, or raw
  subtree handling changed.
- Tests: the recorded hash and golden-PNG gates cover every artifact required
  from `generate_all_samples`, and the repository search confirms neither
  harness invokes the deleted example.
- Structure: removing the duplicate executable reduces the number of sample
  implementations without adding a module, trait, generic, wrapper, feature,
  dependency, or indirection.

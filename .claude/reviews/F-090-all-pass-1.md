# F-090, all aspects, pass 1

**Reviewed**: uncommitted worker diff, 6 files, 20547 additions and 5 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, corpus scan accepts foreign namespace elements as preset geometry

`tools/gen-presets/generate.py:168`

The corpus scanner matches only the local name `prstGeom`. A valid extension
element in another namespace with that local name and a `prst` attribute is
therefore treated as DrawingML geometry. Its vendor value can fail the corpus
gate even though it is not an `a:prstGeom`. Match the expanded DrawingML name
so prefix tolerance does not become namespace blindness.

### D2, duplicate comparison does not prove source-byte identity

`tools/gen-presets/generate.py:71`

The duplicate policy compares bytes produced by `ElementTree.tostring`, which
normalises namespace prefixes and other XML syntax. Two source subtrees that
differ byte for byte can therefore compare equal after reserialization. The
approved clarification requires the two `upDownArrow` source definitions to be
byte-identical and every conflicting duplicate to be rejected. Compare exact
source slices or another representation that preserves every source byte.

## Smells

None.

## Nitpicks

None.

## Not found

- Contract: the diff stays within the approved source, notice, generator,
  generated module, existing module declaration, and design-plan files. HLD
  impact remains none.
- Panics: production Rust gains no input-facing panic path. Generator failures
  are explicit command failures with context.
- OOXML: generated definitions use complete fixed-prefix `a:custGeom` roots and
  preserve schema child order from the pinned source.
- Tests: the four named gates cover byte-identical regeneration, the pinned
  direct and unique counts, corpus coverage, and known and unknown lookup.
- Structure: the new directory and generated module are explicitly approved by
  F-090. No trait, generic parameter, crate, dependency, or feature flag is
  added.

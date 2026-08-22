# F-X044, complete, pass 1

**Reviewed**: working-tree diff against claim Base `454216f`, 1 file, 234 insertions and 21 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the regression gate does not pin the approved cache envelopes

`crates/rdocx-layout/src/engine.rs:534`

`crates/rdocx-layout/src/engine.rs:5719`

`crates/rdocx-layout/src/engine.rs:5805`

The compile-time checks prove only that the three partition constants fit
inside the two total constants. The entry and byte tests then derive both their
workloads and their expectations from those same production constants. They
would continue to pass if the paragraph ceiling were changed from 4,096 entries
and 56 MiB to smaller values, including the old byte ceiling, as long as the
related constants moved together. The approved contract requires the exact
4,096 entry and 56 MiB paragraph envelope plus the exact 4,160 entry and 64 MiB
combined envelope to be proved. Add independent literal assertions for all four
values so an accidental capacity regression fails the named gate.

## Smells

None.

## Nitpicks

None.

## Not found

No implementation correctness, contract-scope, panic-safety, OOXML, or
structural findings. Fingerprints are deterministic over the borrowed typed
paragraph, typed equality remains authoritative after a fingerprint match, hit
lookup does not clone the key, hits do not refresh queue position, and
publication remains gated on whole-layout success. The focused collision,
editor-scale, traversal-invalidation, failure-and-eviction, and warm-cold tests
passed. Scoped clippy with warnings denied also passed.

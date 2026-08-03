# F-097, correctness, pass 1

**Reviewed**: working diff, 7 files, 615 changed lines
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, background reference test does not prove transform preservation

`crates/rpptx-layout/src/context.rs:2270`

The `bgRef` regression selects a solid `phClr` fill with an untransformed
`bg1` reference. It still passes if background substitution drops the
reference transform stack or the placeholder transform stack. F-097 routes
theme colour and transform behavior, and the approved risk rider requires
exact colour evidence. Add a background reference case whose selected fill and
reference colour both carry transforms, then assert the exact resolved colour.

## Nitpicks

None.

## Not found

Correctness, contract scope, panic paths, OOXML child order, namespace
tolerance, raw subtree preservation, renderer boundary leakage, story-gate
relevance, and structural-rule violations produced no other findings.

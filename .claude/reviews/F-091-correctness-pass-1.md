# F-091, correctness, pass 1

**Reviewed**: complete working diff, 4 implementation and plan files, 445 inserted lines and 27 deleted lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the corpus preset gate passes when preset support is removed

`crates/rpptx-layout/src/context.rs:2407`

The test asserts only that the resolver produced no unknown-preset or
preset-evaluation fallback categories. If preset parsing or resolution is
removed, every preset shape follows the existing `preset geometry pending
evaluation` branch, which the counter deliberately ignores, and the test still
passes. Count modelled preset inputs and successful preset outputs directly,
then assert that the corpus contains at least one and that every one either
evaluates or takes the named unknown-preset fallback.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML namespace, schema-order,
raw-preservation, evaluator-scaling, deterministic-render, or structure issue
was found.

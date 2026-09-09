# F-243, correctness, pass 1

**Reviewed**: working diff, 8 files and 561 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, constructor validation does not prove the required relationship graph

`crates/rdocx/src/document.rs:1653`

The pre-publication validator checks that required parts and content-type keys
exist, then checks only the targets of relationships that happen to exist. A
missing settings, theme, font-table, core-properties, or application-properties
relationship would still pass. The constructor contract requires validation of
the complete candidate graph before returning it.

### D2, round-trip gate does not exercise unmodelled content preservation

`crates/rdocx/tests/integration_test.rs:91`

The test serializes and reopens only the owned blank graph. The routed parser
and serializer obligation requires a profile-aware reopen and save that proves
an unrelated unmodelled part and its relationship survive byte-identically.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML child-order, test-gate, or
structural findings.

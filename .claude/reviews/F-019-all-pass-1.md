# F-019, all aspects, pass 1

**Reviewed**: working diff, 4 files, 159 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the MIME shape gate accepts malformed top-level types and extra slashes

`crates/oxml-opc/src/relationship.rs:332`

The test only splits at the first slash and checks that both sides are nonempty.
A value such as `wrong/type/extra` therefore passes even though the plan says
the gate asserts the expected MIME shape. Require the top-level type to be
`application` and reject a subtype containing another slash.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML, tests, or
structure. The constant values use their required package or officeDocument
namespaces, the module exposure is additive, the existing universal defaults
reuse the new constants, and no dependency or published consumer changed.

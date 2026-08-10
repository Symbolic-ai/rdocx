# F-120, all, pass 2

**Reviewed**: `git diff --working` against claim commit `696d464`, one tracked
file with 1,767 changed lines, comprising 1,737 additions and 30 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, parsed provenance breaks structural equality for constructed axes

`crates/rpptx-chart/src/lib.rs:2116`

`Axis` derives `PartialEq`, which now includes the private `parsed_kind` field.
`Axis::new()` sets that field to `None`, while parsing the serialized axis sets
it to `Some(kind)`. A valid constructed axis therefore does not equal the
result of `Axis::from_xml(&axis.to_xml()?)`, even though its complete ChartML
model is unchanged. This violates the structural round-trip contract and lets
private parser provenance affect public value equality. The provenance guard
must not participate in model equality, or it must use a representation whose
constructed and reparsed states compare equally.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D1 is remediated. A parsed axis cannot be relabelled under a different
root while retaining its type-specific raw tail. No other correctness,
contract, panic-safety, OOXML namespace, schema-order, raw-preservation, test,
or structural finding was found.

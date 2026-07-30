# F-017, correctness, pass 2

**Reviewed**: working diff, 5 files, 1019 additions and 5 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, self-closing modeled values are reclassified as unsupported raw XML

`crates/oxml-core/src/custom_properties.rs:246`

`parse_empty_value` recognizes only the string and `empty` variants. A
self-closing `vt:i4`, `vt:r8`, or `vt:bool` therefore becomes `Raw`, although
the equivalent explicit empty element reaches the typed parser and is rejected
as an invalid value. Keep the two XML spellings consistent by rejecting empty
numeric and Boolean values, with regression cases for each spelling.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML ordering, namespace, preservation, test,
or structural findings.

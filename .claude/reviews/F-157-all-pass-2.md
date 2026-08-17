# F-157, all aspects, pass 2

**Reviewed**: working diff, 7 tracked files, 720 added lines and 16 removed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, chart relationship scope is not limited to a direct child

`crates/rdocx-oxml/src/drawing.rs:1041`

The namespace-aware scanner treats both start and empty `a:graphicData` events
as an open chart container. An empty container therefore leaves the flag set
for a later sibling. It also accepts a ChartML `c:chart` nested under an
unmodelled child, and a nested foreign element named `graphicData` can clear
the flag early at `crates/rdocx-oxml/src/drawing.rs:1069`. The schema contract
requires the typed chart to be the direct child of the nonempty ChartML
graphic-data element. Malformed URI attributes are also silently ignored at
`crates/rdocx-oxml/src/drawing.rs:1045` instead of returning the attribute
error.

### D2, aliased existing externalData can be duplicated

`crates/rdocx/src/document.rs:2990`

Package assembly rejects an existing workbook link only when serialized bytes
contain the fixed spelling `<c:externalData`. The typed ChartML model preserves
this child as raw XML, so a valid producer alias such as `<q:externalData>`
survives serialization and bypasses the check. Assembly then appends a second
external-data child and publishes a chart with competing workbook links. The
atomicity test covers only the fixed prefix and would not catch this case.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings.

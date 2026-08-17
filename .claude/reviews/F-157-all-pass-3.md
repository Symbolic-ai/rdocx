# F-157, all aspects, pass 3

**Reviewed**: working diff, 7 tracked files, 812 added lines and 18 removed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, nested ChartML externalData lookalikes block package assembly

`crates/rdocx/src/document.rs:3016`

The namespace-aware duplicate scanner accepts `c:externalData` at every XML
depth. A preserved producer extension can therefore contain a nested ChartML
element with that local name and cause `add_chart_package` to reject the chart,
even though only a direct `c:chartSpace` child occupies the schema's workbook
relationship slot. This violates the raw-child preservation contract and makes
the private package seam reject an otherwise assemblable chart. The guard must
track the chart-space root depth and only recognize a direct child.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings.

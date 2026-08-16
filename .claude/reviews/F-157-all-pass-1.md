# F-157, all aspects, pass 1

**Reviewed**: working diff, 6 tracked files, 650 added lines and 10 removed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, foreign namespace lookalikes become typed chart relationships

`crates/rdocx-oxml/src/drawing.rs:665`

The parser checks only the local name `chart` once it enters ChartML
`a:graphicData`. It also accepts any attribute whose local name is `id` at
`crates/rdocx-oxml/src/drawing.rs:1069`. A producer child such as
`<x:chart x:id="foreign"/>` inside that container is therefore promoted to the
typed chart relationship even though neither QName belongs to the ChartML or
office-document relationship namespace. The inline path has the same defect at
`crates/rdocx-oxml/src/drawing.rs:941`. This violates the plan's scoped
`c:chart r:id` parser contract and can route a preserved producer extension as
a native chart.

### D2, ambiguous structured payloads leave partial XML on error

`crates/rdocx-oxml/src/drawing.rs:1019`

`CT_Inline::to_xml` writes the `wp:inline`, extent, and document-properties
events before it validates picture XOR chart through `drawing_payload`.
`CT_Anchor::to_xml` does the same at
`crates/rdocx-oxml/src/drawing.rs:820`. An ambiguous or empty structured
payload returns an error only after mutating the caller's writer, contrary to
the approved requirement to reject the payload before serialization. A caller
that reuses the writer receives a corrupt XML prefix despite handling the
error.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings.

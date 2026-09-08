# F-245, all, pass 4

**Reviewed**: working-tree implementation diff excluding review records, 17 files, 2,216 additions and 40 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, replacement can overwrite a part referenced from another relationship scope

`crates/rdocx/src/document.rs:8744`

The exclusivity check counts relationships to the existing font part only in
the font-table owner's relationship set. A package-level or other part-level
internal relationship can target the same part while this count remains one.
Replacing that face then reuses and overwrites the shared part, changing the
bytes observed through the producer relationship. The existing package-wide
scan protects deletion only. Reuse must require exactly one internal target
edge across the complete package graph.

### D2, preserved raw content can lose a relationship it still references

`crates/rdocx-oxml/src/font_table.rs:317`

The relationship reference count visits only modeled embedded-font entries.
A valid producer extension retained in `raw_children` can carry the same
`r:id`, but removing or replacing the modeled face treats that relationship as
unreferenced and deletes it. The raw XML remains while its package edge is
gone. Relationship ownership checks must include references in retained raw
font-table content or fail closed when exclusivity cannot be proved.

### D3, sibling comments, processing instructions, and whitespace are dropped

`crates/rdocx-oxml/src/font_table.rs:207`

The root reader retains start and empty elements but discards every other
event. The font reader repeats the same behavior at line 398. Comments,
processing instructions, CDATA, and producer whitespace between fonts or
between font properties therefore disappear as soon as a typed mutation
serializes the table. These are valid unmodeled child events covered by the
byte-preservation contract and must be retained at their current raw boundary.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 3 fixed-prefix, GUID lexical-form, family and pitch enumeration, and
theme-color oracle defects are remediated. Correctness, contract, panics,
OOXML schema order, namespace safety, test relevance, and repository structure
were checked. No additional defect was found in allocation atomicity, modeled
child order, public type structure, deterministic layout input, or the pinned
LibreOffice execution path.

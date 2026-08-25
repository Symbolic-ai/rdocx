# F-X054, all, second recovery pass 1

**Reviewed**: uncommitted working diff, 15 files, 2,710 changed lines with
2,641 additions and 69 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, A removed namespace owner can be confused with a surviving raw duplicate

`crates/rdocx/src/document.rs:624`

Owner replay scores a candidate when its complete byte span contains a marker
as an arbitrary byte window. It does not establish that the same retained raw
child still belongs to that logical owner. For example, a document can bind
`x` to `urn:root` and contain one ordinary paragraph with `<x:producer/>`, then
a second paragraph that locally rebinds `x` to `urn:target` and contains the
same `<x:producer/>` bytes. If `Document::remove_content` removes the second
paragraph, the retained marker for that deleted owner finds the first
paragraph as its sole candidate. Replay then adds `xmlns:x="urn:target"` to
the surviving paragraph, changing its raw child from the root namespace to the
deleted owner's namespace. Save succeeds instead of treating the owner
identity as missing and failing closed. The same false identity is possible
for table, cell, content-control, hyperlink, and run owners because they share
this matcher.

## Smells

None.

## Nitpicks

None.

## Not found

All original pass 1 through pass 3 findings and recovery pass 1 through pass 3
triggers were rechecked. Raw children before run properties, parser-derived
qualified names, local and inherited namespace decoding, root and body scope
separation, empty undeclarations, visible child-content facts, fixed serializer
prefix collisions, unchanged unsafe-scope byte preservation, modified
unsafe-scope rejection, and unique retained-owner position changes remain
correct for their recorded inputs.

No additional findings were found in body, cell, paragraph, hyperlink, or run
item order, exact exposed raw bytes, public enum exhaustiveness, drawing or
field projections, legacy flattened accessors, producer-defined numbering
round trips, layout and exporter marker suppression, fail-closed ordinary or
deleted text decoding, Python error classification, OOXML child order, panic
safety, public API documentation, dependency structure, test naming, or the
repository structural rules. Focused tests for ordered save and reopen,
owner-position changes, fixed-prefix rejection, numbering preservation, and
invalid visible text passed during this review.

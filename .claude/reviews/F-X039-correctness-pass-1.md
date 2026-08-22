# F-X039, correctness, pass 1

**Reviewed**: working tree, 9 files, 505 insertions and 76 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, transfer identity omits the document-wide wrapping state

`crates/rdocx-layout/src/engine.rs:409`

`ReusableEngineContext` deliberately excludes body content, but retained safe
paragraph blocks depend on the document-wide wrapping-drawing predicate at
`crates/rdocx-layout/src/engine.rs:624` and
`crates/rdocx-layout/src/engine.rs:665`. An engine primed by a document without
a wrapping drawing can therefore transfer to a rebuilt document that adds one
in another paragraph. Unchanged paragraph cache hits then retain no reflow
parameters, so the second pagination pass cannot move their text around that
drawing. Add the exact document-wide wrapping state to the private transfer
identity and prove that a changed state rejects transfer.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML, test-gate, or structural findings.

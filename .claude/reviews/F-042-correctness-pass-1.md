# F-042, correctness, pass 1

**Reviewed**: working diff against
`6ec9d05721dcab1fd6f9a2ba8f924fe8215d8c80`, 6 tracked files, 217 insertions,
69 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the grouped-link gate does not prove page annotation assembly

`crates/oxml-pdf/src/writer.rs:1493`

The test checks only the transformed `/Rect`. An annotation dictionary can
remain as an unreferenced object when the page `/Annots` assembly pass is flat,
so reverting that one required pass would leave this test green. Assert that
the page dictionary references the nested annotation as well as checking its
transformed rectangle.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML, test, or structure findings.

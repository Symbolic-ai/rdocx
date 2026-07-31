# F-056, all aspects, pass 1

**Reviewed**: working diff, 2 files, 360 insertions and 7 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the standard-map test covers only four of twelve slots

`crates/oxml-drawing/src/color.rs:1467`

The approved test says the standard semantic slots select their standard theme
slots, but the test asserts only the two background and two text mappings. A
regression in any accent or hyperlink default would pass.

### D2, the selective override test covers only two untouched slots

`crates/oxml-drawing/src/color.rs:1509`

The approved override contract says only named mappings are replaced, but the
test checks the changed mapping and only two of the eleven mappings that must
remain equal to the master. An implementation that also changed an unchecked
accent or hyperlink slot would pass.

### D3, direct-colour results are not asserted exactly

`crates/oxml-drawing/src/color.rs:1538`

The regression compares each result under two maps but does not assert the
expected transformed or looked-up value. It also does not exercise the newly
documented `lastClr` fallback when the system name is absent from the lookup.
The test needs exact expected results for RGB, system, and preset choices plus
one missing-system-name fallback case.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML, tests, or
structure.

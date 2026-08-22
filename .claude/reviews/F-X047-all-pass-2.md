# F-X047, all aspects, pass 2

**Reviewed**: Revised working-tree diff, 9 files, 405 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, backend suppression is broader than the empty-carrier contract

`crates/oxml-pdf/src/writer.rs:427`

The new suppression predicates identify a carrier only by an empty glyph list.
That also changes handling and resource ordinals for a non-empty third-party
text run whose glyph list is empty, contrary to the contract not to change
non-Word inputs. Match the carrier's empty text and empty glyph list together
in PDF font collection, alpha collection, image indexing, emission, and the
Word result font replay.

### D2, the story regression does not assert zero width

`crates/rdocx/tests/regression_test.rs:5143`

The test filters for empty text and checks empty glyph and advance arrays, but
it never checks the segment's width. The approved contract and test name both
require zero width, so an accidental positive width would pass this gate and
move later content.

### D3, the testing HLD describes the superseded PDF comparison

`docs/hld/12-testing-strategy.md:207`

The specification says the compatibility case proves PDF equality between
ordinary and attributed layouts. That checks source neutrality, but it does
not prove backend invisibility because both layouts contain the carrier. The
implemented gate now compares output before and after carrier removal. Record
that stronger current behaviour for both PDF and raster.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML preservation,
tests, or structure.

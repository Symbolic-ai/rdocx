# F-006, correctness, pass 2

**Reviewed**: working diff in `crates/rdocx-pdf/src/image.rs`, 1 file with
66 additions and 7 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Resolved

### D1, EOI terminates the marker walk

`crates/rdocx-pdf/src/image.rs:66`

EOI now returns `None` immediately. The marker walk cannot reach an SOF segment
in trailing bytes, and `jpeg_bytes_after_eoi_cannot_supply_dimensions` proves
the reported trigger no longer returns dimensions.

## Smells

None.

## Nitpicks

None.

## Not found

- No regression in standalone-marker handling. TEM, SOI, and RST0 through RST7
  still advance without reading a segment length.
- No regression in the RST-before-SOF gate. It still returns the encoded width
  and height.
- No gap in the truncation coverage. Every strict prefix of the constructed
  header returns `None`, and the complete header returns its dimensions.
- No unchecked segment read or advance. Length validation and checked bounds
  remain in place.

All 11 `rdocx-pdf` tests pass, including the EOI, RST, and truncation tests.

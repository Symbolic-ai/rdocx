# F-006, correctness, pass 1

**Reviewed**: working diff in `crates/rdocx-pdf/src/image.rs`, 1 file with
51 additions and 7 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, EOI does not terminate the marker walk

`crates/rdocx-pdf/src/image.rs:66`

The `0xD0..=0xD9` arm groups EOI with restart markers and continues scanning.
EOI terminates the JPEG codestream. An input containing SOI, EOI, then a valid
SOF segment returns dimensions from bytes after the image has already ended.
That malformed input must return `None`, otherwise `decode_jpeg` can pass
invalid JPEG bytes through with dimensions taken from trailing data.

## Smells

None.

## Nitpicks

None.

## Not found

- No unchecked segment-length reads or advances. The implementation uses
  checked indexing, checked addition, and a bounded segment slice.
- No non-termination in the marker loop. Every accepted marker consumes at
  least its marker prefix and code.
- No gap in the required restart-marker classification. RST0 through RST7 are
  covered by `0xD0..=0xD7`, with TEM and SOI also handled as standalone.
- No defect in the named regression gate. It fails against the old walk and
  passes against this implementation.
- No gap in the required truncation loop for the constructed regression input.
  It checks every strict prefix and then checks the complete input.

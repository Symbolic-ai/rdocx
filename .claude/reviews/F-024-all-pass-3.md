# F-024, all, pass 3

**Reviewed**: the complete remediated working diff in
`crates/oxml-media/src/lib.rs`, 1 file with 630 additions and 1 deletion
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a truncated 40-byte BI_BITFIELDS header is accepted without its masks
`crates/oxml-media/src/lib.rs:342`

For a 40-byte `BITMAPINFOHEADER` using compression `3`, the required red,
green, and blue masks follow the DIB header rather than living inside it. The
parser bounds only `14 + dib_size`, then the centralized match accepts both
16-bit and 32-bit BI_BITFIELDS at `crates/oxml-media/src/lib.rs:381` and
`crates/oxml-media/src/lib.rs:384`. A 54-byte BMP header can therefore set
compression to `3`, omit all three masks, and still receive `ImageInfo`.
Require the external mask bytes for the 40-byte variant before accepting this
compression mode. The current direct BMP regressions cover core-header bit
counts and embedded alpha-mask semantics, but not this truncated bitfield
layout.

## Smells

None.

## Nitpicks

None.

## Not found

All four pass 1 findings and all four pass 2 findings are resolved with direct
negative regressions. No additional findings were found in PNG, JPEG, GIF,
WebP, DPI conversion, every-prefix bounds safety, arithmetic panic safety,
public API scope, dependency scope, or local readability. The centralized BMP
validity match is easier to audit than independent post-parse conditions, apart
from the missing external-mask bound above. `cargo test -p oxml-media` passed
all 14 tests, and `cargo check -p oxml-media --all-targets` passed.

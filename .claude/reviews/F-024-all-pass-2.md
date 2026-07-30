# F-024, all, pass 2

**Reviewed**: the complete remediated working diff in
`crates/oxml-media/src/lib.rs`, 1 file with 583 additions and 1 deletion
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, a BI_RGB mask is still treated as declared alpha
`crates/oxml-media/src/lib.rs:378`

The new mask handling fixes the 40-byte BI_RGB case from pass 1, but the guarded
arm also accepts compression `0` when a 56-byte or later DIB happens to contain
a nonzero alpha-mask field. Mask fields are not active for BI_RGB, whose high
byte remains unused. A 108-byte BI_RGB header with any nonzero value at file
offset 66 is therefore still reported as four-channel alpha. The mask must only
affect alpha for a compression mode that declares bitfields.

### D2, BITMAPCOREHEADER accepts bit counts that its format forbids
`crates/oxml-media/src/lib.rs:327`

The 12-byte core header permits only 1, 4, 8, or 24 bits per pixel. This branch
passes its bit count into the shared match at
`crates/oxml-media/src/lib.rs:374`, which also accepts 16 and 32. A hostile core
header with either value receives plausible metadata even though the DIB header
is inconsistent and should return `None`.

### D3, unsupported VP8 profile values are accepted
`crates/oxml-media/src/lib.rs:410`

The remediation checks the key-frame and show-frame bits, but omits the
three-bit VP8 version field. Only values 0 through 3 are defined by the current
VP8 format. Setting bits 1 through 3 of `payload[0]` to a value from 4 through 7
while retaining the key-frame bit, show-frame bit, start code, and dimensions
still returns `ImageInfo`. This violates the promised `None` result for an
unsupported header.

### D4, an odd RIFF file size is accepted
`crates/oxml-media/src/lib.rs:395`

WebP RIFF chunks occupy an even number of bytes after padding, so the RIFF size
field is necessarily even. The parser bounds `riff_end` but does not validate
that parity. Increasing the existing VP8 fixture's RIFF size from 22 to 23 and
adding one trailing byte produces malformed container metadata that is still
accepted. The new per-chunk padding-byte check does not cover this file-level
invariant.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 1's VP8X canvas-area defect, VP8 interframe defect, and nonzero RIFF
padding-byte defect are resolved and have direct negative tests. The original
40-byte BI_RGB BMP case now reports three channels without alpha, while the
56-byte BI_BITFIELDS fixture reports its alpha mask. No additional findings
were found in PNG, JPEG, GIF, DPI conversion, every-prefix bounds safety,
arithmetic panic safety, public API scope, dependency scope, or local
readability. The focused `cargo test -p oxml-media` run passed all 14 tests.

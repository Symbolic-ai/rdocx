# F-024, all, pass 1

**Reviewed**: the complete working diff in `crates/oxml-media/src/lib.rs`, 1
file with 537 additions and 1 deletion
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, uncompressed 32-bit BMP is incorrectly reported as having alpha
`crates/oxml-media/src/lib.rs:365`

The parser derives alpha only from `bits_per_pixel` and reports every 32-bit
bitmap as four-channel alpha. A 40-byte `BITMAPINFOHEADER` with `BI_RGB`, which
is exactly what the test fixture builds, does not declare an alpha mask. Its
fourth byte is reserved rather than an alpha channel. Later DIB headers can
declare alpha through a nonzero alpha mask, so the parser must consider the DIB
variant, compression, and masks instead of treating 32 bpp as sufficient. The
current assertion at `crates/oxml-media/src/lib.rs:719` locks in the incorrect
answer.

### D2, oversized VP8X canvases bypass the format's area limit
`crates/oxml-media/src/lib.rs:423`

VP8X stores width and height independently as 24-bit values, but requires their
product to fit in `u32`. The parser adds one to each dimension and returns them
without a checked area calculation. A hostile header can therefore declare
`16_777_216` by `16_777_216` and receive a valid `ImageInfo`, despite the design
contract that inconsistent headers return `None`.

### D3, a lossy WebP interframe can be accepted as a still image
`crates/oxml-media/src/lib.rs:399`

A simple lossy WebP must contain a VP8 key frame. The parser checks the key
frame start code at payload bytes 3 through 5, but never checks the frame-type
bit in the three-byte frame tag. Setting bit zero of `payload[0]` while leaving
the start-code and dimension bytes intact produces an interframe header that is
reported as a valid still image.

### D4, a nonzero RIFF padding byte is accepted
`crates/oxml-media/src/lib.rs:391`

For an odd WebP chunk size, RIFF requires one zero padding byte. The code proves
that the padded byte exists through `padded_end`, but never reads or validates
it. Changing the padding byte in the existing five-byte VP8L fixture from zero
to a nonzero value still returns metadata, so malformed container input does
not follow the promised `None` path.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings were found in PNG `IHDR` and `pHYs` conversion, JPEG
marker bounds and JFIF conversion, GIF logical-screen dimensions, arithmetic
panic safety, every-prefix loop coverage, public API scope, dependency scope,
or local structure. The five format readers and shared checked-read helpers
remain directly readable in the approved existing crate root. The focused
`cargo test -p oxml-media` run passed all 14 tests.

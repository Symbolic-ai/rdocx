# F-225, correctness, pass 4

**Reviewed**: current working tree implementation across 16 files, 5,349
inserted lines and 21 deleted lines from `597a27c`
**Verdict**: 9 defects, 0 smells, 0 nitpicks

## Defects

### D1, the affine group does not scale editable text glyphs

`crates/rpptx/src/pdf.rs:1872`

Editable text keeps the untransformed PDF font size at lines 1872 to 1874 and
places its local text box inside the SVD-derived group at
`crates/rpptx/src/pdf.rs:1945`. The existing resolver consumes the group scale
only by changing the child bounds at `crates/rpptx-layout/src/context.rs:1075`,
while run resolution retains the declared point size at
`crates/rpptx-layout/src/context.rs:2011`. A valid `200 Tz`, nonuniform CTM, or
sheared text matrix therefore enlarges or shears the text-box bounds without
applying that affine scale to the glyphs. The pass-4 regression saves and
reopens the group and checks XML at `crates/rpptx/src/pdf.rs:3925`, but it never
renders the result. This does not prove or provide the required arbitrary
affine editable text projection.

### D2, a valid ToUnicode map is discarded when a simple encoding is also present

`crates/rpptx/src/pdf.rs:2337`

The importer calls lopdf's general font-encoding resolver, then declares the
result invalid whenever `/ToUnicode` exists but that resolver returned a
single-byte encoding at lines 2346 to 2351. That resolver gives the font's
ordinary `/Encoding` entry priority. A common valid simple font containing both
`/Encoding /WinAnsiEncoding` and a valid `/ToUnicode` stream is consequently
marked unsupported and direct-decoded instead of using its Unicode map. The
new cache regression uses `Identity-H` at `crates/rpptx/src/pdf.rs:4008`, so it
does not exercise this ordinary simple-font combination.

### D3, isolated composite text leaves the text matrix at the wrong origin

`crates/rpptx/src/pdf.rs:1216`

Selecting a Type0 font marks its metrics unsupported at
`crates/rpptx/src/pdf.rs:890`. Every string show then returns before the text
matrix advancement at `crates/rpptx/src/pdf.rs:1450`. If the same text object
switches to a supported simple font and shows another string without a new
`Tm`, that supported text resumes from the pre-Type0 origin and can overlap the
omitted source text. The regression at `crates/rpptx/src/pdf.rs:4163` uses only
the composite font, so it does not test the supported sibling that observes the
stale state. Unsupported state is therefore not isolated from later supported
operators.

### D4, simple-font width fallback rejects valid character codes

`crates/rpptx/src/pdf.rs:1357`

For any shown code outside the retained `/FirstChar` plus `/Widths` array, the
importer returns a fatal missing-width error at lines 1357 to 1367. PDF simple
fonts use the descriptor's `/MissingWidth` for those codes. The retained width
model contains no missing-width value at `crates/rpptx/src/pdf.rs:213`, and the
parser reads only `FirstChar` and `Widths` at
`crates/rpptx/src/pdf.rs:2406`. The public fixture masks this case by defining
all 256 widths at `crates/rpptx/tests/integration.rs:104`. FirstChar and Widths
advancement is present, but valid fallback advancement remains unsupported by
neither a diagnostic nor the PDF-defined metric.

### D5, DCT images can bypass the declared pixel limit

`crates/rpptx/src/pdf.rs:1482`

The pixel budget trusts the PDF dictionary's declared `/Width` and `/Height`
at lines 1482 to 1509. The DCT branch then forwards the JPEG bytes unchanged at
`crates/rpptx/src/pdf.rs:1543` without checking the JPEG's intrinsic dimensions
against those declarations or charging the intrinsic pixel count. A stream can
declare a 1 by 1 image while carrying a much larger JPEG, pass
`max_pixels_per_page`, and publish bytes that the presentation renderer later
decodes at the larger intrinsic size. This defeats the public image resource
bound.

### D6, wrong-typed image state is silently repaired

`crates/rpptx/src/pdf.rs:3099`

A present non-boolean `/ImageMask` is treated as false at lines 3099 to 3103,
and a present non-integer `/BitsPerComponent` is treated as 8 at lines 3107 to
3110. The decode path repeats the bit-depth default at
`crates/rpptx/src/pdf.rs:3181`. Thus malformed dictionaries such as
`/ImageMask 7` or `/BitsPerComponent /Eight` can enter the supported Flate
pipeline instead of failing strict state validation. The strict Filter repair
is correct at `crates/rpptx/src/pdf.rs:3073`, but the neighboring typed image
fields retain the same lossy-normalization failure.

### D7, active-content traversal runs before the object cap and can be quadratic

`crates/rpptx/src/pdf.rs:338`

The importer scans the full object graph before checking `max_objects` at lines
338 to 349. `reject_active_content` also creates a fresh visited set for every
top-level object at `crates/rpptx/src/pdf.rs:2072`. Many objects can reference
the same broad safe subgraph, causing that subgraph to be walked again from
each root. The default admits up to 250,000 objects at
`crates/rpptx/src/pdf.rs:52`, and the caller-selected object cap does not bound
this work because the scan precedes the check. Bounded input can therefore
force quadratic CPU consumption inside the security preflight.

### D8, editable serialization changes legal zero-length dash members

`crates/rpptx/src/pdf.rs:3480`

The parser correctly accepts nonnegative members and rejects only an all-zero
array at `crates/rpptx/src/pdf.rs:2961`. DrawingML lowering then clamps every
dash and gap ratio to at least one at lines 3480 to 3485. A legal PDF pattern
such as `[0 2]` with round caps relies on a zero-length painted segment to
produce dots, but editable output serializes a positive segment instead. The
phase-equals-cycle fix is correct at `crates/rpptx/src/pdf.rs:3010`. Its test
only checks the normalized vector and presence of custom-dash XML at
`crates/rpptx/src/pdf.rs:3862`, not preservation of the zero member.

### D9, subset fonts with the same family name are conflated

`crates/rpptx/src/pdf.rs:2550`

Embedded-font collection strips the six-character PDF subset prefix at
`crates/rpptx/src/pdf.rs:2543`, then drops every later font with the same family
at lines 2550 to 2552. Page text retains only that family name at
`crates/rpptx/src/pdf.rs:203`, and shaping resolves by family at
`crates/rpptx/src/pdf.rs:1259`. Two valid subset TrueType fonts such as
`AAAAAA+Family` and `BBBBBB+Family` can contain different glyph programs, yet
all text is shaped through whichever subset was encountered first. The one-font
fixture at `crates/rpptx/tests/integration.rs:100` cannot expose this embedded
font identity loss.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 3 disposition

- D1 is closed. Both differential paths feed raw full-image SSIM directly to
  the final 0.995 predicate at `crates/rpptx/tests/integration.rs:444`, with no
  blur or alternate score.
- D2 is closed. The isolated one-pixel case asserts raw SSIM failure at
  `crates/rpptx/tests/integration.rs:511`, then proves geometry, text, link, and
  point-tolerance inputs remain true at line 520.
- D3 is closed for one charged owned mapping reused by repeated shows and fatal
  cache parse errors at `crates/rpptx/src/pdf.rs:2324`. The valid ordinary
  encoding plus ToUnicode combination is the separate pass-4 D2.
- D4 is closed for group construction, save, and reopen at
  `crates/rpptx/src/pdf.rs:3900`. The missing render fidelity proof and glyph
  scaling failure are pass-4 D1.
- D5 is closed by retained-element accounting before layout, editable, and link
  insertion at `crates/rpptx/src/pdf.rs:1713`.
- D6 is closed by tainting rendering modes 4 through 7 until graphics-state
  restore at `crates/rpptx/src/pdf.rs:978`.
- D7 is closed by set-based source-string collection at
  `crates/rpptx/src/pdf.rs:2433`.
- D8 is closed for in-range `FirstChar` and `Widths` plus `TJ` numeric
  adjustment at `crates/rpptx/src/pdf.rs:1351` and
  `crates/rpptx/src/pdf.rs:1007`. MissingWidth and composite isolation are the
  separate pass-4 D4 and D3.
- D9 is closed for the phase-equals-cycle boundary at
  `crates/rpptx/src/pdf.rs:3010`. Zero-member DrawingML lowering is pass-4 D8.
- D10 is closed by exact scalar and array-member type rejection at
  `crates/rpptx/src/pdf.rs:3073`.

## Not found

No additional findings in raw SSIM threshold gaming, Poppler binary pinning,
the 38.4-point styled source geometry, page-box and rotation normalization,
operator arity, current-subpath state, finite derived geometry, CTM stroke
width scaling, retained-element counting, rendering-mode taint, source-string
hash deduplication, dash phase normalization, Filter parsing, URI scheme
restriction, link relationship ownership, OOXML child order, media relationship
publication, native public API shape, feature gating, transactional candidate
publication, panic paths reachable from imported bytes, dependency direction,
HLD file scope, or structural rules. The implementation changes exactly the
nine HLD files named by the approved plan and adds no unapproved trait, generic
parameter, wrapper, builder, feature flag, crate, or integration test binary.

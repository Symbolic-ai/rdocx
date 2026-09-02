# F-225, correctness, pass 1

**Reviewed**: working tree implementation across 15 files, 3,006 inserted lines and 7 deleted lines
**Verdict**: 11 defects, 0 smells, 0 nitpicks

## Defects

### D1, consecutive `cm` transforms are multiplied in the wrong order
`crates/rpptx/src/pdf.rs:105`

PDF premultiplies each additional transform with the existing CTM. With this
column-vector `point` implementation, the stored matrix must compose the
existing transform with the new one. `then` instead returns `next * self`.
For example, a scale by two followed by a ten-point translation retains a
ten-point translation instead of the required twenty-point device-space
translation. Paths, text origins, and images therefore move to the wrong
coordinates whenever noncommuting `cm` operators are consecutive.

### D2, text positioning discards required text-state geometry
`crates/rpptx/src/pdf.rs:715`

`Td` and `T*` edit only the translation fields and invent leading as 1.2 times
the font size. `TJ` concatenates its strings while discarding every numeric
position adjustment at lines 739 to 745. `show_text` then uses only the `e` and
`f` fields of the text matrix at line 919, advances by the bundled replacement
font's shaped width at line 950, and emits an unrotated text box. A rotated or
scaled `Tm`, a declared `TL`, the spacing operands of `"`, or a kerning value in
`TJ` therefore produces wrong text size, orientation, and subsequent origins in
both modes.

### D3, affine image bounds are calculated from only two corners
`crates/rpptx/src/pdf.rs:1003`

An affine transform maps the unit image square through four corners. The code
uses only `(0, 0)` and `(1, 1)` and converts the result to an axis-aligned
rectangle. Under a 45-degree rotation those opposite corners have the same x
coordinate, so a nonzero image collapses to the minimum editable width. Mixed
shear and reflection have the same failure, and all image orientation is lost.

### D4, unsupported image masks and decode semantics are silently accepted
`crates/rpptx/src/pdf.rs:1011`

The DCT branch retains bytes without checking `Mask`, `SMask`, `Decode`, colour
space, or bit depth. The Flate branch checks only bit depth and two colour-space
names. A masked or inverted image is imported with visibly different pixels
and no diagnostic, even though masks and unsupported image semantics are
outside the approved subset and must remain explicit.

### D5, declared path stroke state is lost
`crates/rpptx/src/pdf.rs:1803`

`parse_dash` ignores the dash phase operand. The editable projection at
`crates/rpptx/src/pdf.rs:2123` writes only width and colour, dropping the parsed
cap, join, and dash array entirely. A supported dashed, round-capped, or
bevel-joined path therefore changes paint after save and reopen. Preserved mode
also cannot reproduce a nonzero dash phase.

### D6, the decompression limit is not an aggregate import budget
`crates/rpptx/src/pdf.rs:341`

Embedded fonts receive the full limit before page content starts. Every font
encoding at line 850 and every image at line 1019 also receives the full limit,
while `self.decompressed` charges only page-content bytes. A PDF can therefore
decode the limit once for fonts, once across content, and again for every image
or repeated font encoding. The advertised aggregate cap does not bound retained
memory or decompression work.

### D7, `max_pixels_per_page` is enforced per image instead of per page
`crates/rpptx/src/pdf.rs:983`

Each image is compared independently with the page pixel limit, and no page
pixel accumulator is updated after acceptance. A page containing many images
can retain an arbitrary multiple of `max_pixels_per_page` pixels while every
individual check passes.

### D8, active-content and action rejection is incomplete
`crates/rpptx/src/pdf.rs:1387`

The JavaScript scan checks only each indirect object's immediate dictionary. A
direct nested action such as a catalog `OpenAction` dictionary containing
`S /JavaScript` and `JS` is never visited. Link annotations with another action
kind are also converted to a diagnostic and dropped at
`crates/rpptx/src/pdf.rs:1125` instead of being rejected. Both inputs violate the
approved fail-closed JavaScript and non-URI action boundary.

### D9, unsupported font encodings can substitute without a diagnostic
`crates/rpptx/src/pdf.rs:848`

The importer asks `lopdf` for an encoding and has no way to observe its standard
encoding fallback for unsupported maps. It records only a typeface-resolution
substitution later. A font with an unsupported encoding can therefore decode to
different Unicode text without the required stable diagnostic naming the
requested encoding.

### D10, the editable round-trip test omits most of its contract
`crates/rpptx/src/pdf.rs:2441`

The approved test includes text, images, path geometry and paint, and URI target
and rectangle. The source PDF contains no image, and this test checks only text,
shape count, two XML substrings, and relationship target. Image bytes and
dimensions, exact path commands and paint, and the link rectangle can all
regress while this test remains green. The all-limits test at line 2336 also
does not construct its named encrypted, cyclic, decompression, or per-page
aggregate pixel cases.

### D11, the differential gate does not prove the declared editable or sensitivity boundaries
`crates/rpptx/tests/integration.rs:236`

Only the preserved PNG is compared with Poppler. The editable presentation is
never rendered, its link rectangle is never inspected, and the test reuses the
preserved SSIM when accepting editable mode at line 275. The perturbation test
at `crates/rpptx/tests/integration.rs:287` passes synthetic booleans and a full
white-to-black image change to the predicate. It does not shift imported
geometry by 1.01 points, mutate imported text or link state, or apply a
calibrated pixel perturbation near the 0.995 floor. Regressions in the real
comparison path can therefore pass both named gates.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in public feature gating, native API shape,
transactional candidate publication, graphics-state stack balance, OOXML child
order, link relationship ownership, HLD file scope, panic paths, or structural
rules. The approved module and dependency have one current consumer, and the
implementation adds no trait, generic parameter, wrapper, builder, feature
flag, crate, or integration test binary.

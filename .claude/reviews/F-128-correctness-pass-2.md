# F-128, correctness, pass 2

**Reviewed**: working-tree diff from `575a8b6330609ec929dce4a15d8c02658be71eff`, 18 files, 1,396 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, header-only preview validation can still suppress the labelled fallback
`crates/rpptx/examples/render_deck.rs:460`
`crates/rpptx-layout/src/context.rs:826`
`crates/oxml-media/src/lib.rs:323`
`crates/oxml-pdf/src/raster.rs:774`

Pass-1 D2 is not fully remediated. Package assembly admits a preview whenever
`oxml_media::probe` reports PNG or JPEG, but that probe validates metadata, not
backend decodability. The PNG probe returns success as soon as it encounters a
bounded `IDAT` chunk without inflating its payload. A PNG with a valid signature,
IHDR, and chunk framing but corrupt image data therefore enters `RenderInput`.
The resolver sees an allowed content type, freezes `ResolvedContent::Image`, and
suppresses the labelled placeholder. The raster backend later fails its full
decode and silently returns without drawing the image. A valid PNG declared as
`image/jpeg` follows the same blank-output path because the resolver accepts both
types without matching MIME to the probed format, while the backend selects its
JPEG decoder from that MIME value. The end-to-end malformed case uses `not a PNG`,
which the header probe rejects, so neither backend-invalid case can fail the
current gate.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D1 is fixed. Missing preview targets remain unresolved and reach the
labelled chart fallback instead of aborting package assembly.

Pass-1 D3 is fixed for the stated valid, missing, and plainly malformed cases.
The end-to-end fixture contains a real `mc:AlternateContent` chart choice with
an immediate typed picture fallback, asserts the cached image bytes for the
valid case, and asserts the stable label for the missing and malformed cases.

Pass-1 D4 is fixed. Equal chart relationship identifiers are resolved through
slide, layout, and master scopes, and the negative resolver test exercises a
typed missing-target resource.

No additional relationship-scope, namespace-URI, raw-byte preservation,
schema child-order, deterministic-ordering, panic-path, dependency-direction,
or structural-simplicity defects were found.

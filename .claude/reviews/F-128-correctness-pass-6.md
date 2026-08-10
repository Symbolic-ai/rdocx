# F-128, correctness, pass 6

**Reviewed**: working-tree diff from `575a8b6330609ec929dce4a15d8c02658be71eff`, 20 files, 1,822 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, raster admission still accepts JPEG component layouts the PDF backend misdeclares
`crates/rpptx/examples/render_deck.rs:502`
`crates/oxml-pdf/src/image.rs:47`
`crates/oxml-pdf/src/writer.rs:610`

The compatibility predicate accepts every fully decoded JPEG whose MIME type and
resource bounds pass. That includes valid one-component grayscale and
four-component CMYK JPEG previews. The raster backend converts either form to
RGBA, so the native-bounds visibility test accepts it. The PDF path instead
passes the original DCT stream through while `decode_jpeg` hard-codes
`DeviceRGB`, and the writer emits that three-component colour space regardless
of the JPEG frame's component count. A grayscale or CMYK cached preview can
therefore suppress the labelled fallback even though the shared PDF backend
describes its encoded samples with the wrong colour space. The gate covers only
the bundled three-component JPEG and cannot fail this case.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-5 D1 is closed. Native-pixel visibility retains sparse edge content while
encoded input, decoded pixels, PNG scanlines and zlib output remain bounded by
the 16 MiB admission caps.

Pass-5 D2 is closed. Only the direct
`p:graphicFrame/a:graphic/a:graphicData` path can select a chart choice, while a
chart URI in a descendant extension remains opaque. Namespace aliases and the
read-only raw serialization projections remain intact.

Pass-5 D3 is closed. The corpus example and integration tests call the same
crate-local `render_package` implementation for relationship assembly, media
admission, chart parsing, resolution and ordinary renderer lowering.

No additional source-scope, chart-routing, malformed-input, arithmetic-panic,
raw-byte preservation, schema-child-order, group-lowering, font-manager,
dependency-direction, HLD-alignment or structural-simplicity findings were
found. Pass-1 through pass-4 findings remain closed apart from the newly
identified JPEG component-layout branch of renderer compatibility.

# F-128, correctness, pass 4

**Reviewed**: working-tree diff from `575a8b6330609ec929dce4a15d8c02658be71eff`, 18 files, 1,572 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the one-pixel admission probe does not bound source image decoding
`crates/rpptx/examples/render_deck.rs:536`
`crates/oxml-pdf/src/image.rs:122`
`crates/oxml-pdf/src/image.rs:135`

The candidate page is one pixel, but the raster backend fully inflates the PNG
IDAT stream and computes source-sized scanline storage before scaling it into
that page. A package-controlled PNG with the correct signature and declared
MIME can therefore expand without a byte limit. Crafted maximum dimensions can
also overflow the unchecked expected-length multiplication in debug builds.
The admission check can panic or exhaust memory before it returns false, so it
is not a bounded or panic-safe predicate over untrusted package media.

### D2, the cached chart relationship projection can diverge from serialized XML
`crates/rpptx-oxml/src/graphic_frame.rs:44`
`crates/rpptx-oxml/src/graphic_frame.rs:59`
`crates/rpptx-oxml/src/graphic_frame.rs:60`
`crates/rpptx-oxml/src/graphic_frame.rs:185`

`CT_GraphicData::payload` remains publicly mutable while the new relationship
identifier is stored separately and never refreshed. Replacing the bytes in a
public `GraphicDataPayload::Chart` with a payload that names a different
relationship makes `to_xml` write the new bytes while
`chart_relationship_id()` keeps returning the old identifier. The resolver can
then route a frame to a chart part that its serialized XML no longer names.

## Smells

None.

## Nitpicks

None.

## Not found

All pass-1 through pass-3 D1 to D4 cases are closed. Missing image targets now
stay unresolved, real PNG and JPEG cached previews render, plain malformed
bytes and fully corrupt PNG or JPEG payloads retain the labelled fallback, and
both declared-MIME mismatch directions are rejected. For inputs within decoder
resource bounds, the black and white one-pixel comparison correctly detects
silent raster decode failure and fully transparent output.

The end-to-end fallback test covers valid PNG, the bundled-template JPEG,
missing target, plain malformed bytes, header-valid corrupt PNG, PNG declared
as JPEG, a SOF-valid JPEG without scan data, and JPEG declared as PNG. It also
proves deterministic resolved layout and PNG output. Focused chart resolver and
OOXML preservation tests passed, as did the `rpptx` all-targets check and the
dependency tree inspection. The complete `rpptx-layout` suite passed 100 tests
and failed only its three pre-existing corpus-required tests because this
worktree has no `corpus/pptx` directory.

No additional relationship-scope, namespace-resolution, raw-byte preservation,
schema child-order, group-lowering, font-manager, deterministic-ordering,
dependency-direction, HLD-alignment, or structural-simplicity defect was found.
The added dependency edges remain inward within the PresentationML family.
Every crate whose public surface changed has `publish = false`, so the public
API of a published crate rider does not apply.

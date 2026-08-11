# F-128, correctness, pass 7

**Reviewed**: working-tree diff from `575a8b6330609ec929dce4a15d8c02658be71eff`, 20 files, 1,876 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-6 D1 is closed. Package admission requires the probed JPEG frame to use
8-bit samples and exactly three components before the raster visibility check
can admit it at `crates/rpptx/examples/render_deck.rs:513`. The shared
production package gate carries raster-decodable one-component grayscale and
four-component CMYK fixtures at `crates/rpptx/tests/integration.rs:520` and
`crates/rpptx/tests/integration.rs:524`, then requires both to retain the
labelled fallback at `crates/rpptx/tests/integration.rs:581`.

Pass-1 through pass-5 findings remain closed. Missing and malformed preview
targets stay unresolved, PNG and JPEG MIME types are matched to sniffed bytes,
encoded and decoded storage is capped before backend decoding, sparse visible
pixels remain admissible, and chart payload bytes cannot diverge from their
read-only relationship projection. Only the schema-positioned graphic-data
payload selects a chart choice, while descendant chart URIs remain opaque.
Integration tests and the corpus driver still call the same crate-local package
rendering function.

No additional correctness, contract, panic-path, OOXML namespace, raw-byte
preservation, schema-child-order, test-gate, source-scope, group-lowering,
font-manager, dependency-direction, HLD-alignment, or structural-simplicity
finding was found. The focused deterministic fallback, byte-preservation, and
descendant-chart tests passed during this review. The added dependency edges
remain inward and every changed crate remains unpublished.

# F-128, correctness, pass 1

**Reviewed**: working-tree diff from `575a8b6330609ec929dce4a15d8c02658be71eff`, 18 files, 1,212 changed lines
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, a missing cached-image target aborts before the labelled fallback
`crates/rpptx/examples/render_deck.rs:456`

Package assembly returns an error for every internal image relationship whose
target part is absent. When that relationship belongs to the `p:pic` paired
with an unsupported chart choice, resolution never runs and cannot freeze the
required labelled placeholder or its chart diagnostic. A deck with a missing
cached preview therefore fails the complete rendering pipeline instead of
retaining visible chart bounds.

### D2, an unrenderable preview is accepted as a successful cached fallback
`crates/rpptx-layout/src/context.rs:819`

The fallback path treats any relationship-resolved picture as usable without
checking its package content type or decodability. The shared PDF and raster
backends render only PNG and JPEG image bytes, so an EMF, WMF, SVG, or malformed
preview can cross as `ResolvedContent::Image` and disappear at the backend.
The resolver then suppresses the labelled bounds fallback even though no
compatible cached picture is available.

### D3, the required cached-image integration gate contains no cached image
`crates/rpptx/tests/integration.rs:4471`
`crates/rpptx-layout/src/context.rs:5818`

The end-to-end fallback fixture starts from a direct authored chart frame and
only replaces its ChartML part with a 3-D plot. It creates neither
`mc:AlternateContent` nor a fallback `p:pic`, so the deterministic render test
exercises the labelled placeholder rather than cached-image selection. The
resolver-only test supplies the non-image bytes `chart preview` and stops after
checking `ResolvedContent`. No gate proves that a 3-D chart renders an actual
cached image, and neither D1 nor D2 can fail the current tests.

### D4, required scoped and missing-target routing cases stop short of resolution
`crates/rpptx-layout/src/context.rs:5866`
`crates/rpptx-layout/src/context.rs:5917`

The equal-identifier test only calls `ScopedChartResources::get` on three maps.
It never resolves slide, layout, and master chart frames, so a regression that
routes every frame through one scope would still pass. The contextual-failure
test constructs an absent relationship and an external resource, but never a
`ChartResource::MissingTarget`. The design plan requires both routing paths to
be exercised through chart resolution.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract-scope defects were found. No namespace-URI, raw-byte
preservation, schema child-order, deterministic ordering, panic-path,
dependency-direction, or structural-simplicity defects were found.

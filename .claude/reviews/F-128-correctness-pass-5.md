# F-128, correctness, pass 5

**Reviewed**: working-tree diff from `575a8b6330609ec929dce4a15d8c02658be71eff`, 20 files, 1,819 changed lines
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the one-pixel visibility probe rejects valid sparse cached previews
`crates/rpptx/examples/render_deck.rs:499`
`crates/rpptx/tests/integration.rs:486`

Media admission scales every source image into one output pixel and accepts it
only when that pixel differs from a black or white baseline. A valid larger PNG
whose centre sampling neighbourhood is transparent but whose edges or corners
contain visible chart content therefore compares equal to both baselines. The
image is renderer-compatible and would be visible at the actual chart bounds,
but it never enters the scoped media map and the unsupported chart receives the
labelled placeholder instead of its cached image. The gate uses a one-pixel PNG,
so it cannot expose this false rejection.

### D2, descendant graphic data can misclassify a non-chart choice as a chart
`crates/rpptx-oxml/src/shape_tree.rs:501`
`crates/rpptx-layout/src/context.rs:697`

`graphic_frame_projects_chart` returns true for any descendant
`a:graphicData` with the chart URI. It does not require that element to be the
graphic frame's schema-positioned payload. A non-chart graphic frame can retain
such an element inside an extension or another unmodelled descendant and then
be projected as `chart_choice`. The alternate-content resolver consumes every
projected frame as a chart without checking its typed graphic-data payload, so
it suppresses the ordinary selected fallback and emits a missing-chart or
labelled-chart result. Malformed and non-chart choices are required to remain
opaque.

### D3, the end-to-end tests do not exercise production package assembly
`crates/rpptx/tests/integration.rs:444`
`crates/rpptx/tests/integration.rs:4509`
`crates/rpptx/examples/render_deck.rs:188`

The named end-to-end test calls a test-local `render_presentation_package`
implementation. That helper duplicates scoped relationship assembly, media
admission, chart parsing, resolver invocation, and renderer lowering instead of
invoking the production path in `render_deck`. Reverting the example to its old
resolver call or breaking its scoped chart assembly leaves the test-local copy
and all named F-128 gates green. The required integration evidence therefore
does not protect the code path that renders real presentation packages.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-4 D1 is closed for resource bounds. Encoded input, pixel storage, PNG
scanline arithmetic, and zlib output are capped before the existing unbounded
raster decoder is called. The inflation and maximum-dimension gates exercise
the relevant rejection paths.

Pass-4 D2 is closed for raw payload mutation. Chart and opaque payloads are
read-only, the dedicated mutable accessor exposes only typed table payloads,
and the relationship projection remains paired with the raw chart bytes used
for serialization.

No additional source-scope, namespace-attribute, raw-byte preservation, schema
child-order, group-lowering, font-manager, dependency-direction, arithmetic
panic, or structural-simplicity findings were found.

# S32 sprint review, pass 1

**Reviewed**: `sprint/s32` against
`ad7661152b266462134ce0de4d0d88744191a32e`, 35 files, 3,492 changed lines,
crates: `oxml-layout`, `rpptx-oxml`, `rpptx-layout`, `rpptx-chart`,
`rpptx-render`, `rpptx`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, slide master colour mapping falls through to the layout override
`crates/rpptx/examples/render_deck.rs:690`

The package renderer scans the slide and layout colour-map overrides but returns
only an explicit override. A slide with `a:masterClrMapping` therefore continues
to a layout `a:overrideClrMapping` and uses the layout map instead of the
master map requested by the slide. F-128 passes this result into the resolver,
then F-127 uses it for every unstyled series accent. Such an authored or
preserved chart renders the wrong series colours through the integrated path.
The fix must treat a present master mapping at either inheritance level as a
terminal selection of `master.color_map`, and an end-to-end chart test must
distinguish the master, layout, and slide mappings.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M12 gate is: "a chart created by rpptx opens in PowerPoint, its data is
editable, and it renders."

The narrow gate has evidence. Microsoft PowerPoint 16.104 build
16.104.25121423 opened the SHA-256-bound F-124 candidate without repair, and
Edit Data showed the authored categories and both source series at
`docs/sprints/AS_BUILT.md:4120`. The S32 test
`authored_chart_relationship_enters_presentation_renderer` starts from the
owning `Presentation::add_chart` facade and requires finite native paths and
labels at `crates/rpptx/tests/integration.rs:438`. Its helper invokes the same
package-rendering function used by the corpus driver at
`crates/rpptx/tests/integration.rs:4609`.

The milestone gate therefore holds for the recorded candidate and authored
chart. The sprint is not ready because B1 violates S32's separate effective
colour-map definition of done.

## Not found

- `duplication`: no second chart renderer, colour resolver, package assembly,
  or fallback implementation was added.
- `layering`: the new `rpptx-layout` to `rpptx-chart` edge stays within the
  PresentationML family, and no `oxml-*` crate gained a format-specific edge.
- `harness`: both S32 AS_BUILT entries declare the hash harness unchanged, and
  all 28 integrated entries are recorded as matching.
- `docs`: all HLD files named by the two approved plans were updated, with no
  additional HLD contradiction found beyond B1's implementation defect.
- `deps`: `rpptx-layout` consumes `rpptx-chart` for native projection, and the
  `rpptx` development dependency on `miniz_oxide` bounds test and example PNG
  inflation.
- `surface`: the added chart resources, frozen group variant, immutable OOXML
  projections, and shared-font lowering entry point are consumed by the
  approved rendering path. No unrelated public API was found.
- `fallback and preservation`: cached preview admission is bounded, the
  labelled fallback remains visible, and raw ChartML and alternate-content
  bytes remain the serialization source.

# S32 sprint review, pass 2

**Reviewed**: `sprint/s32` against
`ad7661152b266462134ce0de4d0d88744191a32e`, 36 files, 3,669 changed lines,
crates: `oxml-layout`, `rpptx-oxml`, `rpptx-layout`, `rpptx-chart`,
`rpptx-render`, `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M12 gate is: "a chart created by rpptx opens in PowerPoint, its data is
editable, and it renders."

The narrow gate holds. Microsoft PowerPoint 16.104 build 16.104.25121423
opened the SHA-256-bound authored chart without repair, and Edit Data showed
the recorded categories and source series at `docs/sprints/AS_BUILT.md:4120`.
The passing `authored_chart_relationship_enters_presentation_renderer` test
starts with `Presentation::add_chart`, requires finite native paths and shaped
labels at `crates/rpptx/tests/integration.rs:508`, and calls the same package
rendering function as the corpus driver at
`crates/rpptx/tests/integration.rs:4678`.

The S32 gates also hold. The direct and mapped accent series tests pass, the
3-D cached-image and diagnostic test passes, and the deterministic production
package fallback test proves usable PNG and JPEG previews remain visible while
missing or unusable previews retain the label at
`crates/rpptx/tests/integration.rs:624`. The hash harness reports all 28 entries
unchanged.

## Not found

- `interaction`: the pass-1 colour-map precedence defect is fixed. A slide or
  layout `masterClrMapping` now terminates at the master map at
  `crates/rpptx/examples/render_deck.rs:685`. The passing end-to-end regression
  distinguishes master, layout, and slide mappings at
  `crates/rpptx/tests/integration.rs:214`, before F-127 consumes the result.
- `duplication`: one chart renderer, one series-colour resolver, one scoped
  package assembly path, and one fallback selector serve the integrated path.
- `layering`: the new `rpptx-layout` to `rpptx-chart` edge remains inside the
  PresentationML family. No `oxml-*` crate gained a format-specific dependency.
- `harness`: both S32 AS_BUILT entries declare no delta, the recorded full
  integrated verification agrees, and the fresh check reports 28 matches.
- `gate`: authored rendering, exact series colours, cached-image fallback,
  visible labelled fallback, diagnostics, and byte preservation have passing
  focused evidence.
- `docs`: F-127 updated HLD 09. F-128 updated exactly HLD 03, 06, 07, 08, and
  09 as approved. No contradictory HLD section was found.
- `deps`: `rpptx-layout` consumes `rpptx-chart` for native chart projection.
  The `rpptx` development dependency on `miniz_oxide` bounds the PNG preview
  admission tests and production example.
- `surface`: the new chart-resource types, immutable OOXML projections, frozen
  group variant, and shared-font lowering entry point are all consumed by the
  approved presentation rendering path. No unrelated public API was found.
- `fallback and preservation`: chart and AlternateContent raw bytes remain the
  serialization source. Relationship scope, bounded preview admission, cached
  image output, stable diagnostics, and labelled fallback remain intact.

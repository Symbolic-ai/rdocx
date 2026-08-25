# F-200, Vertical and bidirectional text

**Status**: approved
**Sprint**: S58
**Size**: M
**Depends on**: F-199

## Problem

Word paragraph properties do not type `w:bidi`, and run properties leave
`w:rtl` opaque at `crates/rdocx-oxml/src/properties.rs:89` and
`crates/rdocx-oxml/src/properties.rs:1576`. Word layout has no paragraph or run
direction in its line-breaking input at
`crates/rdocx-layout/src/engine.rs:4194`. DrawingML paragraph `rtl` is retained
as a raw attribute and ignored by resolution at
`crates/oxml-drawing/src/text/paragraph.rs:834` and
`crates/rpptx-layout/src/context.rs:1692`. PowerPoint then emits logical runs
left to right at `crates/rpptx-render/src/text.rs:705`.

The vertical portion of the story conflicts with the current specification.
PowerPoint deliberately transposes and quarter-turns the complete text group
at `crates/rpptx-render/src/text.rs:967`, which
`docs/hld/08-rendering-spec.md` documents as an approximation. Upright East
Asian vertical text, Mongolian layout, and WordArt stacking remain explicit v1
non-goals in `docs/hld/02-scope-and-non-goals.md`. Exact vertical layout would
therefore be a scope change substantially larger than this M-sized story.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, vertical text non-goals.
- `docs/hld/03-architecture.md`, shared shaping and source-span ownership.
- `docs/hld/05-drawingml-model.md`, text property hierarchy and raw reconciliation.
- `docs/hld/08-rendering-spec.md`, vertical approximation and exact source mapping.
- `docs/hld/10-bindings-spec.md`, public layout type versioning.
- `docs/hld/12-testing-strategy.md`, deterministic golden and corpus evidence.
- `docs/hld/14-development-backlog.md`, "F-200, Vertical and bidirectional text".

## Approach

Type Word paragraph `bidi`, Word run `rtl`, and DrawingML paragraph direction
in their existing schema positions without losing unknown XML. Propagate one
shared `Auto`, `LeftToRight`, or `RightToLeft` paragraph direction through
existing line-layout types. Do not create a new module or forwarding layer.

Resolve the paragraph base direction, compute Unicode bidi levels for logical
text, intersect bidi spans with F-199's script and font spans, and shape each
directional span explicitly. Fit logical content into lines, then reorder spans
into visual order. Keep logical text and logical source ranges authoritative
for selection, diagnostics, PDF ToUnicode maps, and round-trip output.

Interpret start and end alignment, indents, bullets, and labels from the
paragraph base direction while keeping stored numeric measurements unchanged.
Carry visual painting order through Word, PowerPoint, PDF, raster, and SVG
consumers of F-199's cluster and offset representation.

Retain the already documented whole-group quarter-turn vertical approximations
and add regression coverage proving bidi metadata does not disturb them. Exact
East Asian or Mongolian vertical writing remains outside v1 and does not change
this story's HLD scope.

Support both paragraph-level and character-level direction in Word and
DrawingML. Accept the planned pre-1.0 additions to exhaustive public line and
glyph structures. Correct visual order requires deterministic structural
assertions over bidi levels, visual glyph runs, logical source spans, and PDF
search text, with the pinned 0.95 SSIM and 80 percent page result as the visual
cross-check.

## Rejected alternatives

- Reverse stored source text into visual order. That corrupts extraction and
  source attribution.
- Apply bidi before line fitting without line-local reordering. UAX 9 visual
  order is resolved per line.
- Keep direction only in Word or PowerPoint. The output and line-breaking
  contract is shared.
- Call the quarter-turn approximation exact vertical layout. That contradicts
  the current HLD and hides a material scope decision.
- Introduce a new direction abstraction module. Existing shared line types are
  the direct ownership point.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `word_bidi_and_run_rtl_parse_write_parse_without_raw_loss` | Typed direction and unknown siblings preserve schema order and bytes |
| round-trip | `drawingml_rtl_attribute_becomes_typed_without_reordering_unknown_content` | Direction resolution does not lose raw attributes or children |
| unit | `mixed_direction_line_uses_uax9_visual_order_without_changing_logical_text` | Visual spans follow bidi levels while logical source remains exact |
| integration | `right_to_left_paragraph_resolves_start_alignment_and_indents_from_the_right` | Direction-sensitive alignment, indents, bullets, and labels use the leading edge |
| integration | `run_level_direction_override_shapes_the_exact_source_span` | Overrides intersect F-199 script spans without corrupting clusters |
| regression | `rtl_pdf_paints_visual_order_but_maps_search_text_logically` | Painted order and searchable logical text both remain correct |
| golden | `rtl_corpus_document_matches_the_reviewed_oracle` | The pinned visual oracle meets the approved threshold and structural assertions |
| regression | existing vertical direction tests | Quarter-turn modes remain exact to their documented approximation unless scope changes |

The backlog test gate is **golden**: a bidirectional document renders with the
correct visual order.

## HLD impact

- `docs/hld/03-architecture.md`, logical text, directional shaping, visual ordering, and source ownership.
- `docs/hld/05-drawingml-model.md`, typed DrawingML direction and raw reconciliation.
- `docs/hld/08-rendering-spec.md`, UAX 9, direction-sensitive layout, and logical versus visual order.
- `docs/hld/10-bindings-spec.md`, public layout-type source compatibility if exhaustive structs change.
- `docs/hld/12-testing-strategy.md`, RTL corpus and structural oracle assertions.

## Risk routing

- Parser and serializer. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`, then prove schema order, namespace
  tolerance, fixed prefixes, and raw preservation.
- Layout and shaping. Use deterministic baselines and require logical-source
  invariants in addition to pixel evidence.
- Public API of published crates. State any source impact, run package
  dry-runs, and enforce archive-size limits.
- Crate dependency graph. Keep any bidi or vertical-orientation dependency in
  the format-neutral layout layer.
- External oracle comparison. Pin tool, DPI, and corpus identities, then retain
  complete page evidence and the exact threshold.

## Hash harness

Expected unchanged at 49 of 49. If an existing fixture intentionally carries
direction metadata, identify its exact expected delta before implementation.
Do not re-record unrelated output or absorb F-198's delta.

## Implementation checklist

- [ ] Type Word and DrawingML direction properties without raw loss.
- [ ] Propagate shared paragraph and run direction through existing layout types.
- [ ] Intersect bidi levels with F-199 script, font, cluster, and offset spans.
- [ ] Fit logical content and reorder each final line visually.
- [ ] Resolve alignment, indents, bullets, and labels from base direction.
- [ ] Paint visual order while preserving logical extraction and source spans.
- [ ] Add structural, round-trip, integration, golden, and backend regressions.
- [ ] Retain the documented quarter-turn vertical approximations.
- [ ] Run all risk riders and update exactly the approved HLD files.

## Open questions

None. This story implements paragraph-level and run-level bidi, retains the
documented quarter-turn vertical approximations, uses structural and pinned
pixel evidence, and permits the stated pre-1.0 public structure additions.

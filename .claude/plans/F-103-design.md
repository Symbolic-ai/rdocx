# F-103, Hyperlinks, fields and diagnostics

**Status**: completed
**Sprint**: S25
**Size**: M
**Depends on**: F-092

## Problem

DrawingML already retains direct run hyperlinks, and the shared line and PDF
layers already support URI link annotations. The resolver drops the direct
hyperlink when it freezes `ResolvedRunStyle` at
`crates/rpptx-layout/src/lib.rs:306`, and text shaping hardcodes both
`hyperlink_url` and `field_kind` to `None` at
`crates/rpptx-render/src/text.rs:147`.

Fields retain stored text and `field_type`, but the renderer shapes them exactly
like ordinary text. The current one-based page number is available when the
page frame is built at `crates/rpptx-render/src/lib.rs:208`, yet never reaches
field shaping. Diagnostics already cross from `ResolvedSlide` to
`LayoutResult`, so missing or unsupported link actions need no new error
surface.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Rendering" and the diagnostic
  visibility requirement.
- `docs/hld/05-drawingml-model.md`, "Text body".
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract" and text
  property non-inheritance.
- `docs/hld/08-rendering-spec.md`, "The recursion hazard" and "Text in a
  shape".
- `docs/hld/14-development-backlog.md`, "F-103, Hyperlinks, fields and
  diagnostics".

## Approach

Add a concrete source-scoped hyperlink target map beside `ScopedMediaIds` in
`rpptx-layout`. Thread each flattened shape's source through text resolution,
read only the run or field's direct `hlinkClick`, and resolve its relationship
identifier in that source scope. Freeze the URI in `ResolvedRunStyle`. Missing
relationships and unsupported action-only links keep their text visible, omit
the annotation, and append a stable diagnostic.

Project hyperlink relationships from the existing render relationship scopes
without adding an `oxml-opc` dependency. Copy the resolved URI into
`TextSegment`, letting the existing segment emitter create one transformed
`LinkAnnotation` for each laid-out run fragment.

Pass the one-based page number through shape and text lowering. Substitute it
before shaping for `field_type="slidenum"` and for an untyped field in an
effective slide-number placeholder, then set `FieldKind::Page`. Other fields
retain their stored display text and no field kind.

## Rejected alternatives

- Inherit hyperlink actions through the text cascade. Actions are direct run
  state and are explicitly non-inherited.
- Add raw relationship objects to the frozen resolver contract. A concrete
  scoped URI map is smaller and keeps package types upstream.
- Substitute the slide number after shaping. Glyphs, advances, wrapping, and
  link rectangles would describe the wrong text.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `slide_number_field_renders_current_page_and_hyperlink_emits_annotation` | The backlog gate on slide two with `FieldKind::Page` and the resolved URI annotation |
| regression | `same_relationship_id_resolves_hyperlink_in_its_shape_source_scope` | Slide, layout, and master `rId7` targets never cross scopes |
| regression | `missing_hyperlink_relationship_keeps_text_and_records_diagnostic` | Broken links remain visible and diagnosable |
| regression | `untyped_slide_number_placeholder_uses_the_current_page_number` | Effective `sldNum` placeholders normalize untyped fields |
| regression | `grouped_hyperlink_annotation_keeps_transformed_run_bounds` | Nested shape transforms apply to annotation rectangles |

The test gate is a slide-number field renders the correct number and a
hyperlink emits an annotation.

## HLD impact

- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`

## Risk routing

- Layout, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Use deterministic fonts for glyph,
  annotation, and raster evidence, run `cargo test -p rpptx-layout` and
  `cargo test -p rpptx-render`, and record no incidental baseline.

No parser, dependency-graph, published-surface, or new-file trigger is planned.

## Hash harness

Expected to be unchanged. Presentation hyperlinks and fields do not feed the
Word output harness.

## Implementation checklist

- [x] Resolve direct hyperlink relationships in the producing shape's scope.
- [x] Preserve visible text and add diagnostics for missing or unsupported
  link actions.
- [x] Emit transformed URI annotations through the existing segment path.
- [x] Substitute one-based slide numbers before shaping and set
  `FieldKind::Page`.
- [x] Cover typed and untyped slide-number fields plus grouped link bounds.

## Open questions

None. F-103 renders external URI run hyperlinks. Internal slide jumps and
action-only links remain visible with diagnostics because expanding the PDF
action contract is outside this M-sized story.

# F-097, Backgrounds

**Status**: completed
**Sprint**: S23
**Size**: S
**Depends on**: F-086

## Problem

`ResolveCtx::effective_background` selects slide, layout, master, or theme
fallback in the correct order, but `CommonSlideData` keeps explicit `p:bg`
content only as raw bytes. `ResolveCtx::resolve_slide` therefore emits a
diagnostic instead of concrete paint for every modelled slide, layout, or
master background. Only the theme fallback reaches `ResolvedSlide::background`.

The shared page frame and both backends already support solid and gradient
background paint. The missing work is typed, preserving `p:bg` resolution and
the final renderer assignment.

## Spec reference

- `docs/hld/05-drawingml-model.md`, the fill module and colour-resolution
  contract.
- `docs/hld/06-presentationml-model.md`, "The shape tree", "Parse only what we
  render", and "Raw-XML preservation".
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract" and
  "Draw order".
- `docs/hld/08-rendering-spec.md`, "Extending PositionedElement" and "The
  rasteriser".
- `docs/hld/14-development-backlog.md`, "F-097, Backgrounds".

## Approach

Type a read-only rendering projection of `p:bg` inside the existing
`crates/rpptx-oxml/src/slide_parts.rs` file. Retain the complete captured
subtree as the sole serialization source, following the existing
`CT_AlternateContent` pattern. The projection distinguishes `p:bgPr` from
`p:bgRef`. It exposes the first modelled DrawingML fill for properties, or the
style index and colour choice for a reference. Parse at `p:cSld` read time with
the inherited namespace bindings, since the captured subtree may rely on root
declarations. Accept any prefix by namespace URI and preserve every attribute,
effect, unsupported sibling, and byte of the original subtree. The writer
emits that retained subtree in its existing schema slot before `p:spTree`.

Update `BackgroundContent` in `crates/rpptx-layout/src/context.rs` to borrow the
typed projection. Resolve `p:bgPr` fill through the existing concrete fill and
colour path. Resolve `p:bgRef` against the theme background fill-style list,
including `phClr` substitution through the reference colour. Reuse the
existing fill-style lookup and substitution helpers from `style.rs` rather
than duplicating shape-style logic. Retain the existing slide, layout, master,
then theme fallback precedence. Unsupported paint yields a specific diagnostic
rather than the current generic unresolved-background message.

In `crates/rpptx-render/src/lib.rs`, assign the resolved paint to
`PageFrame::background`. The shared raster and PDF backends then draw it before
every shape without a synthetic rectangle in the element list.

Keep all code and tests in existing files. This avoids a new module and keeps
background parsing next to the `p:cSld` child order it participates in.

## Rejected alternatives

- Parse raw background XML in `rpptx-layout`. PresentationML ownership and
  round-trip preservation belong in `rpptx-oxml`.
- Convert a background to the first ordinary shape. A page background is below
  every element and already has a dedicated backend-neutral field.
- Resolve only the common solid case. The story gate explicitly requires an
  inherited master gradient.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `background_projection_preserves_the_source_subtree_verbatim` | Prefix tolerance, schema position, and the raw sole serialization source hold for `p:bgPr` and `p:bgRef` |
| unit | `background_precedence_is_slide_layout_master_then_theme` | The deepest explicit background wins and absence reaches the theme fallback |
| integration | `master_gradient_background_renders_when_slide_and_layout_omit_one` | The backlog gate samples both gradient endpoints from inherited master paint |
| unit | `background_reference_resolves_phclr_through_the_master_colour_map` | Theme background style and colour substitution become concrete paint |
| regression | `background_is_not_duplicated_in_page_elements` | Paint occupies `PageFrame::background` and draw order remains stable |

The backlog test gate is
`master_gradient_background_renders_when_slide_and_layout_omit_one`.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`

## Risk routing

- Theme colour, tint, shade, and colour mapping. Read
  `docs/hld/05-drawingml-model.md`. Use only the spec-correct
  `oxml-drawing` colour resolver and do not change the legacy Word tint and
  shade path.
- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add the round-trip test above for
  prefix tolerance, schema position, and byte-for-byte preservation of the
  complete retained subtree. The rendering projection is never a second
  writer source.
- Layout and rendering. Read `docs/hld/08-rendering-spec.md`. Rasterise the
  in-memory background fixture in deterministic font mode and record no
  system-font baseline.

## Hash harness

Expected to be unchanged. Explicit PowerPoint background resolution is not
connected to the released Word renderer.

## Implementation checklist

- [x] Add preserving `p:bgPr` and `p:bgRef` rendering projections in existing slide-part code.
- [x] Resolve explicit slide, layout, and master background paint plus theme fallback.
- [x] Resolve background style references and `phClr` through the effective colour map.
- [x] Assign resolved paint to the page background before shape elements.
- [x] Prove round-trip preservation, precedence, inherited gradient pixels, and no duplication.
- [x] Reconcile the PresentationML and resolver HLD sections with current behavior.

## Open questions

None. The existing precedence, theme-fill, and page-background contracts fix
the implementation.

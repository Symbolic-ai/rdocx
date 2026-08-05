# F-098b, Paragraph inline resolution

**Status**: completed
**Sprint**: S24
**Size**: L
**Depends on**: F-098a

## Problem

Resolved paragraphs and runs already cross the frozen boundary at
`crates/rpptx-layout/src/lib.rs:235`, but no PowerPoint path turns them into
shaped inline items. `oxml-layout` exposes font resolution, metrics, shaping,
explicit breaks, and text segments at `crates/oxml-layout/src/lib.rs:13`, while
`rpptx-render` currently emits no glyphs.

## Spec reference

- `docs/hld/07-inheritance-and-resolution.md`, "The nine-level list style" and
  the frozen `ResolvedSlide` contract.
- `docs/hld/08-rendering-spec.md`, "Text in a shape".
- `docs/hld/14-development-backlog.md`, "F-098b, Paragraph inline resolution".

## Approach

In the approved private text module, convert each `ResolvedTextRun` into
`oxml_layout::InlineItem` values. Resolve the best concrete typeface, default an
otherwise unresolved size to 18 points and paint to black, preserve bold,
italic, underline, strike, and baseline information, then shape with
`FontManager::resolve_font_for_text`, `metrics`, and `shape_text`. Explicit
breaks become `InlineItem::LineBreak`. Fields retain their stored display text,
while field substitution remains F-103.

Add a private layout path that reuses one font manager across a presentation,
loads `RenderInput::fonts`, and collects the used font data into
`LayoutResult`. Keep the public `RenderInput` and `layout_presentation`
signatures unchanged. Add a concrete text-layout error variant rather than
erasing shaping failures or panicking.

## Rejected alternatives

- Shape in `rpptx-layout`. The frozen resolver owns inheritance, while font
  selection and glyph output belong to rendering.
- Add another paragraph or run model. The owned resolved contract already
  contains every required value.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `resolved_runs_emit_glyph_items_with_concrete_style_and_break_boundaries` | The child backlog gate across text, field, and explicit break runs |
| regression | `missing_run_size_and_fill_use_visible_renderer_defaults` | Unresolved size becomes 18 points and missing paint becomes black |
| regression | `presentation_layout_collects_every_font_used_by_shape_text` | Glyph font IDs have matching embedded font data |
| regression | `text_shaping_failures_return_a_render_error_without_panicking` | Font and shaping failures remain explicit |

The test gate is resolved text runs emit glyph items with the expected font
size, colour, style, and explicit break boundaries.

## HLD impact

None. The frozen boundary and shaping ownership are already specified.

## Risk routing

- Layout, pagination, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Every glyph and raster test constructs a
  deterministic font manager, focused checks include `cargo test -p
  rpptx-render`, and no baseline is re-recorded.

## Hash harness

Expected to be unchanged. This adds glyphs only to the unpublished PowerPoint
renderer.

## Implementation checklist

- [x] Convert resolved text and fields to shaped inline text segments.
- [x] Preserve explicit line breaks and concrete run styling.
- [x] Reuse one font manager per presentation and collect used font data.
- [x] Return explicit text-layout errors.
- [x] Add deterministic structural and shaping regressions.

## Open questions

None. The 18 point, black, and generic sans-serif fallbacks are approved for
missing resolved values.

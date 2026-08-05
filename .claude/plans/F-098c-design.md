# F-098c, Line stacking

**Status**: completed
**Sprint**: S24
**Size**: M
**Depends on**: F-098b

## Problem

F-098b produces shaped inline items, but a slide renderer still needs to break
them against the shape content width and turn successive paragraphs into one
measured block. `oxml-layout::break_into_lines` already implements greedy
wrapping, explicit breaks, indents, and line-height rules at
`crates/oxml-layout/src/line.rs:212`, so the PowerPoint renderer must compose it
rather than fork it.

## Spec reference

- `docs/hld/07-inheritance-and-resolution.md`, "The nine-level list style".
- `docs/hld/08-rendering-spec.md`, "Text in a shape".
- `docs/hld/14-development-backlog.md`, "F-098c, Line stacking".

## Approach

Run each shaped paragraph through `break_into_lines` using the content width,
resolved left and right margins, first-line or hanging indent, line spacing,
and body wrap flag. Convert point spacing directly. Convert percentage spacing
against the effective first-run font size. Stack lines and paragraphs from the
content-box top, preserving a measured block height and maximum occupied width
for anchoring and later autofit.

Apply left, centre, right, justified, and distributed horizontal paragraph
alignment while lowering both `LineItem::Text` and the future
`LineItem::Marker` through one glyph-emission path. Emit underline and strike
lines from the same concrete segment metrics. Shape text remains above the
shape path in draw order and is never clipped to the content box.

## Rejected alternatives

- Copy the line breaker into `rpptx-render`. That creates a second source of
  wrapping behavior at the exact shared format boundary.
- Emit glyph runs before measuring the complete paragraph. Later anchoring and
  autofit both require stable block metrics.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `paragraphs_stack_wrapped_lines_with_spacing_and_alignment` | The child backlog gate across two paragraphs and multiple lines |
| regression | `wrap_none_breaks_only_at_explicit_line_breaks` | Width overflow remains on one line until a stored break |
| unit | `point_and_percentage_paragraph_spacing_produce_computed_baselines` | Space before and after use the selected units exactly |
| regression | `text_and_marker_items_share_one_baseline_emitter` | F-099 can add a marker without a second positioning path |

The test gate is wrapped paragraphs stack at hand-computed baselines while
`wrap="none"` breaks only at explicit line breaks.

## HLD impact

None. The paragraph order and line-stacking algorithm are already specified.

## Risk routing

- Layout, pagination, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Every exact baseline and raster test uses
  deterministic font mode, focused checks include `cargo test -p
  rpptx-render`, and the hash baseline is never updated incidentally.

## Hash harness

Expected to be unchanged. The 28 entries do not consume the PowerPoint
renderer.

## Implementation checklist

- [x] Map resolved margins, indents, spacing, alignment, and wrap to line-break parameters.
- [x] Break and stack every paragraph in shape-local coordinates.
- [x] Emit glyph, underline, strike, and future marker items through one path.
- [x] Preserve overflow without clipping.
- [x] Add deterministic baseline and alignment regressions.

## Open questions

None. Character tracking increases shaped advances, and percentage paragraph
spacing uses the effective first-run font size.

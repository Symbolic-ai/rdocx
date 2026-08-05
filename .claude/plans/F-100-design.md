# F-100, Autofit

**Status**: completed
**Sprint**: S24
**Size**: M
**Depends on**: F-098d

## Problem

The resolver already preserves `noAutofit`, `spAutoFit`, and stored or bare
`normAutofit` at `crates/rpptx-layout/src/lib.rs:224`, including concrete stored
percentages from `crates/rpptx-layout/src/context.rs:1530`. The renderer does not
yet scale runs or line spacing, measure a bare normal-autofit candidate, or
distinguish visible overflow from fitting behavior.

## Spec reference

- `docs/hld/07-inheritance-and-resolution.md`, the frozen `ResolvedSlide`
  contract and "Body properties".
- `docs/hld/08-rendering-spec.md`, "Autofit".
- `docs/hld/14-development-backlog.md`, "F-100, Autofit".

## Approach

Apply `ResolvedAutofit::None` and `Shape` at scale 1.0. Neither adds a clip, and
shape autofit trusts the already stored extent. A stored normal autofit applies
its exact font scale and line-spacing reduction on the first layout pass. Scale
every effective run and bullet size. Reduce only the extra baseline-to-baseline
leading, never the font's ascent plus descent.

A bare normal autofit tries the quantised sequence 100, 97.5, 95 through 25
percent, at most 31 candidates, and accepts the first block whose measured
width and height fit the content box. Cache each shaped `(font, text, size,
style)` result within the one layout call so repeated candidates do not repeat
font discovery. If the 25 percent candidate still overflows, keep it visible
without clipping.

Keep the helpers private and concrete in the shared text module. Do not change
the frozen resolver contract or add a continuous binary search.

## Rejected alternatives

- Recompute stored PowerPoint answers. Applying the authoring application's
  stored scale is cheaper and more faithful.
- Binary-search a continuous font scale. PowerPoint stores a quantised 2.5
  percent ladder.
- Clip at the smallest scale. Visible overflow is the specified fallback.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `stored_font_scale_renders_at_exactly_sixty_two_point_five_percent` | The backlog gate turns a 20 point run into an exact 12.5 point glyph run |
| unit | `stored_line_spacing_reduction_reduces_only_extra_leading` | Natural glyph height remains the hard floor |
| regression | `shape_autofit_trusts_the_stored_extent` | Shape mode performs one unscaled layout and no resize |
| regression | `no_autofit_overflows_without_a_clip` | Glyphs remain visible beyond the content box |
| unit | `bare_normal_autofit_uses_quantised_two_point_five_percent_steps` | The first fitting candidate is one exact ladder value |
| regression | `bare_normal_autofit_keeps_the_twenty_five_percent_floor_visible` | A still-oversized block remains unclipped at the floor |

The test gate is a stored `fontScale` of 62500 renders at exactly 62.5 percent.

## HLD impact

- `docs/hld/08-rendering-spec.md`

Clarify the 25 percent ladder floor, line-spacing reduction floor, and visible
overflow behavior when the smallest candidate still does not fit.

## Risk routing

- Layout, pagination, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. All structural glyph and raster evidence uses
  deterministic fonts, focused checks include `cargo test -p rpptx-render`, and
  no baseline is recorded incidentally.

## Hash harness

Expected to be unchanged. Autofit affects the unpublished PowerPoint renderer
only.

## Implementation checklist

- [x] Apply stored font scale and line-spacing reduction verbatim.
- [x] Preserve unscaled no-autofit and shape-autofit behavior without clipping.
- [x] Implement the 31-value quantised ladder and 25 percent visible floor.
- [x] Cache shaping work within one autofit calculation.
- [x] Add deterministic scale, spacing, ladder, and overflow regressions.

## Open questions

None. The 25 percent floor, visible overflow, and natural glyph-height floor
for line-spacing reduction are approved.

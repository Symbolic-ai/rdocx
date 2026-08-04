# F-101, Vertical text

**Status**: approved
**Sprint**: S24
**Size**: S
**Depends on**: F-098d

## Problem

The frozen text body retains all DrawingML direction values at
`crates/rpptx-layout/src/lib.rs:212`, but the renderer has only horizontal shape
text. Vertical layout must reuse the completed horizontal pipeline in a
transposed box, while unsupported upright East Asian stacking must remain
visible and produce a stable approximation diagnostic.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Explicitly not in v1", the `eaVert`
  row.
- `docs/hld/07-inheritance-and-resolution.md`, the frozen `ResolvedSlide`
  contract.
- `docs/hld/08-rendering-spec.md`, "Text in a shape", the "Vertical text"
  paragraph.
- `docs/hld/14-development-backlog.md`, "F-101, Vertical text".

## Approach

Lay out `vert` horizontally in a transposed content box, then wrap the text
children in a same-centre 90 degree group. Use the opposite quarter-turn for
`vert270`. Degrade `eaVert` to the `vert` path and record
`east Asian vertical text rendered as rotated vertical text` in the resolved
slide diagnostics before rendering.

Degrade Mongolian vertical and both WordArt vertical variants to the nearest
quarter-turn path with a direction-specific diagnostic rather than losing the
text. Keep the private direction-to-transform helper concrete, compose it
inside the existing shape transform, and add no clip or public type.

## Rejected alternatives

- Implement upright CJK stacking. It is an explicit v1 non-goal.
- Rotate each glyph independently. The specification requires one rotated group
  around an ordinary horizontal layout.
- Drop unsupported vertical variants. Every approximation must stay visible and
  diagnosable.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `vertical_text_uses_a_transposed_box_and_rotated_group` | The backlog gate and hand-computed group transform |
| regression | `east_asian_vertical_text_degrades_to_rotated_with_a_diagnostic` | `eaVert` remains visible and records the stable message |
| unit | `vertical_270_uses_the_opposite_quarter_turn` | The two principal vertical directions rotate oppositely around one centre |
| regression | `other_vertical_variants_remain_visible_with_diagnostics` | Mongolian and WordArt variants use the approved visible fallbacks |

The test gate is vertical text renders rotated and records a diagnostic for
`eaVert`.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/08-rendering-spec.md`

Record the exact direction mapping and visible diagnostics for the vertical
variants whose v1 fallback is not currently stated.

## Risk routing

- Layout, pagination, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Transform, glyph, and raster assertions use
  deterministic fonts, focused checks include `cargo test -p rpptx-render`, and
  no system-font baseline is recorded.

## Hash harness

Expected to be unchanged. Vertical slide text does not feed the Word hash
harness.

## Implementation checklist

- [ ] Transpose the content box and reuse the horizontal layout pipeline.
- [ ] Apply opposite same-centre quarter turns for vertical and vertical-270.
- [ ] Degrade East Asian vertical text visibly with the stable diagnostic.
- [ ] Preserve other vertical variants through documented visible fallbacks.
- [ ] Add deterministic transform, diagnostic, and visibility regressions.

## Open questions

None. The quarter-turn mapping, stable `eaVert` message, and visible diagnostic
fallbacks for Mongolian and WordArt vertical variants are approved.

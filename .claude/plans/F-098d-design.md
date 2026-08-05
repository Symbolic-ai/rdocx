# F-098d, Text anchoring

**Status**: completed
**Sprint**: S24
**Size**: S
**Depends on**: F-098c

## Problem

F-098c leaves one measured text block at the top of the content box. The frozen
body contract also carries top, centre, bottom, justified, and distributed
vertical anchors at `crates/rpptx-layout/src/lib.rs:202`, and the renderer must
apply them after line measurement so bottom-centred text reaches the baseline
specified by the parent gate.

## Spec reference

- `docs/hld/07-inheritance-and-resolution.md`, "Body properties".
- `docs/hld/08-rendering-spec.md`, "Text in a shape".
- `docs/hld/14-development-backlog.md`, "F-098d, Text anchoring".

## Approach

Measure the complete unanchored text stack, then translate its glyph and line
elements inside the content box. Top uses zero spare-height offset, centre uses
half, and bottom uses all. Preserve negative offsets when the text overflows and
do not add a clip.

For positive spare height, justified anchor distributes it between line boxes.
Distributed anchor uses equal line gaps plus half a gap before the first and
after the last line. With one line, both forms use the centre policy. Never
distribute negative spare height. Apply horizontal paragraph alignment before
the vertical translation, then append the resulting text children after the
shape path inside the existing shape transform group.

## Rejected alternatives

- Anchor each paragraph independently. DrawingML anchors the complete text body.
- Clip overflowing text to the content box. The rendering specification
  explicitly prefers visible overflow to truncation.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `bottom_center_text_in_an_inset_box_lands_at_the_computed_baseline` | The child and parent backlog gate with independent baseline arithmetic |
| unit | `top_center_and_bottom_anchors_use_zero_half_and_full_spare_height` | Three ordinary anchors translate the same block exactly |
| regression | `overflowing_anchored_text_remains_visible_without_a_clip` | Negative spare height does not hide glyphs |
| unit | `justified_and_distributed_anchors_allocate_positive_spare_height` | Inter-line and edge gaps follow the approved policy |

The test gate is text anchored bottom-centre in an inset box lands at the
computed baseline.

## HLD impact

- `docs/hld/08-rendering-spec.md`

Record the exact justified and distributed vertical-anchor policy because the
current section names anchoring without defining these two forms.

## Risk routing

- Layout, pagination, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. The baseline gate and every optional raster
  assertion use deterministic fonts, and no system-font baseline is recorded.

## Hash harness

Expected to be unchanged. The PowerPoint renderer is not used by the existing
Word output corpus.

## Implementation checklist

- [x] Measure the complete unanchored block and its spare height.
- [x] Apply top, centre, and bottom offsets without clipping overflow.
- [x] Apply the approved justified and distributed gap policies.
- [x] Attach text after the shape path inside the existing transform group.
- [x] Prove the bottom-centre baseline with deterministic independent arithmetic.

## Open questions

None. The stated justified and distributed vertical-anchor policies are
approved.

# F-098, Shape text layout

**Status**: approved
**Sprint**: S24
**Size**: XL
**Depends on**: F-083, F-030

## Problem

The resolver already produces owned text bodies at
`crates/rpptx-layout/src/lib.rs:182`, but the renderer ignores every
`ResolvedContent::Text` value at `crates/rpptx-render/src/lib.rs:226`. The
original story combines content-box geometry, shaping, line stacking, and
anchoring, which is too large for one reviewable implementation.

## Spec reference

- `docs/hld/07-inheritance-and-resolution.md`, "Body properties" and "The
  nine-level list style".
- `docs/hld/08-rendering-spec.md`, "Text in a shape".
- `docs/hld/14-development-backlog.md`, "F-098, Shape text layout" and
  "F-098a" through "F-098d".

## Approach

Retain F-098 as an umbrella gate and implement it through F-098a to F-098d.
The children own the content box, paragraph inline resolution, line stacking,
then horizontal and vertical anchoring. They consume the frozen
`ResolvedTextBody` contract and the existing `oxml-layout` font, line-breaking,
and positioned-output types without adding another public text model.

The parent has no independent source diff. It closes only after all four child
plans, implementations, reviews, and delivery records close and the integrated
bottom-centre gate passes.

## Rejected alternatives

- Keep one XL implementation. It would combine four natural algorithmic seams
  and exceed the repository's review-size rule.
- Reimplement shaping and line breaking in `rpptx-render`. The shared
  `oxml-layout` machinery already exists for this format boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `all_f098_children_complete_before_the_parent_closes` | F-098a through F-098d each have durable plans, reviews, tests, and delivery records |
| regression | `bottom_centred_text_lands_at_the_computed_baseline` | The integrated child implementation satisfies the parent gate |

The test gate is text anchored bottom-centre in an inset box lands at the
computed baseline.

## HLD impact

- `docs/hld/14-development-backlog.md`

The completion update confirms the four-child split still describes the
implemented boundaries.

## Risk routing

- Layout, pagination, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Every glyph or raster assertion uses
  deterministic font mode, and no system-font baseline is recorded.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for one private `crates/rpptx-render/src/text.rs` module shared by all S24 text
  stories.

## Hash harness

Expected to be unchanged. The PowerPoint renderer is unpublished and does not
feed the 28 Word sample outputs.

## Implementation checklist

- [ ] Approve the F-098a through F-098d split in the HLD backlog, delivery backlog, sprint plan, and current sprint.
- [ ] Complete F-098a, F-098b, F-098c, and F-098d with individual evidence.
- [ ] Confirm the integrated bottom-centre baseline gate passes in deterministic font mode.
- [ ] Close the parent only after every child is complete.

## Open questions

None. The shared private `crates/rpptx-render/src/text.rs` module is approved.

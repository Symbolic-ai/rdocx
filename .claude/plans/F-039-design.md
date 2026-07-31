# F-039, Global CTM flip

**Status**: approved
**Sprint**: S08
**Size**: L
**Depends on**: F-038

## Problem

`crates/rdocx-pdf/src/writer.rs:393` converts every element from top-left
coordinates into PDF bottom-left coordinates independently. Text, lines,
rectangles, images, outlines, and annotations each carry their own Y
subtraction. That representation cannot compose nested group transforms and
makes the later shared backend easy to fix inconsistently.

The staged copy created by F-037 inherits the same behavior. F-039 must replace
the content-stream conversions with one page transform while preserving every
rendered pixel. PDF operator bytes are expected to change.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Four latent defects to fix" and "The PDF
  backend".
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate".
- `docs/hld/13-risks-and-open-questions.md`, "R2, the PDF coordinate-system
  flip".
- `docs/hld/14-development-backlog.md`, "F-039, Global CTM flip".

## Approach

At the start of each page content stream, emit one saved graphics state and
the matrix `[1 0 0 -1 0 H]`. Write lines and rectangles directly in their
top-left, Y-down layout coordinates. Use text matrix
`[1 0 0 -1 x y]` so the outer flip does not invert glyphs. Use image matrix
`[w 0 0 -h x y]` so image pixels remain upright. Restore the page graphics
state once after all content.

Keep link annotations and outline destinations outside the content stream on
their existing page-coordinate conversion. They are PDF dictionary values and
do not inherit the page content CTM.

Apply the same focused writer change to the staged `oxml-pdf` backend and the
shipped `rdocx-pdf` backend. This keeps the copy reviewable and
makes the real seven-sample golden gate exercise the high-risk change without
cutting either crate over to a new dependency. Land the behavioural change in
its own F-039 commit with the expected PDF-byte delta and zero pixel delta
stated.

## Rejected alternatives

- Keep per-element flips and special-case groups. Nested transforms would
  still compose in mixed coordinate systems.
- Flip annotations with the content CTM. Annotation rectangles are outside
  the content stream and would move to the wrong page location.
- Compare PDF bytes. The new page matrix intentionally changes the operator
  stream.
- Change only one backend copy. That would leave the shipped renderer
  untested by the required corpus gate and create immediate staging drift.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `page_content_uses_one_global_flip` | Exactly one page-level flip wraps the content stream. |
| unit | `text_and_images_cancel_the_outer_flip` | Text uses negative `d`, images use negative height, and both remain upright. |
| unit | `lines_and_rectangles_use_top_left_coordinates` | Primitive Y values are no longer subtracted per element. |
| regression | `annotations_remain_outside_the_content_transform` | Link rectangles keep their existing bottom-left dictionary coordinates. |
| golden, gate | `global_ctm_preserves_every_sample_pixel` | The whole seven-sample corpus has zero decoded pixel changes. |

The backlog test gate is that golden-PNG diffs show zero changes across the
corpus.

## HLD impact

None. The rendering specification already defines the exact matrices, the
annotation exception, the isolated commit, and the zero-pixel-change gate.

## Risk routing

- Layout and rendering. Run the deterministic golden-PNG gate at its exact
  zero-pixel threshold and the existing 28-entry hash harness. Do not update
  either baseline to make the story pass.
- Public implementation of published `rdocx-pdf`. The mirrored source change
  is behaviour-preserving at the visual contract but changes PDF bytes. It
  requires consolidated approval, a full package dry-run, and an explicit
  commit message declaring changed PDF operators with unchanged pixels.
- File copy parity. Diff the two writer implementations after the change and
  account for only crate-type differences introduced during staging.

## Hash harness

Expected to remain unchanged. Its deterministic PNGs bypass PDF operator
encoding, while the new golden gate proves the PDF raster result is unchanged.

## Implementation checklist

- [ ] Emit one saved global page CTM and one matching restore.
- [ ] Remove per-element Y subtraction for content-stream primitives.
- [ ] Install the specified text and image matrices.
- [ ] Leave annotation rectangles and outline destinations outside the CTM.
- [ ] Apply and review the same focused writer change in both approved copies.
- [ ] Pass exact unit assertions and the seven-sample zero-pixel golden gate.
- [ ] Confirm all 28 hash-harness entries remain unchanged.

## Open questions

None. Mirroring the CTM rewrite into `rdocx-pdf` is approved. No crate
dependency, version, tag, or publication change is included.

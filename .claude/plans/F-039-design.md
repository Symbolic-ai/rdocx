# F-039, Global CTM flip

**Status**: completed
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
the content-stream conversions with one page transform. PDF operator bytes are
expected to change. Poppler 26.01.0 also changes exactly four antialias pixels
on two reflected strokes, so the reviewed golden manifest must move once before
returning to exact zero-difference enforcement.

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
`[w 0 0 -h x y+h]` so image pixels remain upright and occupy the same page
rectangle after PDF matrix concatenation. Restore the page graphics state once
after all content.

Keep link annotations and outline destinations outside the content stream on
their existing page-coordinate conversion. They are PDF dictionary values and
do not inherit the page content CTM.

Apply the same focused writer change to the staged `oxml-pdf` backend and the
shipped `rdocx-pdf` backend. This keeps the copy reviewable and
makes the real seven-sample golden gate exercise the high-risk change without
cutting either crate over to a new dependency. Land the behavioural change in
its own F-039 commit with the expected PDF-byte delta and reviewed four-pixel
Poppler delta stated.

Before updating the manifest, prove that `invoice` and `quote` are the only
changed samples and that exactly four decoded RGBA pixels change. In `invoice`,
pixel `(112, 397)` changes from `fcf5f5ff` to `ffffffff` and `(112, 398)`
changes from `ffffffff` to `fcf5f5ff`. In `quote`, pixel `(112, 303)` changes
from `f4fafaff` to `ffffffff` and `(112, 304)` changes from `ffffffff` to
`f4fafaff`. The other five samples remain exact. Update the manifest once with
a non-empty F-039 review reason. After that update, normal check mode continues
to require exact equality for all seven samples. No tolerance is introduced.

## Rejected alternatives

- Keep per-element flips and special-case groups. Nested transforms would
  still compose in mixed coordinate systems.
- Flip annotations with the content CTM. Annotation rectangles are outside
  the content stream and would move to the wrong page location.
- Compare PDF bytes. The new page matrix intentionally changes the operator
  stream.
- Keep the original image translation at `y`. PDF concatenation places the
  image one height away from its original rectangle.
- Add a pixel tolerance. The reviewed four-pixel Poppler result is recorded
  once, then exact decoded-pixel comparison remains the gate.
- Change only one backend copy. That would leave the shipped renderer
  untested by the required corpus gate and create immediate staging drift.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `page_content_uses_one_global_flip` | Exactly one page-level flip wraps the content stream. |
| unit | `text_and_images_cancel_the_outer_flip` | Text uses negative `d`, images use negative height with `y+h`, and both remain upright in their original rectangles. |
| unit | `lines_and_rectangles_use_top_left_coordinates` | Primitive Y values are no longer subtracted per element. |
| regression | `annotations_remain_outside_the_content_transform` | Link rectangles keep their existing bottom-left dictionary coordinates. |
| golden | `global_ctm_matches_declared_poppler_delta` | Before the reviewed update, only the declared `invoice` and `quote` pixels change under Poppler 26.01.0, while the other five samples remain exact. |
| golden, gate | `global_ctm_preserves_reviewed_sample_pixels` | After the one reviewed manifest update, the whole seven-sample corpus has exact decoded-pixel equality with no tolerance. |
| regression | `injected_pixel_is_still_rejected` | `--inject-one-pixel proposal` still fails precisely after the manifest update. |

The revised backlog test gate is the exact declared four-pixel `invoice` and
`quote` delta before one reviewed manifest update, followed by exact zero
differences across the corpus in normal check mode.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- Layout and rendering. Prove the exact declared four-pixel `invoice` and
  `quote` delta under Poppler 26.01.0, update the golden manifest once with a
  non-empty F-039
  reason, then prove exact equality across all seven samples. Run the existing
  28-entry hash harness without changing its baseline.
- Public implementation of published `rdocx-pdf`. The mirrored source change
  changes PDF bytes and the reviewed golden digests for `invoice` and `quote`.
  It requires
  consolidated approval, a full package dry-run, the package size assertion,
  and an explicit commit message declaring both changes.
- File copy parity. Diff the two writer implementations after the change and
  account for only crate-type differences introduced during staging.

## Hash harness

Expected to remain unchanged. Its deterministic PNGs bypass PDF operator
encoding. Do not update `scripts/hash_baseline.json`.

## Implementation checklist

- [x] Emit one saved global page CTM and one matching restore.
- [x] Remove per-element Y subtraction for content-stream primitives.
- [x] Install the specified text matrix and the corrected `y+h` image matrix.
- [x] Leave annotation rectangles and outline destinations outside the CTM.
- [x] Apply and review the same focused writer change in both approved copies.
- [x] Prove the old manifest differs only at the four declared `invoice` and
      `quote` pixels.
- [x] Update the golden manifest once with a non-empty F-039 review reason.
- [x] Pass exact unit assertions and exact seven-sample check mode afterward.
- [x] Prove one injected `proposal` pixel is still rejected precisely.
- [x] Confirm all 28 hash-harness entries remain unchanged.

## Open questions

None. Mirroring the CTM rewrite into `rdocx-pdf`, correcting the image matrix
to `y+h`, and recording the exact reviewed four-pixel Poppler delta are
approved. No crate dependency, version, tag, tolerance, or publication change
is included.

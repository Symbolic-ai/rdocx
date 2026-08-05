# S24 sprint review, pass 1

**Reviewed**: sprint/s24 against 01d0b4cf6aee32adba725104a3a74041d8e4e3dd,
32 files, 4,307 changed lines, crates: rpptx-layout, rpptx-render
**Verdict**: 2 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, Distributed bullets expand the fixed hanging slot

`crates/rpptx-render/src/text.rs:667`

F-098c applies distributed glyph expansion to `LineItem::Text` and
`LineItem::Marker` through the same branch, while F-099 relies on the marker
width being the fixed hanging-indent slot. The marker glyphs are also counted
as distributable gaps at `crates/rpptx-render/src/text.rs:807`. A distributed
bulleted paragraph therefore expands the marker's effective width and moves the
first text run to the right of `marL`. Separate marker emission from text gap
distribution, count text-only gaps, and add a deterministic regression that
keeps the marker slot and first text origin fixed under distributed alignment.

### B2, Justification silently stops when shaping creates ligatures

`crates/rpptx-render/src/text.rs:783`

Word gaps are counted only when Unicode scalar count equals glyph-advance
count, and the same assumption gates expansion at
`crates/rpptx-render/src/text.rs:831`. Normal shaping can produce ligatures or
complex clusters, so a line containing an ordinary space can report zero gaps
and render left aligned even when the paragraph is justified. Decouple space
expansion from the one-character-to-one-glyph assumption and add deterministic
coverage with text whose shaped glyph count differs from its character count.

## Should-fix

### S1, The F-098d delivery record contradicts the approved anchor policy

`docs/sprints/AS_BUILT.md:3078`

The record says justified anchoring allocates spare height only between
paragraphs, while the implemented and HLD policy distributes it between line
boxes. Correct the append-only entry before sprint close so the durable record
matches `docs/hld/08-rendering-spec.md:297` and the reviewed implementation.

## Nice-to-have

None.

## Milestone gate

The M10 gate is: "the SSIM harness meets its target across the corpus."
That milestone gate does not hold yet and is not expected to hold in S24.
F-104 remains pending in `docs/sprints/BACKLOG.md:214` and owns that harness.

The S24 gate does hold except for B1 and B2 above. Evidence includes
`bottom_center_text_in_an_inset_box_lands_at_the_computed_baseline`,
`wingdings_f0b7_bullet_renders_as_a_visible_unicode_glyph`,
`stored_font_scale_renders_at_exactly_sixty_two_point_five_percent`,
`vertical_text_uses_a_transposed_box_and_rotated_group`, the required pinned
50-deck corpus, and the full verification recorded at 80395f4 with all 28 hash
entries unchanged.

## Not found

- `duplication`: no second text helper or competing layout path was added.
- `layering`: no `oxml-*` crate gained an `rdocx-*` or `rpptx-*` dependency.
- `harness`: every S24 plan and delivery entry declares an unchanged harness,
  matching the observed 28-entry result.
- `docs`: no HLD contradiction was found beyond the delivery-record error S1.
- `deps`: no dependency or manifest changed.
- `surface`: the private text module adds no public API. The new contextual
  `RenderInputError::TextLayout` variant is required by F-098b's error contract.

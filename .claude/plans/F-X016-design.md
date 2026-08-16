# F-X016, Floating drawing placement and text wrapping

**Status**: completed
**Sprint**: S41
**Size**: L
**Depends on**: F-X015

## Problem

F-X015 put the wrap mode, the four text distances and the two alignments into
the model. Nothing reads them. Two behaviours are still missing.

**Alignment is ignored.** `resolve_anchor_h` and `resolve_anchor_v` in
`crates/rdocx-layout/src/paginator.rs` resolve an offset against its
`relativeFrom` frame and never consult `align_h` or `align_v`. An anchor
positions itself either by an offset or by an alignment, and a document using
the latter carries offset zero, so a drawing aligned to the right margin renders
at the left edge of its frame.

**Text does not flow around anything.** Every drawing is placed over or under
the text. `wrapSquare` should keep text clear of the drawing's frame on the
lines it spans, and `wrapTopAndBottom` should push text below it. This is what
the external PR 2 screenshots were demonstrating and it is the last piece of
that contribution still missing.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put", for `rdocx-layout` owning the
  paginator and for line breaking living in `oxml-layout`.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" for the golden category,
  and "The hash harness" for the labelled-delta rule.
- `docs/hld/14-development-backlog.md`, "F-X016, Floating drawing placement and
  text wrapping".

## Approach

### 1. Alignment-based placement

`resolve_anchor_h` and `resolve_anchor_v` gain the frame they resolve against as
a `(start, size)` pair, then apply either the offset or the alignment:

```rust
fn frame_h(rel: ST_RelativeFromH, g: &PageGeometry, indent_left: f64) -> (f64, f64);
fn resolve_anchor_h(
    rel: ST_RelativeFromH, off: f64, align: Option<AnchorAlignH>,
    width: f64, g: &PageGeometry, indent_left: f64,
) -> f64;
```

`Left` and `Inside` sit at the frame start, `Right` and `Outside` at
`start + size - width`, `Center` at the midpoint. Mirrored for the vertical
axis. With no alignment the existing offset behaviour is unchanged, which is
what keeps every current baseline still.

Treating `Inside` as `Left` and `Outside` as `Right` is correct only for
odd-numbered pages, since the pair mean binding-side and outer-edge. Facing-page
layout is not modelled anywhere in this crate, so the approximation is stated
rather than hidden.

### 2. Per-line width reservation in line breaking

`LineBreakParams` in `oxml-layout` gains

```rust
pub line_prefix_widths: Vec<f64>,
pub line_suffix_widths: Vec<f64>,
```

indexed by line number. `break_into_lines` subtracts the pair from the line's
available width and adds the prefix to its indent. An empty vector, the default,
reproduces exactly today's behaviour, so every existing caller is unaffected.

### 3. Reflow state, carried only when a document needs it

Re-breaking a paragraph at pagination time needs its `InlineItem`s and its
`LineBreakParams`, because whether a drawing overlaps a line is only known once
the paragraph has a position on a page. `InlineItem::Text` holds the same shaped
glyphs `LayoutLine` already holds, so carrying it on every paragraph roughly
doubles layout memory for text.

It is therefore carried conditionally:

```rust
pub struct ParagraphReflow { items: Vec<InlineItem>, params: LineBreakParams }
pub reflow: Option<Box<ParagraphReflow>>,
```

`Engine::layout` scans the document once for any anchor whose wrap is not
`None`. Only if one exists does any paragraph carry reflow state. A document
without a wrapping drawing, which is nearly all of them, pays nothing.

### 4. Reflow at pagination

`Pager` tracks the wrapping drawings already placed on the page being built,
with their resolved rectangles. Before a paragraph is measured, if it or the
page carries a wrapping drawing:

- Resolve the paragraph's own drawings against its prospective top.
- For each wrapping drawing whose vertical span overlaps the paragraph's, and
  for `Square`, `Tight` and `Through`, reserve `width + distL` or
  `width + distR` on each overlapped line, on the side the drawing sits.
- For `TopAndBottom`, offset the paragraph's content below the drawing's bottom
  edge plus `distB`.
- Re-break with the reserves and repeat once, because the paragraph's height
  changes which lines the drawing overlaps.

Two passes rather than a fixpoint. The second pass settles the common case and
a bounded loop cannot fail to terminate.

`Tight` and `Through` are approximated as `Square`. Wrapping to an outline needs
the `wp:wrapPolygon` the model does not carry, and reserving the frame is a
strictly better approximation than not wrapping at all. Stated here so the
approximation is a decision.

**Corrected during implementation.** The plan originally limited wrapping to
drawings anchored to the current paragraph or to ones already placed, on the
grounds that a later paragraph's position is not yet known. Rendering
`sample1.docx` showed that assumption failing on the contribution's own headline
page: its right-hand arrow is anchored to paragraph 282 while the text that must
flow around it is in paragraph 280, so the right arrow kept printing over the
text while the left one wrapped correctly.

A bounded look-ahead now collects wrapping drawings from following blocks, but
only those whose vertical frame is the page or a margin. Those have a position
that does not depend on where their own paragraph lands, so there is no
circularity to resolve. A drawing anchored **paragraph-relative** in a later
block is still left to the pass that places it, because its position genuinely
needs its own paragraph placed first. That residual case is the honest
limitation, and it is much narrower than the original one.

## Rejected alternatives

- **Carry reflow state on every paragraph unconditionally.** Doubles layout
  memory for text in every document to serve the few that wrap.
- **Reconstruct inline items from the laid-out lines.** Lossy: a line boundary
  does not record whether it came from a forced break or from wrapping, so
  re-breaking would silently drop forced breaks.
- **Reserve width at layout time rather than at pagination.** Whether a drawing
  overlaps a line depends on where the paragraph lands, which layout does not
  know.
- **Iterate reflow to a fixpoint.** Unbounded work for a case two passes settle.
- **Scan alpha channels to wrap tightly**, as the external PR did. It decoded
  every image and walked every pixel inside layout with no cache, and it applied
  outline extents to `wrapSquare`, which reserves the frame. Wrong quantity,
  large cost.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden | `text_wraps_beside_a_left_aligned_square_drawing` | Lines the drawing spans start to its right by `width + distR`, and lines below it start at the margin |
| golden | `text_wraps_beside_a_right_aligned_square_drawing` | Lines the drawing spans end before it, and their available width is reduced by `width + distL` |
| golden | `a_top_and_bottom_drawing_pushes_text_below_it` | The first text line sits below the drawing's bottom edge plus `distB` |
| regression | `a_wrap_none_drawing_leaves_text_untouched` | A document with a `wrapNone` drawing lays out identically to one with no drawing at all |
| unit | `an_aligned_anchor_resolves_against_its_frame` | Each alignment in each axis resolves to the frame start, midpoint and end for every `relativeFrom` |
| unit | `an_anchor_without_an_alignment_still_uses_its_offset` | Offset behaviour is unchanged, which is what keeps existing baselines still |
| unit | `a_document_without_wrapping_carries_no_reflow_state` | No paragraph holds reflow state unless the document has a wrapping drawing |

**Test gate**, from the backlog: the three golden tests.

## HLD impact

- `docs/hld/03-architecture.md`, the note placement paragraph's neighbourhood,
  extended to say the paginator reflows a paragraph around wrapping drawings and
  why the reflow input is carried conditionally.

## Risk routing

Matched row: **Layout, pagination, line breaking, text shaping**.

- Read `docs/hld/08-rendering-spec.md`. Deterministic font mode for any baseline
  recorded, and any re-record is deliberate and separately committed.
- This is the sprint's largest behavioural change. The mitigation is that every
  new path is gated on a wrap mode other than `None`, so a document without a
  wrapping drawing cannot reach any of it. The harness proves that.

The parser row does not match: F-X015 did the parsing and this story only reads
the model.

## Hash harness

**Unchanged, 28 of 28.** No corpus document contains a floating drawing with a
wrap mode other than `None`, and every new code path is gated on one, so the
flat result is what proves the gating holds.

Evidence for the new behaviour is the golden set, since the harness cannot see
it.

## Implementation checklist

- [x] `line_prefix_widths` and `line_suffix_widths` in `LineBreakParams`, with
      `break_into_lines` honouring them
- [x] Alignment-aware `resolve_anchor_h` and `resolve_anchor_v`
- [x] `ParagraphReflow`, carried only when the document has a wrapping drawing
- [x] Wrapping drawings tracked per page in the `Pager`
- [x] Reflow before measuring, two passes, square and top-and-bottom
- [x] Tests, including the wrap-none identity regression
- [x] Confirm the harness is unchanged
- [x] `/microscope F-X016 --working`
- [x] `/verify`

## Open questions

None. The two judgement calls, approximating the outline wraps as `Square` and
limiting wrapping to drawings already placed, are recorded above as stated
decisions.

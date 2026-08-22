# F-X041, Remove duplicated glyphs at break opportunities

**Status**: completed
**Sprint**: S52
**Size**: M
**Depends on**: F-030, F-104, F-X037

## Problem

The Word conversion layer calls `unicode_linebreak` and slices an already
shaped `TextSegment` in `crates/rdocx-layout/src/convert.rs:94`. When glyph
count differs from scalar count, `slice_text_segment` estimates glyph bounds
from byte fractions at `crates/rdocx-layout/src/convert.rs:127`. Both Word text
paths call this conversion at `crates/rdocx-layout/src/engine.rs:1703` and
`crates/rdocx-layout/src/engine.rs:1896`.

The shared line breaker already owns Unicode break discovery and exact
subsegment reshaping at `crates/oxml-layout/src/line.rs:552` and
`crates/oxml-layout/src/line.rs:621`. Pre-slicing in Word duplicates that
ownership and can put a boundary glyph into adjacent chunks, especially for
ligatures and other non-bijective shaping.

## Spec reference

- Unicode Standard Annex 14, "Unicode Line Breaking Algorithm".
- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/08-rendering-spec.md`, "Text in a shape" and "Performance".
- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and "The hash
  harness".

## Approach

Remove `convert::text_segments` and its approximate glyph slicing. Both Word
projection paths push one shaped `InlineItem::Text` for each formatting and
provenance span. The shared `break_into_lines` path remains the single owner of
UAX 14 segmentation and reshapes each exact text slice through the same font
manager, preserving formatting, spacing, source span subdivision, hyperlink,
field, and note metadata.

Add Word-level regressions that collect positioned glyph runs and source spans
from `PageFrame`, then verify exact text concatenation, contiguous provenance,
and glyph sequences shaped independently for each emitted chunk. Render the
same cases through deterministic PDF and raster paths. Keep the existing
shared line-break tests as the lower-level invariant.

## Rejected alternatives

- Improving byte-fraction glyph estimates cannot make ligatures or combining
  clusters bijective.
- Teaching both layers to coordinate break positions preserves two owners and
  allows them to drift again.
- Deduplicating positioned glyphs after layout would hide the bug while
  corrupting widths and provenance.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `word_projection_leaves_break_segmentation_to_shared_layout` | Word creates one text item per formatting span and no approximate glyph slice remains. |
| golden | `break_opportunities_emit_every_scalar_and_glyph_once` | Spaces, hyphens, ligatures, combining text, CJK, and taken and untaken breaks concatenate to exact source text with contiguous provenance and exact reshaped glyphs. |
| regression | `reported_words_do_not_duplicate_boundary_glyphs` | `ttf-parser`, doubled spaces, `financial`, and `allocated` have no duplicate or missing glyph in `PageFrame`. |
| golden | `fixed_break_runs_match_pdf_and_raster_backends` | Deterministic PDF text and raster output consume the corrected shared page runs without backend-specific repair. |

The test gate is **golden**. Deterministic layout of spaces, hyphens,
ligatures, combining text, CJK, and untaken versus taken break opportunities
emits each source scalar and shaped glyph exactly once with contiguous
provenance. The reported `ttf-parser`, doubled-space, `financial`, and
`allocated` cases are covered through `PageFrame` and both built-in backends.
The intentional sample hash delta is isolated, explained, and reviewed.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`

## Risk routing

- Layout, line breaking, and text shaping: re-read
  `docs/hld/08-rendering-spec.md`. Run every golden and baseline in
  deterministic font mode. Record the exact affected sample hashes and never
  accept a system-font baseline.

## Hash harness

Expected intentional delta:

- `page1.png` changes for `contract`, `invoice`, `letter`, `quote`, and
  `report`.
- `pdf/pages`, `pdf/resources`, and `pdf/bytes` change for all seven samples:
  `contract`, `feature_showcase`, `invoice`, `letter`, `proposal`, `quote`, and
  `report`.
- Every `word/document.xml`, `word/styles.xml`, and `word/numbering.xml` hash
  remains unchanged.

Correcting each emitted run's glyph vector changes PDF page streams and the
embedded font subsets. Those structural PDF fingerprints can change even when
the corrected glyphs paint the same page-one raster pixels, which is why
`feature_showcase` and `proposal` have PDF deltas without PNG deltas. Capture
and review this exact sample set before updating the baseline, and keep the
behavior change isolated in its own labelled feature commit.

## Implementation checklist

- [x] Delete Word-owned UAX 14 segmentation and approximate glyph slicing.
- [x] Feed complete shaped formatting spans into the shared line breaker.
- [x] Add exact scalar, glyph, provenance, PDF, and raster regressions.
- [x] Quantify and review the deterministic sample hash delta.
- [x] Run focused Word and shared layout tests plus golden and hash gates.

## Open questions

None. The source already shows the duplicate ownership and the shared line
breaker already implements exact subsegment reshaping.

# F-X047, Attribute empty Word paragraphs

**Status**: approved
**Sprint**: S52
**Size**: S
**Depends on**: F-X037

## Problem

An empty Word paragraph produces a line with no positioned element in
`crates/rdocx-layout/src/engine.rs:2837`. Interactive callers therefore have no
source-bearing caret target for an empty body paragraph, table cell, header,
footer, footnote, or endnote. PR 41 supplies a prototype but no focused proof.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/08-rendering-spec.md`, "Word source provenance" and text layout.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".

## Approach

When an otherwise empty Word paragraph creates its single empty line, add one
zero-width empty text segment using the paragraph's resolved default font and
metrics. In provenance mode assign the paragraph source node and scalar range
`0..0`. In ordinary mode emit the same segment without a source id so layout
structure remains compatible. Do not shape a glyph and do not change non-empty
paragraphs or non-Word layout inputs.

## Rejected alternatives

- A synthetic space changes text extraction and visible layout.
- A facade-only caret record would disagree with third-party `PageFrame`
  consumers.
- Provenance-only insertion would make ordinary and attributed layouts differ.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `empty_word_stories_emit_one_attributed_zero_width_segment` | Body, table, header, footer, footnote, and endnote paragraphs each carry the right source node and `0..0` range. |
| regression | `empty_paragraph_uses_resolved_default_metrics` | Direct paragraph and style defaults choose the caret font and line height. |
| regression | `empty_segment_is_backend_invisible_and_layout_compatible` | Ordinary and provenance layouts match structurally, non-empty paragraphs do not change, and PDF plus raster emit no new glyph. |

The test gate is **regression**. Both backends and the deterministic hash
harness must remain unchanged.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout and text shaping: re-read `docs/hld/08-rendering-spec.md`, run every
  backend baseline in deterministic font mode, and require unchanged hashes.

## Hash harness

Expected to be unchanged. The segment has empty text and zero width, so neither
built-in backend should emit a glyph or move content.

## Implementation checklist

- [ ] Add one empty segment only to empty Word paragraph lines.
- [ ] Resolve default font metrics without shaping a glyph.
- [ ] Attach the correct optional source node and `0..0` scalar range.
- [ ] Cover every Word story, compatibility, backends, and hashes.

## Open questions

None. A zero-width empty segment is the smallest common representation visible
to all `PageFrame` consumers.

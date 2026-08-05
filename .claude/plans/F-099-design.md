# F-099, Bullets

**Status**: completed
**Sprint**: S24
**Size**: M
**Depends on**: F-098d

## Problem

The frozen resolver already carries character and automatic bullets with
independently inherited font, colour, size, scheme, and starting value at
`crates/rpptx-layout/src/lib.rs:267`. The renderer has no marker construction or
numbering state. In addition, `FontManager` aliases Wingdings to Symbol at
`crates/oxml-layout/src/font.rs:650`, so passing private-use `F0B7` through font
resolution produces the wrong glyph or a missing-glyph box.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Text body".
- `docs/hld/07-inheritance-and-resolution.md`, "The nine-level list style".
- `docs/hld/08-rendering-spec.md`, "Text in a shape", the "Bullets" paragraph.
- `docs/hld/14-development-backlog.md`, "F-099, Bullets".

## Approach

Extend the private S24 text module with one numbering state per text body and
level. Character bullets map Wingdings `U+F0B7` to Unicode `U+2022` before font
resolution. Automatic bullets support these exact eight tokens:
`arabicPlain`, `arabicPeriod`, `arabicParenR`, `arabicParenBoth`,
`alphaLcPeriod`, `alphaUcPeriod`, `romanLcPeriod`, and `romanUcPeriod`.

The first automatic paragraph at a level uses `start_at`, and a consecutive
paragraph with the same scheme increments it. Advancing a shallower level
resets deeper levels. A character bullet, no bullet, or scheme change resets
the affected sequence. Other schema schemes fall back visibly to
`arabicPeriod` rather than dropping the marker.

Resolve bullet font, colour, and point or percentage size independently, with
omitted values falling back to the paragraph's first effective run style.
Prepend one shaped `InlineItem::Marker`, and reuse F-098c for the baseline and
F-098 paragraph indents for marker and wrapped-text placement. Add no new
public type or second source file.

## Rejected alternatives

- Pass Wingdings private-use text directly to the font manager. Its current
  Symbol alias is the failure this story must prevent.
- Implement all 41 schema schemes. The story names eight common schemes and
  requires a visible v1 fallback for the remainder.
- Store counters in the resolver. Numbering is presentation layout state, not
  inherited OOXML model state.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `wingdings_f0b7_bullet_renders_as_a_visible_unicode_glyph` | The backlog gate maps to U+2022 and produces a visible deterministic glyph |
| unit | `eight_common_auto_number_schemes_format_exact_markers` | Each approved token produces the exact punctuation and case |
| unit | `automatic_bullets_increment_and_reset_by_level` | Start, increment, shallower reset, scheme change, and no-bullet reset semantics |
| unit | `bullet_style_overrides_and_fallbacks_are_independent` | Font, colour, point size, and percentage size resolve independently |
| regression | `bullet_marker_uses_left_and_hanging_indent` | Marker and wrapped text use the expected distinct positions |
| regression | `unsupported_auto_number_scheme_keeps_a_visible_marker` | Other schema schemes use the approved Arabic-period fallback |

The test gate is a Wingdings `F0B7` bullet renders as a visible bullet glyph,
not a missing-glyph box.

## HLD impact

- `docs/hld/08-rendering-spec.md`

Record the exact eight schemes, counter lifetime and reset rules, visible
fallback, and minimum Wingdings mapping because the current paragraph leaves
those behaviors unspecified.

## Risk routing

- Layout, pagination, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Every glyph or pixel assertion uses
  deterministic font mode, focused checks include `cargo test -p
  rpptx-render`, and no font baseline is recorded from system fonts.

## Hash harness

Expected to be unchanged. Bullets render only through the unpublished
PowerPoint path.

## Implementation checklist

- [x] Add per-body, per-level automatic numbering state and the eight formatters.
- [x] Reset sequences on shallower levels, scheme changes, character bullets, and no bullet.
- [x] Map Wingdings F0B7 to a Unicode bullet before font resolution.
- [x] Shape marker style and size independently from the paragraph run.
- [x] Reuse the shared marker emitter and prove indent placement.

## Open questions

None. The eight schemes, counter and reset rules, minimal Wingdings mapping,
and visible `arabicPeriod` fallback are approved.

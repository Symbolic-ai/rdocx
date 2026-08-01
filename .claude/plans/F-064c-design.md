# F-064c, Text bullets

**Status**: completed
**Sprint**: S14
**Size**: S
**Depends on**: F-064b

## Problem

F-064b provides paragraph properties but no DrawingML bullet vocabulary.
Business-deck list paragraphs need character, automatic-numbering, and explicit
no-bullet choices together with optional font, size, and colour children.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Text body" and "Preservation".
- `docs/hld/08-rendering-spec.md`, "Text in a shape".
- `docs/hld/14-development-backlog.md`, "F-064c, Text bullets".

## Approach

Add `text/bullet.rs`. Model the mutually exclusive `a:buChar`, `a:buAutoNum`,
and `a:buNone` choices plus font, percentage or point size, and colour. Attach
one bullet model to paragraph properties and emit its components in DrawingML
schema order.

Keep Wingdings codepoint conversion out of this wire model. M10 owns the
renderer-side mapping before font resolution. Unknown bullet-related children
remain raw at their paragraph-property boundary.

## Rejected alternatives

- Convert bullet characters to Unicode while parsing. That would make a
  structural round-trip lossy and put rendering policy in the XML layer.
- Store all bullet XML as raw. Later layout needs the modelled choice, size,
  colour, and font values.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `every_modelled_bullet_form_round_trips_in_schema_order` | The child backlog test gate covers character, auto-number, and none |
| regression | `bullet_font_size_and_colour_keep_their_schema_positions` | Optional components write in canonical order |
| regression | `unknown_bullet_children_round_trip_byte_for_byte` | Unsupported bullet extensions survive in place |
| regression | `malformed_bullet_values_return_errors_without_panicking` | Missing characters and invalid size or numbering tokens fail safely |

The test gate is `every_modelled_bullet_form_round_trips_in_schema_order`.

## HLD impact

None. The bullet vocabulary and renderer boundary are already specified.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove schema order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte raw preservation.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `text/bullet.rs` before implementation.

## Hash harness

Expected to be unchanged. The unpublished DrawingML model has no Word consumer.

## Implementation checklist

- [x] Add failing bullet-family, order, raw-preservation, and malformed-input tests.
- [x] Add bullet choice, font, size, colour, and numbering types.
- [x] Integrate bullets into paragraph properties in schema order.
- [x] Preserve unsupported bullet XML at exact boundaries.
- [x] Run focused checks.

## Open questions

None. The new `text/bullet.rs` module and file are approved.

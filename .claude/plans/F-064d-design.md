# F-064d, Nine-level list styles

**Status**: approved
**Sprint**: S14
**Size**: M
**Depends on**: F-064b, F-064c

## Problem

The text-body shell and paragraph model do not yet carry `a:lstStyle`. Without
its nine level-specific paragraph property slots, later inheritance cannot
resolve the level-indexed defaults that control most business-deck text.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Text body", "Two traps that are silent
  until PowerPoint refuses the file", and "Preservation".
- `docs/hld/07-inheritance-and-resolution.md`, "The text style chain".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-064d, Nine-level list styles".

## Approach

Add `text/list_style.rs`. Represent `a:lstStyle` as nine explicit optional
level slots using the paragraph-property type from F-064b. Parse level elements
by their numbered local names and write them in ascending schema order, while
retaining unknown siblings at exact boundaries.

Complete the `CT_TextBody` composition with body properties, list style, and
paragraphs. Prove it using inline schema-valid XML that exercises every level,
paragraph content, bullets, whitespace, and raw extensions. The fetched deck
corpus remains the M7 boundary gate.

## Rejected alternatives

- Use an unchecked vector of levels. Nine explicit slots make invalid level
  numbers unrepresentable and match the schema's fixed sequence.
- Add a binary PowerPoint fixture. Repository policy keeps fixtures inline
  until the external corpus harness exists.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `schema_valid_text_body_using_all_nine_list_levels_round_trips_structurally` | The child backlog gate and integrated F-064 fixture gate |
| regression | `list_style_levels_write_in_ascending_schema_order` | All nine level slots emit at the correct boundaries |
| regression | `unknown_list_style_children_round_trip_byte_for_byte` | Unsupported siblings retain their exact XML and positions |
| regression | `invalid_list_levels_return_errors_without_panicking` | Unknown numbered levels and malformed properties fail safely |

The test gate is
`schema_valid_text_body_using_all_nine_list_levels_round_trips_structurally`.

## HLD impact

None. The nine-level chain and corpus boundary are already specified.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove schema order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte raw preservation.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `text/list_style.rs` before implementation.

## Hash harness

Expected to be unchanged. The unpublished DrawingML model has no Word consumer.

## Implementation checklist

- [ ] Add failing nine-level, complete-text-body, raw, and malformed-input tests.
- [ ] Add the fixed nine-slot list-style model.
- [ ] Parse and serialise list levels in schema order.
- [ ] Complete the text-body shell using all prior child types.
- [ ] Run focused checks and retain the external corpus gate for the M7 boundary.

## Open questions

None. The new `text/list_style.rs` module and file and inline schema-valid S14
fixtures are approved. The external corpus remains the M7 boundary gate.

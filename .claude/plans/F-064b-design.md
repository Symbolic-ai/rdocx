# F-064b, Text paragraphs and runs

**Status**: approved
**Sprint**: S14
**Size**: L
**Depends on**: F-064a

## Problem

The F-064a shell cannot carry actual slide text. The model needs DrawingML
paragraphs, paragraph properties, runs, run properties, fields, breaks, and
text nodes without reusing WordprocessingML's different unit conventions.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Text body", "Two traps that are silent
  until PowerPoint refuses the file", and "Preservation".
- `docs/hld/08-rendering-spec.md`, "Text in a shape".
- `docs/hld/14-development-backlog.md`, "F-064b, Text paragraphs and runs".

## Approach

Add `text/paragraph.rs`. Model paragraphs and their ordered run, field, and
break children. Model the paragraph and run properties required by the story,
including alignment, level, indentation, spacing, centipoint font size, style
flags, baseline, typefaces, fill, and hyperlink identifiers.

Text serialisation adds `xml:space="preserve"` whenever leading or trailing
whitespace is significant. Parsing retains text content and the relevant XML
space intent. Unknown children remain byte-identical at their schema boundaries.

## Rejected alternatives

- Reuse Word paragraph and run types. Their namespaces, property sequences,
  font-size units, and line-spacing units differ.
- Trim text and reconstruct spaces during layout. That loses source semantics
  before a renderer can see them.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `leading_and_trailing_text_whitespace_survives_via_xml_space_preserve` | The child and parent whitespace gate |
| round-trip | `paragraph_runs_fields_and_breaks_round_trip_structurally` | All four content forms preserve order and properties |
| regression | `paragraph_and_run_properties_use_drawingml_units_and_schema_order` | Centipoints, percentages, and fixed child ordering |
| regression | `malformed_text_content_returns_errors_without_panicking` | Invalid values and incomplete XML fail safely |

The test gate is
`leading_and_trailing_text_whitespace_survives_via_xml_space_preserve`.

## HLD impact

None. The paragraph and run vocabulary is already specified.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove schema order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte raw preservation.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `text/paragraph.rs` before implementation.

## Hash harness

Expected to be unchanged. The unpublished DrawingML model has no Word consumer.

## Implementation checklist

- [ ] Add failing whitespace, content-order, property-unit, and malformed-input tests.
- [ ] Add paragraph, run, field, break, and text types.
- [ ] Add paragraph and run property parsing and serialisation.
- [ ] Preserve significant text whitespace and unsupported XML.
- [ ] Connect paragraphs to the F-064a shell and run focused checks.

## Open questions

None. The new `text/paragraph.rs` module and file are approved.

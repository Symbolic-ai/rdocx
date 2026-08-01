# F-064a, Text body properties and shell

**Status**: completed
**Sprint**: S14
**Size**: M
**Depends on**: F-053

## Problem

`oxml-drawing` has no `text` module at
`crates/oxml-drawing/src/lib.rs:1`. Later paragraph and list-style work needs a
single owner for `a:txBody` and a typed `a:bodyPr` model that retains PowerPoint's
stored autofit decision.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Text body", "Two traps that are silent
  until PowerPoint refuses the file", and "Preservation".
- `docs/hld/08-rendering-spec.md`, "Text in a shape" and "Autofit".
- `docs/hld/14-development-backlog.md`, "F-064a, Text body properties and
  shell".

## Approach

Add `text/mod.rs` and `text/body.rs`. Define the `CT_TextBody` composition shell
and `CT_TextBodyProperties` with insets, anchor, wrap, vertical direction, and
the three autofit forms. Preserve stored `normAutofit` scale and line-spacing
reduction values verbatim. Do not implement layout or resize behaviour.

The root parser accepts any prefix. Writers use `a:` and keep `bodyPr`,
`lstStyle`, and paragraphs in schema order as later children populate the shell.
Unsupported body-property children remain at their original boundaries.

## Rejected alternatives

- Implement autofit calculations here. This crate owns the wire model, while
  layout belongs to M10.
- Put the whole text model in one source file. The approved XL split has four
  independently reviewable schema areas.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `every_body_property_autofit_form_round_trips_in_schema_order` | The child backlog test gate covers no, shape, and normal autofit |
| regression | `body_properties_preserve_unknown_children_at_their_boundaries` | Unsupported body properties survive byte for byte |
| regression | `text_body_reads_any_prefix_and_writes_the_fixed_a_prefix` | Prefix tolerance and canonical output |
| regression | `malformed_body_properties_return_errors_without_panicking` | Invalid enum and numeric attributes fail safely |

The test gate is `every_body_property_autofit_form_round_trips_in_schema_order`.

## HLD impact

None. The text-body and autofit wire contracts are already specified.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove schema order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte raw preservation.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `text/mod.rs` and `text/body.rs` before implementation.

## Hash harness

Expected to be unchanged. The unpublished DrawingML model has no Word consumer.

## Implementation checklist

- [x] Add failing body-property, prefix, raw-preservation, and malformed-input tests.
- [x] Add the text module, text-body shell, and body-property types.
- [x] Parse and serialise body properties and autofit forms in schema order.
- [x] Preserve unsupported XML at exact boundaries.
- [x] Export the approved text module and run focused checks.

## Open questions

None. The new `text/mod.rs` and `text/body.rs` modules and files are approved.

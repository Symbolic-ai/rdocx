# F-061, Lines

**Status**: completed
**Sprint**: S14
**Size**: M
**Depends on**: F-054

## Problem

`oxml-drawing` models colours and fills but exports no line vocabulary from
`crates/oxml-drawing/src/lib.rs:1`. Shape outlines therefore cannot retain
width, paint, dash, cap, join, or endpoint settings, and no code maps the
DrawingML preset dash enumeration to renderer-ready dash ratios.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Modules", "Two traps that are silent until
  PowerPoint refuses the file", and "Preservation".
- `docs/hld/14-development-backlog.md`, "F-061, Lines".

## Approach

Add `crates/oxml-drawing/src/line.rs`. Model `CT_LineProperties` with width,
the existing `Fill` choice, preset and custom dash data, cap and join choices,
and head and tail end settings. Define the story-requested
`ST_PresetLineDashVal` enumeration with a total mapping from every token to its
relative dash array. Keep the wire model independent from `oxml-layout` so the
DrawingML crate does not depend upward on rendering.

Parsing accepts any namespace prefix. Writing uses the fixed `a:` prefix and
the schema sequence for fill, dash, join, head end, tail end, and extensions.
Unknown siblings remain byte-identical through `OrderedRawChildren`.

## Rejected alternatives

- Return `oxml_layout::Stroke` directly. That reverses the dependency direction
  and mixes XML representation with later resolution.
- Preserve preset dash tokens as strings. The test gate requires a total,
  reviewable mapping and malformed tokens must return errors.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `every_preset_line_dash_value_maps_to_a_dash_array` | The backlog test gate covers every `ST_PresetLineDashVal` variant |
| round-trip | `line_properties_round_trip_width_fill_dash_cap_join_and_ends` | Every modelled line field survives serialise and reparse |
| regression | `line_properties_write_schema_order_and_preserve_unknown_children` | Fixed prefix, child order, and byte-for-byte raw preservation |
| regression | `malformed_line_values_return_errors_without_panicking` | Unknown tokens and invalid numeric attributes fail safely |

The test gate is `every_preset_line_dash_value_maps_to_a_dash_array`.

## HLD impact

None. The existing DrawingML model already defines the line module and scope.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove schema order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte raw preservation.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `crates/oxml-drawing/src/line.rs` before implementation.

## Hash harness

Expected to be unchanged. The unpublished DrawingML model has no Word consumer.

## Implementation checklist

- [x] Add failing dash-map, line round-trip, schema-order, and malformed-input tests.
- [x] Add line enums, endpoint types, errors, and the total preset dash mapping.
- [x] Parse and serialise line properties in schema order.
- [x] Preserve unsupported children at their original schema boundaries.
- [x] Export the approved module and run focused checks.

## Open questions

None. The new `crates/oxml-drawing/src/line.rs` module and file are approved.

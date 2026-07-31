# F-060, Fills

**Status**: completed
**Sprint**: S13
**Size**: L
**Depends on**: F-054

## Problem

`oxml-drawing` can model colour choices but has no fill model at
`crates/oxml-drawing/src/lib.rs:1`. It cannot round-trip no-fill, solid,
gradient, pattern, or picture-fill XML, so later shape properties cannot
describe the fill of a business-deck shape.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Modules", "Two traps that are silent until
  PowerPoint refuses the file", and "Preservation".
- `docs/hld/04-opc-and-packaging.md`, "Media".
- `docs/hld/14-development-backlog.md`, "F-060, Fills".

## Approach

Add the HLD-defined `crates/oxml-drawing/src/fill.rs` module. Model
`a:noFill`, `a:solidFill`, `a:gradFill` with linear and path geometry,
`a:pattFill`, and `a:blipFill` with source rectangles, stretch, and tile
properties. Reuse `ColorChoice` for every modelled colour and keep picture
relationship identifiers as owned strings without introducing an OPC or media
dependency.

Each fill form parses prefix-tolerantly and writes the fixed `a:` prefix in
schema order. Gradient stops retain document order during round trips. Unknown
siblings and nested extensions are captured byte for byte with
`OrderedRawChildren`. The public enum contains the five story-requested fill
families and no renderer conversion surface.

## Rejected alternatives

- Lower fills directly to `oxml-layout::Paint`. This story owns the OOXML wire
  model, and resolution belongs to later PresentationML layers.
- Sort gradient stops while parsing. That would mutate source structure during
  a model round trip.
- Parse media bytes. `a:blip` carries relationship identifiers, while package
  resolution and media probing belong to other crates.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `every_fill_form_round_trips_and_gradient_stops_keep_document_order` | The backlog test gate across no, solid, linear gradient, path gradient, pattern, stretched blip, and tiled blip forms |
| regression | `fill_forms_read_any_prefix_and_write_fixed_a_prefix_in_schema_order` | Canonical names and required child order |
| regression | `unknown_fill_children_round_trip_byte_for_byte_in_place` | Unmodelled nested XML survives at its schema boundary |
| regression | `malformed_fill_values_return_errors_without_panicking` | Bad positions, angles, percentages, and missing attributes are rejected safely |

The test gate is
`every_fill_form_round_trips_and_gradient_stops_keep_document_order`.

## HLD impact

None. The existing DrawingML model already lists every fill form in scope.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove schema order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte raw preservation.
- Theme colour and transform isolation: read
  `docs/hld/05-drawingml-model.md`. Extra checks confirm fills reuse
  `ColorChoice`, do not modify Word's legacy theme path, and leave the released
  Word theme diff empty.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `crates/oxml-drawing/src/fill.rs` before implementation.

## Hash harness

Expected to be unchanged. This is an unpublished DrawingML model with no Word
consumer.

## Implementation checklist

- [x] Add failing fill-family, schema-order, raw-preservation, and malformed-input tests.
- [x] Add fill types and errors for every story-requested family.
- [x] Implement prefix-tolerant parsing and fixed-prefix serialisation.
- [x] Preserve gradient stop order and unknown XML boundaries.
- [x] Export the approved module and run focused checks.

## Open questions

None. The three HLD-defined S13 module files were approved together.

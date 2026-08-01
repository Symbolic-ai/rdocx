# F-062, Effects

**Status**: approved
**Sprint**: S14
**Size**: S
**Depends on**: none

## Problem

`oxml-drawing` has no effect-list model in
`crates/oxml-drawing/src/lib.rs:1`. A shape carrying an outer shadow or an
unsupported effect such as glow cannot cross the model boundary without losing
either renderer-relevant data or unmodelled XML.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Modules", "Two traps that are silent until
  PowerPoint refuses the file", and "Preservation".
- `docs/hld/08-rendering-spec.md`, "The rasteriser".
- `docs/hld/14-development-backlog.md`, "F-062, Effects".

## Approach

Add `crates/oxml-drawing/src/effect.rs`. Model `CT_EffectList` and
`CT_OuterShadowEffect` with blur radius, distance, direction, scale, skew,
alignment, rotation, and colour. Preserve every unsupported effect as raw XML
at its original effect-list boundary.

The parser is prefix-tolerant and the writer uses the fixed `a:` prefix in the
effect-list schema order. The model exposes data only. It does not add a raster
approximation for unsupported effects or for outer-shadow blur.

## Rejected alternatives

- Model every DrawingML effect. The story deliberately models outer shadow and
  preserves everything else.
- Flatten unsupported effects into one trailing extension blob. That loses the
  original sibling position and can violate schema order.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `a_shape_with_glow_round_trips_with_glow_intact_as_raw_xml` | The backlog test gate preserves glow byte for byte |
| round-trip | `outer_shadow_properties_and_colour_round_trip_structurally` | Every modelled shadow field survives serialise and reparse |
| regression | `effect_list_writes_schema_order_and_keeps_raw_effect_positions` | Canonical prefix and raw boundary placement |
| regression | `malformed_outer_shadow_values_return_errors_without_panicking` | Invalid numbers and tokens fail safely |

The test gate is `a_shape_with_glow_round_trips_with_glow_intact_as_raw_xml`.

## HLD impact

None. The existing DrawingML model already defines the effect boundary.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove schema order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte raw preservation.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `crates/oxml-drawing/src/effect.rs` before implementation.

## Hash harness

Expected to be unchanged. The unpublished DrawingML model has no Word consumer.

## Implementation checklist

- [ ] Add failing glow, outer-shadow, schema-order, and malformed-input tests.
- [ ] Add effect-list and outer-shadow types with explicit errors.
- [ ] Parse and serialise the modelled shadow in schema order.
- [ ] Preserve unsupported effects at their original boundaries.
- [ ] Export the approved module and run focused checks.

## Open questions

None. The new `crates/oxml-drawing/src/effect.rs` module and file are approved.

# F-091, Preset evaluation and fallback

**Status**: completed
**Sprint**: S22
**Size**: M
**Depends on**: F-090

## Problem

`CT_ShapeProperties` models `a:custGeom` but preserves `a:prstGeom` only as an
unmodelled subtree at `crates/oxml-drawing/src/shape_props.rs:183`. The resolver
therefore reaches `crates/rpptx-layout/src/context.rs:463` without a preset name
or adjustment values and emits a generic bounds fallback for every preset.

F-090 supplies the generated definition lookup, and F-058 supplies the guide
and path evaluator. This story must join them without weakening raw XML
preservation or losing text when a producer uses an unknown preset name.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Geometry".
- `docs/hld/06-presentationml-model.md`, "Parse only what we render" and
  "Raw-XML preservation".
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract".
- `docs/hld/08-rendering-spec.md`, "Preset geometry".
- `docs/hld/14-development-backlog.md`, "F-091, Preset evaluation and
  fallback".

## Approach

Add `CT_PresetGeometry2D` inside the existing `geometry.rs`, holding the preset
name, adjustment guides, and ordered raw children. Parse and serialise
`a:prstGeom` in the existing `shape_props.rs` geometry slot with prefix-tolerant
input, fixed `a:` output, and schema child order. Preserve unmodelled children
byte for byte.

Add a lookup-and-evaluate entry point in `geometry.rs`. It fetches F-090's
generated `a:custGeom` bytes, parses them through the existing
`CT_CustomGeometry2D` path, applies the shape's adjustment values as overrides,
and returns the same `EvaluatedCustomGeometry` used by custom geometry. Keep
all implementation in existing modules except F-090's generated module.

Update `ResolveCtx::resolve_ordinary_shape` to convert a known preset into
backend-neutral paths and its text rectangle. An unknown preset retains
`ResolvedGeometry::BoundsFallback`, retains the independently resolved text
body, sets a stable unsupported category, and appends a diagnostic naming the
preset. If both custom and preset geometry are present, schema choice semantics
continue to prefer the already modelled custom geometry.

## Rejected alternatives

- Parse generated XML in `rpptx-layout`. DrawingML geometry ownership belongs
  in `oxml-drawing` and duplicating the evaluator crosses the crate seam.
- Convert the generated table to bespoke renderer paths. Presets and
  `a:custGeom` use the same guide machinery, so a second evaluator adds drift.
- Drop unknown shapes. The frozen contract requires visible bounds, text, and a
  diagnostic.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `preset_geometry_round_trips_with_unknown_children_verbatim` | Prefix-tolerant parsing, fixed-prefix schema order, and raw subtree preservation all hold |
| unit | `rectangle_preset_evaluates_to_expected_bounds_and_text_rect` | A known preset uses generated guides and paths |
| unit | `preset_adjustments_override_generated_defaults` | Shape-level `a:avLst` values feed F-058's evaluator |
| regression | `unknown_preset_keeps_bounds_text_and_diagnostic` | The named backlog gate preserves shape visibility and text while recording the unknown name |
| corpus | `all_corpus_preset_geometries_evaluate_or_fallback` | All 50 decks resolve without panic or silent shape loss |

The backlog test gate is `unknown_preset_keeps_bounds_text_and_diagnostic`.

## HLD impact

None. The implementation follows the already documented preset evaluation and
fallback contract.

## Risk routing

- Any parser or serialiser. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add the round-trip test above proving
  schema order, prefix tolerance, and byte-for-byte preservation of unmodelled
  children.
- Layout and rendering. Use deterministic font mode for any render evidence.
  This story records no new pixel baseline and must keep the hash harness
  unchanged.

## Hash harness

Expected to be unchanged. PowerPoint preset evaluation is not connected to the
released Word render samples.

## Implementation checklist

- [x] Model preset name and adjustment values in existing geometry code.
- [x] Preserve unknown preset children and schema order on round-trip.
- [x] Evaluate generated definitions through F-058.
- [x] Convert known presets to backend-neutral paths and text rectangles.
- [x] Preserve bounds and text for unknown presets with a named diagnostic.
- [x] Run the 50-deck corpus resolution gate.

## Deviations

The non-vacuous corpus gate exposed two standard fractional guides used by
F-090's generated definitions but absent from F-058's evaluator seed. The
existing evaluator now seeds `wd12` and `hd10`. No approach, dependency,
public contract, HLD impact, or hash expectation changed.

## Open questions

None. The existing custom-geometry evaluator, generated table, and frozen
fallback contract determine the implementation.

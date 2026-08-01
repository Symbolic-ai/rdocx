# F-063, Shape properties and style references

**Status**: completed
**Sprint**: S14
**Size**: M
**Depends on**: F-060, F-061

## Problem

The crate has transform, geometry, fill, and soon line and effect types, but
`crates/oxml-drawing/src/lib.rs:1` exports no `a:spPr` composition boundary or
the four style references carried by `p:style`. Later PresentationML code would
otherwise need to duplicate schema ordering and the special background-fill
index rule.

## Spec reference

- `docs/hld/01-glossary.md`, "Theme and colour vocabulary".
- `docs/hld/05-drawingml-model.md`, "Modules", "Two traps that are silent until
  PowerPoint refuses the file", and "Preservation".
- `docs/hld/07-inheritance-and-resolution.md`, "Theme format-scheme indices".
- `docs/hld/14-development-backlog.md`, "F-063, Shape properties and style
  references".

## Approach

Add `crates/oxml-drawing/src/shape_props.rs` for `CT_ShapeProperties`, composing
the existing transform, geometry, fill, line, and effect models in schema order.
Add `crates/oxml-drawing/src/style_ref.rs` for line, fill, effect, and font
references with their index and colour choice.

Represent fill-style selection as a checked resolver result that distinguishes
the normal fill-style list from `idx > 1000`, where `idx - 1000` selects the
background-fill list. Keep theme lookup outside this crate until F-065 provides
the format scheme. Preserve unsupported `a:spPr` and style-reference children
at their schema boundaries.

## Rejected alternatives

- Put all types in one shape module. Style references are also consumed by the
  inheritance resolver independently from `a:spPr`.
- Resolve style indices against a theme here. F-065 owns the theme format
  scheme, while this story owns index interpretation only.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `fill_ref_1001_resolves_to_background_fill_style_1` | The backlog test gate implements the `idx > 1000` rule |
| round-trip | `shape_properties_round_trip_in_schema_order` | Transform, geometry, fill, line, effect, and raw children retain order |
| round-trip | `all_four_style_reference_forms_round_trip` | Line, fill, effect, and font references preserve indices and colours |
| regression | `malformed_shape_and_style_references_return_errors_without_panicking` | Invalid roots, indices, and child values fail safely |

The test gate is `fill_ref_1001_resolves_to_background_fill_style_1`.

## HLD impact

None. The model and index rule are already specified.

## Risk routing

- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove schema order,
  prefix-tolerant reads, fixed `a:` writes, and byte-for-byte raw preservation.
- Theme colour and colour mapping: read `docs/hld/05-drawingml-model.md`. Extra
  checks prove references reuse `ColorChoice` without touching Word's legacy
  theme path.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `shape_props.rs` and `style_ref.rs` before implementation.

## Hash harness

Expected to be unchanged. The unpublished DrawingML model has no Word consumer.

## Implementation checklist

- [x] Add failing shape-order, style-reference, background-index, and malformed-input tests.
- [x] Add and export the shape-properties model.
- [x] Add and export the four style-reference forms and index classification.
- [x] Preserve unsupported XML at exact schema boundaries.
- [x] Run focused checks after F-061 is integrated.

## Open questions

None. The new `shape_props.rs` and `style_ref.rs` modules and files are
approved.

# F-057, a:xfrm

**Status**: approved
**Sprint**: S13
**Size**: M
**Depends on**: none

## Problem

`oxml-drawing` currently exposes only colour, namespace, and raw-order modules
at `crates/oxml-drawing/src/lib.rs:1`. It cannot parse, serialise, or compose
the `a:xfrm` data that positions shapes and maps nested group coordinates.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Modules" and "Two traps that are silent
  until PowerPoint refuses the file".
- `docs/hld/08-rendering-spec.md`, "Why Group is the whole design".
- `docs/hld/01-glossary.md`, "Units and coordinate systems".
- `docs/hld/14-development-backlog.md`, "F-057, a:xfrm".

## Approach

Add the HLD-defined `crates/oxml-drawing/src/xfrm.rs` module. Model offset and
extent pairs with `Emu`, optional child offset and extent pairs for group
transforms, `Angle` rotation, and horizontal and vertical flips. Parse local
names without requiring a prefix, write the fixed `a:` prefix in schema order,
and retain unknown children at their original boundaries with
`OrderedRawChildren`.

Expose the parsed transform as `CT_Transform2D`. Its matrix method returns the
six affine coefficients as `[f64; 6]` in the order `a, b, c, d, e, f`. It
applies child-coordinate translation and scale, shape translation, centre
rotation, and centre flips in the order fixed by the rendering HLD. Composition
stays local to `oxml-drawing`, so this story adds no `oxml-layout` dependency.

## Rejected alternatives

- Depend on `oxml-layout::Transform`. The architecture DAG keeps the DrawingML
  wire model independent of the renderer contract.
- Reuse Word's `drawing.rs`. That file models `wp:` wrappers and is explicitly
  retained in `rdocx-oxml`.
- Append unknown children after known fields. That violates `xsd:sequence` and
  loses their original schema boundaries.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `nested_group_transform_composes_to_the_hand_computed_matrix` | The backlog test gate, including child offset, child extent, translation, rotation, and flips |
| round-trip | `transform_reads_any_prefix_and_writes_fixed_a_prefix_in_schema_order` | All fields parse and serialise in the required child order |
| regression | `unknown_transform_children_round_trip_at_their_original_boundaries` | Raw XML survives byte for byte around modelled children |
| regression | `zero_child_extent_returns_a_transform_error_instead_of_non_finite_coefficients` | Invalid scaling is reported without NaN or infinity |

The test gate is `nested_group_transform_composes_to_the_hand_computed_matrix`.

## HLD impact

None. The existing DrawingML and rendering sections already define this
contract.

## Risk routing

- Unit conversion and `Emu`: read `docs/hld/01-glossary.md` units and preserve
  the truncating constructors. Extra check:
  `cargo test -p oxml-core units::tests::emu_float_constructors_truncate_toward_zero`.
- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Extra checks prove prefix-tolerant
  reads, fixed `a:` writes, schema order, and byte-for-byte raw preservation.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `crates/oxml-drawing/src/xfrm.rs` before implementation.

## Hash harness

Expected to be unchanged. This adds an unpublished model with no current Word
consumer.

## Implementation checklist

- [ ] Add failing matrix, round-trip, raw-preservation, and invalid-extent tests.
- [ ] Add transform value types, errors, parsing, and fixed-prefix writing.
- [ ] Implement finite affine matrix generation and composition.
- [ ] Export the approved module and run focused checks.

## Open questions

None. The three HLD-defined S13 module files were approved together.

# F-058, Guide evaluator

**Status**: completed
**Sprint**: S13
**Size**: L
**Depends on**: F-014

## Problem

The DrawingML crate has no geometry module at
`crates/oxml-drawing/src/lib.rs:1`. It therefore cannot evaluate guide formulas,
seed the standard shape environment, apply adjust values, or lower `a:arcTo`
commands into renderer-neutral cubic segments.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Geometry".
- `docs/hld/08-rendering-spec.md`, "Preset geometry".
- `docs/hld/13-risks-and-open-questions.md`, "Q1, preset shape definition
  provenance".
- `docs/hld/01-glossary.md`, "Units and coordinate systems".
- `docs/hld/14-development-backlog.md`, "F-058, Guide evaluator".
- ISO/IEC 29500-1, `gd (Shape Guide)`, for the 17 recognised formula tokens
  and their argument order.
- Microsoft `[MS-OE376]`, Part 4 Section 5.1.11.11, for Office's `mod` and
  negative-input `sqrt` guide behaviour.

## Approach

Add the HLD-defined `crates/oxml-drawing/src/geometry.rs` module with owned
guide names and operands, the full 17-operation `GuideOp` enum, a seeded
evaluation environment, adjust-value overrides, and local evaluated path
commands. The enum maps the formula tokens `*/`, `+-`, `+/`, `?:`, `abs`,
`at2`, `cat2`, `cos`, `max`, `min`, `mod`, `pin`, `sat2`, `sin`, `sqrt`, `tan`,
and `val` without splitting the compound operations into invented variants.
The environment contains `w`, `h`, `ss`, edges, centres, standard fractional
width and height guides, and declared adjust values. Guides evaluate in
declaration order with `f64` arithmetic and angles in 60000ths of a degree.
Office's interoperability behaviour applies `mod x y z` as the Euclidean norm
and `sqrt x` as `sqrt(abs(x))`.

Model move, line, cubic, close, and arc input commands. Evaluation emits only
move, line, cubic, and close output commands. Arc lowering splits sweeps into
segments of at most 90 degrees and derives cubic control points from the
ellipse tangent, leaving no arc command for a renderer. The output stays local
to `oxml-drawing`, preserving the architecture DAG until the later
PresentationML lowering layer consumes it.

## Rejected alternatives

- Wait for preset-shape provenance. Custom geometry needs the evaluator now,
  and the HLD identifies that work as independent of the preset table source.
- Use a trait-based formula engine. There is one evaluator and no second
  implementer today.
- Return `oxml-layout::Path`. That would add a renderer dependency to the wire
  model and contradict the documented crate layering.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `hand_written_custom_geometry_guides_produce_expected_path_coordinates` | The backlog test gate across seeded values, adjust overrides, ordered guides, and path commands |
| unit | `all_seventeen_formula_tokens_parse_and_evaluate_with_drawingml_argument_order` | Every recognised formula token maps to its compound operation and argument order |
| regression | `arc_to_is_flattened_to_finite_cubics_with_matching_endpoints` | Arc lowering emits only finite cubic commands and lands on the expected endpoint |
| regression | `office_mod_and_negative_sqrt_semantics_produce_finite_values` | The Office interoperability rules for Euclidean `mod` and `sqrt(abs(x))` are pinned |
| regression | `division_by_zero_returns_an_error_instead_of_non_finite_coordinates` | Division by zero is an explicit error |

The test gate is
`hand_written_custom_geometry_guides_produce_expected_path_coordinates`.

## HLD impact

- `docs/hld/05-drawingml-model.md`, "Geometry", to replace the inaccurate
  split arithmetic variants with the 17 formula operations DrawingML defines.

## Risk routing

- Unit conversion and angles: read `docs/hld/01-glossary.md` units and preserve
  truncation in existing constructors. Extra checks:
  `cargo test -p oxml-core units::tests::angle_round_trip_degrees` and
  `cargo test -p oxml-core units::tests::new_unit_float_constructors_truncate_toward_zero`.
- A new module or file: `CLAUDE.md` structural rules require explicit approval
  for `crates/oxml-drawing/src/geometry.rs` before implementation.

## Hash harness

Expected to be unchanged. This adds an unpublished evaluator with no current
Word consumer.

## Implementation checklist

- [x] Add failing guide-operation, seeded-environment, path, and invalid-math tests.
- [x] Add owned guide operands, operations, environment, and evaluation errors.
- [x] Implement ordered guide and adjust-value evaluation.
- [x] Implement finite arc-to-cubic lowering and path evaluation.
- [x] Export the approved module and run focused checks.

## Open questions

None. The three HLD-defined S13 module files were approved together.

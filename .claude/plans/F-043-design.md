# F-043, Gradient shading dictionaries

**Status**: approved
**Sprint**: S10
**Size**: L
**Depends on**: F-041

## Problem

`crates/oxml-pdf/src/writer.rs:710` renders only solid path paint. The private
`solid_color` helper at `crates/oxml-pdf/src/writer.rs:769` deliberately drops
linear and radial gradients, so the staged layout model can express paint that
the PDF backend cannot emit.

The writer also has no registry for the pattern, shading, and function objects
that a PDF gradient requires. Adding those objects during recursive emission
would make reference allocation depend on traversal details and would make it
hard to prove that page resources and content names stay aligned.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The PDF backend".
- `docs/hld/12-testing-strategy.md`, "oxml-pdf".
- `docs/hld/14-development-backlog.md`, "F-043, Gradient shading dictionaries".

## Approach

Add one private gradient registry inside `writer.rs`. Pre-scan path fill and
stroke paints in depth-first emission order, normalize every gradient's stops,
allocate deterministic names and references, and retain the accumulated group
transform needed by the shading pattern matrix. Normalization sorts offsets,
clamps them to `[0, 1]`, resolves duplicate offsets deterministically, and
leaves the existing one-stop degradation in `oxml-layout` unchanged.

For each linear gradient, write a type 2 axial shading. For each radial
gradient, write a type 3 radial shading. Both use DeviceRGB, the model's extend
flags, and a type 3 stitching function over one type 2 exponential function per
adjacent stop interval. Stop alpha is composited away as the documented v1
fallback because luminosity soft masks are outside this story.

Write one type 2 shading pattern per gradient occurrence with the approved
element-local matrix. Register only the names used by a page in its `/Pattern`
resources. During path emission, select `/Pattern` plus `scn` for fills and
`/Pattern` plus `SCN` for strokes, preserving the solid half of mixed paint and
repeating geometry when the two components need distinct state.

Keep every helper private in the existing writer file. Do not add a module,
dependency, public type, feature flag, or baseline update.

## Rejected alternatives

- Emit direct `sh` operators after clipping the path. Pattern colour-space
  operators preserve the existing path paint structure and work for strokes.
- Allocate resources while emitting content. PDF references and page resource
  names must be known before page dictionaries are written.
- Add gradient state to `oxml-layout`. The backend-neutral model already
  contains every geometry and stop value this writer needs.
- Implement stop-alpha soft masks. The rendering specification explicitly
  keeps that work outside v1.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `rotated_linear_gradient_renders_with_its_axis_rotated` | Poppler rasterization at the recorded version produces the expected sampled colours after a group rotation. |
| unit | `linear_gradient_writes_pattern_shading_and_stitching_resources` | A linear fill writes a type 2 pattern, type 2 shading, type 3 stitching function, and interval type 2 functions. |
| unit | `radial_gradient_writes_type_three_shading` | A radial fill writes type 3 shading coordinates and the declared extend flags. |
| unit | `gradient_stops_are_sorted_deduplicated_and_clamped` | Unordered, repeated, and out-of-range offsets produce deterministic valid bounds and encode values. |
| regression | `gradient_fill_and_solid_stroke_both_render` | A gradient fill does not suppress a supported solid stroke. |
| regression | `gradient_stroke_uses_pattern_stroke_operators` | A gradient stroke selects the named pattern with `CS` and `SCN`. |
| regression | `page_patterns_include_only_gradients_used_on_that_page` | Each page resource dictionary names only its own gradients. |

The backlog test gate is a rotated linear gradient rendering with its axis
rotated, asserted on sampled raster pixels.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- External oracle comparison. Follow `.claude/skills/differential-testing.md`,
  assert the existing recorded `pdftoppm` version before sampling pixels, use
  deterministic inputs, and record the exact version in verification evidence.

## Hash harness

Expected to remain unchanged. The staged backend is not a released sample
consumer. Do not update `scripts/hash_baseline.json`.

## Implementation checklist

- [ ] Add one private deterministic gradient registry in `writer.rs`.
- [ ] Normalize gradient stops without changing the public layout model.
- [ ] Write linear and radial shading dictionaries and stitching functions.
- [ ] Write shading patterns with element-local matrices and page resources.
- [ ] Apply gradient patterns to path fills and strokes.
- [ ] Add structural resource tests and the sampled rotated-axis gate.
- [ ] Update exactly the declared HLD files to current intent.
- [ ] Prove the hash and exact golden baselines remain unchanged.

## Open questions

None. The rendering specification fixes the PDF object graph, normalization,
matrix ownership, alpha fallback, and test gate.

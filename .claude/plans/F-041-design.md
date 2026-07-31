# F-041, Path rendering

**Status**: completed
**Sprint**: S09
**Size**: M
**Depends on**: F-039

## Problem

`crates/oxml-pdf/src/writer.rs:500` explicitly skips `Path` elements. The
staged layout model already carries move, line, cubic curve, and close commands
plus fill rules and stroke state, but none of that representation reaches the
PDF content stream. F-040 also needs the same geometry encoding for group
clips.

Gradient and tile paints exist in the model but their PDF resources are not
owned by this story. F-043 owns gradient shading dictionaries, so F-041 must
make solid paths work without inventing an incomplete resource format.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The PDF backend".
- `docs/hld/12-testing-strategy.md`, "oxml-pdf".
- `docs/hld/14-development-backlog.md`, "F-041, Path rendering".

## Approach

Add a private path geometry helper inside `writer.rs` that maps `MoveTo`,
`LineTo`, `CurveTo`, and `Close` to `m`, `l`, `c`, and `h`. Use it for visible
paths now and for F-040 clips later.

For solid fills and strokes, set RGB paint and stroke width, cap, join, the PDF
default miter limit of 10, and any dash array with phase zero. Select the final
paint operator from the supported components:

- Fill only uses `f` or `f*` from the fill rule.
- Stroke only uses `S`.
- Fill and stroke use `B` or `B*` from the fill rule.

Wrap each visible path in a balanced graphics-state save and restore. Treat
`Linear`, `Radial`, and `Tile` components as staged and unsupported until their
resource-owning stories. A supported solid component may still render when the
other component is staged. Do not add a diagnostic API, feature flag, module,
or dependency.

## Rejected alternatives

- Implement gradient dictionaries here. F-043 owns their functions,
  shadings, patterns, and matrices.
- Duplicate geometry emission in the group renderer. A single private helper
  keeps clipping and visible paths in schema order.
- Add a public path writer abstraction. Only the PDF writer consumes this
  encoding today.
- Add miter-limit or dash-phase fields to the layout model. There is no second
  producer requiring those public fields in this story.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `path_fill_only_emits_f` | A non-zero solid fill emits geometry followed by `f`. |
| regression, gate | `path_stroke_only_emits_s` | A solid stroke emits geometry and stroke state followed by `S`. |
| regression, gate | `path_fill_and_stroke_emit_b` | Supported fill and stroke share geometry and finish with `B`. |
| unit | `even_odd_paths_use_starred_operators` | Even-odd fill and combined paint use `f*` and `B*`. |
| unit | `path_geometry_preserves_command_order` | Move, line, cubic, and close map to `m`, `l`, `c`, and `h` in order. |
| unit | `stroke_state_maps_cap_join_miter_and_dash` | `w`, `J`, `j`, `M`, and `d` carry the model values and documented defaults. |
| regression | `staged_gradient_component_does_not_block_solid_component` | A solid half of a mixed path still renders without an invalid resource name. |

The backlog test gate is fill-only `f`, stroke-only `S`, and combined `B`.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- No table row adds an external rider. Run focused `oxml-pdf` tests, exact
  seven-sample golden comparison, dependency inspection, and the consolidated
  workspace verification required by the normal sprint gate.

## Hash harness

Expected to remain unchanged. The staged backend is not a released rendering
consumer. Do not update `scripts/hash_baseline.json`.

## Implementation checklist

- [x] Add one private geometry emitter in the existing writer.
- [x] Map solid fill and stroke state to PDF operators.
- [x] Select `f`, `f*`, `S`, `B`, or `B*` from supported paint components.
- [x] Leave gradient and tile resources to their owning stories.
- [x] Add the three backlog gate tests and focused operator tests.
- [x] Update exactly the declared HLD files to current intent.
- [x] Prove the hash and exact golden baselines remain unchanged.

## Open questions

None. F-041 runs before F-040 and owns the one path geometry emitter that
F-040 reuses for clips. The public backlog dependency remains F-039.

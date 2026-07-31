# F-044, ExtGState alpha

**Status**: completed
**Sprint**: S09
**Size**: S
**Depends on**: F-039

## Problem

The PDF writer uses RGB values for text, lines, and filled rectangles at
`crates/oxml-pdf/src/writer.rs:416`, `crates/oxml-pdf/src/writer.rs:457`, and
`crates/oxml-pdf/src/writer.rs:469`, but drops `Color.a`. Solid path paint will
have the same defect when F-041 lands. F-040 also needs named graphics state
resources to express group opacity.

Without one shared registry, each element or story could allocate duplicate
ExtGState dictionaries for the same alpha value and expose inconsistent names
in page resources.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Three remaining defects to fix" and "The
  PDF backend".
- `docs/hld/14-development-backlog.md`, "F-044, ExtGState alpha".

## Approach

Pre-scan the layout for non-opaque alpha values and allocate one document-wide
ExtGState per distinct normalized `f32` value. Normalize negative zero to zero,
clamp finite values to `[0, 1]`, and treat non-finite values as opaque. Give
each state a deterministic resource name and set both stroking and
non-stroking alpha so one state serves fills, strokes, and group opacity.

Expose the registry only through private writer helpers. Add the states used by
a page to that page's `/ExtGState` resources, write each dictionary once, and
emit `gs` before text, line, rectangle, and supported solid path paint when its
alpha is below one. Opaque elements do not allocate or emit a state.

When one path has different fill and stroke alpha, emit its geometry once for
fill and once for stroke so each paint operation selects the correct shared
state. Keep the combined `B` or `B*` operator when both alpha values normalize
to the same key.

Keep the Rust unit gate independent of an external Poppler subprocess. Assert
the PDF resources and operators structurally, then use the existing
deterministic tiny-skia raster path for a 50 percent rectangle over white to
assert the midpoint pixel. The exact seven-sample Poppler harness remains the
integration guard and must not move.

## Rejected alternatives

- Allocate one ExtGState per element. That violates the distinct-alpha reuse
  contract and grows large documents unnecessarily.
- Key raw `f64` values. The PDF API writes `f32`, so values that serialize
  identically must share a state.
- Invoke Poppler from Rust unit tests. The package tests should remain
  hermetic, while the existing golden harness already owns external PDF
  rasterization.
- Change the layout color model. It already carries alpha for every affected
  primitive.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `half_alpha_over_white_rasterises_to_midpoint` | A 50 percent solid rectangle over white produces the expected midpoint RGBA pixel in deterministic raster output. |
| unit | `same_alpha_reuses_one_extgstate` | Repeated equal alpha values create one dictionary and one resource name. |
| unit | `distinct_alpha_values_create_distinct_states` | Different serialized alpha values receive different states. |
| unit | `alpha_state_sets_fill_and_stroke_constants` | Each dictionary carries matching `ca` and `CA`, and content emits `gs`. |
| regression | `opaque_elements_emit_no_alpha_state` | Alpha one preserves existing content and resources. |
| regression | `solid_path_alpha_uses_shared_registry` | F-041 solid fill and stroke colors use the same registry as existing primitives. |

The backlog test gate is a 50 percent alpha fill over white rasterising to the
midpoint colour. The structural assertions prove the PDF-specific ExtGState
behavior, while the deterministic raster assertion proves the compositing
value without adding an external test dependency.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- No table row adds an external rider. Run focused `oxml-pdf` tests, exact
  seven-sample golden comparison, dependency inspection, and the consolidated
  workspace verification required by the normal sprint gate.

## Hash harness

Expected to remain unchanged. No current released sample uses non-opaque
colors through this staged backend. Do not update `scripts/hash_baseline.json`.

## Implementation checklist

- [x] Wait for F-041 so all supported solid paint consumers use one registry.
- [x] Pre-scan and allocate deterministic document-wide alpha states.
- [x] Write reused ExtGState dictionaries and page resource entries.
- [x] Apply alpha to text, lines, rectangles, and supported solid paths.
- [x] Add reuse, PDF structure, opacity, and midpoint raster tests.
- [x] Make the registry available to F-040 group opacity without a public API.
- [x] Update exactly the declared HLD files to current intent.
- [x] Prove the hash and exact golden baselines remain unchanged.

## Open questions

None. F-044 follows F-041 and precedes F-040, owning the shared ExtGState
registry used by solid path alpha and group opacity. Its gate combines
structural PDF assertions with the deterministic tiny-skia midpoint pixel,
while the existing exact Poppler harness remains the external integration
gate.

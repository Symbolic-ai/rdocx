# F-040, Group rendering

**Status**: approved
**Sprint**: S09
**Size**: M
**Depends on**: F-039

## Problem

`crates/oxml-pdf/src/writer.rs:500` explicitly skips every `Group`, so staged
PPTX content loses nested transforms, clips, opacity, and all grouped children.
The page-level CTM from F-039 established the coordinate system needed for
group matrices to compose directly, but the writer still has no recursive
content emission seam.

The group contract also overlaps two S09 stories. Clip paths need the geometry
emitter introduced by F-041, and group opacity needs the ExtGState registry
introduced by F-044. The implementation order must preserve those story
boundaries without duplicating either mechanism.

## Spec reference

- `docs/hld/08-rendering-spec.md`, "Why `Group` is the whole design" and "The
  PDF backend".
- `docs/hld/12-testing-strategy.md`, "oxml-pdf".
- `docs/hld/14-development-backlog.md`, "F-040, Group rendering".

## Approach

Refactor page content emission into one private recursive function that accepts
an element slice and writes into the current `Content`. Keep the existing
single page save, global flip, and restore outside that recursion.

For each group, emit a balanced sequence:

1. `q`.
2. The group's six affine coefficients through `cm`.
3. The clip geometry followed by `W n` or `W* n` when a clip exists.
4. The shared ExtGState name through `gs` when opacity is below one.
5. Recursive child content in document order.
6. `Q`.

Reuse F-041's private path geometry emitter for clipping and F-044's
document-wide alpha registry for opacity. Do not render `effects` in this
story. The current `OuterShadow` representation remains staged for later work.
Do not change the rasteriser, which is owned by F-045.

## Rejected alternatives

- Flatten groups with `walk()` for content emission. Flattening loses the
  graphics-state boundary required for clipping and opacity.
- Duplicate clip path operators inside the group arm. F-041 already owns the
  same `m`, `l`, `c`, and `h` encoding.
- Build opacity support directly into F-040. F-044 owns alpha reuse and the
  existing dropped-alpha defect.
- Add a renderer trait or a new module. There is one PDF implementation and
  the recursive helper is private to the existing writer.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `three_deep_groups_balance_graphics_state` | Three nested groups produce equal `q` and `Q` counts around their children. |
| unit | `group_emits_transform_before_children` | The six group coefficients appear in one `cm` before child operators. |
| unit | `group_clip_uses_declared_fill_rule` | Non-zero and even-odd clips emit `W n` and `W* n`. |
| unit | `group_opacity_uses_registered_graphics_state` | Opacity below one emits the shared ExtGState name before child content. |
| regression | `group_effects_do_not_change_pdf_output_yet` | Staged effects do not add unsupported operators or unbalance state. |

The backlog test gate is balanced `q` and `Q` counts in the content stream for
three-deep nesting.

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

- [ ] Wait for the approved path and ExtGState prerequisites in this sprint.
- [ ] Introduce private recursive element emission in the existing writer.
- [ ] Emit group save, matrix, optional clip, optional opacity, children, and
      restore in that order.
- [ ] Leave effects and raster group support staged for their owning stories.
- [ ] Add the three-deep balance gate and focused ordering tests.
- [ ] Update exactly the declared HLD files to current intent.
- [ ] Prove the hash and exact golden baselines remain unchanged.

## Open questions

None. Approved S09 execution makes F-041 and F-044 implementation
prerequisites for F-040 so clipping and opacity reuse their single owning
mechanisms. The backlog dependency on completed F-039 remains unchanged.

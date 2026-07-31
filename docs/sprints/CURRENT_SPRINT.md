# Current Sprint, S07

**Milestone**: M4 Layout primitives.

**Goal**: Extend the staged shared element model so it can express rotated,
clipped, gradient-filled shapes and content-addressed media without changing
released rdocx construction sites. Add one traversal seam that makes nested
group transforms explicit before the PDF backend migrates.

## Spec references

- `docs/hld/03-architecture.md`, for the format-neutral `oxml-layout` boundary
  and the rule forbidding dependencies on either document-format family.
- `docs/hld/08-rendering-spec.md`, for the exact path, paint, stroke, group,
  media, positioned-element, background, diagnostic, and traversal contracts.
- `docs/hld/11-migration-plan.md`, for the staged `PositionedElement` extension,
  unchanged released consumers, and deferred publication and cutover boundary.
- `docs/hld/12-testing-strategy.md`, for the nested-group traversal regression
  and the workspace, hash, no-default-features, WASM, docs, and package gates.
- `docs/hld/13-risks-and-open-questions.md`, for the group-blind collection-pass
  hazard that makes the shared `walk` helper mandatory before PDF migration.
- `docs/hld/14-development-backlog.md`, for the F-032 through F-036 contracts,
  dependencies, sizes, test gates, and the M4 unchanged-hash milestone gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-032 | Path and PathCommand | M | in-progress | codex |
| F-036 | MediaId | S | in-progress | codex |
| F-033 | Paint and Stroke | M | in-progress | codex |
| F-034 | Path and Group arms | M | pending | - |
| F-035 | The walk helper | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-032 and F-036 can begin independently from the completed S06 foundation.
F-033 builds paint and stroke on the path and content-addressed media models.
F-034 then combines the completed transform and paint work in the two new
positioned-element arms, and F-035 follows last because it traverses the nested
group representation.

## Definition of done for this sprint

- `Path` and `PathCommand` provide the four commands, fill rules, conservative
  control-point bounds, and rectangle, rounded-rectangle, and ellipse helpers.
- Paint supports solid, linear, radial, and tile forms, while `Stroke` owns its
  width, cap, join, and dash data and single-stop gradients degrade to solids.
- `PositionedElement` gains only the planned path and group arms, both enums are
  non-exhaustive, and page backgrounds and layout diagnostics are available.
- `walk` visits every leaf in a three-deep group exactly once with the correct
  accumulated transform, and `MediaId` deduplicates identical image bytes.
- Released rdocx source, manifests, construction sites, and rendered output are
  unchanged, while the staged crate stays at 0.0.0 with publication disabled.
- The full workspace and package gates pass with all 28 hash-harness entries
  unchanged, completing the M4 milestone without publishing a development
  crate.

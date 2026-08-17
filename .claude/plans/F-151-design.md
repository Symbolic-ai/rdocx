# F-151, Revision display in the renderer

**Status**: approved
**Sprint**: S48
**Size**: M
**Depends on**: F-149

## Problem

The typed paragraph model retains revision wrappers at their original run
boundaries (`crates/rdocx-oxml/src/text.rs:624`), and each wrapper exposes its
projected runs and revision kind (`crates/rdocx-oxml/src/revision.rs:27`). The
layout engine nevertheless walks only `para.runs` at
`crates/rdocx-layout/src/engine.rs:496`. It therefore omits inserted content
from the accepted view and has no way to include deleted content, apply tracked
decorations, or mark a changed paragraph in the margin.

The facade also caches one normal and one deterministic layout with no render
choice (`crates/rdocx/src/document.rs:535`). Existing PDF and raster methods
therefore cannot request a tracked-change view, while changing their default
would break the reader-oriented accepted-view contract.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-151, Revision display in the
  renderer".
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions",
  including the typed revision ownership tree.
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap", Word flow
  layout, and deterministic raster rendering.
- `docs/hld/10-bindings-spec.md`, native Word revision APIs and binding surface
  stability.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness", and
  "The golden-PNG gate".

## Approach

Add a public `RevisionView` enum with `Accepted` and `Tracked` variants and a
small concrete `RenderOptions` value whose default selects `Accepted`. Add
option-taking PDF and raster facade methods beside the existing methods. The
existing methods delegate to default options so their output and the Python,
WASM, and CLI surfaces remain unchanged. Options are passed by value and have
no builder.

Carry the selected view in `rdocx_layout::LayoutInput`. Build one ordered
paragraph run projection from ordinary runs and the typed revision wrappers at
each boundary. The accepted projection includes insertion and move-destination
content, omits deletion and move-source content, and uses the current modeled
properties. The tracked projection includes both sides, forces a single
underline on insertion and move-destination text, and forces a single strike
on deletion and move-source text while retaining all other direct and inherited
formatting. Nested wrappers are flattened once in their preserved document
order.

Mark a `ParagraphBlock` when its tracked projection contains a visible revision.
The paginator emits one solid change bar in the outside margin for the portion
of that paragraph placed on each page. The bar uses the existing neutral line
element and follows paragraph splitting without adding a backend-specific arm.
Property-only changes receive a change bar but do not replace the current
accepted formatting.

Cache only the default accepted layouts. Option-taking tracked renders compute
their own layout so a two-entry view cache does not multiply every font-mode
cache or create stale cross-view results.

## Rejected alternatives

- Clone the facade and call `accept_all` before every accepted render. Revision
  resolution is a package-level mutation path and would make rendering depend
  on serialization, reparsing, and facade state unrelated to layout.
- Parse the preserved raw revision XML again in the renderer. The typed
  projection already owns revision order, metadata, and run content.
- Add tracked-change variants to the PDF backend. Underline, strike, and margin
  bars already lower through format-neutral text and line elements.
- Cache every font-mode and revision-view combination. Only the default view is
  on the compatibility path, and the extra cache cases would add invalidation
  state without demonstrated need.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `revision_views_project_wrapped_runs_in_document_order` | Accepted and tracked projections include the correct insertion, deletion, move, nested, and ordinary runs once |
| unit | `tracked_revision_decorations_override_only_underline_and_strike` | Insertions underline, deletions strike, and existing color, font, emphasis, and unrelated decorations survive |
| regression | `a_split_changed_paragraph_draws_one_margin_bar_on_each_page` | Each page portion receives a finite outside-margin line without changing text placement |
| golden, gate | `both_revision_views_render_and_accepted_matches_resolved_document` | Both deterministic PNG views render, and accepted pixels equal the same document after revisions are accepted and removed |
| regression | `default_render_methods_keep_the_accepted_view` | Existing PDF, PNG, and page-layout methods equal their option-taking accepted counterparts |

The **test gate**, from the backlog, is golden. Both views of one document
render, and the accepted view is pixel-identical to the same document with
revisions accepted and removed.

Tests stay in existing crate-local modules and the existing `rdocx` regression
binary. Every pixel comparison uses bundled deterministic fonts at a fixed DPI.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

Record revision-view projection ownership, the additive render options, tracked
decoration and split-page change-bar behavior, binding stability, and the
deterministic two-view golden gate.

## Risk routing

- Layout, pagination, line breaking, and text shaping. Read HLD 08. Run all
  revision render checks with bundled deterministic fonts, run the golden PNG
  gate, and do not record a system-font baseline.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  concrete enum, options value, and option-taking native methods are additive
  and story-required. Run affected package dry-runs and archive size
  assertions.

## Hash harness

Expected unchanged across all 49 entries. Existing render methods keep the
accepted default, and no current sample contains modeled tracked revisions.

## Implementation checklist

- [ ] Add concrete revision-view options and additive PDF and raster entry points.
- [ ] Carry the selected view through `LayoutInput` without changing the default caches.
- [ ] Project ordinary and revision-wrapped runs in preserved paragraph order.
- [ ] Apply tracked insertion and deletion decorations without losing other formatting.
- [ ] Carry a changed-paragraph marker through pagination and draw split-aware margin bars.
- [ ] Add projection, decoration, pagination, default-compatibility, and deterministic golden tests.
- [ ] Run focused checks plus deterministic rendering and published-package riders.
- [ ] Update exactly HLD 03, HLD 08, HLD 10, and HLD 12 at completion.

## Open questions

None. The backlog fixes the two views and accepted default. Existing native
render method families establish the additive options surface, and the typed
revision projection establishes the ownership boundary.

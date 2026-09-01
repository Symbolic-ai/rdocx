# F-X071, Integrate PRs 61 through 64

**Status**: completed
**Sprint**: S62
**Size**: L
**Depends on**: none

## Problem

PRs [61](https://github.com/tensorbee/rdocx/pull/61),
[62](https://github.com/tensorbee/rdocx/pull/62),
[63](https://github.com/tensorbee/rdocx/pull/63), and
[64](https://github.com/tensorbee/rdocx/pull/64) add related Word reader facts
across drawing, table, numbering, revision, field, and facade code. PRs 61 and
64 have approved reader outcomes at current heads `7c40c2e` and `5cb5cba`.
PR 62 head `fa48a39` still loses inherited namespace bindings and can reorder
malformed row revision markers. PR 63 head `60bc663` still fails to use the
default paragraph style identity when associating a numbering level.

The contributor fork rejects maintainer pushes even though the pull requests
advertise maintainer modification. The safe integration route is therefore a
current-tree hardened equivalent that retains the contributor commits and
records the exact source heads and deviations.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions" and crate ownership.
- `docs/hld/04-opc-and-packaging.md`, namespace-aware parsing, raw XML
  preservation, and schema-order serialization.
- `docs/hld/10-bindings-spec.md`, native Word facade stability.
- `docs/hld/12-testing-strategy.md`, regression and round-trip gates.
- `docs/hld/14-development-backlog.md`, "F-X071, Integrate PRs 61 through 64".
- PRs 61 through 64 at source heads `7c40c2e`, `fa48a39`, `60bc663`, and
  `5cb5cba`.

## Approach

Build one maintainer-owned worker branch from the S62 claim base. Preserve the
contributor-authored commits where they apply cleanly, then add separately
labelled hardening commits so the contribution record distinguishes submitted
behavior from maintainer remediation.

Adopt PR 61's hyperlink target, external-image relationship, and drawing
safety facts with expanded-name relationship checks. Adopt PR 62's document,
table, border, row-grid, and formatting facts, then carry owner namespace
bindings through table, row, cell, content-control, and raw-child boundaries.
Retain unknown row properties and malformed insertion and deletion markers at
their original `CT_TrPr` schema positions.

Adopt PR 63's numbering metadata and effective paragraph and run formatting.
Keep numbering marker run properties separate from body-run formatting. Resolve
the final direct `numId` and `ilvl` before selecting level properties, honor
`numId=0`, and use one explicit-or-default paragraph style identity for both
style resolution and `w:lvl/w:pStyle` association. Keep the narrowed
`has_unmodeled_properties` contract for retained XML and attributes.

Adopt PR 64's insertion, preserved XML, complex-field display, and bounded
nested-revision facts only after auditing its current two post-approval commits.
The reader must reject excessive revision depth without a panic, preserve nested
revision projection, and keep field display segments in source order.

Do not add a crate, module, feature flag, trait, generic parameter, or dynamic
dispatch. Add regressions to existing source modules and integration binaries.
At sprint closure, record all four source PRs and authenticated contributor
`@pedroassumpcao`. Comment with the integrated commit and close or merge the
original records only through the repository's close-sprint route.

## Rejected alternatives

- Blindly merging the four GitHub branches bypasses the F-ID lifecycle and
  lands confirmed correctness defects from PRs 62 and 63.
- Pushing directly to the contributor fork is unavailable because GitHub
  rejects this maintainer account with HTTP 403.
- Reimplementing every contribution from scratch would discard useful commit
  attribution and obscure which behavior came from each source PR.
- Creating four new stories duplicates one tightly coupled Word reader review
  and verification boundary.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `linked_image_relationship_requires_office_namespace` | Foreign `link` lookalikes remain unsupported while aliased Office relationship attributes type correctly. |
| round-trip | `row_property_owner_bindings_survive_repeated_serialization` | Retained row-property XML keeps bindings declared on table, row, or property owners through save, reopen, and a second save. |
| regression | `malformed_row_revision_markers_keep_schema_slots` | Raw insertion and deletion markers remain on their respective sides of typed markers and preserve `CT_TrPr` order. |
| regression | `default_paragraph_style_selects_numbering_level` | Paragraphs with absent `pPr` or no direct `pStyle` use the default style for level association and merge the correct level properties. |
| regression | `effective_numbering_honors_direct_overrides_and_zero` | Direct `numId`, direct `ilvl`, and `numId=0` select or cancel level formatting atomically. |
| regression | `nested_revision_projection_is_bounded_and_complete` | Nested visible content survives within the declared depth, excessive depth rejects without panic, and namespace shadows do not gain revision semantics. |
| regression | `complex_field_display_segments_keep_order_and_properties` | Simple and complex field kinds and cached display segments retain source order and direct run properties. |
| round-trip | `contributed_reader_facts_survive_save_and_reopen` | Every adopted fact and retained raw subtree reopens with the same semantics and bytes. |

The **test gate** is regression. Run the complete `rdocx-oxml` and `rdocx`
suites plus every focused regression above.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Read HLD 04 and HLD 06. Add alias,
  inherited-scope, fixed-prefix, schema-order, raw-subtree, and repeated
  save-reopen regressions.
- **Public API of a published crate**. Read HLD 10 and the structural rules.
  The native `rdocx` reader methods and enums are additive pre-1.0 API. The
  low-level `rdocx-oxml` preservation fields are intentional pre-1.0 source
  breaks for exhaustive struct literals, consistent with the numbering
  preservation contract already recorded in HLD 10. Run rustdoc with warnings
  denied, inspect the API diff, run workspace publish dry-runs, and enforce the
  package archive size limit.

No dependency-graph, layout, font, unit, colour, binding, feature, oracle,
release-script, module, or file-move row is triggered.

## Hash harness

Expected unchanged across all 49 entries. These are reader projections and
fail-closed malformed-input repairs. Any output delta blocks integration.

## Implementation checklist

- [x] Pin and audit the four exact contributor source heads.
- [x] Adopt PR 61 relationship-safe hyperlink and drawing facts.
- [x] Adopt PR 62 document and table facts.
- [x] Propagate namespace bindings through every retained table owner boundary.
- [x] Preserve raw row properties and malformed revisions in schema slots.
- [x] Adopt PR 63 numbering and effective-formatting facts.
- [x] Resolve default-style numbering-level association.
- [x] Adopt and audit PR 64 revision and field facts at its current head.
- [x] Add every focused regression to existing test binaries or source modules.
- [x] Run focused suites, full verification, risk riders, and the unchanged hash harness.
- [x] Update exactly the listed HLD files and record all four contribution sources.

## Open questions

None. The user explicitly requested the four PR outcomes in S62, the source
heads are pinned, the fork write failure fixes the hardened-equivalent route,
and the remaining behavior is constrained by existing preservation contracts.

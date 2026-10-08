# F-X180, Correct cell nil and none border precedence

**Status**: completed
**Sprint**: S90
**Size**: S
**Depends on**: none

## Problem

`crates/rdocx-layout/src/table.rs:961` gives invisible cell borders an outer-edge exception and conflates nil with none. Both painting and border bands inherit this rule. Issue 272 documents seven incorrect direct and style-derived line sets.

## Spec reference

- `docs/hld/08-rendering-spec.md`, table-style cascade and line-breaking/page-transition rules.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness" and "The golden-PNG gate".
- `docs/hld/14-development-backlog.md`, "F-X180, Correct cell nil and none border precedence".

## Approach

Adopt PR 274 from `hadim` at `59d9de9426233da389369ee10f5c8bd32ca8f75c` against all of Issue 272.
Remove the outer-edge exception and its boolean parameter from resolved_cell_edge. Explicit nil suppresses the edge, explicit none inherits the corresponding table edge, and omitted cell edges inherit as before. Keep painting and row border bands on this concrete helper. Adopt the contributed focused test and dense-form expectation only after independent deterministic verification. Extend the existing regression entrypoint for all seven direct/style cases, line sets and border-band geometry. No XML normalization.
No new production file, module, trait, generic, crate or dependency.
Re-measure affected archives on the actual sprint inventory rather than copying
the PR's dated measurements. Pause the F-282 worker at a saved checkpoint before these independent fixes.
Implement and integrate F-X180, then F-X181, in separate waves while F-282
source and Cargo writes are stopped. Resume the preserved F-282 worktree
afterwards and reconcile overlapping files against both approved contracts.
F-283 retains its completion barrier on F-282 and follows both fixes.

## Rejected alternatives

Merging directly into main bypasses sprint close. Accepting only the contributed
test leaves reported nested, style or boundary variants unproven. Copying archive
measurements from the contributor does not describe the integrated source.

## Test plan

**Test gate**: regression. The seven reported direct and Table Grid variants, including a single unmodified corner and first-row top suppression, match pinned Word line sets. The contributed gate fails before the correction. The deterministic dense-form golden changes only the identified top segment, with unchanged row geometry. All 49 hash-harness entries remain unchanged.

| Category | Test | Asserts |
|---|---|---|
| regression | `cell_nil_removes_every_edge_and_cell_none_falls_back_to_the_table` | Contributed failure reproduces before correction and passes after |
| differential | Source-built Issue 272 variants | Pinned Word 16.113.2 native export, exact line-set or joined-text/relative-position checks |
| round-trip | Existing regression entrypoint extensions | Source tokens, unrelated XML and saved/reopened layout remain stable |
| regression | Existing layout and facade tests | Deterministic baseline geometry and neighboring unaffected cases |

## HLD impact

- `docs/hld/08-rendering-spec.md`

## Risk routing

- Layout and pagination: read HLD08, use deterministic bundled fonts and assert exact affected geometry. Declare and independently review the dense-form golden top-segment delta.
- External oracle: apply `.claude/skills/differential-testing.md`, pin Word 16.113.2 and record actual native input/export hashes. Compare topology and relative positions, with no absolute font-metric parity claim.
- Published behavior: signatures unchanged, corrective pre-1.0 rendering behavior. Run locally patched publish dry runs for rdocx-layout and rdocx and refresh archive measurements and size assertions.
- Workflow records: user explicitly approved design, review, progress and handoff files. No new production construct.

## Hash harness

Unchanged, all 49 deterministic entries. The separate dense-form golden loses the identified first-cell top line, changing its line count from 26 to 25 and its reviewed pixel checksum, with unchanged row geometry.
Never re-record a harness entry to conceal an unexplained difference.

## Implementation checklist

- [x] Authenticate native source-built issue variants.
- [x] Prove the contributed regression fails before implementation.
- [x] Adopt and inspect the contribution, extending full issue coverage.
- [x] Pass risk riders, scoped verification and zero-finding microscope.
- [x] Complete HLD, delivery records and contributor disposition evidence.

## Open questions

None. User explicitly added the issue and PR and approved workflow records.
Issue 264 remains excluded. GitHub closure waits for verified sprint close.

# F-X181, Ignore page and column breaks inside table cells

**Status**: completed
**Sprint**: S90
**Size**: S
**Depends on**: none

## Problem

`crates/rdocx-layout/src/engine.rs:9227` emits page and column inline items in table cells. The field-result path at line 9502 does the same. Issue 273 documents shifted cell text, excess row height and incorrect nested-cell behavior.

## Spec reference

- `docs/hld/08-rendering-spec.md`, table-style cascade and line-breaking/page-transition rules.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness" and "The golden-PNG gate".
- `docs/hld/14-development-backlog.md`, "F-X181, Ignore page and column breaks inside table cells".

## Approach

Adopt PR 275 from `hadim` at `5ea2f5710d4bb13d0182bd74417fa5ca58522984` against all of Issue 273.
Adopt the concrete in_table_cell context flag on the existing paragraph implementation, true only for layout_paragraph_with_source_in_table. Drop PageBreak and ColumnBreak inline items in this context, including field-result form feed and vertical tab, while retaining line breaks and body pagination. Existing table traversal supplies nested-cell context. Extend the contributed existing regression entrypoint for leading and nested cases, no-break row height, exact joined text and ordinary line/body controls. Preserve the authored XML.
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

**Test gate**: regression. Leading and inline page and column breaks in ordinary and nested cells match pinned Word text positions and no-break row geometry. Line-break and body page-break controls retain their existing behavior. The contributed gate fails before the correction. Source XML survives reopen, and all 49 hash-harness entries remain unchanged.

| Category | Test | Asserts |
|---|---|---|
| regression | `page_and_column_breaks_inside_a_table_cell_are_dropped` | Contributed failure reproduces before correction and passes after |
| differential | Source-built Issue 273 variants | Pinned Word 16.113.2 native export, exact line-set or joined-text/relative-position checks |
| round-trip | Existing regression entrypoint extensions | Source tokens, unrelated XML and saved/reopened layout remain stable |
| regression | Existing layout and facade tests | Deterministic baseline geometry and neighboring unaffected cases |

## HLD impact

- `docs/hld/08-rendering-spec.md`

## Risk routing

- Layout and pagination: read HLD08, use deterministic bundled fonts and assert exact affected geometry. Retain ordinary line and body page-break controls.
- External oracle: apply `.claude/skills/differential-testing.md`, pin Word 16.113.2 and record actual native input/export hashes. Compare topology and relative positions, with no absolute font-metric parity claim.
- Published behavior: signatures unchanged, corrective pre-1.0 rendering behavior. Run locally patched publish dry runs for rdocx-layout and rdocx and refresh archive measurements and size assertions.
- Workflow records: user explicitly approved design, review, progress and handoff files. No new production construct.

## Hash harness

Unchanged, all 49 deterministic entries. Table-cell break behavior changes only documents containing those breaks. Existing body break samples retain their output.
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

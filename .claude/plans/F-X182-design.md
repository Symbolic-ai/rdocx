# F-X182, Honor direct table alignment

**Status**: completed
**Sprint**: S90
**Size**: M
**Depends on**: none

## Problem

`crates/rdocx-layout/src/table.rs:504` clears the direct jc after the base-first cascade, so directly centered and right-aligned tables are rendered at the left margin.
Full reported acceptance is [Issue 277](https://github.com/tensorbee/rdocx/issues/277), reported by `hadim`. PR 279 subsequently supplies the correction, reviewed at `93749ddce266103b62a61e78c3b8a6b692a94646` from `hadim`.

## Spec reference

- `docs/hld/08-rendering-spec.md`, table-style cascade, table geometry and Word section geometry.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness" and "The golden-PNG gate".
- `docs/hld/14-development-backlog.md`, the matching story below.

## Approach

Adopt PR 279 only after independently proving its gate and complete Issue 277 acceptance. Remove the direct-alignment clearing while retaining the existing table-style cascade and placement calculation. Direct left, center, right, start and end values override style values. Keep authored indent precedence consistent with existing alignment semantics. Cover direct-versus-style conflicts, absent alignment, nested cells, right-to-left controls and save/reopen preservation. Do not couple this correction to legacy compatibility positioning, which is F-X183.

Implement in its own isolated wave after F-X181 while F-282 stays paused. Integrate and complete F-X182 at a scoped dependency checkpoint before claiming F-X183. Then resume the preserved F-282 worker and reconcile shared source/tests against all approved plans. F-283 retains its F-282 completion barrier.

## Rejected alternatives

A direct main merge bypasses sprint closure. Broadly shifting all tables breaks modern, centered and nested controls. Copying reporter coordinates without a fresh pinned oracle does not establish acceptance.

## Test plan

**Test gate**: regression. `direct_table_alignment_overrides_style_and_reopens` covers every reported variant using source-built fixtures in the existing regression entrypoint. Prove the gate fails before implementation.

- Differential: source-built Word controls cover mode 14, mode 15, mode 12, missing compatibilityMode and missing compat, direct alignments and style conflicts, table/cell margins, indentation, nested tables and right-to-left cases.
- Round-trip: preserve settings, alignment, margins and opaque producer XML through an unrelated edit and reopen.
- Regression: assert exact table offsets, text positions, row geometry, border positions and following body content using deterministic fonts.
- Verification: affected checks/tests, Clippy and format, archive/README gates, locally patched publication dry runs, and reviewed hash manifests. Final full sprint verification remains due.

## HLD impact

- `docs/hld/08-rendering-spec.md`

## Risk routing

- Layout and pagination: read HLD08. Use deterministic bundled fonts, exact table/text offsets and unchanged controls. Declare every hash delta before recording it.
- External oracle: apply `.claude/skills/differential-testing.md`. Pin Word for Mac 16.113.2 and authenticate actual input, saved DOCX and offline printing PDF hashes. Compare relative geometry, with no absolute Arial versus Caladea metric parity claim.
- Published behavior: corrective pre-1.0 rendering change. Re-measure affected archives, validate README inventory and run locally patched publish dry runs without upload.
- Unit conversion: read HLD01 and preserve truncating unit constructors. Use existing point conversion.
- New workflow files: explicitly approved by the user. No new production file, module, crate, dependency, trait or generic.

## Hash harness

Expected changes are the invoice page-one PNG and invoice/quote PDF page-content and byte entries. Their directly right-aligned totals tables move by 36 pt. Source inspection identifies the contract centered table too, but its full resolved width leaves no placement delta. Independently establish this exact set before recording, rather than copying the PR baseline. Source XML and PDF resource hashes remain unchanged. The pinned invoice golden changes only for the moved totals table. Identify the exact affected samples and entries from the frozen generator before baseline recording, bind the before/after manifests, and independently review the explained deltas. Other samples and unrelated geometry remain unchanged.
This story owns baseline movement only in its exclusive wave. No unexplained output delta may be recorded. Use a separate labelled behavioral commit with the exact expected delta stated.

## Implementation checklist

- [x] Authenticate full reported native controls and their source child order.
- [x] Prove the focused gate fails before implementation.
- [x] Implement every reported variant without disturbing controls.
- [x] Attribute and review exact deterministic hash deltas before recording.
- [x] Pass scoped verification, risk riders and zero-finding microscope.
- [x] Update HLD, handoff and delivery records at the appropriate checkpoint.

## Open questions

None. The user authorized all issues except 264 and explicitly approved these workflow records. Issue 264 remains untouched. GitHub closure waits for the verified sprint close.

# F-X183, Correct table margins and legacy positioning

**Status**: approved
**Sprint**: S90
**Size**: M
**Depends on**: F-X182

## Problem

Top-level tables always begin at their edge indent. Word compatibility modes below 15, including an absent setting, instead align left table cell text to the indent and extend right-aligned tables by the final cell margin.
Full reported acceptance covers [Issue 276](https://github.com/tensorbee/rdocx/issues/276) and [Issue 278](https://github.com/tensorbee/rdocx/issues/278), reported by `hadim`. PR 280 at `795b29d78d5c2ca49c1b414c9818de201fe36b4e` supplies both corrections and is stacked on PR 279.

## Spec reference

- `docs/hld/08-rendering-spec.md`, table-style cascade, table geometry and Word section geometry.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness" and "The golden-PNG gate".
- `docs/hld/14-development-backlog.md`, the matching story below.

## Approach

Adopt only PR 280 incremental changes after F-X182, not its duplicated PR 279 commits or unverified baselines. A left or right margin absent from the table, its style chain and default table style resolves to zero. Explicit and inherited margins remain authoritative. Distinguish bare styles, ordinary defaults, stylesWithEffects-only defaults and the dense-form case with authenticated native records. Carry the actual compatibility threshold from Document settings into layout with a concrete table-positioning fact. Reuse existing settings parsing and base-first resolved cell margins. Resolve eligible top-level compensation from the measured first-cell left margin and indent context. In the authenticated legacy right-aligned controls, increasing the table or first-cell left margin moves the whole table, while changing last-cell left or any right margin does not. Use the measured first-left dependency for these right/end cases, not the PR last-right inference. Preserve separate own-cell padding and table movement assertions, with no claim that Rust reproduces Word internal subpoint border quantization. Centered, modern mode 15 and nested table placement retain their measured behavior. Respect accepted row/cell traversal, margin overrides, authored indentation, bidiVisual and floating-table context rather than applying an unconditional shift. Do not reuse an unrelated tab or footnote flag as a table policy. A concrete LayoutInput field, if required, is additive and must be populated consistently by every caller. Preserve unknown XML and source settings. Do not normalize source XML. Source-built controls must distinguish absent resolved indent from explicit zero or style-inherited zero, because the first minimal native capture does not show an unconditional left shift. Reconcile the actual Word rule before implementation.

Implement in its own isolated wave after F-X181 while F-282 stays paused. Integrate and complete F-X182 at a scoped dependency checkpoint before claiming F-X183. Then resume the preserved F-282 worker and reconcile shared source/tests against all approved plans. F-283 retains its F-282 completion barrier.

## Native contract qualification

The minimal and official Grid contexts remain separate evidence sets. The official Grid style chain establishes explicit indent zero and default margins108. The legacy right-margin discriminator index is `285a9e7011fad4b44c44e9cf9075cbc3fa9277dfb5d6ef675ca86887d845e6e4`, and the unchanged right-margin override index is `86c995765fa531dacd0ab7ed87f4ca4f900ffcd80016fcff1c829a75efab743f`. These measured controls correct the reporter and contribution inference about which cell margin controls right placement. Full reported inputs still require acceptance, including the original default-margin right-aligned case.

## Rejected alternatives

A direct main merge bypasses sprint closure. Broadly shifting all tables breaks modern, centered and nested controls. Copying reporter coordinates without a fresh pinned oracle does not establish acceptance.

## Test plan

**Test gate**: regression. `legacy_table_positions_use_resolved_cell_margins` covers every reported variant using source-built fixtures in the existing regression entrypoint. Prove the gate fails before implementation.

- Differential: source-built Word controls cover mode 14, mode 15, mode 12, missing compatibilityMode and missing compat, direct alignments and style conflicts, table/cell margins, absent/default/bare styles, stylesWithEffects context, indentation, nested tables and right-to-left cases.
- Round-trip: preserve settings, alignment, margins and opaque producer XML through an unrelated edit and reopen.
- Regression: assert exact table offsets, text positions, row geometry, border positions and following body content using deterministic fonts.
- Verification: affected checks/tests, Clippy and format, archive/README gates, locally patched publication dry runs, and reviewed hash manifests. Final full sprint verification remains due.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`

## Risk routing

- Layout and pagination: read HLD08. Use deterministic bundled fonts, exact table/text offsets and unchanged controls. Declare every hash delta before recording it.
- Parser and package styles: read HLD04 and HLD06. Source-built main-style and stylesWithEffects-only defaults must retain ordered source XML and every opaque part. When the rendered style default exists only in the effects part, consume the proven fallback without replacing authored main styles or rewriting either part. Test missing, malformed, conflicting and unrelated effects content through save/reopen.
- External oracle: apply `.claude/skills/differential-testing.md`. Pin Word for Mac 16.113.2 and authenticate actual input, saved DOCX and offline printing PDF hashes. Compare relative geometry, with no absolute Arial versus Caladea metric parity claim.
- Published API: read HLD10. If LayoutInput gains a required public struct field, document the pre-1.0 source compatibility impact, update every struct literal and run affected binding/WASM checks.
- Published behavior: corrective pre-1.0 rendering change. Re-measure affected archives, validate README inventory and run locally patched publish dry runs without upload.
- Unit conversion: read HLD01 and preserve truncating unit constructors. Use existing point conversion.
- New workflow files: explicitly approved by the user. No new production file, module, crate, dependency, trait or generic.

## Hash harness

Expected changes are limited to PNG and PDF entries of existing samples with eligible top-level tables and missing or pre-15 compatibility mode. Zero fallback also moves previously unstyled cell text 5.4 pt left in generated samples. Dense-form and F-266c nested geometry goldens change to zero side padding and wider nested cells. The precise PNG/PDF entry set and both existing goldens must be authenticated independently before recording. Source XML and PDF resource hashes remain unchanged. The baseline starts after F-X182, so direct-alignment deltas remain separately attributed. Bind exact before/after manifests, verify each affected table offset and independently review every changed entry before recording. Modern, centered and nested controls remain stable.
This story owns baseline movement only in its exclusive wave. No unexplained output delta may be recorded. Use a separate labelled behavioral commit with the exact expected delta stated.

## Implementation checklist

- [ ] Authenticate full reported native controls and their source child order.
- [ ] Prove the focused gate fails before implementation.
- [ ] Implement every reported variant without disturbing controls.
- [ ] Attribute and review exact deterministic hash deltas before recording.
- [ ] Pass scoped verification, risk riders and zero-finding microscope.
- [ ] Update HLD, handoff and delivery records at the appropriate checkpoint.

## Open questions

None. The user authorized all issues except 264 and explicitly approved these workflow records. Issue 264 remains untouched. GitHub closure waits for the verified sprint close.

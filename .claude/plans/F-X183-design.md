# F-X183, Correct table margins and legacy positioning

**Status**: completed
**Sprint**: S90
**Size**: M
**Depends on**: F-X182

## Problem

Top-level tables always begin at their edge indent. Word compatibility modes below 15, including an absent setting, instead compensate eligible table placement using the measured first-cell left margin and resolved indentation context.
Full reported acceptance covers [Issue 276](https://github.com/tensorbee/rdocx/issues/276) and [Issue 278](https://github.com/tensorbee/rdocx/issues/278), reported by `hadim`. PR 280 at `795b29d78d5c2ca49c1b414c9818de201fe36b4e` supplies both corrections and is stacked on PR 279.

## Spec reference

- `docs/hld/08-rendering-spec.md`, table-style cascade, table geometry and Word section geometry.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness" and "The golden-PNG gate".
- `docs/hld/14-development-backlog.md`, the matching story below.

## Approach

Adopt only PR 280 incremental changes after F-X182, not its duplicated PR 279 commits or unverified baselines. A left or right margin absent from the table, its style chain and default table style resolves to zero. Explicit and inherited margins remain authoritative, subject to the separately qualified built-in TableNormal behavior below. Use the same absence fallback and direct-cell edge overlays in intrinsic width measurement and final cell layout. Removing the implicit 5.4 pt table fallback otherwise underestimates autofit cells whose explicit padding previously happened to match that fallback. Prove the direct-cell and equivalent table-margin measurement with a focused regression before correcting this narrow overlay, preserving all other autofit, spacing and border policies. Distinguish bare styles, ordinary defaults, stylesWithEffects-only defaults and the dense-form case with authenticated native records. Carry the actual compatibility threshold from Document settings into layout with a concrete table-positioning fact. Reuse existing settings parsing and base-first resolved cell margins. Resolve eligible top-level compensation from the measured first-cell left margin and indent context. In the authenticated legacy right-aligned controls, increasing the table or first-cell left margin moves the whole table, while changing last-cell left or any right margin does not. Use the measured first-left dependency for these right/end cases, not the PR last-right inference. Preserve separate own-cell padding and table movement assertions, with no claim that Rust reproduces Word internal subpoint border quantization. Centered, modern mode 15 and nested table placement retain their measured behavior. Respect accepted row/cell traversal, margin overrides, authored indentation, bidiVisual and floating-table context rather than applying an unconditional shift. Do not reuse an unrelated tab or footnote flag as a table policy. A concrete LayoutInput field, if required, is additive and must be populated consistently by every caller. Preserve unknown XML and source settings. Do not normalize source XML. Source-built controls must distinguish absent resolved indent from explicit zero or style-inherited zero, because the first minimal native capture does not show an unconditional left shift. Reconcile the actual Word rule before implementation.

Implement in its own isolated wave after F-X181 while F-282 stays paused. Integrate and complete F-X182 at a scoped dependency checkpoint before claiming F-X183. Then resume the preserved F-282 worker and reconcile shared source/tests against all approved plans. F-283 retains its F-282 completion barrier.

## Native contract qualification

The minimal and official Grid contexts remain separate evidence sets. Absent resolved indentation limits left/start compensation only, while the captured legacy right/end cases use the first-left margin even without tblInd. The official Grid style chain establishes explicit indent zero and default margins108. The legacy right-margin discriminator index is `285a9e7011fad4b44c44e9cf9075cbc3fa9277dfb5d6ef675ca86887d845e6e4`, and the unchanged right-margin override index is `86c995765fa531dacd0ab7ed87f4ca4f900ffcd80016fcff1c829a75efab743f`. These measured controls correct the reporter and contribution inference about which cell margin controls right placement. Full reported inputs still require acceptance, including the original default-margin right-aligned case.

The built-in default audit index is `bb5fbb8982419fe71449f499ff4a502b02cb512df2ceff71eb09fecda47ba4eb`. Its 108 bound files authenticate six sources, seven source/native byte-identical archive comparisons and ten PDF pages. With the fixed TableNormal identity and metadata, authored side values zero and 288 produce the same left text starts as the earlier 108 control, including an explicit style reference and a fresh Word session. Main-only and effects-only cases behave alike. Opposite main/effects values therefore do not establish a general part precedence rule. A renderer-only effective built-in left-default treatment consistent with these controls is required, preserving both source parts and ordinary selected-style/direct-cell cascades. Recognition features were not isolated, and short left-aligned words do not independently establish right padding, wrapping or whole-style replacement. Do not generalize the exception to arbitrary default styles or discard other built-in properties. Test ordinary custom defaults separately and document the bounded native discrepancy.

The custom-default discriminator index is `aae88f9f287b29a8f592e0c387751cbaf1a50e460f16e7d1fca106b4fba8b023`. Its 141 bound files authenticate two byte-identical source/native comparisons and four PDF pages. The same default flag, type and metadata with a distinct ID and name respect authored zero and 288 margins in both implicit and explicit selection. Built-in counterparts retain their earlier left text starts. All painted table extents remain equal. ID and name change together, so this distinguishes the tested custom identity from the built-in identity without isolating recognition axes. These short words still do not establish right padding or wrapping.

## Rejected alternatives

A direct main merge bypasses sprint closure. Broadly shifting all tables breaks modern, centered and nested controls. Copying reporter coordinates without a fresh pinned oracle does not establish acceptance.

## Test plan

**Test gate**: regression. `legacy_table_positions_use_resolved_cell_margins` covers every reported variant using source-built fixtures in the existing regression entrypoint. Prove the gate fails before implementation.

- Differential: source-built Word controls cover mode 14, mode 15, mode 12, missing compatibilityMode and missing compat, direct alignments and style conflicts, table/cell margins, absent/default/bare styles, stylesWithEffects context, indentation, nested tables and right-to-left cases.
- Accepted traversal: a parsed empty cell-level SDT in a leading accepted row must not hide the first actual laid-out cell in a later row. Preserve the empty row and source control ownership, with eligible legacy left/start/right/end and modern/nested controls.
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

The independently reviewed delta is exactly14 of49 entries. PDF bytes and page-content fingerprints change for contract, feature_showcase, invoice, proposal, quote and report. The invoice and quote page-one PNG entries also change. All35other entries remain exact, including all21selected XML parts, all7PDF resources, letter PDF and the other5page-one PNGs. All complete sample DOCX member bytes, PDF page counts and MediaBoxes remain unchanged. DELTA pass1 independently authenticates the bound inputs and approves recording this exact set.

Global zero side padding moves cell text5.4pt left and increases available content width10.8pt, including modern and nested content. Invoice fitting and quote wrapping, row heights and continuation placement therefore change. This is separate from top-level-only legacy table positioning. Modern, centered, nested, bidi and floating table placement controls retain their policy. Dense-form nested geometry shifts with its parent cell content, with reviewed FNV12522878329634332671 and nonwhite32421. F-266c changes only7table text geometry records, with reviewed digest2002409b412388ad17b7f85e170d36b8c3658e6c7e4cff84098220772c006b76. Neither fixture builder changes. Source preservation and every intentional pixel delta are bound in the DELTA review. The baseline starts after F-X182, whose alignment precedence remains independently asserted. Its legacy relative-center/right expectation changes arise from the shifted left control. Intrinsic direct-cell overlays, optional effects consumption and bounded built-in defaults have separate focused controls and are not inferred from these sample deltas.

The existing F-268a fixed/autofit/nested deterministic geometry pin is also affected by global zero padding. Supplemental DELTA pass2 binds genuine Base/current source and outputs and approves recording the exact geometry. Autofit columns change20.34to9.54 and242.28to231.48, with second-column x92.34to81.54. Nested declared225grid fits the zero-padded234host, changing111.6to112.5per column and x311.4/423.0to306.0/418.5. All6fixed rectangles and all14y/heights remain exact. Preserve all6fixed cell rectangles, every y coordinate and height, declared grid and autofit engagement policy. The source-built original reviewed geometry is not a bound native Word GUI capture, despite its historical test name. Exact intrinsic and nested width changes require independent attribution, with source parts unchanged.

The existing F-268a corpus-shaped width guard also pins the explicit-autofit positive arm. Supplemental DELTA pass3 binds unchanged test and source helpers plus genuine Base/current runs and approves recording20.34/215.19to9.54/204.39, removing10.8padding per intrinsic column. The absent-layout negative arm must retain declared72/360widths. This structural width control does not require PDF instrumentation and introduces no engagement or broader autofit policy change.

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

# F-X183, delta, pass 1

**Reviewed**: Frozen pre-recording working diff on `work/f-x183-codex`, Base and HEAD `d4b8f5458678d1640de9d2fb709655ab437ca98b`. Eleven tracked files, 854 added lines and 27 removed lines.

**Verdict**: 0 defects, 0 smells, 0 nitpicks within the DELTA scope. The exact fourteen hash changes, two sample golden changes and two existing fixture geometry changes are justified for subsequent reviewed recording. This is not the final ALL microscope, scoped verification or feature completion.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Frozen identity and provenance

The freeze is `/private/tmp/fx183-delta-freeze.json`, SHA-256 `9f7c00d55a3aa10ca193845e3ac6a56da2c05e2c127ceab7fb81c0cbff3f0de3`. Its complete patch digest is `1fa602e9f352703094196b25f99d58d526b2a06e3b3302d76b1b88496dce150b`. Independently verified all 141 bound source, baseline, receipt and artifact hashes, branch, HEAD, porcelain status and complete binary diff. Repeated the binding, status and diff checks immediately before this record. Both baseline files remain identical to Base.

Read the approved plan, its native evidence qualifications, workflow and microscope contracts, implementation diff and relevant HLD changes. Native observations distinguish minimal unstyled tables, official Grid inheritance, explicit or absent resolved indent, first-cell left versus right margins, built-in versus custom defaults, effects-only fallback and nested or RTL exclusions. Their relative geometry does not establish absolute bundled-font versus native Arial pixel parity.

The sample generator and both harness implementations remain byte-identical to Base. Both existing fixture builder spans also remain byte-identical to Base. Identical artifact-export instrumentation was added before old assertions and removed, with bound add/remove records. The detached Base is clean. Accepted current logs show actual current-source compilation and the increased regression inventory. Genuine Base fixture runs pass, while genuine current fixture runs fail against the unchanged geometry pins. Initial shared-target Base-binary reuse attempts are explicitly excluded from accepted artifacts. Touching inputs alone is not treated as proof of source freshness. The compilation records, current inventory and distinct measured outputs corroborate the corrected runs. No Cargo command was executed by this reviewer.

## Exact hash, package and PDF boundary

Independently invoked the unchanged read-only collectors on the frozen artifacts. Before equals the current complete 49-entry baseline. Exactly fourteen entries change:

- PDF bytes and page-content fingerprints for `contract`, `feature_showcase`, `invoice`, `proposal`, `quote` and `report`.
- `invoice:page1.png` and `quote:page1.png`.

The other 35 entries remain exact, including all 21 selected XML parts, all seven PDF-resource fingerprints, letter PDF and five page-one PNGs. Independently compared every ZIP member, not only the harness-selected parts, across all seven sample DOCX files and both fixture DOCX files. Names, ordering and member bytes are exact. PDF page counts and every page MediaBox are unchanged. Sample page counts remain 12, 6, 2, 2, 5, 2 and 5 in generator order. Each fixture remains one page. Every inflated resource stream remains exact. Changed inflated streams are page content.

All sample tables lack a selected table style, table indent, table side margins and cell margins. Their output changes follow the zero fallback at `crates/rdocx-layout/src/table.rs:570` and line 574. Removing 5.4 pt on each side increases available content width by 10.8 pt and moves the left text origin by 5.4 pt. These samples do not independently prove the effects consumer at `crates/rdocx/src/document.rs:26997`, the bounded built-in default exception, intrinsic direct-cell overlays or legacy placement at `crates/rdocx-layout/src/table.rs:918`. Those have separate native-bound and compiled controls.

Independently extracted ordered font-associated glyph bytes from all sample PDF text-show operations. Six samples retain exact ordered glyph sequences. Quote retains every font-associated glyph with identical multiplicity. A separate raw-text sequence comparison isolates its only non-whitespace reorder to the same three words, `3-Year Premium Support`, moving from the page-two continuation into the page-one description cell. This is supported by both pages' actual text and visual inspection, rather than a character multiset alone. No unexplained glyph deletion, duplication or content-order change was found.

## Exact raster and wrapping qualification

Independently decoded all fourteen sample golden PNGs without resizing, color conversion or tolerance. Dimensions remain 1275 by 1650. Five before/after RGBA arrays are exact. Invoice changes exactly 98,149 pixels within half-open bounds `[112, 178, 1008, 1504]`. Quote changes exactly 247,248 pixels within `[112, 400, 1155, 1512]`. These match the frozen report. The evidence uses pinned Poppler 26.01.0 at 150 DPI.

Viewed the full before and after invoice and quote page-one rasters and independently rendered quote page two in memory. Invoice cell origins and fitted title geometry change while source content and page count remain. Quote row three loses one wrapped line. Several other descriptions wrap at different words. The last description now fits on page one, eliminating its continuation fragment and moving the totals and following terms upward on page two. These are width, fitting, wrapping and pagination effects, not a translation-only qualification. Borders, totals, terms and acceptance content remain visible. No extra raster artifact was written.

## Existing fixture delta

Viewed both complete dense and F266c before/after rasters and independently decoded their original RGBA arrays.

Dense changes exactly 2,355 pixels within `[96, 100, 710, 178]`. The unchanged builder and package retain outer edges, page size, text and vertical geometry. Cell text, the nested table and its dependent foreground anchor shift left by 5.4 pt through cell-content geometry. Foreground stamp pixel count remains 1,624 and behind-text count remains zero. The reviewed FNV changes from `17727332437927583437` to `12522878329634332671`. This is distinct from adding legacy placement compensation to nested tables. The existing geometry gate begins at `crates/rdocx/tests/regression_test.rs:28781`.

F266c changes exactly 1,513 pixels within `[285, 201, 531, 274]`. The first fourteen canonical text records remain exact. The remaining seven table records retain text, glyph order and rotation coefficients. Horizontal `Across` moves from x 233.4 to 228.0. Vertical origins and rotation translations change consistently with the removed side padding. The geometry digest changes from `02995cf452d8add0ceb9c147d65773d55bc9f8b065b7a92b0e42073fdd770f7f` to `2002409b412388ad17b7f85e170d36b8c3658e6c7e4cff84098220772c006b76`. The assertion at `crates/rdocx/tests/integration_test.rs:21469` remains unrecorded and correctly rejects this intentional movement. Existing missing-glyph boxes are present in both rasters and are not newly introduced loss.

## Not found and remaining gates

Delta attribution, source and OOXML preservation, fixture provenance, baseline exclusivity, deterministic raster comparison and regression sensitivity produced zero findings. Correctness, contract, panics, tests and structure checks within this bounded attribution scope also produced zero findings. The named legacy gate fails on the pre-change behavior and passes on the corrected source. Intrinsic overlay and reusable-context controls have distinct bound evidence and are not inferred from sample pixels.

The old hash and geometry assertions currently fail intentionally. They must be recorded only through the approved workflow, with this exact delta stated. Final ALL microscope and the complete scoped gates remain required after recording. This pass neither authorizes unexplained additional changes nor claims native pixel parity or sprint completion.

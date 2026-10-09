# F-X182, delta, pass 1

**Reviewed**: Frozen pre-recording working diff on `work/f-x182-codex`, Base and HEAD `aab27aa3fe0278a73418490aed312d98ebb220d3`. Three files, 272 added lines and 9 removed lines. Production scope is seven removed lines in `crates/rdocx-layout/src/table.rs`. The remaining changes are 267 regression lines and the table-alignment HLD paragraph.

**Verdict**: 0 defects, 0 smells, 0 nitpicks within this delta-only audit. The exact five hash changes and invoice golden change are explained and qualified for local recording. This is not final all-aspect microscope, scoped completion, integrated sprint verification or GitHub closure approval.

## Defects

None found in the bounded delta review.

## Smells

None found in the bounded delta review.

## Nitpicks

None.

## Frozen identity and evidence

- Freeze identity `/private/tmp/fx182-provisional-delta-freeze.identity.json`, SHA-256 `9807f3eccbf58c10f1d360c6af0b77e868f244a68203acf09cee9f3032945385`.
- Exact patch `/private/tmp/fx182-provisional-delta-freeze.patch`, SHA-256 `175a06caa74d28fba1e6b06b8741d8033fd873b8ff08ec879f4613673e97cb6c`.
- Independently rehashed all 78 frozen source, receipt, artifact and baseline bindings. Verified branch, HEAD, porcelain status, complete working diff and numstat against the freeze. Rechecked frozen source and both baseline hashes immediately before writing this record.
- Both baseline files remain byte-identical to Base. The sample generator, hash harness and golden harness also remain byte-identical to Base. No contributor baseline was used as an expectation.
- Independently verified 287 unique recursive native evidence bindings from the minimal and official Grid indices. The native Word version is 16.113.2, build 16.113.26092012. Relative alignment evidence is distinct from absolute Arial versus bundled-font metrics and from F-X183 legacy positioning.

## Source and test attribution

`crates/rdocx-layout/src/table.rs:487` retains the existing base-first resolved table properties. Removing the separate direct-alignment clearing lets the existing center and right/end placement calculation at `crates/rdocx-layout/src/table.rs:545` consume the resolved direct value. Direct width treatment at line 497 remains intact. No compatibility, margin-default, RTL mapping, parser or serializer behavior is added by this production patch.

The source-built gate `direct_table_alignment_overrides_style_and_reopens` in `crates/rdocx/tests/regression_test.rs:56452` exercises 480 combinations of direct/style alignment, nested placement, RTL controls and five compatibility contexts, plus explicit indent and wide-margin checks. It observes text, real painted border and fill geometry, following content, row height, unrelated edit and reopen, exact table XML, settings and opaque XML. Its RTL and compatibility expectations explicitly preserve existing behavior rather than claiming F-X183 native parity. The bound reversion log fails at runtime with `0 != 162`, and the restored-source gate passes. The contributed regression likewise fails before the production change and passes after it. No Cargo command was run during this review.

`crates/rdocx/examples/generate_all_samples.rs:1162` and line 1381 author the quote and invoice totals tables with direct right alignment. Their resolved grids leave 36 pt of free width. The unchanged generator therefore supplies a concrete trigger for the observed translation. The centered contract table has no free width and its complete output remains unchanged.

## Exact hash and PDF delta

Independently called the unchanged read-only hash collectors on the frozen before and after artifacts and matched both complete manifests. Exactly these five of 49 entries change:

- `invoice:page1.png`
- `invoice:pdf/bytes`
- `invoice:pdf/pages`
- `quote:pdf/bytes`
- `quote:pdf/pages`

All 44 other entries remain exact, including every source XML and PDF resource fingerprint. Every sample DOCX archive is unchanged. The before harness reports all 49 entries matching. The unrecorded after harness fails on precisely the five declared entries.

Independently decoded and compared every PDF stream for invoice and quote. Invoice objects 7 and 8 contain 78 changed x coordinates. Quote object 8 contains 77. Every changed line retains its operator and all other operands. All other decoded stream bytes and y coordinates remain exact. Decimal x deltas are 36 pt except the retained `35.99999` serialized values. The maximum 0.00001 pt discrepancy is a bounded artifact observation, not a widened production, native-oracle or golden acceptance tolerance.

## Exact golden delta and visual inspection

Decoded every before and after golden PNG into its original RGBA buffer without resizing, color conversion or tolerance, and independently matched all recorded buffer digests. All dimensions remain 1275 by 1650. Only invoice changes, with exactly 6,853 pixels in the half-open rectangle `[112, 1430, 1163, 1526]`. The six other decoded buffers, including quote page one, are exact.

Viewed both full-page invoice golden rasters independently. The subtotal and tax rows move right together, matching the decoded stream attribution. The rest of the page remains unchanged. The quote PDF change occurs beyond its unchanged page-one golden. No additional diff image was written. The exact in-memory RGBA comparison establishes the changed region and unchanged remainder.

The bound golden evidence uses pinned Poppler 26.01.0 at 150 DPI and deterministic generated PDFs. It qualifies the invoice manifest update only, with the decoded before digest `3c615dc63e5ca969418b75d99addd4a5742a3c8d7dcc85560619956c337fe4fa` and after digest `4e6e4ab7fbbbb040629853b853cc0a9a4ac78d49fedcc97538cbe98ca1314337`.

## Not found and remaining gates

Correctness, contract, panics, OOXML preservation, regression sensitivity and structural checks produced zero findings within this delta-only scope. No new trait, generic, dependency, production module or file is introduced. F-X180 and F-X181 production changes remain in the frozen Base and are not modified by this patch. This statement does not substitute for their later integrated regression checks.

Local recording must use the exact reviewed five-entry set and invoice-only golden delta with a stated behavioral reason. Final microscope must review the resulting complete diff after baseline writes, package and archive riders, scoped verification and any later edits. Full integrated sprint verification and review remain due. The review changed only this file and ends here.

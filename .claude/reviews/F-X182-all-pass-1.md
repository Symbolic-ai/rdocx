# F-X182, all, pass 1

**Reviewed**: Complete frozen working diff on `work/f-x182-codex`, Base and HEAD `aab27aa3fe0278a73418490aed312d98ebb220d3`. Nine tracked files, 288 added lines and 25 removed lines. Production changes remove seven lines. The approved prior delta review is additional evidence, not a substitute for this all-aspect pass.

**Verdict**: 0 defects, 0 smells, 0 nitpicks. All six microscope aspects pass for this scoped feature checkpoint. Integrated sprint verification, sprint review and GitHub closure remain due.

## Defects

None found.

## Smells

None found.

## Nitpicks

None found.

## Frozen identity and contract

Read repository guidance, canonical workflow and microscope command, the approved F-X182 plan, current in-progress tracker rows, progress note, prior delta review and referenced HLD table cascade, units, testing and backlog sections.

Verified exact branch, HEAD, status, numstat and complete working patch. Independently rehashed all 93 bound files, receipts and artifacts, then repeated the binding and patch check before writing this record.

- Final identity: `/private/tmp/fx182-final-review-freeze.identity.json`, SHA-256 `e0962061833186c78f2de6dcbb6980270779a08ea973b86573076fd055e72946`.
- Final patch: `/private/tmp/fx182-final-review-freeze.patch`, SHA-256 `4fca540926f699c91a7739fba20a840257ddca22b23fac6f827fc5d267b0f7ba`.
- Prior delta review: `.claude/reviews/F-X182-delta-pass-1.md`, SHA-256 `33f8994050676df562dfe6e85d54f4b36b3901b179c265464ac560ea1a1d449d`.

Production, regression and rendering HLD bytes are identical to the independently audited provisional freeze. The final changes add reviewed baseline recording, two measured archive rows and their existing policy tuples, and implementation checklist ticks.

## Not found, by aspect

- **Correctness: zero findings.** `crates/rdocx-layout/src/table.rs:487` retains the base-first style and direct-property resolution. The removed clearing lets the existing placement calculation at line 545 consume direct alignment. Direct width handling at line 497 is unchanged. Center uses half the nonnegative free width, right/end uses all of it, and other values retain authored indentation. No compatibility positioning or margin-default correction is introduced.
- **Contract: zero findings.** The seven-line removal satisfies direct alignment precedence without absorbing F-X183. `docs/hld/08-rendering-spec.md:978` states the resulting behavior. The gate explicitly distinguishes relative placement from absolute native font metrics and preserves existing RTL and compatibility behavior as controls. No unrelated feature or dependency change appears.
- **Panics: zero findings.** No new production indexing, unwrap, slicing, arithmetic or failure path is introduced. Added test unwraps and indexing operate on source-built fixtures with asserted page and paint presence.
- **OOXML: zero findings.** Production works on the cloned resolved table and does not change parsing or serialization. Fixture child order places bidiVisual before width and alignment. The test retains exact authored table XML, settings and opaque producer XML through an unrelated edit and reopen. Pure layout leaves serialized source unchanged.
- **Tests: zero findings.** `crates/rdocx/tests/regression_test.rs:56457` exercises 480 direct/style, nesting, bidi and compatibility combinations plus indent and margin controls. It measures text, painted borders and fills, row height, following content and reopened state. The compiled source-reversion receipt fails at runtime with `0 != 162`, and the restored gate passes. The contributed regression at line 56433 also fails before and passes after the correction. Native provenance remains bounded to authenticated Word 16.113.2 build 16.113.26092012 relative controls.
- **Structure: zero findings.** No trait, generic, wrapper, module, crate, dependency or feature flag is added. Concrete regression helpers stay in the existing integration entrypoint. No layering or published API changes appear.

## Recorded behavior and baseline provenance

The final hash entries exactly equal the independently collected after-manifest. Exactly five of 49 entries differ from Base: invoice page-one PNG, invoice PDF bytes and pages, and quote PDF bytes and pages. All 44 other entries, including source XML and PDF resources, remain exact. Only invoice changes in the seven decoded golden buffers, with the reviewed RGBA digest and unchanged dimensions and rasterizer identity. Reasons name F-X182 and the 36 pt totals-table translation.

The prior independent delta pass decoded all invoice and quote streams, established 78 and 77 changed x coordinates respectively, and independently viewed both full invoice rasters. Every other operand and stream remained exact. The retained serialized 0.00001 pt representation difference did not widen any production or golden tolerance. Only 6,853 invoice pixels changed within the reviewed half-open rectangle `[112, 1430, 1163, 1526]` at 150 DPI. Final frozen delta artifacts still match that pass. No contributor baseline was adopted.

## Scoped verification and publication evidence

Inspected the bound final receipts without running Cargo. They report facade suites of 500, 360 and 867 passes, CLI suites of 3 and 58, layout 317, and three doctests, with their recorded existing ignores. Three-crate warnings-denied Clippy and documentation pass. The hash check matches 49 entries and pinned Poppler 26.01.0 golden check matches seven buffers. Workflow policy passes 140 tests with two existing skips, including 27 README and 22 package inventory validations. Format and diff checks are empty and successful, prose has zero violations, and 26 generated adapters match.

Both actual locally patched publish dry runs verify the packaged crates and abort upload. Bound measurement receipts record rdocx at 1,520,783 compressed bytes, 8,416,471 normalized member bytes and 36 members, and rdocx-layout at 308,929, 1,671,932 and 15. The two README rows and measurement tuples match these receipts.

Independent inspection of the current `tmp-crate` archives confirms every archived source/test member and README equals the worktree. Current rdocx measurements exactly match the recorded tuple. Current rdocx-layout compressed size is 308,928, with exact recorded normalized member bytes and count. This one-byte observed difference is within the unchanged 64-byte compressed archive policy at `scripts/readme_doctests.py:394` and its validator at line 1336. No archive-byte identity or unverified cause for that difference is asserted. Both archives remain below 10 MiB.

This review qualifies the declared scoped checkpoint only. It does not claim final workspace verification, F-X183 acceptance, publication or issue closure. No source, baseline, scratch, ledger or queue file was modified, and no Cargo or native UI action was performed. The review ends here.

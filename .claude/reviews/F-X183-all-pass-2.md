# F-X183, all, pass 2

**Reviewed**: Frozen working diff on `work/f-x183-codex`, Base and HEAD `d4b8f5458678d1640de9d2fb709655ab437ca98b`. Sixteen tracked files, 966 added lines and 72 removed lines, plus four retained review records.

**Verdict**: 0 defects, 0 smells, 0 nitpicks. ALL pass 1 D1 is resolved. This is feature-scoped review, not final integrated sprint verification.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Identity and remediation

Authenticated `/private/tmp/fx183-all2-freeze.json`, SHA-256 `890efe77f544ca1ea4306c36416249cd3178587bbf304e35c4f98181f15a568b`, and complete binary diff `7b710ac28c7936d3869d53daea38a9ca9b4b0a70dcab29292755cb18cc672913`. Independently rehashed all 101 direct bindings, including all 44 retained ALL1 bindings, and verified branch, HEAD, exact status and diff. Rechecked bindings and diff immediately before writing. Read the workflow, microscope contract, approved plan, progress and HLD04/08 changes. Prior complete source review and three independent DELTA records remain applicable to unchanged code and artifacts.

The only production change since ALL1 is `crates/rdocx-layout/src/table.rs:921`: `rows.iter().find_map(|row| row.cells.first())` replaces selection from row zero alone. This traverses the already accepted laid-out rows, skips only rows without a projected cell and borrows the first real cell. It neither removes source rows nor changes cell-control projection. Empty tables still contribute no compensation. Existing top-level, compatibility, bidi, floating, alignment and resolved-indent guards remain unchanged. No new counter, indexing or panic path is introduced.

The concrete regression at `crates/rdocx/tests/regression_test.rs:56516` inserts a parsed empty cell-level SDT row before the real cells. Sixteen combinations cover legacy and modern modes, top-level and nested placement, and left/start/right/end. It compares the painted edge with the corresponding unchanged actual-cell fixture, checks resolved padding, exact document XML, pure-save bytes and reopened geometry. The authenticated before run genuinely compiled and failed at runtime with legacy-left edge72 versus66.6. The corrected production run compiled and passed all sixteen combinations in1.16s. This directly closes the original valid-source defect without deleting the empty row or weakening preservation.

## All-aspect source and contract assessment

Global absent side padding remains zero in intrinsic and final layout. Direct intrinsic edge overlays preserve explicit zero and agree with final cell margins. Resolved style indentation remains distinct from missing indentation. Legacy compensation consumes the first accepted cell's left margin, and nested, floating, bidi, center and modern controls retain their policy. The exact built-in TableNormal recognition changes only its effective base left edge. Derived and direct overlays still win, and custom identities retain authored values.

Effects-default consumption remains render-only on an owned style clone. It resolves a sole internal relationship relative to the actual main part, requires a guarded complete Word style-owner projection with unique IDs and accepts only a sole self-contained noncolliding default when main styles lack a table default. It does not merge arbitrary effects catalogues or rewrite either source part. Existing invalid, ambiguous, external, relocated and namespace-alias controls remain covered. Source XML, relationships, settings and opaque parts retain their preservation contract.

The required public LayoutInput compatibility field, all consumers, retained-engine key and intentional pre-1.0 struct-literal impact remain documented and unchanged since ALL1. HLD04 and HLD08 describe current behavior. No new dependency, crate, module, file, trait, generic or speculative abstraction is introduced by production changes.

## Native and baseline evidence

Independently rehashed all 491 unique native evidence bindings with zero errors. Their bounded source/style/compatibility distinctions remain qualified, without claiming absolute native Arial versus deterministic bundled-font parity. No new native capture or renderer comparison was run during this review.

The three DELTA reviews retain their exact hashes and conclusions. The fourteen changed harness entries and35 unchanged entries remain the entire recorded sample delta. Only invoice and quote golden identities change. Builders, source ZIP members, resource streams and the reviewed dense, F266c, F268a and explicit-autofit geometry controls retain their prior authenticated attribution. D1 remediation changes no baseline or tolerance. Fresh hash49 and golden7 checks pass.

## Verification and publication limits

Read the actual bound refreshed logs. Layout passes319 unit tests and one doctest. The new sixteen cases, named600-fixture gate and inherited24 controls pass. Scoped check, Clippy and formatting pass. Workflow tests pass140 with two skips in55.689s, including fresh README/archive policy checks. Prose reports zero violations and26 adapters are in sync.

Prior unaffected facade, CLI, binding, WASM and warnings-denied documentation evidence remains explicitly qualified by the selection-only remediation. Facade evidence is500 unit and360 integration passes, plus the original869-pass regression attempt and separately corrected sole-target pass. This is not a fresh full870-test invocation. Full workspace and final sprint gates remain due. No Cargo command was run by this reviewer. The freeze's retained measurement-window endpoint is an earlier bounded interval, not the ALL2 freeze timestamp or total effort.

Independently opened all four refreshed retained archives and checked every archived source/test member against the frozen worktree. Measured layout tuple is `(310871, 1681340, 15)` and actual publish tuple is identical. Measured facade tuple is `(1526086, 8447574, 36)`, while actual publish is `(1526084, 8447574, 36)`. The existing VCS normalization was applied exactly. Raw member differences between each measured/published pair are only README.md. All source/test bytes match, member totals match and archives are below10MiB. The two-byte facade compressed difference lies within the unchanged64-byte policy. Its compression cause is not established or inferred. Actual package/tmp-crate provenance and retained copies are separately authenticated. Publication logs explicitly abort upload because these are dry runs.

## Not found and handback

- **Correctness**: zero findings. D1 is resolved by accepted-row traversal.
- **Contract**: zero findings. Bounded defaults, placement and source preservation match the approved plan.
- **Panics**: zero findings. The remediation adds only guarded iteration and borrowing.
- **OOXML**: zero findings. No parser or serializer change, source-row deletion or ownership rewrite is introduced.
- **Tests**: zero findings. Genuine reversion failure and restored success cover D1, and refreshed affected checks retain the stated scope.
- **Structure**: zero findings. Existing concrete consumers suffice without new abstractions.

This review ends here. Only this review record was written. No source, plan, baseline, HLD, progress, queue or native artifact was changed. Return to the sprint orchestrator for feature completion and integration.

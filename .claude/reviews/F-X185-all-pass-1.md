# F-X185, all, pass 1

**Reviewed**: Working diff against Base and HEAD `182636b02ea77dfb63b67eb5c4ecca4810c846ba` on `work/f-x185-codex`. The frozen diff contains 18 tracked files, 1,789 insertions and 78 deletions. Independent source phases and final evidence audit cover all six microscope aspects. The worker released all writes and Cargo before this record.

**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Authenticated identity

Final freeze `/private/tmp/fx185-final-freeze.json` SHA-256 `a35874ba638ca30307f4dbc3cd3beb122670ff4d12d9dd34bbd33bb1c307f1b7`. Exact ordinary binary diff SHA-256 `d3350241ba652021dc5dc77cea3964d46a356da82f5888ff939cbe6ad58c718a`. Independently rehashed all 18 tracked paths and 78 artifact bindings, checked branch, HEAD and exact pre-review status, and compared the current diff. All match.

Read the approved plan, CLAUDE, WORKFLOW, microscope and risk-routing contracts, cited HLD03, HLD04, HLD10 and HLD12, and parser/serializer HLD06 rider. Original source binder `ef083d539038a31a204c0abff6ac9f781b1c5a7b31980008ec38f8f03583c4a5`, lint2 binder `5af7ee50093bdb9b816ce874e2419b356894b20a9e17e12da0b6379bd6e7091d` and Issue289 binder `0f54a7c4fb01c01e8d7ef995274d0646942f1424618be1516fe16a98e316423a` establish the staged source review. Removing the three exact lint allowances recovers the original source bytes. Issue289 incremental diff `016331ff42c08a5fde75ea1d6390018fe8de4d235d128a810c45fba5327f4020` changes only the Python paragraph snapshot consumer and its existing test assertions. Final additional changes are reviewed measurement and HLD metadata.

## Not found

- **Correctness**: Zero findings. Checked selected-id source projection, continuation and one-newline paragraph joining, accepted UTF-8 display offsets versus accepted run indices, duplicate/reversed/hidden/mid-run refusal, field and wrapper display, qualified owner selection and nested cell/textbox separation. The projection verifies exact selected display and endpoint offsets after private re-anchoring. Multi-paragraph insertion validates both original source endpoints before publishing either. Python and CLI use owned checked endpoint snapshots, including block-control paths. The Issue289 paragraph constructor now uses the same checked actual paragraph snapshot rather than synthesizing empty XML.
- **Contract**: Zero findings. Document and CommentRef checked APIs preserve metadata ordering and distinguish unknown, orphan, reference-only point and reply states. Shared graph proof validates actual definitions and all companion linkage. Readers tolerate only raw-proven undefined marker identities without inventing records, while existing destructive and CLI validation consumers retain the strict wrapper. Zero-run wrapper-only intervals have the specifically approved representational-error boundary, not fabricated orphan/point semantics. The two hidden batch/snapshot helpers each have actual Python and CLI consumers. Seven-field Python construction and original metadata equality remain compatible. Owned anchor snapshots stay readable after mutation and stale reuse refuses. Typed CLI endpoints include normalized story, owner, path, paragraph kind, run boundary and containing body index.
- **Panics**: Zero findings. Checked new range slicing, marker span removal, map lookups, optional continuation and insertion validation against their construction invariants. The owner paragraph binary search indexes an entry registered in the same local inventory. Checked part expects follow relationship and existence proof. No new untrusted unchecked identity allocation or raw-count fallback is introduced.
- **OOXML**: Zero findings. Qualified namespace identity, original source scope closure, raw marker provenance, opaque boundary refusal and schema-order preservation were inspected. All reader projections operate on private serialized paragraph candidates and do not publish package edits. Alias, foreign lookalike, opaque payload and unchanged-source controls retain their exact source assertions. No broader wrapper deletion, namespace pruning or foreign-attribute rewriting from PR287 is imported.
- **Tests**: Zero findings. Named positive gates, real compiled before failures, source retention, reopened owner reuse and atomic stale/error cases are present in existing entrypoints. Both Issue289 constructor variants genuinely fail before the consumer repair and pass afterward. Scope and runtime qualifications below distinguish actual final evidence from preceding evidence. No new test binary or substituted production-generated expectation is introduced.
- **Structure**: Zero findings. Changes remain in existing concrete files and follow the approved exact helper signatures. No new trait, generic parameter, crate, module, dependency or feature flag is introduced. The manual CommentRef Debug retains public Debug plus Clone/Copy without requiring Document Debug. Three narrow lint allowances document the existing concrete signatures. The accepted-axis gap is bounded rather than widened through another object model.

## Executable evidence and limits

Independently authenticated all logs and source bindings in the three scoped receipts: `/private/tmp/fx185-20261008T162742Z/receipt.json` has 13 successful steps, `164221Z/receipt.json` has nine, and `164344Z/receipt.json` has eight. No builds or tests were run by the reviewer.

The native receipt records rdocx 501 unit, 364 integration, 894 regression and two doctest passes with 21 existing ignored tests. rdocx-oxml records 628 tests plus one doctest. CLI records three unit and 61 integration passes. Focused source matrices independently bind 11 native and seven CLI passes. All-target checks, affected all-feature Clippy, denied-warning docs and both WASM binding checks pass. Hash harness49 is unchanged and no baseline recording is authorized or present.

Before evidence contains an actual compiled native accepted-text failure, `DeltaFOREIGN` versus `Delta` followed by tab and newline, CLI null versus required anchor_text, and a built Python missing-anchor-field AttributeError. The wrapper-only source evidence records the original display versus unfaithful re-add on the old zero-run axis. These are executed failures, not merely missing test selections.

The first scoped Python parity35 pass, 459.99 seconds with one existing warning, and two pinned Poppler/concurrency passes precede only the bounded Issue289 constructor repair. They are not claimed as rerun after that repair. Current rebuilt Python core93, affected all-feature lint/check/docs, strict mypy, stubtest and actual Issue289 runtime pass after the repair. The exact same existing parametrized constructor assertions failed twice beforehand. Current extension SHA-256 `0927e5f9f901b2afde90d2a2350547029fc92fa6e6fcc9bffb851553c182a03b` is retained and bound to its actual module/build provenance. Native production and test bytes are unchanged across that Python-only repair.

Final README validation covers 27 workspace READMEs and 22 publishable inventories, four actual root Rust examples and one actual oxml Rust example. Workflow140 passes with two skips in 87.951 seconds. Prose reports zero violations, all26 generated adapters are in sync and fmt passes. The remainder receipt records selection of the actual Rust README consumers and inventory-only treatment of CLI/Python cases. It does not establish an earlier failed X185 README run.

## Published-package and measurement checks

Independently opened all six retained measured and actual publish-dry-run archives, recomputed existing normalized VCS measurement policy, and compared every archived source/test member to frozen worktree bytes. The three packages have respectively 27, three and 25 exact source/test members in each archive. All compressed archives remain below10MiB.

| Package | Recorded compressed bytes | Actual publish compressed bytes | Normalized member bytes | Members |
|---|---:|---:|---:|---:|
| rdocx-oxml | 443671 | 443672 | 2763759 | 32 |
| rdocx-cli | 72087 | 72087 | 321234 | 8 |
| rdocx | 1543654 | 1543655 | 8549973 | 36 |

The one-byte compressed differences do not establish an exact byte-identical archive claim. They satisfy the unchanged64-byte existing policy, while normalized member sizes/counts and all actual source/test payloads are exact. No cause for the compressed differences is inferred. The frozen measurement script changes only the three package tuples, and corresponding README rows match. Actual locally patched verified publication logs explicitly abort upload due to dry run for all three packages. No publication occurred.

This is the feature ALL review for Issues283 and289. It does not approve PR287 as a whole, its conflicting removal alternatives, or Issue288's separately recorded destructive-route work. Full integrated sprint verification, sprint review, integration/handoff and GitHub closure remain orchestration steps. No new native Word or aggregate render parity claim is made. Issue264 remains untouched.

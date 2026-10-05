# S89 sprint review, pass 1

**Reviewed**: `sprint/s89` at `5c7d8bf195e0e6eae050245e6760a1707edeb6c2` against merge base `9d019472f7e6b95ac4ba0770c4dee35dcbc28f0e`, 64 files, 10,381 additions and 692 deletions. Changed crates: `oxml-layout`, `rdocx-layout`, `rdocx-oxml` and `rdocx`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have.

## Blocking

Zero findings. No fix-now item established.

## Should-fix

Zero findings. No tracked-follow-up item established.

## Nice-to-have

Zero findings. No human-action or refuted item established.

## Milestone gate

This review covers the S89 related-story boundary within M24. It does not
claim the final modern DOCX boundary scheduled for S94 at
`docs/hld/14-development-backlog.md:60`.

The S89 roadmap states: "The sprint gate includes save-reopen checks for
relationships and untouched XML." The current definition of done at
`docs/sprints/CURRENT_SPRINT.md:45` additionally requires note policies,
cross-story ranges, deterministic fragment remapping, glossary lifecycle and
the downstream fontdb feature configuration.

The integrated all-feature test log contains passing executions of:

- `note_policies_match_pinned_word_markers_and_page_placement`, defined at `crates/rdocx/tests/integration_test.rs:22608`. The approved contract and HLD retain the explicit Word for Mac 16.113.2 beneathText divergence. This review did not launch Word or regenerate the oracle.
- `cross_story_ranges_reopen_with_exact_endpoints`, defined at `crates/rdocx/tests/integration_test.rs:23384`.
- `full_story_fragment_import_remaps_every_conflicting_dependency`, defined at `crates/rdocx/tests/regression_test.rs:45536`, including the shared companion and conflict regressions.
- `public_created_building_blocks_insert_and_reopen`, defined at `crates/rdocx/tests/regression_test.rs:47075`, alongside lifecycle, namespace, block grammar and placeholder selector regressions.

The routed font feature log records passing checks and tests for default,
no-default, memmap and no-default plus memmap configurations. The compile
regression was observed before the four-line PR 269 fix and documented in the
feature evidence. The existing binary and ordinary file source arms remain
unchanged at `crates/oxml-layout/src/font.rs:2032` and
`crates/oxml-layout/src/font.rs:2041`.

Successful full-gate evidence is split between
`/private/tmp/s89-full-verify-routed` and
`/private/tmp/s89-full-verify-pinned-tail`. Both head files identify
`881490515a6550e1d7952c452e42fae1b7320985`. Inspected logs record workspace
formatting, Clippy, changed-crate and all-feature tests, the font matrix,
49 matching hashes, prose and adapter checks, 140 policy tests with two
existing skips, no-default tests, WASM, docs, README examples, the verified
22-package publish dry run, 22 archives below 10 MiB and passing cargo-deny.
The dry run aborts uploads as expected. Interrupted earlier compiler runs
and the mixed-toolchain temporary-example failure are not counted as passes.

Only AS_BUILT, BACKLOG, CURRENT_SPRINT and SPRINT_TRACKER changed between that
verified source head and this reviewed head. Their five feature records agree
on completion and cleared owners. Full verification at the final committed
head, including this report, remains an integrator closure obligation. This
review does not substitute older source evidence for that exact-head gate.

## Not found

- **Interaction**: Zero confirmed findings. Glossary capture closes inherited namespaces and projects the selected content using its physical relationship owner at `crates/rdocx/src/building_block.rs:605`. It uses the shared fragment import transaction rather than copying entry XML alone. Capture and typed creation share the block grammar guard at `crates/rdocx/src/document.rs:6349`. Import refreshes note or comment destinations after companion closure at `crates/rdocx/src/document.rs:16758`, avoiding replacement of newly imported companions. The marker allocator includes all destination package parts at `crates/rdocx/src/field.rs:3149`, including glossary identities. Reviewed note and range companion remapping, staged publication and glossary drawing-ID reservation without establishing an interaction defect.
- **Duplication**: Zero confirmed findings. The glossary adapter reuses `DocumentFragment::from_part_content` and `import_content_staged` at `crates/rdocx/src/document.rs:1090` and `crates/rdocx/src/document.rs:1107`. No second dependency-import engine was introduced. Specialized raw XML edits retain their existing owner contracts.
- **Layering**: Zero findings. No manifest or lockfile changed in the sprint. The font fix stays in `oxml-layout`, Word XML in `rdocx-oxml`, note pagination in `rdocx-layout`, and package transactions in the `rdocx` facade. No new dependency from format-neutral crates to Word or presentation crates was introduced.
- **Harness**: Zero findings. All five plans expect unchanged output, their delivery records report unchanged hashes, and the integrated log reports all 49 entries matching. No baseline or harness implementation change is present.
- **Gate**: Zero confirmed missing S89 acceptance finding. Named integrated tests and the font matrix cover the sprint contract. The final exact-head verification requirement is explicitly retained above.
- **Docs**: Zero confirmed findings. Reviewed capability rows, ownership, native fragment and glossary surfaces, package integrity, note rendering and testing boundaries. DOCX-042 retains F-293 as its live remaining annotation owner. Completed fragment and glossary rows explicitly limit opaque companion support. Prefix, block boundary, placeholder and beneathText limits agree with the implementation and plans. Earlier dependency-prefix AS_BUILT entries remain historical scoped evidence, while the final entries name the integrated source gate.
- **Deps**: Zero findings. No new dependency, version or default feature was added. The font fallback handles Cargo features enabled by existing downstream consumers.
- **Surface**: Zero confirmed unrequested public surface. Note policy enums, immutable range snapshots, expanded fragment owners and checked glossary lifecycle operations have explicit story contracts. Existing-control placeholder binding retains the control discriminator. Content-control creation, modern annotation metadata, new binding entry points and implicit AutoText expansion remain with their recorded owners or outside scope.

This pass makes no source, test, HLD or ledger edits. This report is the sole
tracked artifact generated by the pass. The review ends here and returns
control to the integrator.

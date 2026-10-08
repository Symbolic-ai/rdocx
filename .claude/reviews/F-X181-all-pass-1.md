# F-X181, all aspects, pass 1

**Reviewed**: Frozen working diff on `work/f-x181-codex`, Base and HEAD `e480131608418643c1373101f50baad1913437ab`. Seven files, 312 added and 8 deleted lines. The production scope is 16 added and 1 deleted engine lines. Remaining changes are 283 regression lines, the approved plan checklist, HLD08 and measured archive metadata.
**Verdict**: 0 defects, 0 smells, 0 nitpicks.

## Defects

None. Zero defects.

## Smells

None. Zero smells.

## Nitpicks

None. Zero nitpicks.

## Not found

- Correctness: The four existing paragraph entrypoints pass the explicit cell context. Only the table entrypoint passes true. Both ordinary cell paragraphs and cell-control paragraphs reach it, and nested tables use the same recursive cell traversal. Typed page and column breaks emit no item in cells. Field display FF and VT are consumed without adding an item, while the scalar cursor advances. Ordinary line controls, tabs and non-cell page or column controls retain their existing paths.
- Contract: The implementation addresses all Issue273 criteria, including leading and inline breaks, ordinary and nested cells, joined text without a space, no-break row height and unchanged body pagination. The adopted production delta stays inside the inspected PR275 scope. No F-X180 table-border or paginator changes occur. HLD08 describes current behavior and the archive refresh changes only the affected package measurements.
- Panics: No new unwrap, expect, indexing, slicing or unchecked arithmetic is introduced in production. The existing char_indices loop supplies UTF-8 boundaries for field display slicing. The context flag adds no input-dependent allocation or recursion.
- OOXML: The change affects layout item emission and never edits the source model or serializer. Tests assert complete package byte equality after layout, exact saved and reopened document XML, retained authored break tokens and unknown XML, and equal reopened geometry. Valid typed field-cache breaks exercise the scalar FF and VT path without embedding forbidden XML characters. The schema-ordered native source and its bindings authenticate.
- Tests: The contributed regression fails before production and passes after it. The extended matrix checks 16 ordinary/nested, leading/inline, absent/page/column/line combinations plus their typed field variants. It compares each nested target with its own no-break control, checks exact joined text and leading x position, row height and following-paragraph y, and retains positive line-break controls. Body controls exercise direct and field page, column and line paths and source preservation. Existing suites retain their documented ignores.
- Structure: No new file, module, type, trait, generic, dependency or forwarding abstraction is introduced in production. One concrete boolean extends the existing paragraph implementation. Tests are in the existing regression entrypoint.

## Frozen identity and review integrity

Freeze manifest `/private/tmp/fx181-review-freeze.identity.json` SHA256 `c145897b516e16f0b2157cdeff99a9c7de95bd1bffaa4fb48fd6157bc31460d5` binds the seven files and 19 receipt logs. Binary diff SHA256 `3a09a41eca00b2d0d5a19ce9af9679640c94fc6245a1a6e83fb708dc23a0ba79` matches the initial review read, the final worker freeze, the frozen patch and the final pre-write recheck.

Every source and metadata hash below is identical before and after the read-only review. The final recheck also authenticates all 19 receipt hashes. Only this review record is written by the reviewer.

| File | SHA256 before and after review |
|---|---|
| `.claude/plans/F-X181-design.md` | `772b652335d8f9dd6f576a8f947e923fed7c195dae88b04ffc203a602264f736` |
| `README.md` | `243210f1d994853500f14a056c7abdce65d5b3c3c04652a3ad73b4df750fc21b` |
| `crates/rdocx-layout/README.md` | `3d635544937c094719104db64f79220474ade947815662bdc47038d68cf7560a` |
| `crates/rdocx-layout/src/engine.rs` | `4cc08616eba12b2b22be579734ebcc38c91e4d9657945d135a51daa7bc9e1c3b` |
| `crates/rdocx/tests/regression_test.rs` | `13ce0c1c48a9636f3d8c2be7d8716cc7c23654cc9624de4ba21471e50b345da9` |
| `docs/hld/08-rendering-spec.md` | `caf4b35c56f867622467bcb8c6d5079be203a0339436d77c26e66749e9050082` |
| `scripts/readme_doctests.py` | `666a4f6211f32054c92efc5c8d63a5084f63e47f30ee2d4610ac40c30fb41b0e` |

`crates/rdocx-layout/src/table.rs` and `crates/rdocx-layout/src/paginator.rs` are byte-identical to Base. Their SHA256 values are `391f089227c07408547f80c15e63a6262e68a16686ed5a406ac41fa20b60d49e` and `570cc7d7135da608555dead38cc76ee534891996382d1d3dc7548a48722d1dba`. F-282 remains paused and untouched.

## Acceptance evidence

The review reads the approved design, AGENTS, CLAUDE, canonical workflow and microscope command, HLD08 table-style and line/page-transition rules, HLD12 taxonomy/hash/golden rules, and the F-X181 backlog definition. Both sprint trackers mark F-X181 in-progress. Issue273 intake is `/private/tmp/S90-X180-X181-native-controls/issue-273-intake.json`, with reporter `hadim`, and the approved contribution is PR275 at `5ea2f5710d4bb13d0182bd74417fa5ca58522984`.

The reviewer independently rehashes all 45 bindings in `/private/tmp/S90-X180-X181-native-controls/schema-ordered-native-table-controls-authenticated-audit-index.json`, SHA256 `188680c6f5661a90d9cf1397e64aa6219c51dda0df01c4dc45b83b08639a6643`, with zero errors. The source SHA256 is `3ed26a7c88179f9e2673edcccb5a23cbbd71e54b22d892ee0ddb7ed8eb815904` and PDF SHA256 is `2848c32edea9aae1a06b2b95def1ea0fc17e45c91707e07f2bb9e541d53ba3c9`. The pinned oracle is Word for Mac16.113.2 build16.113.26092012. Its 25 pages include all 16 cell-break cases and body text on pages24/25.

Native page and column controls have zero target-baseline and following-paragraph y deltas against their own absent controls. The tiny joined-word xMax deltas, -0.000264pt ordinary and -0.000288pt nested, remain recorded. Native line controls move the following paragraph by13.68pt. Library tests use deterministic bundled Caladea12 and assert topology and exact own-control geometry. They do not assert absolute Arial metric equality, pixel parity or native field-update/reopen evidence that was not performed.

All receipt paths below are under `/private/tmp/` and their hashes are in the verified freeze manifest.

- `fx181-contributed-before.log` records the meaningful runtime failure on unchanged production. `fx181-contributed-after.log`, `fx181-native-matrix-final.log` and `fx181-body-controls.log` pass. The final strengthened matrix passes in2.87 seconds.
- `fx181-changed-crate-tests.log` reports 500 passed with6 ignored, 360 passed with8 ignored, 865 passed with7 ignored, and 317 passed with0 ignored. The documented ignores remain unchanged. Additional doctest groups pass.
- `fx181-check.log` passes all-target check. `fx181-scoped-clippy.log` passes in8.01 seconds. Scoped documentation checks pass.
- `fx181-workflow-policy.log` reports140 tests, two existing skips, and success in53.871 seconds. Expected negative-fixture diagnostics in this log are not production failures.
- `fx181-hash-harness.log` reports all49 entries match. The before-production harness also matches49. No baseline is changed.
- `fx181-readme-doctests.log` validates27 distinct workspace READMEs and22 publishable inventories, with their Rust examples passing.
- `fx181-publish-dry-run.log`, `fx181-archive-measurements.log` and `fx181-archive-sizes.log` verify locally patched publish dry runs and actual tuples: rdocx-layout308999 compressed bytes,1672132 member bytes,15 members, and rdocx1518202 compressed bytes,8403444 member bytes,36 members. Both stay below10485760 bytes. Archived source members match current source, and README and script tuples agree. Unrelated measurements and thresholds are unchanged.
- Final format and diff-check logs are empty and successful. Final prose reports zero violations. All26 generated skill adapters are synchronized.

The worker confirms final pipeline49944 exited0 before this record is written. No gate is pending in the reviewed freeze. This is feature-scoped review evidence, not integrated sprint verification or release approval. The reviewer runs no Cargo, native UI, remediation, handoff, commit or push.

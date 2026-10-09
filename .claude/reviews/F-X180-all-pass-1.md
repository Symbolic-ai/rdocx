# F-X180, all, pass 1

**Reviewed**: `/microscope F-X180 --working`, branch `work/f-x180-codex`, HEAD `cf58ed1d1e427cd5662deb98d19b9ddefe55828f`. Three source files, 240 additions and 66 deletions. Binary diff SHA-256 `be08885f79bbd21c097e53176e27ca0175da042a545f8ac5d65f701d0b48e524`.

**Verdict**: 1 defect, 0 smells, 0 nitpicks. Remediation and another independent pass are required.

## Contract and scope

Read CLAUDE.md, WORKFLOW.md, the canonical microscope command, approved F-X180 plan, progress note, active sprint and backlog, and cited HLD08, HLD12 and HLD14 sections. The feature is in progress. Reviewed the complete Issue 272 body from authenticated intake `/private/tmp/S90-X180-X181-native-controls/issue-272-intake.json`, including all seven direct and style-derived cases.

The native evidence index `/private/tmp/S90-X180-X181-native-controls/independent-native-table-controls-authenticated-audit-index.json`, SHA-256 `ace2e2b67ac21e2f18fd46f9fe24cf4b07be5c87292522715d4766ff4841b4e8`, records actual Word 16.113.2 observations. Its seven border cases support the proposed nil suppression and none inheritance behavior. It records painted rectangle unions, not a Rust pixel comparison. The native source-built direct table properties also need a separately qualified schema-order correction before claiming an exact schema-valid fixture correspondence.

## Defects

### D1, direct-border regression fixtures place tblBorders before tblW

`crates/rdocx/tests/regression_test.rs:56003`

For each direct-border case, `table_properties` is a `w:tblBorders` subtree, inserted before `w:tblW`. The CT_TblPr sequence requires width before borders, followed later by layout. The existing schema sequence is also explicit at `crates/rdocx-oxml/src/table.rs:1060` and `crates/rdocx-oxml/src/table.rs:1064`. Four of the seven new inputs therefore violate schema child order. They are parsed, edited and serialized before deterministic layout, so the passing assertions can exercise a normalized candidate without proving the intended schema-valid source-built input. Construct direct properties in schema order and rerun the affected regression. Preserve the deliberately tested opaque producer subtree separately.

## Smells

Zero.

## Nitpicks

Zero.

## Aspects checked

- Correctness: zero additional findings. `resolved_cell_edge` distinguishes Nil from None, preserves omitted-edge inheritance and excludes invisible or art borders. Painting and horizontal border bands consume the same helper. The corner case expects all four first-cell edges, matching native evidence.
- Contract: zero additional findings. Changes remain limited to border precedence and its regression expectations. Dense-form line count changes from 26 to 25 and removes the identified top segment. Existing geometry assertions remain. HLD completion and archive/hash gates are still pending and are not claimed complete by this review.
- Panics: zero findings. The production change introduces no indexing, allocation or unchecked arithmetic. Existing paginator indices and geometry operations are unchanged. Added unwraps and map indexing are confined to controlled test construction and assertions.
- OOXML: D1 above. Production parsing, serialization and unmodeled XML handling are unchanged. Tests retain nil/none tokens and an opaque cell-property subtree through save and reopen.
- Tests: D1 above. Reviewed the contributed outer/interior regression, all seven native topology expectations, border-band height assertions and dense-form checksum update. The reported before-fix failure and focused after-fix passes are caller evidence, not tests rerun during this review. Full changed-crate and packaging/hash gates remain the caller's responsibility.
- Structure: zero findings. No new production file, module, dependency, trait, wrapper, generic parameter or feature flag. The concrete shared helper loses its obsolete outer-edge parameter.

## Frozen identity

The three reviewed files contain 67,566 lines in total: paginator.rs 8,735, table.rs 2,713, regression_test.rs 56,118.

| File | SHA-256 |
|---|---|
| crates/rdocx-layout/src/paginator.rs | 570cc7d7135da608555dead38cc76ee534891996382d1d3dc7548a48722d1dba |
| crates/rdocx-layout/src/table.rs | 391f089227c07408547f80c15e63a6262e68a16686ed5a406ac41fa20b60d49e |
| crates/rdocx/tests/regression_test.rs | 676a9ca0496e40cd3ee1bb81d38612c73db5f51d40d1889507ee07f68ffa78d3 |

Verified the binary diff and all three source hashes before and after writing this record. No source, documentation, Cargo, formatting or UI changes were made. This pass ends with the review record.

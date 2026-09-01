# F-X071, correctness, pass 3

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through final implementation `ea0d8227c6c63803d947d07fc9212a776c25d7cc`, 20 implementation files and 3,496 changed lines (3,324 additions, 172 deletions)
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, typed numbering metadata drops producer attributes and child XML
`crates/rdocx-oxml/src/numbering.rs:2223`
`crates/rdocx-oxml/src/numbering.rs:2549`
`crates/rdocx-oxml/src/numbering.rs:2557`

The new `pStyle`, `nsid`, and `tmpl` branches retain only `w:val`. Their start-element paths then skip the complete subtree with `read_to_end_into`, and their empty-element paths likewise capture no extra attributes. Serialization reconstructs each node with only `w:val` at `crates/rdocx-oxml/src/numbering.rs:2388`, `crates/rdocx-oxml/src/numbering.rs:2631`, and `crates/rdocx-oxml/src/numbering.rs:2648`. For example, opening and saving `<w:pStyle w:val="List" ext:fact="kept"/>` deletes `ext:fact`, while a nonempty `w:pStyle` also loses every retained child. The same loss occurs for `w:nsid` and `w:tmpl`. At the claim base these elements followed `level_raw_boundary` or `abstract_raw_boundary` and survived as complete raw XML. Promoting their `w:val` facts must not weaken the unmodelled XML preservation required by the approved plan. The added numbering tests use only plain `w:val` attributes, so they do not detect the regression.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-2 remediation**: All five prior fixes are present. Foreign document-background lookalikes remain untyped and retain their exact subtree bytes. Picture relationships require the direct picture graphic-data path. Undeclared conventional prefixes fail closed. Duplicate pictures, duplicate blips, and duplicate relationship attributes do not merge facts. Raw row-property extras receive stable per-slot RTF loss diagnostics.
- **Correctness and contract**: Apart from D1, no additional defect was found in default-style numbering association, direct `numId` or `ilvl` overrides, `numId=0`, section and table completeness facts, hyperlink metadata, external images, revision enumeration, or field display ordering.
- **Panics and bounds**: No panic, unchecked arithmetic, unbounded revision recursion, malformed revision acceptance, or depth-limit regression was found. Excessive nested revision input rejects before recursive projection.
- **OOXML and preservation**: Apart from D1, no issue was found in expanded-name matching, inherited owner bindings, namespace shadows, row-property schema slots, raw table and content-control subtrees, drawing payload namespaces, or repeated save and reopen behavior.
- **Tests**: `cargo test -p rdocx-oxml --quiet` passed 328 unit tests and one doctest. `cargo test -p rdocx --lib --quiet` passed 326 tests with three ignored. The three focused pass-2 remediation tests for foreign backgrounds, ambiguous picture payloads, and RTF raw-row diagnostics passed. `git diff --check` passed. The only sensitivity gap found is described in D1.
- **Structure and public API**: No new crate, module, feature flag, trait, generic parameter, forwarding wrapper, dynamic dispatch, or public-signature regression was found. The additions remain concrete reader-only pre-1.0 APIs.

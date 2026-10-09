# F-282, ALL, pass 1

**Reviewed**: Frozen working tree on `work/f-282-codex`, implementation prefix HEAD `31524cc698be5eeaf0850aef0d7d913b1ec5a17f`, immutable claim Base `70e0b11fd6f2c0db35c6627fd9f50ca035bc30de`. Thirteen tracked changed files, 336684 insertions and 683 deletions, plus the approved untracked 16077-line bibliography.rs. Production scope is six changed files including that new module. The existing regression entrypoint adds335027 lines. Reviewed unchanged document.rs as the staging dependency.
**Contract**: Revised approved bounded S90 plan equal canonical79588295. F-282 delivers native source authoring and measured formatting admissions. Remaining catalogue work is F-X192 in S91 and F-283 is carried.
**Verdict**: 3 defects, 0 smells, 0 nitpicks.

## Defects

### D1, first paragraph formatting reset drops unknown attributes in the Word namespace

`crates/rdocx/src/bibliography.rs:1056`
`crates/rdocx/src/bibliography.rs:1094`

The metadata guard rejects attributes only when their namespace differs from WordprocessingML. It does not check the local names against the attributes actually owned by each formatting particle. A supported bibliography first paragraph containing `<w:ind w:left="360" w:producer="keep"/>` passes this guard, then whole-particle replacement removes the unmodeled `w:producer` attribute. If no generated counterpart exists, deleting the particle loses it as well. The recursive run-property merge has the same gap. A qualified formatting element does not establish ownership of every attribute in its namespace.

Preserve the unowned attributes or reject the transaction atomically. Exact raw XML and revision controls must cover the paragraph and recursive run-property paths. Existing foreign-attribute controls at `crates/rdocx/tests/regression_test.rs:67810` do not cover an unknown local name in the owned namespace. This finding is independently documented in the authenticated OOXML pass1 report.

### D2, source member identity edits miscalculate closing QNames with legal tag whitespace

`crates/rdocx/src/bibliography.rs:1502`

Closing QName replacement derives its range from the complete element end and assumes the QName directly precedes `>`. XML permits intervening whitespace. Replacing Title with Year in an imported `<b:Title>old</b:Title >` selects the closing byte span `:Title ` instead of `b:Title`. Combined with the opening edit, the candidate becomes `<b:Year>old</bb:Year>`. Staged validation correctly prevents publication, but the valid supported source edit is wrongly refused solely because of retained XML spelling. Contributor role renames share this helper.

Locate the actual closing QName source span and preserve trailing whitespace. Cover scalar and role identity edits, namespace aliases and spaced closing tags. This is static byte-range reasoning on frozen source, not a newly executed formatter or Cargo result. The authenticated independent OOXML report carries the same defect.

### D3, required mixed eligible and unfinished refresh rollback has no regression control

`crates/rdocx/src/bibliography.rs:16061`
`crates/rdocx/tests/regression_test.rs:62743`
`crates/rdocx/tests/regression_test.rs:296157`
`.claude/plans/F-282-design.md:117`

The revised release contract requires a supported earlier owner and a recognized unfinished standard later owner to prove whole-refresh error behavior, with all caches, source parts, package bytes and revision preserved. The staging code appears to propagate errors correctly, but the existing tests do not prove that policy. The default-locale error control at62743 rejects before formatting. The malformed filter controls at296157 have one owner. Other update error tests cover one protected metadata owner, one IEEE grouping error or one MERGEFORMAT conflict. None mixes an eligible cache with a later unfinished locale, source-kind or contributor branch, and none asserts the document revision for such refusal.

Add the required mixed-operation regression in the existing entrypoint, exercise an actual recognized unfinished branch after an eligible owner, and assert Err with unchanged caches, complete package/source bytes and revision or existing handle validity. Keep a separate noncatalogue retention-report control. This is a missing mandatory test-contract proof, not a claim that the current staged code was executed and observed publishing a partial result. The final scoped gate cannot supply a test that is absent from the frozen source.

## Smells

None found.

## Nitpicks

None recorded.

## Not found

- Correctness and admission: no additional finding beyond D1 and D2. All thirteen deferred APA selector identities1054,1105,1107,1108,1109,1111,1112,1113,1115,1121,2117,2128,2145 remain absent from the APA grammar match and reach its explicit unfinished error. Eleven non-APA bibliography branches have the documented numeric1033, lean Book, contributor/property and ASCII materializer checks. Citation grammar remains separate.
- Atomicity and API: no additional implementation finding. Public cache updates stage a clone and publish after owner updates, style preparation, source identity validation and reopen. Ordinary source mutations and no-op replacement keep their separate transaction paths. D3 blocks the required regression proof, not a static claim of partial publication.
- Locale and ordering: no additional finding. Source, field and explicit actual default context select concrete consumers without a guessed en-US fallback. Non-ASCII collection sort keys refuse. The ASCII key boundary does not claim the unresolved Unicode Word comparator.
- Panics and arithmetic: no additional finding. Reviewed public validation, identity edits, locale handling, reference counting, font-label width checks and internal run operations. D2 reaches checked parser refusal rather than publishing malformed XML. No executed runtime acceptance is inferred.
- Namespace, schema and ownership: no additional finding beyond the two independently reviewed defects. Qualified identity, custom XML relationship graph, source no-op preservation, source deletion references, instruction spans, foreign lookalikes, opaque particles, physical field boundaries and selected story offsets were included in the independent OOXML aspect.
- Structured caches and fonts: no additional finding. First/interior properties, IEEE row/cell/grid and label widths, owned run properties and outside source content retain concrete controls. Font resolver visibility changes no algorithm and documents effective cascaded input. Semantic emitters retain explicit measured configurations and source/generated distinctions. No general font coverage or locale alias equivalence is claimed by this review.
- Tests and evidence: no additional finding beyond D3. Existing rich fixtures retain own source identities and raw properties. Completed own17 captures and separate recovered-cache preservation tests remain different evidence domains. Finite page-split projections retain exact raw shape/property/space guards, not a global coalescing operation. The development receipt records1340 executed APA passes, zero failures and zero ignored tests. This is the earned subset checkpoint, not the final native or full catalogue gate.
- Structure: no additional finding. Approved bibliography module, concrete shared font resolver with existing consumers and existing test entrypoint conform to current structural rules. No new trait, generic, runtime feature or dependency was introduced.
- HLD and package intent: no additional finding. All seven listed HLD files state partial native coverage, source metadata versus formatting distinction, atomic unfinished errors, noncatalogue diagnostics, separate citations and S91 remainder. No dedicated bibliography Python or WASM API or new ICU dependency is claimed. Current archives, compatibility and publication dry runs remain pending risk gates.

## Independent aspect and authentication

The independent report `/Users/atulsharma/Documents/projects/tensorbee/rdocx/.claude/reviews/F-282-ooxml-pass-1.md`, SHA `24b1e9fe4fa8933115aec94bb9d36d9f1ae13d1c11611d54e28ae6e037988540`, completed before this ALL verdict. D1 and D2 above reconcile its two defects. Its explicit remaining OOXML zero dispositions were read and authenticated. Neither reviewer remediated source.

Binder `/private/tmp/S90-F282-bounded-doc-review-68oy0ruf/binder.json`, SHA `4fc213d71050507ceaddcf78d857a9a114a73a32ab0e37dd6384d936e60b46c4`, authenticated all25 current records and available retained copies before inspection. All25 current records authenticated again at exit. The revised plan matches the bound canonical contract. Seven source hashes remain unchanged:

| Worker source | SHA256 |
|---|---|
| crates/rdocx-layout/src/engine.rs | 90f328bb37575d257ae0b34fbeb1ef3ebdf6539f4c1dafe71ad4db84e3db9e6e |
| crates/rdocx-oxml/src/text.rs | cc6bb3ec2b64f2fec3aa06b55820386daa8ca42fd5de51020b53bd97bcd8f354 |
| crates/rdocx/src/field.rs | 63db78f9e205c56f2982c46a5999092c0041e00204256455669d7b6da07001d7 |
| crates/rdocx/src/lib.rs | 2071060310142958b0fb1276ee967bf69840d039c85afd19dc0209e28a4d32a0 |
| crates/rdocx/src/bibliography.rs | 31485a1f2061125662459770e3ff6654a524c29be1ecb9791d7508991d1fd767 |
| crates/rdocx/tests/regression_test.rs | 884ee40a35a01e8f991e722e967cc9c3949a0917c68466baf77144e2271ca3d4 |
| crates/rdocx/src/document.rs | 4057ef506a747776554fc57832eabc7daadc516f8ab8618b5af2961bb08910be |

Development receipt3bd0f8be binds46 items. Its seven current source bindings, retained before sources, oracle/reconstruction records and eight logs authenticate. Its historical full-contract plan binding is intentionally superseded by revised plan SHA `de75b471deae4f587e4f015d2bafed68409f797c7f717debf3508302a18af342` in the current binder. It is not a current plan authentication claim. The combined APA log records1340 passed, zero failed, zero ignored, 989 filtered,74.57 seconds. Failing/interrupted historical logs remain failures, not counted as current passes.

## Pending integration and gates

Worker prefix predates F-X191 integration. Missing F-X191 HLD14 text and unchanged document.rs in this prefix are inherited prefix state, not authored removals or an asserted merge conflict. Reconcile both approved plans and reviewed prefixes before completion, preserving source7 and all earned fixtures.

No Cargo, formatter execution, UI, Git mutation, network or temporary write was performed during this formal pass. Only this worker review record was written. Current scoped native/differential, binding runtime/typing, WASM, archive/package, deterministic49-entry harness and final full integrated gates are not earned by this review. Complete mandatory risk riders and resolve all three findings through normal remediation, then obtain a new independent review pass. No gate may skip the existing measured subset or substitute full catalogue claims for it.

This pass ends at the review record. Control returns to the orchestrator.

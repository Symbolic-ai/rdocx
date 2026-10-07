# F-279, implementation, pass 4

**Reviewed**: Frozen working diff on `work/f-279-codex`, based on `114ff99bc5f75c3bda1859f5f3c4d0cd5ba38a06`. Forty-six tracked files, 8230 additions and 769 deletions. Binary diff SHA-256: `a45df3f01ca6ae4faa788165be38784dcc2ebcfccefdd629242d6d886cc6736a`. The three existing untracked reviews are reference material outside implementation scope.

**Verdict**: 0 defects, 1 smell, 0 nitpicks. The microscope exit condition remains unmet.

Branch, status and the in-progress sprint and backlog entries were checked. The canonical workflow and microscope command, approved current plan, cited HLD contracts, applicable risk routing, latest frozen progress and pass 3 were read. This pass revisited the complete changed-file scope and the interactions of the pass-3 repairs with parsing, checked source staging, physical story ownership, cache identity, note completion, related consumers and public surfaces. It is independent static analysis. Existing logs below are worker evidence. No Cargo command, UI operation, source edit or remediation was performed. Only this review record was written.

## Defects

0 found.

## Smells

### S1, section inheritance introduces a generic callback with one instantiation

`crates/rdocx/src/document.rs:27327`
`crates/rdocx/src/document.rs:27355`
`CLAUDE.md:120`

The new recursive visit_control helper takes apply as impl FnMut, which introduces a generic parameter. Its only external call passes the single apply closure declared in materialize_header_footer_inheritance. Recursive calls pass that same concrete closure type. No second instantiation exists today. This violates the explicit repository rule that a new generic parameter must be instantiated two ways today. The approved plan also states that no new generic is required at `.claude/plans/F-279-design.md:143`.

Keep the useful recursive control traversal, but express this single concrete operation without an unconstrained callback type. For example, collect or visit the mutable section references with concrete existing types and apply the inheritance operation in the owning function. Do not add a speculative second consumer just to justify the generic. No functional failure is claimed for the current helper.

## Nitpicks

0 found.

## Pass-3 repair assessment

- D1 is repaired in the inspected paths. The synthetic cached-result paragraph chooses an effective producer Word alias or default prefix without replacing an occupied binding at `crates/rdocx-oxml/src/text.rs:2908`. Aliased simple-field owners retain their namespace scope during dirty updates at `crates/rdocx-oxml/src/text.rs:9159`. Genuine source text keeps its Word alias and opaque attributes at `crates/rdocx-oxml/src/text.rs:9920`. Source-less generated elements receive local canonical scope. The regression at `crates/rdocx/tests/regression_test.rs:50881` covers aliased and default owners, inherited foreign w fields and a foreign w:keep attribute through update, save and reopen. Opaque bytes stay exact and only two genuine field identities are placed.
- D2 is repaired. cached_display_owner_is_locked follows the cached ancestor tree at `crates/rdocx-oxml/src/text.rs:1083`, and rendering consults it before assigning a descendant pagination kind at `crates/rdocx-layout/src/engine.rs:7785`. The three-level simple and complex regression at `crates/rdocx/tests/regression_test.rs:50924` distinguishes locked middle-owner literal retention from unlocked dynamic substitution and checks unchanged bytes and flags through reopen.
- D3 is repaired functionally. Active relationship selection uses document_sections at `crates/rdocx/src/document.rs:25458`. Mutable inheritance walks direct body paragraphs and nested controls while excluding table-cell sections at `crates/rdocx/src/document.rs:27325`. The unique first-header and first-footer regression at `crates/rdocx/tests/regression_test.rs:50973` verifies their original control-ending section and subsequent omitted-reference section, per-section SECTION displays and four placements. S1 above concerns the new traversal's structure, not that behavior. The separate legacy MERGEFIELD discovery inventory remains unchanged.

The six pass-2 remedies remain intact. Generated note glyphs retain source None with a separate structural owner channel, bounded restart completion uses the actual final body band, physical revision projections keep metadata without serialize/reparse, physical deleted and opaque predecessor occurrences are consumed before accepted-view binding, implicit canonical drawing fragments remain compatible without admitting explicit foreign bindings, and authoritative rich story bodies and physical part names participate in retained cache identity.

The complete source scope still uses existing production files. Shared note_reference_source constructors remain format neutral across chart, PDF, DOCX and PPTX consumers. No new trait, crate, feature flag or production module was introduced. The valid local-w:drawing wrapper namespace limitation remains explicit in the plan and progress. This pre-existing writer limitation is not represented as repaired by the root-scoped physical-order regression or by this review.

## Existing execution evidence and remaining gates

The frozen progress reports discriminating old-source failures and passing final focused tests for all three pass-3 findings. The inspected `/private/tmp/f279-pass3-final-focused2.log` records the passing focused cases. `/private/tmp/f279-pass3-final-model.log` records the final XML model's 616 unit tests and README doctest passing. Final nine-crate lint is reported at `/private/tmp/f279-pass3-final-lint2.log`.

The prior full affected-native run preceded the last alias-text and foreign-attribute enhancement. The final rdocx and rdocx-layout repeats are still running in `/private/tmp/f279-pass4-native.log` at review time. No final-native completion is claimed here. The prior rebuilt-wheel suite's 175 passes belong to the prior frozen source, not this candidate. Final-source Python, hash, documentation and README consumers, full policy and verified all-22 patched publish dry-runs remain outstanding unless the integrator supplies their actual results. Archive measurements below 10 MiB remain explicitly no-verify measurements, not consumer-build verification. Applicable no-default and WASM checks remain integrated rider obligations.

The genuine F-279 saved-output Word reopen evidence remains 18 cases, 957 retained fields, 180 successful source-contract checks and zero cache, dirty or lock changes. Word is pinned to 16.113.2 build 16.113.26092012. Original partial update procedures and OLD sentinels remain explicit. This authenticates the bounded external lifecycle and does not replace the pending native or packaging gates.

## Not found

- Correctness: 0 independently established remaining defects.
- Contract: 0 additional functional or public-surface defects beyond the structural smell recorded above.
- Panics: 0 independently established defects in the changed checked indexing, source remapping, slicing and bounded number-format paths.
- OOXML: 0 additional namespace, schema child-order or opaque-preservation defects in the reviewed changes. The documented pre-existing local drawing limitation remains bounded.
- Tests: 0 additional independently established harness defects. The new regressions distinguish the three previous failures, and source-built checks remain separate from authentic external evidence.
- Structure: S1 above. No additional trait, wrapper, module, crate-boundary or speculative-extension finding.

Review ends here. No remediation was performed.

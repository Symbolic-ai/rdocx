# F-279, implementation, pass 5

**Reviewed**: Frozen working diff on `work/f-279-codex`, based on `114ff99bc5f75c3bda1859f5f3c4d0cd5ba38a06`. Forty-six tracked files, 8236 additions and 769 deletions. Binary diff SHA-256: `cf7cf531adc8b8acf3dad7ee62a58df8020c96708b1eedd133d600746fce93e3`. Existing untracked review records are reference material outside implementation scope.

**Verdict**: 0 defects, 0 smells, 0 nitpicks. The microscope exit condition is met for this frozen candidate. This is not a completion or verification verdict.

The branch, status, in-progress sprint and backlog entries, canonical workflow and microscope command, approved current design, cited HLD contracts, applicable risk routing, latest progress and pass 4 were read. This is a continuation of the independent whole-feature review. The current complete changed-file and hunk inventory was reconciled with the previously reviewed feature scope. The concrete traversal replacement was inspected directly against the layout section inventory and its regression. No Cargo command, UI operation, source edit or remediation was performed.

## Scope reconciliation and evidence limit

Pass 4 records the prior candidate's exact binary diff hash, 46-file scope, accepted functional repairs and sole structural smell. Its full binary patch or complete source snapshot was not retained. Therefore this pass does not claim a bytewise comparison proving every other file unchanged. The visible current diff, earlier independent review evidence and latest progress support the continuation assessment. Worker statements that the only implementation delta is the concrete traversal replacement remain separately identified statements, rather than a fabricated comparison artifact. The current frozen hash was independently checked twice.

The full scope remains the existing layout, OOXML, facade, Python, neutral constructor consumers, tests, HLD, approved plan, archive measurement rows and existing policy files listed in the current diff. No new implementation file, production module, crate, trait, feature flag or public traversal API appears. The previously accepted six pass-2 remedies and three pass-3 functional repairs remain the assessment carried forward from their independent review, with no contrary evidence found in this pass. The documented local w:drawing wrapper namespace limitation remains explicit and is not represented as repaired.

## Defects

0 found.

## Smells

0 found.

## Nitpicks

0 found.

## Pass-4 repair assessment

S1 is repaired at `crates/rdocx/src/document.rs:27327`. The recursive control_sections helper now accepts concrete CT_Sdt and returns Vec of mutable CT_SectPr references. It has no callback generic, trait object or speculative second consumer. The owning function applies its existing concrete inheritance closure at `crates/rdocx/src/document.rs:27358`.

The collector traverses paragraph and nested control content in source order, ignores tables, row/cell projections, runs and raw content, and retains the final body section after the body traversal at `crates/rdocx/src/document.rs:27365`. This matches document_sections at `crates/rdocx-layout/src/engine.rs:120` and its control inventory at `crates/rdocx-layout/src/engine.rs:175`. Collecting references before application does not change effective header/footer state ordering. The collector reads section locations while the caller changes reference lists inside those sections. It adds no indexing, arithmetic, parsing or serialization path.

The existing discriminator at `crates/rdocx/tests/regression_test.rs:50973` verifies unique first header and footer activation in a control-ending section, inheritance in the following omitted-reference section, live section values 1 and 2, four placements, two updated physical fields and save/reopen retention. Its inspected execution log `/private/tmp/f279-pass4-s1-focused.log` records one pass. `/private/tmp/f279-pass4-s1-check.log` and `/private/tmp/f279-pass4-s1-lint.log` record successful all-target checking and all-target/all-feature lint. These are worker execution evidence, not commands run by this reviewer.

## Execution evidence and remaining gates

Latest progress records the exact prior a45df3f0 candidate passing 499 rdocx unit, 360 integration, 811 regression and 306 rdocx-layout tests with doctests, and an isolated rebuilt wheel passing 175 Python tests. Those results remain prior-candidate evidence. They are not relabeled as final-source results. The current structural replacement has the focused, check and lint evidence above.

Final-source documentation and README consumers, hash harness, full policy and all 22 verified patched publish dry-runs remain due unless the integrator supplies their actual completed results. Archive measurement refreshes and all packages below 10 MiB remain no-verify measurements, not verified consumer builds. The deliberately stopped prior gate wrapper's exit 143 is not described as a test failure or successful whole gate. Integrated no-default and WASM checks remain separate rider obligations.

The authenticated Word evidence remains 18 actual saved-output reopens, 957 retained fields, 180 successful source-contract checks and zero cache, dirty or lock changes, pinned to Word 16.113.2 build 16.113.26092012. Initial partial update procedures and OLD sentinels retain their stated limits. This review does not upgrade those procedures or replace native and packaging gates with external evidence.

## Not found

- Correctness: 0 remaining independently established defects. Concrete traversal preserves the modeled section and inheritance order.
- Contract: 0 additional defects. The replacement stays within the approved existing-file scope and removes the forbidden single-instantiation generic.
- Panics: 0 additional defects. The replacement uses checked optional section access and safe mutable iteration, without new unwrap, indexing, slicing or arithmetic.
- OOXML: 0 additional defects. This delta performs no XML writing and preserves the existing checked ownership and serialization boundaries.
- Tests: 0 additional harness defects. The existing inheritance discriminator remains applicable and passed for this replacement.
- Structure: 0 remaining smells. The helper uses concrete existing types and performs substantive recursive collection without speculative abstraction.

Review ends here. No remediation was performed.

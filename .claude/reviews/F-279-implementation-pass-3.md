# F-279, implementation, pass 3

**Reviewed**: Frozen working diff on `work/f-279-codex`, based on `114ff99bc5f75c3bda1859f5f3c4d0cd5ba38a06`. Forty-six tracked files, 7865 additions and 702 deletions. Binary diff SHA-256: `5cb3a92a78a1352437581fbd29354caf9472f463620ec0b3c5388f6edc245818`. The two existing untracked reviews are reference material outside the implementation diff.

**Verdict**: 3 defects, 0 smells, 0 nitpicks. Completion is blocked.

The branch, status, current sprint and backlog status were checked. The approved plan, its cited HLD contracts, canonical workflow, microscope command, applicable risk routing, differential-testing reference, previous review and frozen progress were read. Review covered the changed Word model, source registration, staging, pagination, retained state, shared constructors, bindings, tests, HLD and package measurements. This is independent static analysis. No Cargo command, UI operation, implementation edit or remediation was performed. Existing execution evidence is distinguished below from this review's findings. Only this review record was written.

## Defects

### D1, nested simple-cache parsing overwrites the producer's foreign w binding

`crates/rdocx-oxml/src/text.rs:2878`
`crates/rdocx-oxml/src/text.rs:2886`
`crates/rdocx-oxml/src/numbering.rs:621`

The synthetic paragraph around a simple field's cached children always declares w as the canonical Word namespace. word_prefixes_at then removes the inherited binding for that prefix and replaces it with the synthetic declaration. A real outer field using an alias such as x for Word, with w bound to a foreign namespace on the outer field or an ancestor, therefore reparses its cached foreign w:r or w:fldSimple elements as Word elements. The first parsing pass correctly excluded those elements. The nested pass can include their text and field instructions, changing the projected cache and making a foreign PAGE look like a pagination field. Cache rewriting can also fail when its original-namespace source scanner rejects the falsely admitted child.

The projection wrapper must carry the original effective namespace scope without taking an occupied prefix. Cover an aliased Word outer simple field containing both a genuine nested field and an opaque foreign w field, with the foreign binding inherited from outside the extracted result span. Assert unchanged opaque bytes, unchanged foreign cache and no foreign pagination identity. Canonical-only four-wire-form tests do not exercise this namespace boundary.

### D2, an intermediate locked cached owner does not protect its descendants during rendering

`crates/rdocx-oxml/src/text.rs:1030`
`crates/rdocx-layout/src/engine.rs:7784`
`crates/rdocx-layout/src/engine.rs:7791`
`crates/rdocx/tests/regression_test.rs:49379`

Cached display recursion returns only the leaf display owner. Pagination-kind assignment checks the top-level field's lock and that leaf's lock, losing any intermediate cached-field owner's lock. An unlocked UNKNOWN outer field whose cached result contains a locked UNKNOWN field whose cached result contains an unlocked PAGE therefore paints the PAGE sentinel as a live page value. The cache updater propagates the locked-owner state recursively and retains the same stored child, so rendered output and the retained locked display disagree. The existing lock regression covers a locked top-level owner and one child, which cannot reveal this case.

Retain effective lock ancestry when projecting cached segments or when assigning their pagination kinds. Add at least a three-level cache case for a locked middle owner, asserting rendered literal display and retained cache and flags through update, save and reopen. Keep ordinary unlocked descendants dynamic.

### D3, control-owned sections still use a different furniture inventory

`crates/rdocx-layout/src/engine.rs:121`
`crates/rdocx/src/document.rs:25460`
`crates/rdocx/src/document.rs:26525`
`crates/rdocx/src/document.rs:27326`
`crates/rdocx/tests/regression_test.rs:49479`

Pagination and number-format lookup now recognize section-ending paragraphs inside modeled body controls, but header_footer_rel_ids_for_layout still visits only direct body paragraphs and the final body sectPr. build_layout_input filters header and footer relationships using that incomplete inventory. A header or footer referenced only by a control-owned ending section is consequently absent from the layout input and source registry, so its SECTION or SECTIONPAGES cache has no placement and its furniture is not rendered. materialize_header_footer_inheritance also walks only direct body paragraphs. An omitted reference after such a control-owned section cannot inherit that section's furniture, and an explicitly changed control-owned reference does not update the effective chain.

Use the same modeled main-story section order for active relationship selection and inheritance as for pagination. Test a control-owned section with unique header and footer references, followed by a section that omits both references and must inherit them. Include a first or even variant so activation and inherited selection are checked. Preserve the deliberately separate legacy MERGEFIELD discovery boundary. The existing control-owned section regression contains only body fields and number formats, so it does not prove furniture ownership.

## Prior defects and scope assessment

- Pass-2 D1 is repaired structurally. Generated note text keeps source None. note_reference_source travels through shaping, painting, multilingual output, rebinding and related consumers independently of exact text attribution.
- Pass-2 D2 is repaired in the inspected completion path. The blanket default-docEnd eligibility exclusion is removed. Completed restart regions append all body-referenced endnotes into the retained final flow band, while reused completed tails avoid duplicate appends. Default, explicit and unused-part fitting and overflow regressions retain bounded invocation assertions.
- Pass-2 D3 is repaired by mutating physical revision projections in place before accepted-view selection, avoiding serialize/reparse loss of text-box metadata.
- Pass-2 D4 is repaired by consuming physical revisions and opaque predecessors in source order before assigning selected owners. The new insertion, move, deletion, opaque and same-run MC/direct cases cover the previously identified occurrence mismatch.
- Pass-2 D5 is repaired for implicit canonical Word fragments while explicit foreign rebinding remains excluded in the drawing parser. D1 above identifies a separate namespace problem in nested simple caches.
- Pass-2 D6 is repaired by including authoritative story bodies and physical part names in retained context and furniture cache identity, including note comparisons and retained allocation accounting.

The local-w:drawing namespace limitation remains explicit in the approved plan. CT_Drawing reconstructs its wrapper without its local namespace declarations at `crates/rdocx-oxml/src/drawing.rs:1684`. The same writer exists in HEAD. The new binder can therefore leave a valid locally bound drawing's physical owner unavailable, as documented, rather than guess an owner. This pass does not claim that limitation was repaired or that the root-scoped same-run test covers it. It is a retained pre-existing preservation limitation, separate from the three new contract defects above.

The source-built row and rich-story checks establish wrapping, clipping, continuation and identities for their bounded cases. They do not establish unmeasured full Word rendering fidelity. The shared note-reference channel remains format neutral, constructor consumers add no reverse family dependency, and no new production module, trait or generic was introduced.

## Existing execution evidence and remaining gates

The existing frozen `/private/tmp/f279-final-eight-native-crates.log` now contains passing unit, integration, regression and doc-test results for the eight changed native crates. In particular, rdocx records 499 unit, 360 integration and 808 regression passes, rdocx-layout records 306 unit passes, and rdocx-oxml records 616 unit passes. These existing tests do not cover D1 through D3 above. This reviewer ran no tests and does not convert these results into a zero-finding verdict.

Root's authenticated actual Word saved-output reopens cover all 18 cases, 957 retained fields and 180 successful source-contract checks, with zero cache, dirty or lock changes. Word is pinned to 16.113.2 build 16.113.26092012. The no-F9 reopen procedure and actual offline PDFs preserve the initial capture provenance. Original partial update scopes and unupdated OLD caches remain explicit. This supports saved-output stability, not a claim that every initial story was explicitly updated or that every native cache equals Word.

The frozen progress records successful focused remediation tests and changed-crate lint, plus archive measurements below the 10 MiB limit. README measurements correctly identify cargo package --no-verify. Those measurements are not verified external consumer builds. Final rebuilt-wheel Python tests, hash harness, policy and documentation gates, verified package and publish dry-runs, applicable no-default checks and the integrated WASM rider remain completion obligations unless the integrator supplies their actual passing evidence. Pending checks are not fabricated code findings.

## Smells

0 found.

## Nitpicks

0 found.

## Not found

- Panics: 0 additional independently established defects in the changed indexing, slicing, checked source remapping and bounded formatting paths.
- Structure: 0 findings against the existing-implementer, wrapper, generic, module or crate-boundary rules.
- OOXML: D1 is the namespace finding. No additional independently established schema child-order or opaque-replacement defect was found in the reviewed changes.
- Tests: the missing cases supporting D1 through D3 are recorded with those defects. No additional independent test-harness defect was found.
- Correctness and contract: D2 and D3 above. No additional independently established defect beyond these and D1.

Review ends here. No remediation was performed.

# F-279, implementation, pass 2

**Reviewed**: Frozen working diff on `work/f-279-codex`, based on `114ff99bc5f75c3bda1859f5f3c4d0cd5ba38a06`. Twenty-seven tracked files, 6650 additions and 625 deletions. The binary diff SHA-256 is `4da5ef1245bbfa5202a7a0c6dc7c0e8f9196b57d145ab4790a883481581db364`. The existing untracked pass-1 review is reference material, not implementation scope.

**Verdict**: 6 defects, 0 smells, 0 nitpicks. Completion is blocked.

The worker was frozen throughout review. Branch, status and in-progress sprint and backlog entries were checked. The approved plan, latest progress record, pass 1, canonical workflow, microscope command, risk router and differential-testing reference were read. The cited scope, architecture, OPC, rendering, bindings and testing contracts were inspected. This pass is independent static analysis. No Cargo command, Word operation, source mutation or remediation was performed. Reported executions below are existing worker or root evidence. Only this review file was written.

## Defects

### D1, generated note labels corrupt text provenance

`crates/rdocx-layout/src/engine.rs:7885`
`crates/rdocx-layout/src/engine.rs:3110`
`crates/rdocx-layout/src/engine.rs:18499`
`crates/rdocx-layout/src/engine.rs:18856`

The new reference-owner collection depends on assigning a zero-length SourceSpan to the generated superscript note number. That visible number does not equal the empty text selected by its span. SourceSpan identifies exact source text, while generated text remains unattributed. The frozen pinned run fails both exact sourced-glyph and generated-text provenance regressions. Keep the structural note-reference owner independently of the generated label's text attribution. Removing these assertions would weaken the existing shared output contract rather than repair ownership.

### D2, default document-end notes disable existing bounded restart behavior

`crates/rdocx-layout/src/engine.rs:2690`
`crates/rdocx-layout/src/engine.rs:17110`
`crates/rdocx-layout/src/engine.rs:17199`
`crates/rdocx-layout/src/engine.rs:17238`
`crates/rdocx-layout/src/engine.rs:17308`

The revised eligibility guard treats absent endnote placement as docEnd and then rejects every input with an endnotes part in that mode. A previously supported single-section document with stable related stories cannot retain a restart record or paginate only its changed region. The four cited frozen tests fail with zero candidate bytes or excessive page-layout invocations. Moving default endnotes into the remaining final body-page flow band requires restart completion to retain that band and append notes correctly. It does not justify removing bounded restart support for the whole input class, including an unused endnotes part. Reconcile completion and note-state reuse without replacing these performance bounds with full pagination expectations.

### D3, accepted revision reparsing drops text-box source ownership

`crates/rdocx/src/document.rs:8337`
`crates/rdocx/src/document.rs:8355`
`crates/rdocx-oxml/src/revision.rs:297`
`crates/rdocx-oxml/src/revision.rs:327`
`crates/rdocx-layout/src/engine.rs:590`
`crates/rdocx/src/field.rs:1078`

The layout-only binder adds source_text_box_owner to a cloned accepted run and installs it through replace_accepted_run. For insertion or move-destination runs, that operation serializes and reparses the revision. The ordinal is projection metadata and is not serialized, so the replacement loses it before SourceRegistry registers the shape. A supported anchored typed text box inside an accepted revision therefore has no physical-owner join, and its owner-dependent pagination caches are retained instead of updated. The ignored replacement result also hides failures in this binding step. Preserve projection metadata across this boundary and cover accepted insertion and move-destination owners, including an inline control inside the revision. This is an unclosed part of pass-1 D5.

### D4, an identical deleted anchor can steal a visible anchor's identity

`crates/rdocx/src/document.rs:8285`
`crates/rdocx/src/document.rs:8302`
`crates/rdocx/src/document.rs:8318`
`crates/rdocx/src/document.rs:8341`
`crates/rdocx-oxml/src/revision.rs:204`
`crates/rdocx/src/field.rs:1078`

The source anchor inventory includes modeled physical anchors inside deleted or moved-away runs. Consumption visits only accepted_run_paths, which omit those revisions, and selects the first unused byte-equal anchor without the run's physical occurrence. If an identical deleted anchor precedes an ordinary visible anchor, the visible shape receives the deleted owner's ordinal. Exact field-byte validation does not reject this case because both text boxes and fields are identical. Cache staging can then write the visible placement into the deleted source owner and leave the visible owner unplaced. Bind the actual physical occurrence before applying the accepted view. The existing duplicate tests cover ordinary paragraphs and inline controls, but omit an identical unrendered revision occurrence before the visible owner. This is a second unclosed part of pass-1 D5.

### D5, rich text-box parsing loses the existing canonical-prefix fragment projection

`crates/rdocx-oxml/src/drawing.rs:1383`
`crates/rdocx-oxml/src/drawing.rs:706`
`crates/rdocx-oxml/src/text.rs:12512`

The previous text-box paragraph parser carried the existing implicit canonical w prefix into fragment parsing. The replacement initializes the body parser with an empty prefix list. In the established AlternateContent paragraph-fragment API, whose enclosing parser supplies canonical Word semantics, a txbxContent without a locally explicit w binding now stores its w:p as raw content. Both shape.text and the authoritative body's modeled paragraphs are empty. The frozen rdocx-oxml run fails alternate_content_is_preserved_once_and_parsed_for_layout at the cited shape.text assertion. Preserve the existing fragment API's canonical prefix behavior while continuing to reject explicitly rebound foreign namespaces. This does not assert that a full Word package may omit its required namespace declaration.

### D6, authoritative rich story changes are missing from reusable cache identity

`crates/rdocx-layout/src/engine.rs:1360`
`crates/rdocx-layout/src/engine.rs:1425`
`crates/rdocx-layout/src/engine.rs:1452`
`crates/rdocx-layout/src/engine.rs:9815`
`crates/rdocx-layout/src/engine.rs:9845`
`crates/rdocx-layout/src/engine.rs:9870`
`crates/rdocx-layout/src/engine.rs:9970`

LayoutInput now makes a present story_bodies entry authoritative, but ReusableEngineContext captures and compares only the legacy headers, footers and note parts. It also omits story_part_names. A paragraph-only rich header remains cache-safe, and its HeaderFooterCacheKey still contains the unchanged legacy part rather than the authoritative body. Warm layout can therefore reuse the previous header text after only the authoritative rich header changes, while a fresh engine paints the new text. Related rich note changes likewise do not invalidate retained context through notes_match. Source-part-name changes can reuse anchored blocks with obsolete projection identities. Include authoritative projection and physical-source inputs in the relevant context and cache boundaries, with direct public LayoutInput warm-versus-fresh tests. Native callers must not be required to mutate an overridden legacy projection to make an authoritative edit visible.

## Pass-1 findings and rider assessment

- D1: The optional authoritative CT_Body path now exists for text boxes, tables and controls. General absence is repaired. Pass-2 D5 identifies a remaining parser compatibility failure.
- D2: Later continuous members now select furniture, restart state and first-page policy on subsequent pages. The authenticated furniture and restart controls cover the measured shared-page policy.
- D3: Note semantic ownership now follows unique placed body references and retains unresolved or conflicting ownership. Pass-2 D1 identifies the incorrect text-provenance mechanism used to collect it.
- D4: Cached-result children now carry their own field identity and display position. Instruction descendants alone inherit owner placement. The four wire-form page/column and vanished-child tests exercise the former failure.
- D5: Physical owner and owner-local paragraph joins replace filtered owner ordinal guesses, and ordinary duplicate, earlier unregistered and inline-control cases are covered. Pass-2 D3 and D4 remain blockers at revision boundaries.
- D6: Ordered cache segments now preserve child display and direct properties for all four wire-form pairs before and after edits. The dedicated rendered-text test exercises this repair.
- D7: The final continuous member's section-end note boundary is consumed before the following section. Deferred document-end notes resume after the consumed flow band. The new focused test exercises ordering and once-only placement.
- D8: Section format and pagination now share document_sections, including modeled control-owned ending paragraphs and excluding table-cell sectPr. General field evaluation retains the prior direct-body MERGEFIELD discovery boundary. The focused cross-section format regression exercises the correction.

Rich related-story rows now reuse row painting and cell clips. Source-built tests exercise wrapped multirow note continuation, exact-height clipping across all five related owner kinds and distinct text-box row paragraph identities. No additional independently established row-geometry defect was found. These source-built checks do not claim a full rich-story Word fidelity comparison.

## Existing execution evidence and remaining gates

The frozen pinned log `/private/tmp/f279-final-changed-crates-pinned.log` records 122 oxml-layout unit passes, 499 rdocx unit passes, 360 integration passes and 806 regression passes. The rdocx-layout result is 293 passes and 10 failures. Two failures substantiate D1 and four substantiate D2. The other four assert the old requirement that endnotes occupy pages distinct from body or footnotes. The intentional remaining-body-page policy is supported by authenticated Word cases, so those four failures are not automatically additional implementation defects. Their test expectations and policy rationale still require explicit reconciliation before the gate can pass. The separately reported rdocx-oxml result is 615 passes and one failure, substantiating D5.

Root's actual reopen evidence under `/private/tmp/S90-F279-reopen` covers 18 of 18 exact saved-output cases, 957 retained fields and 180 successful source-contract checks, with zero cache, dirty or lock changes. Word was pinned to 16.113.2 build 16.113.26092012 and physical PDF evidence used Poppler 26.01.0. No F9 was performed on reopen. Partial initial story-update scopes and previously unupdated OLD caches remain explicit. This evidence supports saved-output stability. It does not establish the native Rust gate or erase the failures above. The worker's frozen progress note predates this root evidence and may be updated only after this review returns.

The five HLD impact files have been updated and public report, immutable snapshot and struct-literal compatibility impacts are documented. Archive measurements and byte-identical packaged regression source checks are existing evidence. The 22-package no-verify measurement is not the mandatory publish dry-run. Final scoped verification, reconciled native suites, final Python wheel consumers, hash and policy checks, required archive consumer verification and a subsequent zero-defect microscope remain completion barriers. The complete capability classification cannot be delivered while these defects remain. No worker handoff or completed story is claimed by this review.

## Smells

None found.

## Nitpicks

None found.

## Not found

- Correctness and contract outside D1 through D6: no additional independently established defect in the inspected changed paths.
- Panics: zero additional independently established panic defects. Checked source patching rejects stale spans and invalid owners. No runtime hostile-input campaign was executed.
- OOXML outside D5: zero additional independently established schema-order, foreign-namespace admission or opaque-source preservation defects. Rich projections remain separate from original DrawingML serialization, and related-story publication uses checked field source spans on a staged candidate.
- Tests outside the identified regression and pending gates: zero additional independently established test defects. Genuine captured cache values were not replaced with synthetic Word expectations.
- Structure: zero new unjustified traits, generic parameters, forwarding wrappers, crates, modules, feature flags or production source files. Existing approved file claims contain the implementation.
- Atomic publication: zero additional receiver-publication defects. The public updater stages layout and all physical cache patches before committing its candidate.
- Bindings: zero additional compatibility defects in the inspected Python report constructor, count getters, total and stubs. Existing constructor arguments remain required and additive counters default to zero.

The frozen binary diff hash was rechecked immediately before writing and remained unchanged. Review ends here. Remediation is a separate implementation phase.

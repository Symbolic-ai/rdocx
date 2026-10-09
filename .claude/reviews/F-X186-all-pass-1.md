# F-X186, all, pass 1

**Reviewed**: Working diff on `work/f-x186-codex`, Base and HEAD `c303d34b76f762e626df7f46a607069b89661b87`. All 16 tracked paths, 1717 insertions and 23 deletions.
**Verdict**: 3 defects, 0 smells, 0 nitpicks. Remediation and another independent review are required.

## Frozen identity and contract

Read the approved F-X186 design, CLAUDE, WORKFLOW, microscope and the cited architecture, packaging, bindings and testing contracts. Reviewed all production, binding, CLI, test and HLD changes, including the qualified PR287 adaptation.

Independently authenticated `/private/tmp/fx186-pass1-freeze.json`, SHA-256 `484d95bcc05e505000bda4de4d27a2a34736db9d4558873925dacdd7c507df5b`. All 16 tracked files, four receipts and 30 logs match their bindings. HEAD, branch, exact status and ordinary binary diff match the freeze. Diff SHA-256 is `f5f3a727aeaf8a4f58d7060f3ee65bd3a6926ff8d04c5467aabe0b191157cb7a`.

## Defects

### D1, unordered raw serialization silently discards independently owned references

`crates/rdocx-oxml/src/text.rs:2949`

The unordered fallback skips every raw namespace-qualified `commentReference` with a numeric Word id. `is_retained_comment_reference_source` at `text.rs:3000` checks only parsed identity. It does not prove that the raw element is a flagged carrier owned by a typed child.

A concrete public construction is a `CT_R` struct literal with `properties: None`, empty `content`, empty `extra_xml_positions`, empty `alt_drawings`, and one `extra_xml` value containing `<w:commentReference xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" w:id="7" xmlns:x="urn:x" x:keep="opaque"/>`. These public raw fields are declared at `text.rs:2044`. The unequal vector lengths select the unordered fallback at `text.rs:2805`. With no typed children, the final loop skips the entire reference and its opaque attribute. Base emitted this raw value through `write_raw_with_word_override`. No parser or typed-carrier assumption is needed to trigger the loss.

The analogous skip at `text.rs:2906` also treats an independent raw reference preceding a typed reference as a carrier. Only proved carrier ownership may suppress a raw child. Preserve independent raw elements while preventing duplicate emission of actual typed carriers. The new carrier test's cleared-position case exercises typed carriers, not this unflagged public raw construction. This finding is established by the exact before and current serialization branches, without running an additional reviewer build.

### D2, namespace declarations change plain reference canonicalization

`crates/rdocx-oxml/src/text.rs:2715`

The attributed-empty discriminator treats a local namespace declaration as an unmodeled reference attribute. The existing input `<q:commentReference xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main" q:id="7"/>` therefore gains a raw carrier at `text.rs:2726` and is emitted from that carrier at `text.rs:2924`, instead of the established canonical `<w:commentReference w:id="7"/>`.

This violates the approved qualification that plain authored references retain established serialization. It is independently reproduced by the unchanged `comment_anchors_are_typed_without_moving_neighbouring_xml` test at `text.rs:15643`, whose canonical adjacency assertion at `text.rs:15674` fails in the authenticated `/private/tmp/fx186-20261008T173041Z/oxml.log`. The actual full oxml result is 628 passed and one failed. Distinguish namespace closure declarations from payload attributes without losing actual unmodeled reference source or neighboring XML. Do not replace the existing canonical assertion merely to accept this unintended delta.

### D3, the required denied-warning lint gate fails on the new private tuple

`crates/rdocx/src/document.rs:6087`

The new `comment_move_source_edits` return type triggers `clippy::type_complexity` under the required all-feature denied-warning gate. The authenticated `/private/tmp/fx186-20261008T172906Z/clippy.log` reports this exact signature and compilation failure for the library and library tests. The frozen source has no localized allowance or other reconciliation. Resolve this earned lint failure within the existing concrete implementation and re-run the deciding gate. This is a current gate defect, not a request for speculative abstraction.

## Smells

Zero.

## Nitpicks

Zero.

## Aspect results

- Correctness: D1. No additional defect found in strict root lookup, staged movement, target search, source-span removal, physical paragraph rebasing or reference-run restoration.
- Contract: D1 and D2. No additional discrepancy found in identity, reply and companion preservation, refusal atomicity, supported cross-story placement or Python revision behavior.
- Panics: zero findings in the changed paths. Checked ranges and ownership precede slicing and publication. Existing test assertions are not production panic findings.
- OOXML: D1 and D2. No additional finding in namespace closure, mixed-run neighboring content, accepted-index mapping, schema placement or conservative Google wrapper pruning.
- Tests: zero additional findings beyond the exposed serialization regression and incomplete gates. Positive fail-before evidence exists and the focused tests cover real implementation behavior. D1 needs its own independent raw construction regression during remediation.
- Structure: D3. No new production file, crate, dependency, trait, generic, speculative flag or forwarding wrapper. The concrete private helpers have actual movement and existing range consumers.

## Evidence and limits

The exact Base receipts retain compiled failure of the two existing-API positive native controls, CLI failure and actual rebuilt Python missing-API failure. Current frozen evidence establishes six focused native movement tests, one low-level carrier lifecycle test, one CLI control and actual current Python rebuild followed by the new Python control and all 94 core tests passing. These cover same-paragraph pure and mixed reference runs, block-control rebasing, supported stories and textbox ownership, aliases and foreign lookalikes, metadata and companion continuity, Google payload retention, stale and invalid ranges and atomic failures.

Scoped verification is **INCOMPLETE**. The first scoped runner stops at D3. The independent remainder stops at D2. There is no final full native or CLI matrix, current hash-harness result, WASM, denied-warning documentation, typing, package measurement, publish dry-run, README inventory or workflow-policy acceptance claim for this diff. Base hash49 is only Base evidence. No archive metadata was refreshed, no baseline movement was authorized or performed, and no native Word oracle is claimed for these source-ownership controls.

The HLD impact files describe the intended checked APIs and preservation contract. Their acceptance remains subject to these defects and the outstanding scoped riders. This review does not complete the feature or the sprint. No source, test, plan, HLD or baseline was edited, and no Cargo command was run by the reviewer. Only this review record was written. Remediation starts after handback.

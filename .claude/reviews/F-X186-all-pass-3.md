# F-X186, all, pass 3

**Reviewed**: Working diff on `work/f-x186-codex`, Base and HEAD `c303d34b76f762e626df7f46a607069b89661b87`. All 20 tracked paths, 2446 insertions and 55 deletions.
**Verdict**: 1 defect, 0 smells, 0 nitpicks. Remediation and another independent review are required. Current scoped gates pass, but they do not cover D1.

## Contract and authenticated identity

Read CLAUDE, WORKFLOW, microscope, the approved F-X186 plan and its cited architecture, packaging, bindings and testing sections. Reviewed all production, tests, binding signatures, CLI routing, HLD changes and final archive measurement metadata. Prior reviews and their actual failed evidence were retained, then reassessed against separate remediation.

Independently authenticated `/private/tmp/fx186-pass3-freeze.json`, SHA-256 `f532b90e356f99900531a44cc38712f2928e7c6253c64bbe58e45e373e8b6c6a`. All 20 tracked files and 86 bound artifacts match. Status and ordinary binary diff match exactly. Diff SHA-256 is `14ff6102dd699bebfbf4ce62a9d7df20768955078e50e1db35ebe5812f336591`. The current source-phase binder is `63c152db34d3ef9246b7a5f26e68c68d6f0e948ece0c3c05b87be7fea4e9ddc9`.

## Defects

### D1, rich anchor queries still read canonical source without retained local bindings

`crates/rdocx/src/comments.rs:2654`

The new static rich projection is given paragraphs from `story_range_paragraphs`, which obtains the main source through `story_sources`. At `crates/rdocx/src/document.rs:15610`, that source is still `self.document.to_xml()`, without the nested namespace replay used by save and the new move safeguard. `document.rs:16913` then scans this canonical source, and `document.rs:16933` closes a paragraph with only the bindings present in that source. A dropped paragraph or block declaration cannot be recovered by that closure.

A concrete input is a valid imported document with an owned root comment id 0 and a paragraph whose local `xmlns:q` binds the Word namespace. Inside its selected pair, place `<w:r><w:t>A</w:t></w:r><q:fldSimple q:instr=" PAGE "><q:r><q:t>7</q:t></q:r></q:fldSimple><w:r><w:t>B</w:t></w:r>`, followed by its one qualified reference. Retain ordinary comments and companion metadata from an existing valid root thread. The package source is namespace-valid and the selected display is `A7B`. The same paragraph and block-local alias-field source forms have independently retained exact Base saveability evidence in this freeze.

The canonical model serialization drops the local modeled-owner declaration while retaining the raw `q:fldSimple` subtree. The caller at `comments.rs:2703` passes that already incomplete paragraph to the new source projection. `project_comment_text` at `crates/rdocx-oxml/src/text.rs:5589` treats unresolved q elements as non-Word rather than reporting an unresolved binding. The raw field is therefore absent from the rich accepted text. Its endpoint fidelity probe repeats that incomplete context, allowing an `AB` projection to validate itself instead of retaining `A7B`. This is an incorrect query result. No particular expected run index or accepted-axis expansion is claimed. A block-local declaration inherited by its paragraph has the same source-loss route.

Supply the proved original namespace context to rich query projection without weakening qualified identity, unsupported-source checks or global canonical fingerprints. Do not infer a missing binding from the q spelling. A checked refusal would avoid a false result but would not satisfy supported source handling by itself. Add a positive imported anchor spanning the raw field, including anchor text and endpoint fidelity. Existing new tests cover adjacent literal `A` and crossing literal refusal, not this rich imported span. This finding is source-grounded, with no additional reviewer execution or new runtime claim.

## Smells

Zero.

## Nitpicks

Zero.

## Review by aspect

### Correctness

D1. Checked root-only lookup, unique complete movable graph, staging and prepare/reopen before one publication, supported cross-story placement and the shared main-body literal finder. Source deletion and destination rebasing use the actual accepted reference-run index and physical paragraph identity. Mixed reference runs retain neighboring children. Only removal of the whole reference run adjusts a later run boundary. Source metadata and companion parts remain owned by the original thread.

Reviewed namespace handling at the three move source-write boundaries and literal safeguard. Source removal applies the same qualified marker edits to the original package namespace authority. Entire transported runs leave that authority, while mixed runs retain remaining declarations and children. Selected-owner snapshot refresh requires a unique same-kind owner, exact retained raw namespace-marker multiset and namespace facts. Unselected owners retain the existing strict replay checks. The source fingerprint axis remains canonical. A possible offset shift from replay was examined and did not establish a defect because subsequent story scans regenerate the same canonical axis used for original endpoint offsets.

### Contract

D1. Additive native, Python and CLI APIs follow the approved signatures and root/reply distinction. The move-to-text operation allocates no temporary comment. The literal operation preserves tab and break zero width, qualified field-result safeguards and refusal atomicity. The rich X185 anchor display remains a separate operation and continues to report actual display characters. Unknown, reply, stale, malformed, unsupported and ambiguous cases fail without publication or Python revision advance.

Google pruning requires a proved marker-only raw skeleton after this move. Opaque attributes, namespace declarations, comments, processing instructions, neighboring text and unrelated payload prevent pruning. Cross-story support follows existing checked placement rather than a new body-only restriction. PR287 routing and finder ideas are adapted under this stricter ownership contract, without importing temporary identity allocation, auto-reanchoring or pop carriers.

### Panics

Zero findings. Reviewed changed slicing, draining, offsets, indexing, arithmetic and expect/unwrap sites in production. Source spans, unique ownership, endpoint limits and edit overlap checks precede mutation. Candidate-only fallible operations prevent publication on error. Test assertions are not production panic findings.

### OOXML

D1. Checked qualified identities, inherited and local namespace closure, shadowing, unmodeled reference attributes and child payload, standalone comments and processing instructions, raw child order, schema placement and run-property transport. Flagged raw reference carriers remain owned by exact typed boundaries. Independent unflagged raw references retain bytes even when private positions are absent. Plain own Word alias declarations retain established canonical serialization, while foreign declarations and payload retain source.

The new static rich source projection shares the existing endpoint fidelity probe and unsupported-child checks. The facade intends to supply original namespace-closed paragraph source, but D1 exposes the remaining canonical-source gap for retained local declarations. Conditional binding closure preserves existing local or shadowed declarations rather than forcing duplicate root bindings. The literal source reader validates complete element and attribute namespace resolution before extracting qualified Word text, including source after the selected end and inside skipped text subtrees.

### Tests

D1 identifies a missing rich imported-span control. Retained actual compiled fail-before evidence establishes the original identity transport and Google pruning failures, CLI failure and rebuilt Base Python missing API. Pass1 raw-reference aggregate and unchanged canonical alias tests prove their remediation. Pass2 direct move aggregate proves the tab/break and ns0-control failures before repair. The unchanged original compatibility tests now pass in the full suite.

Current source controls cover supported stories, same-paragraph pure and mixed reference runs, block-control paths, stale and invalid endpoints, namespace aliases, foreign lookalikes, local shadowing, metadata and companion preservation, no temporary identity allocation and exact refusal bytes. Saveable Base paragraph and block alias-field sources now support adjacent literal moves with declaration and raw field retained after reopen. Crossing those fields refuses. Initially unsaveable table and cell alias-field fixtures are explicitly qualified as earlier atomic refusal, not falsely described as previously successful sources. Raw alias SDT cases outside the existing accepted literal axis remain explicit no-occurrence refusals. Duplicate namespace owners retain strict refusal.

### Structure

Zero findings. No production file, module, crate, dependency, trait, generic or feature flag was added. Concrete helpers have actual move, literal safeguard and rich-reader consumers. The localized tuple lint allowances have concrete rationales. Existing global namespace replay was not relaxed and no second ownership inventory or speculative abstraction was introduced.

## Prior findings

Pass1 D1, D2 and D3 are resolved. The independent raw construction aggregate covers no typed reference, same typed id and different typed id. Flagged carrier lifecycle still covers emission once, typed id mutation, removal, replacement, splitting and segment writing. The unchanged canonical alias assertion passes. Current denied-warning Clippy passes.

Pass2 D1 is resolved. The literal guard now reads the unpublished actual main source using qualified text-only semantics, while static source projection preserves rich reader namespace context. Both original regression failures and the direct move control pass. The broader namespace repair has explicit positive and atomic refusal evidence as described above.

## Current deciding verification

Authenticated current runtime receipt `/private/tmp/fx186-runtime-20261008T183346Z/receipt.json` has four successful steps, including actual current extension rebuild and Python core 94 passing. Production and test bytes remain unchanged across this run. Approved plan and HLD04 metadata clarification accounts for its nonidentical full metadata snapshots. Imported current extension SHA-256 is `329c34ab02410bcb33f4c54befd29d4c6990b73384ea7b78fd9368a4416ca6cb`.

Authenticated scoped receipt `/private/tmp/fx186-20261008T183503Z/receipt.json` has 12 successful steps, not 13. Results include oxml 631 unit tests and one doctest, facade 501 unit, 364 integration, 901 regression and two doctests, CLI three unit and 62 integration tests, affected all-target checks, all-feature denied-warning Clippy, denied-warning docs, WASM, two pinned rendering/concurrency controls, mypy, stubtest and unchanged hash49. The facade retains 21 existing ignored tests.

Authenticated remainder receipt `/private/tmp/fx186-20261008T183950Z/receipt.json` has eight successful steps. It covers archive measurement, actual locally patched publish dry runs, actual dry-run archive provenance, affected README examples and inventories, workflow tests, prose, adapter drift and fmt. Workflow ran 140 tests with two existing skips in 97.032 seconds. README validation covers 27 workspace READMEs and 22 publishable inventories, with four root examples and one oxml example compiled. Prose reports zero violations and all 26 adapters are in sync.

## Archive and metadata reconciliation

Independently opened all six retained measured and actual publication archives. Every archived source and test member equals the current worktree. Recomputed the existing normalized VCS metadata measurement, without changing its policy. The three measured tuples are:

| Package | Compressed bytes | Normalized member bytes | Members |
|---|---:|---:|---:|
| rdocx-oxml | 447616 | 2784206 | 32 |
| rdocx-cli | 72545 | 324047 | 8 |
| rdocx | 1554674 | 8622430 | 36 |

Actual publish dry-run archives have compressed sizes 447612, 72545 and 1554675 respectively, with the exact same normalized member-byte and member-count tuples. These bounded differences satisfy the unchanged 64-byte compressed policy. No exact compressed archive identity or cause of those differences is inferred. All are below 10 MiB. The four final metadata paths change only the three measurement rows and corresponding tuples, not tolerances or version selection.

Reviewed actual publish helper arguments, including `--dry-run`, `--allow-dirty` and all existing local dependency patches. The actual log explicitly aborts upload due to dry run for all three packages. Retained archives from the actual dry-run path are separately bound from measured package archives.

## Limits and handback

This is a feature-scoped review with a blocking defect, not feature acceptance. No full 35-case Python parity rerun, full integrated workspace gate, sprint review, native Word capture, release, push or publication is claimed. Historical failures remain historical evidence and are not current gate failures. No harness baseline moved. F-282 semantic reconciliation and the remaining sprint workflow belong to subsequent orchestration.

Only this review record was written. No source, test, plan, HLD, progress, metadata, archive or baseline was changed, and no Cargo command was run by the reviewer. Return control for separate remediation. Prepare and integration remain blocked by D1.

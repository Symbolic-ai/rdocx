# F-281, microscope, pass 1

**Reviewed**: Frozen working diff on `work/f-281-codex`, Base and Head `86d8108a9cc781c663fcf5f9b0dccbed21146cd4`. Sixteen modified files, 4199 added lines and 35 removed lines. Binary diff SHA256 `3d09244dc4b7a4cee0772f2bea77dd3ebc57b6481206c718d4ff8b8a9f32571d`. Freeze manifest SHA256 `67bdc7a580fe65c65fa1e274d5d1cdb7d400c5de5d4bbaff34943e578e3fba66`.

**Verdict**: 3 defects, 0 smells, 0 nitpicks. Completion is blocked pending separate remediation and another independent pass.

The approved design, cited HLD sections, workflow, microscope command, canonical differential-testing guidance and progress checkpoint were read before reviewing the implementation. Branch, status, all sixteen frozen file hashes, progress hash and every supplied scoped receipt hash were independently checked. The source hashes and binary diff were checked again immediately before this report was written. No Cargo, Word UI, source mutation or test execution occurred in this review. The counterexamples below are reproducible static XML and control-flow traces, not claimed Rust runtime executions.

## Defects

### D1, story projection changes a retained foreign namespace into Word

`crates/rdocx/src/field.rs:18201`

`generated_story_inventory` creates a wrapper with `xmlns:w` bound to Word and deliberately skips the original `w` binding at line 18203. This changes the expanded names of retained children when a valid document uses another prefix for Word and binds `w` to a producer namespace.

Reproduce with a namespace-correct `q:document` and `q:body`, where `xmlns:q` is the WordprocessingML namespace and `xmlns:w="urn:producer"`. Put an opaque `<w:p><w:r><w:t>opaque</w:t></w:r></w:p>` sibling before two valid `q:p` paragraphs. The first valid paragraph contains source text and a typed `q:fldSimple` XE Alpha marker. The second contains a supported INDEX owner and its old cache. Call `rebuild_generated_tables`.

The original opaque paragraph has expanded name `{urn:producer}p`. The constructed wrapper reinterprets that exact byte slice as `{http://schemas.openxmlformats.org/wordprocessingml/2006/main}p`. Python ElementTree independently confirmed this expanded-name change for the exact wrapper operation. Consequently the wrapper's typed paragraph inventory includes the opaque paragraph, while the checked original owner inventory excludes it. `crates/rdocx/src/field.rs:18719` returns the physical paragraph inventory disagreement before any supported table can rebuild. A foreign retained sibling must remain opaque under its original bindings. The required namespace-qualified source projection must not redefine it.

### D2, simple-owner expansion discards locally owned namespace declarations

`crates/rdocx/src/field.rs:19791`

The convertible-attribute check skips local namespace declarations, but expansion does not carry them into the replayed cache at `crates/rdocx/src/field.rs:19825`. Reproduce in an otherwise ordinary `w:document` with an ASCII XE source and this supported generated owner:

```xml
<w:p><q:fldSimple xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main" q:instr="INDEX \z 1033"><q:r><q:t>OLD</q:t></q:r></q:fldSimple></w:p>
```

The owner is typed and its instruction is supported. Expansion removes `q:fldSimple`, including its sole `xmlns:q` declaration, writes new `w:r` field markers, and copies `<q:r><q:t>OLD</q:t></q:r>` unchanged between them. Nothing declares `q` on those sibling cache runs. The resulting staged XML has an unbound prefix. ElementTree accepted the original and rejected the mechanically expanded XML with `unbound prefix`.

That malformed package is published into the staged part and immediately reopened at `crates/rdocx/src/field.rs:19861` and `crates/rdocx/src/field.rs:19863`. Even if a permissive parser can later replace the malformed old result, reparsing it has already lost the namespace qualification required by the ownership contract. Preserve local declarations for every replayed subtree, or retain a nonconvertible owner with a diagnostic. A supported simple owner must not expand through a namespace-invalid intermediate package. The original live document remains protected by staging, so this finding does not claim partial live publication.

### D3, valid self-closing generated owners are silently skipped

`crates/rdocx/src/field.rs:6129`

The Empty event branch handles complex `fldChar` markers but never registers a generated `fldSimple` owner. Simple span creation occurs only in the End event branch at `crates/rdocx/src/field.rs:6184`. This misses the valid empty-element form already admitted as a typed Field by `crates/rdocx-oxml/src/text.rs:7771` through line 7795.

Reproduce with ordinary Word namespace bindings and these body paragraphs:

```xml
<w:p><w:r><w:t>source</w:t></w:r><w:fldSimple w:instr="XE Alpha"/></w:p>
<w:p><w:fldSimple w:instr="INDEX \z 1033"/></w:p>
```

The source marker is discoverable, but the INDEX field produces no dynamic span. Normalization and generated owner discovery both iterate that empty span inventory. `crates/rdocx/src/field.rs:18139` therefore returns a successful zero-entry, zero-diagnostic report, leaving the supported INDEX unmaterialized. An explicitly opened and closed empty `fldSimple` owner takes a different path and can rebuild. Empty-element spelling must not determine whether a namespace-correct, unlocked supported generated instruction is discovered. Apply the same ownership and attribute checks to both spellings, including TOA and caption-selected TOC.

## Smells

Zero findings.

## Nitpicks

Zero findings.

## Not found

- **Correctness outside D1 to D3**: No additional finding in hierarchy grouping, measured lowercase-first ASCII ordering, authority short-key grouping, category heading selection, five-distinct-page passim, chapter endpoint values, measured k behavior or caption selector separation.
- **Contract outside D1 to D3**: No additional finding in additive APIs, category None insertion as numbered owners, locked and unsupported owner retention, bounded non-ASCII retention, source exclusion, required target refusal, staged atomic publication or existing rebuild_toc compatibility.
- **Panics**: Zero new findings. Checked target and offset paths, field inventory arithmetic and validation precedents were inspected. Existing scanner assertions concern stack members already borrowed and validated in the same scope.
- **OOXML outside D1 to D3**: No additional finding in complex owned result spans, original instruction and boundary retention, checked internal hyperlink targets, requested leaders, producer style preservation or generated cache serialization.
- **Tests outside D1 to D3**: No independent gate defect found. The named `generated_tables_match_pinned_word_after_source_mutation` compares six source-built initial and mutated INDEX, authority and figure vectors with the authenticated native semantic records. The actual runtime reversion receipt changes the INDEX cross-reference separator, fails that named test at the initial Alias entry with exit 101, and restores the exact frozen field.rs SHA before the same test passes with exit 0. This demonstrates a behavioral assertion rather than a compilation-only reversion. The discovered namespace and empty-element cases need distinct regression coverage during remediation.
- **Structure**: Zero findings. No new crate, production file, trait, generic parameter, forwarding-only wrapper or speculative dependency. New helpers have concrete generated-table consumers, and the existing scanner also remains the TOC compatibility path.
- **Packaging, API and HLD reconciliation**: Zero additional findings. All six named HLD files, exported types, facade signatures, completed-capability owner assertion and affected archive footprint tuples were inspected against the approved plan. Hash-bound receipts cover scoped tests, clippy, documentation, external consumer compilation, verified publish dry-run, archive size, README compilation, prose and adapter checks. The 49-entry hash harness remains unchanged. These scoped receipts do not replace the integrated sprint gate.

## Evidence limits

The one deterministic layout call in `rebuild_generated_tables` follows provisional cache insertion, and both page endpoint lookups consume that immutable result. The source-built pagination rider checks a table that moves source pages. Its later observation of final layout does not cause a second paginator inside the transaction.

Native no-F9 reopen receipts prove bounded producer persistence and native PDF stability. They do not themselves prove Rust rendering parity. Captured ASCII and en-US contracts do not claim general Unicode collation. This report accepts those explicit approved boundaries and does not introduce a new dependency or broaden their measured claims.

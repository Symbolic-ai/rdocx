# F-280, microscope, pass 2

**Reviewed**: The complete frozen working diff on `work/f-280-codex`, Base and HEAD `cf75a087eeb00ad63f9883cd044a12e41066461d`, 29 tracked files, +12529/-2238 lines. Binary diff SHA-256 `b5bcc4cdf8b0f3bdc234c5715e4876d82a4376fd7c3b49039500ee4c1e0a4e91`. All six default aspects were checked against AGENTS.md, CLAUDE.md, the canonical workflow and microscope command, the approved F-280 plan and its riders, the cited HLD, risk routing and differential-testing guidance. This pass covers the whole implementation, including the four pass-1 remediations.

**Verdict**: 0 defects, 0 smells, 0 nitpicks.

This is an independent static review with authenticated native evidence and existing worker verification receipts. No Cargo or UI command was run. No implementation, progress, design or sprint record was changed. Before writing this record, the frozen binary patch, all29 tracked file hashes, branch, HEAD, numstat, previous review, progress and nine bound verification logs matched the freeze identity. Native evidence qualifies its measured inputs and owners. It does not establish universal renderer parity or unmeasured behavior.

## Defects

Zero defects.

## Smells

Zero smells.

## Nitpicks

Zero nitpicks.

## Pass 1 closure

- **D1, same-paragraph rendered position**: `crates/rdocx-layout/src/lib.rs:157` compares exact accepted field and paired bookmark boundaries within one physical owner. `crates/rdocx-layout/src/engine.rs:9345` and `crates/rdocx/src/field.rs:12159` consume that shared projection. `crates/rdocx/tests/regression_test.rs:51319` exercises before, after and containment through pure evaluation, update, saved reopen and actual deterministic painting. The old paragraph-ID comparison is removed.
- **D2, related targets**: `crates/rdocx/src/field.rs:11589` qualifies ordinary related bookmark text, with exact projected paired boundaries at `crates/rdocx/src/field.rs:11499`. `crates/rdocx-layout/src/engine.rs:10075` separates physical story inventories and deduplicates related part aliases. `crates/rdocx-layout/src/engine.rs:10577` uses that inventory for target text. `crates/rdocx-layout/src/lib.rs:375` exposes existing resolved bookmark numbering rather than reconstructing facade owner paths. The five-family regression at `crates/rdocx/tests/regression_test.rs:51581` and related nested-cell discriminator at `crates/rdocx/tests/regression_test.rs:53035` cover text, numbering, duplication refusal, preservation and painting.
- **D3, cross-owner position**: `crates/rdocx-layout/src/lib.rs:238` rejects unavailable cross-owner comparison. `crates/rdocx/src/field.rs:12159` preserves the complete cache before scalar or numbered resolution, and `crates/rdocx-layout/src/engine.rs:10205` propagates the same failure before rendering replacement text. `crates/rdocx/tests/regression_test.rs:51581` covers scalar and combined number-plus-position retention, diagnostics, update, reopen and paint.
- **D4, raw unclosed quoting**: `crates/rdocx-oxml/src/text.rs:1341` preserves the existing escaped-quote algorithm in the approved hidden concrete getter. The snapshot checks it before sequence mutation at `crates/rdocx-layout/src/engine.rs:1836`, and the facade shape validator uses it at `crates/rdocx/src/field.rs:14940`. `crates/rdocx/tests/regression_test.rs:51531` proves malformed cache retention and a valid successor1 through pure evaluation, update, reopen and paint. `crates/rdocx-oxml/src/text.rs:13662` covers escaped quotes and backslashes.

## Not found

- **correctness**: Zero findings. Checked authoring validation and staged publication, source-qualified sequence ordering and page context, note and annotation refresh without accumulation, numbered delimiter semantics, related target qualification, shared position projection and nested/generated/locked source handling. The private accepted field boundaries and physical bookmark owner bindings participate in snapshot equality at `crates/rdocx-layout/src/lib.rs:100`. Full layout regenerates the current snapshot at `crates/rdocx-layout/src/engine.rs:3626`. The source-only no-SEQ and lock/generated/freshness discriminator is at `crates/rdocx-layout/src/engine.rs:13698`.
- **contract**: Zero findings. Checked all approved caption, SEQ and REF branches and riders, the five HLD impact files, existing Python/WASM/CLI preservation boundaries, hidden additive projections and actual consumers. Same-owner containment remains the explicitly approved conservative cache-plus-diagnostic policy at `.claude/plans/F-280-design.md:812`, rather than a claim to reproduce Word's self-reference error. No reduced REF scope or synthetic native result is presented as completion evidence.
- **panics**: Zero findings. Checked new untrusted-input indexing, XML slices, checked conversions and allocation arithmetic, clone-first mutation, and invariant-bound unwraps. No independently cited input-triggerable panic was established.
- **ooxml**: Zero findings. Checked field cache sibling order, raw simple/complex/nested publication boundaries, namespace closure, optional comment companion ownership, last-paragraph/durable ID remapping, note/media relationship closure and selected drawing identity. Original source owners and unmodelled XML remain preserved. The invalid direct paragraph drawing remains excluded. No parser relaxation was introduced to admit it.
- **tests**: Zero findings. Read the complete added regression and layout/XML coverage, including original caption/renumbering, optional bookmark, physical sequence, rich note/comment, delimiter and copy-refresh controls. The new36-field regression at `crates/rdocx/tests/regression_test.rs:51383` distinguishes native capture from a derived input and asserts opaque Fallback and exact instruction preservation. The named caption gate has a compiling semantic reversion that fails its runtime REF1 assertion, followed by exact restoration and a passing unchanged gate.
- **structure**: Zero findings. No new production file, module, crate, speculative trait, generic parameter or forwarding wrapper. The concrete snapshot and two hidden projections have existing facade and renderer consumers. Source identity remains separate from public visible evaluation indices. Header/footer anchor painting reuses existing placement machinery at `crates/rdocx-layout/src/paginator.rs:5183`. The implementation remains within the approved existing files.

## Native provenance and verification qualification

Independently decoded the same-owner embedded fixture and verified SHA-256 `61092dd6db5b3cf3caffc9db4b0b0318985da4385288c9f12f32c70e989bceae`. Its OPC member set equals the genuine normalized native stage `6283b061ebcaf7a1f23bda0181e1235e447f1cf15c6dbfe5730379a6f28ba790`. Comparing every XML element, attribute and tail establishes that only42 `w:t` payload texts changed across five parts:36 selected caches and six opaque Fallback caches. Other package members are byte-identical. The preparation receipt and independently authenticated nine-stage audit remain external immutable evidence. The audit preserves all36 instructions and12 selected bookmark pairs. Native close and no-F9 reopen retain rich caches, but439 of861696 PDF RGB pixels differ. Equal page text is not pixel parity, and this review does not claim it.

The final scoped receipt reports122 shared-layout units,500 facade units with6 ignored,360 integrations with8 ignored,844 regressions with7 ignored,317 Word-layout units,627 XML units and all four doc-test groups passing. A final focused rerun covers the subsequent Fallback and instruction assertions. Warnings-denied all-target/all-feature clippy and affected documentation pass. README verification covers27 workspace READMEs and22 package inventories. Policy reports140 tests with2 skips. Format, prose and26 generated adapters pass. The deterministic hash harness retains all49 baselines unchanged.

All four affected actual publish dry runs verify extracted package source with local dependency patches and no upload. The archive receipt confirms every archived source member matches the reviewed working source. Compressed sizes are4634504,440535,308769 and1486141 bytes, all below10485760. These are reviewed worker receipts, not independently rerun Cargo verification. Integrated full verification and sprint review remain the orchestrator's responsibility.

The frozen source identity was checked again after writing this review. This pass makes no implementation changes and returns control for structured preparation.

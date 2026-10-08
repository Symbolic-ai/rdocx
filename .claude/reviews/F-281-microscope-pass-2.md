# F-281, microscope, pass 2

**Reviewed**: Frozen working diff on `work/f-281-codex`, Base and Head `86d8108a9cc781c663fcf5f9b0dccbed21146cd4`. Sixteen modified files, 4433 added lines and 37 removed lines. Binary diff SHA256 `a308cf94da2ebb43ee6bdf3956deec211924aa83fb3041e1b3903131c3c50022`. Freeze manifest SHA256 `a7472bb2f66d71568cfc7391444e3dd547d3a10f6320fa65871e964dd79138b3`.

**Verdict**: 0 defects, 0 smells, 0 nitpicks. The microscope exit condition is satisfied for this frozen diff.

The approved plan, progress checkpoint, pass 1 report and canonical microscope instructions were read. The previously reviewed unchanged workflow and cited HLD contracts remain applicable. The two changed HLD boundary statements were independently inspected. F-281 remains in-progress in both sprint records. Branch, Base, status, every modified file, unchanged pass 1 report, progress and all iteration 2 scoped receipt bindings match the manifest. Frozen source hashes, status and binary diff were checked again immediately before this report was written. This pass performed read-only source and receipt analysis. No Cargo, UI, code edits, formatting, queue changes or remediation occurred.

## Defects

Zero findings.

## Smells

Zero findings.

## Nitpicks

Zero findings.

## Previous findings

- **Pass 1 D1, closed**. `crates/rdocx/src/field.rs:18252` chooses an unoccupied wrapper prefix and carries every original binding. It no longer converts foreign `w` elements into Word content. `crates/rdocx/tests/regression_test.rs:55852` exercises the original foreign-w fixture, checks that it reaches the explicit serializer refusal rather than the old inventory mismatch, and asserts exact package, namespace and opaque cache preservation. Its safe Word alias and foreign-x sibling control rebuilds and reopens with the measured Alpha entry. The refusal comes from the unchanged checked mutation guard at `crates/rdocx/src/document.rs:7303` and the unchanged serializer guard at `crates/rdocx/src/document.rs:14675`. This preserves the existing fail-closed mutation boundary under the approved atomicity and XML-preservation contract. It does not claim the foreign-w fixture rebuilds successfully or relax canonical serialization. The API rustdoc and HLD packaging and facade statements now describe this boundary explicitly.
- **Pass 1 D2, closed**. `crates/rdocx/src/field.rs:19804` closes each replayed top-level cache subtree over its namespace scope. Normalization merges inherited declarations with declarations local to the simple owner, retaining subtree-local overrides. It validates the staged XML before setting its package part. `crates/rdocx/tests/regression_test.rs:55883` covers the original local-q cache, multiple cache children with an interleaved comment, exact unknown producer metadata, exact unsupported owner retention and save/reopen. It materializes the supported owner without an unbound namespace.
- **Pass 1 D3, closed**. `crates/rdocx/src/field.rs:6153` gives self-closing simple fields the same accepted parent, run position, instruction and span checks as opened simple owners. Conversion admits either XML spelling. `crates/rdocx/tests/regression_test.rs:55908` materializes INDEX, TOA and caption-selected TOC from empty XE, TA and SEQ fields. It checks locked and unsupported empty owner retention, exact entries and reopen, then checks missing-range atomic refusal. Empty spelling no longer silently removes a supported generated owner from the inventory.

## Not found

- **Correctness**: Zero findings. Reviewed hierarchy and duplicate grouping, physical source order, bounded ASCII ordering, chapter context and range endpoints, cross-reference punctuation, authority category and short-citation grouping, measured passim threshold and caption selection. Rechecked the remediation against the surrounding scanner and normalization paths.
- **Contract**: Zero findings. Additive signatures, numbered category None insertion, old cache exclusion, source-qualified targets, unsupported and locked owner retention, bounded non-ASCII diagnostic retention and atomic publication match the approved contract. Safe namespace aliases remain accepted. A required mutation that cannot safely serialize remains an explicit atomic error. Existing rebuild_toc retains its compatibility scanner mode.
- **Panics**: Zero findings. Checked offsets, span slices, stack membership, source inventory arithmetic and the new cache subtree depth handling were inspected. No new unchecked failure on the reviewed inputs was identified.
- **OOXML**: Zero findings. Checked simple and complex ownership, namespace closure, schema serialization, instruction retention, owned separator-to-end edits, checked hyperlink anchors, styles, leaders and preservation of unrelated producer XML.
- **Tests**: Zero findings. Each D1, D2 and D3 regression has a separate compiled behavioral reversion in the authenticated iteration 2 logs. The respective failures report the old paragraph inventory mismatch, an unbound namespace prefix and zero entries instead of three materialized owners. Each exits 101. The native semantic gate reversion also exits 101 on Alias comma versus the measured period. Exact restoration to field.rs SHA256 `48cc785699a64a0c39b44be010e19f9dad4cefe246a7626af8494ead28375cf9` makes all four named tests pass with exit 0. These are runtime assertion failures and passes, not compilation-only controls. Existing native initial and mutated semantic records remain unchanged.
- **Structure**: Zero findings. Concrete helpers remain in existing files. No new crate, production file, trait, generic parameter, speculative dependency or forwarding-only abstraction was introduced. Cache namespace closure is local to the conversion that consumes it.
- **API, packaging and HLD**: Zero findings. All six named HLD files, exported facade signatures, completed capability assertion and the affected archive measurements were reconciled. The iteration 2 archive rider records rdocx at 1,513,409 compressed bytes, 8,382,821 normalized member bytes and 36 members, with rdocx-oxml unchanged at 441,312 compressed bytes, 2,752,259 normalized member bytes and 32 members. Verified dry-run compilation, package size, external API compilation and README receipts are hash-bound.

## Validation and limits

Supplied scoped receipts show 500 rdocx unit tests, 360 rdocx-oxml unit tests, 861 regression tests, 628 OOXML integration tests and three doctests passing. Clippy, warning-denied documentation, formatting, prose, generated adapters, repository policy and diff checks pass. The existing 49-entry hash harness is unchanged. The reviewer verified the receipts rather than rerunning those commands during the frozen review.

The generated-table transaction still makes one deterministic pagination call after provisional result insertion. All page and range values are consumed from that immutable snapshot. The source-built pagination rider and native semantic gate remain separate from native no-F9 persistence evidence. This review does not claim general Unicode collation, other-locale sorting, Rust raster parity or integrated sprint verification. Return control to /run-sprint for the remaining authorized lifecycle steps.

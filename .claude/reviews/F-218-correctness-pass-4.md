# F-218, correctness, pass 4

**Reviewed**: claim base and source HEAD
`424969e8be199f8618d3d7558299b71633cf5582`, the complete working-tree delta,
the approved plan, cited HLD sections, scratch record, and correctness passes 1
through 3. The implementation delta is 6 files with 3,320 additions and 8
deletions, including the untracked `crates/rpptx/src/embedded.rs`.

**Verdict**: 0 defects, 0 smells, 0 nitpicks. Ready.

## Defects

None. Count: 0.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Not found

- **Pass-3 D1, same-frame OLE identity**: each schema-owned OLE reference is
  accumulated on its nearest graphic-frame owner at
  `crates/rpptx/src/embedded.rs:1555` and
  `crates/rpptx/src/embedded.rs:1594`. Frame close sorts and deduplicates exact
  relationship ids, rejects more than one distinct id, and emits at most one
  whole-frame ownership range at `crates/rpptx/src/embedded.rs:1608`. Repeated
  compatibility aliases therefore inventory once, while distinct Choice and
  Fallback identities fail before inventory, extraction, replacement, or
  removal can select overlapping ownership. The focused regression proves
  both requested identities fail without changing presentation bytes at
  `crates/rpptx/tests/integration.rs:88`.
- **Pass-3 D2, signature-origin relationship closure**: after validating the
  complete origin relset identity, graph construction rejects every
  relationship whose type is not the package digital-signature type at
  `crates/rpptx/src/embedded.rs:1230`. This check precedes graph construction
  and signature removal, so neither an internal nor external unrelated edge
  can be discarded with the origin. The malformed-topology regression includes
  both cases and verifies inventory, mutation rejection, and unchanged live
  state at `crates/rpptx/tests/integration.rs:990` and
  `crates/rpptx/tests/integration.rs:1068`.
- **Prior signature topology and persistence fixes**: package-root origin
  cardinality, legal relationship placement, safe internal targets, required
  parts, duplicate ids and targets, complete VBA relset identity, and private
  invalidation-marker topology remain fail closed. Default and all-feature
  fresh reopen retain known package and VBA invalidation without modifying
  signature evidence or expanding the public API. No adjacent recurrence was
  found.
- **Producing scopes and ownership-aware deletion**: OLE discovery and mutation
  cover slides, loaded layouts, and presentation-listed masters. Shared
  payloads, previews, unrelated producer orphans, and retained raw owner XML
  remain independent. The source-built test covers exact layout and master
  mutation at `crates/rpptx/tests/integration.rs:138`, while the tracked corpus
  test confirms two layout and one master logical OLE owners at
  `crates/rpptx/tests/integration.rs:225`.
- **Atomicity**: each public mutation selects from a validated inventory,
  operates on a consolidated staged clone, serializes and reopens the complete
  candidate, runs presentation validation, and publishes only on success at
  `crates/rpptx/src/embedded.rs:152` and
  `crates/rpptx/src/embedded.rs:521`. The new ambiguity and topology failures
  occur before the live value can change.
- **OOXML namespace, schema order, and raw preservation**: discovery continues
  to resolve element and relationship-attribute expanded names, accept
  Transitional and Strict aliases, enforce complete schema-owner paths, and
  exclude foreign or nested same-namespace lookalikes. Removal changes only
  validated owner ranges and reparses through the correct typed root. No
  prefix-shadow, declaration-loss, raw-subtree, or schema-order defect was
  found.
- **Panics and bounds**: same-frame cardinality is checked before use, package
  graph cardinality proves the one indexed origin, ancestry movement is
  checked, and malformed XML and relationship graphs return errors. No
  reachable untrusted-input panic, integer overflow, unchecked depth
  decrement, or invalid slice access was found.
- **Public API and structure**: the three enums, information value, and four
  facade methods remain the approved additive native Rust API. No Python,
  WASM, CLI, feature, trait, generic, wrapper, crate, dynamic dispatch, binary
  fixture, or integration binary was added. The private module remains the
  approved structural seam, and dependency direction is unchanged.
- **Focused validation**: the two pass-3 regressions each passed independently
  under all features. The all-feature embedded filter passed 7 tests, the
  signature filter passed 11 tests, and the tracked producing-scope corpus test
  passed. `cargo check -p rpptx --all-targets --all-features`, the shared-crate
  dependency-direction regression, `cargo fmt --all --check`, and
  `git diff --check` passed. The scratch record also reports the complete
  post-remediation workspace, corpus, LibreOffice, hash 49 of 49, publication,
  WASM, documentation, and dependency gates green.
- **Differential testing**: the approved plan assigns no external application
  oracle to this opaque package-graph API. The tracked PowerPoint corpus deck
  supplies the required real producer evidence, so no missing oracle was
  counted as a finding.

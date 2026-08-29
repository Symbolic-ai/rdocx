# F-X066, working, pass 3

**Reviewed**: complete pass-2 remediated working diff against claim Base
`3ddac3a3420eda6dc25abd9c5b1dce5721725834`, 6 files, 437 insertions and
20 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-2 D1 closure: `CT_R` retains its earlier required field shape at
  `crates/rdocx-oxml/src/text.rs:425`, and the exact public struct-literal
  regression at `crates/rdocx/src/run.rs:785` compiles without a namespace
  field.
- Pass-2 D2 closure: derived `PartialEq` includes the encoded raw-position
  sidecar at `crates/rdocx-oxml/src/text.rs:423`. The regression at
  `crates/rdocx/tests/regression_test.rs:1937` proves identical raw bytes with
  valid and foreign expanded names receive different classifications and do
  not compare equal.
- Pass-2 S1 closure: ordinary runs allocate no classification or namespace
  sidecar at `crates/rdocx-oxml/src/text.rs:750`. Only an actual unknown Word
  `pict` triggers temporary scope materialization and parsing at
  `crates/rdocx-oxml/src/text.rs:832`. The ordinary-versus-raw regression is at
  `crates/rdocx/tests/regression_test.rs:1961`.
- Sidecar encoding: the high flag and boundary mask are disjoint at
  `crates/rdocx-oxml/src/text.rs:440`. Reads decode the boundary at
  `crates/rdocx/src/run.rs:600`, serialization decodes it at
  `crates/rdocx-oxml/src/text.rs:1034`, and RTF diagnostics decode it at
  `crates/rdocx/src/rtf.rs:707`.
- Boundary mutations: content replacement, property insertion, and content
  removal preserve the classification flag at
  `crates/rdocx-oxml/src/text.rs:668`,
  `crates/rdocx-oxml/src/text.rs:687`, and
  `crates/rdocx-oxml/src/text.rs:700`. Content-control display insertion uses
  the same decode-and-preserve contract at
  `crates/rdocx/src/content_control.rs:441`.
- Namespace correctness: the parser first requires the raw root to be a Word
  `pict` by expanded name at `crates/rdocx-oxml/src/text.rs:832`, then resolves
  the VML `rect` and Office `hr` names through the temporary inherited scope at
  `crates/rdocx-oxml/src/text.rs:477`. Local declarations and shadows remain
  authoritative through `NsReader`.
- Strict fallback: duplicate Office markers, foreign namespaces, numeric or
  false values, extra shapes, visible content, comments, malformed events, and
  every unexpected state return the unsupported classification at
  `crates/rdocx-oxml/src/text.rs:455` and
  `crates/rdocx-oxml/src/text.rs:514`. The negative matrix is at
  `crates/rdocx/src/run.rs:833`.
- Raw and package preservation: the classifier records only a sidecar flag and
  leaves `extra_xml` unchanged at `crates/rdocx-oxml/src/text.rs:833`. The
  package regression at `crates/rdocx/tests/regression_test.rs:1898` proves
  ancestor-only namespace bindings, exact raw bytes, source item order, save,
  and reopen.
- Public contract: `LegacyHorizontalRuleRef::raw_xml` exposes the preserved
  subtree at `crates/rdocx/src/run.rs:123`, and the additive variant remains in
  the existing non-exhaustive enum at `crates/rdocx/src/run.rs:153`. Layout and
  rendering have no new consumer.
- Panics and errors: production classification converts XML, namespace,
  attribute, and write failures to the unsupported result at
  `crates/rdocx-oxml/src/text.rs:455` and
  `crates/rdocx-oxml/src/text.rs:477`. No new unchecked indexing, slicing, or
  untrusted-input unwrap was introduced.
- Tests and evidence: focused classification, equality, ordinary-allocation,
  boundary-order, package, affected crate, hash, WASM, publish-package, and
  pinned Word corpus evidence is recorded. Each new regression is in an
  existing test-bearing file, and reverting parse-boundary tagging breaks the
  positive facade and package assertions at `crates/rdocx/src/run.rs:813` and
  `crates/rdocx/tests/regression_test.rs:1930`.
- Structure and scope: no new file, module, dependency, trait, generic, feature
  flag, or forwarding-only public API was added. No HLD or sprint delivery file
  changed before completion.

# F-X071, correctness, pass 7

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through final source `4c6e1fb223640494209bf984f9a69d42ccd1b389`, 22 implementation files plus the approved-plan contract and 4,013 changed lines (3,831 additions, 182 deletions), together with untracked correctness passes 1 through 6
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, undeclared prefixes in typed-leaf descendants still gain Word semantics
`crates/rdocx-oxml/src/numbering.rs:709`
`crates/rdocx-oxml/src/numbering.rs:753`
`crates/rdocx-oxml/src/numbering.rs:795`

The pass-6 repair rejects undeclared prefixes only among attributes on the
typed leaf's root start tag. `write_typed_leaf_raw` then emits every descendant
event unchanged beneath any fresh declaration added to that root. For example,
parse a default-Word `nsid` with no `val` and nested
`<rdocxWord:producer/>`, where `rdocxWord` is undeclared, then set `nsid`.
With no named Word alias in the retained scope, the writer declares
`rdocxWord` as WordprocessingML on `nsid`, which reinterprets the formerly
unbound producer child as a Word element. An undeclared prefixed attribute on a
nested child has the same failure. The new regression at
`crates/rdocx-oxml/src/numbering.rs:3761` covers only an undeclared attribute
on the leaf root, although pass 6 identified retained child names as part of
the same collision class.

### D2, semantic fallback can migrate a removed owner's declaration to a foreign decoy
`crates/rdocx/src/document.rs:662`
`crates/rdocx/src/document.rs:1094`

The adjacent fallback selects the sole structure-matching candidate whose
semantic snapshot equals the original owner. That snapshot deliberately
represents every non-Word, non-XML prefixed name as `#prefix:<prefix>` and
discards its resolved namespace URI. Consider two otherwise identical
paragraphs. The target owns `xmlns:x="urn:target"` and contains
`<x:producer/>`, while the decoy contains
`<x:producer xmlns:x="urn:decoy"/>`. Their raw markers differ, so the existing
ambiguity flags do not identify the decoy, but their structural and semantic
snapshots compare equal because both foreign names become `#prefix:x`. If the
target paragraph is removed, the new fallback uniquely selects the decoy and
replays `xmlns:x="urn:target"` onto it instead of rejecting the missing owner.
This violates the fail-closed owner contract and can transfer retained
namespace state across foreign expanded names. The removed-owner regression at
`crates/rdocx/tests/regression_test.rs:2895` uses byte-identical raw markers,
not a self-contained foreign-shadowed decoy that reaches this fallback.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-6 root-attribute remediation**: An undeclared prefix on a retained
  root attribute now rejects before mutation. Unchanged raw bytes still write.
  Declared aliases, locally foreign-shadowed output prefixes, default Word
  namespaces, and declared fresh-prefix collisions remain safe. The descendant
  gap is D1.
- **Owner fallback**: The new path requires one structural and semantic match,
  and existing ambiguity guards still reject byte-identical duplicate markers.
  Exact Word and XML namespaces remain distinguished. The foreign-namespace
  identity loss is D2.
- **Numbering facts and duplicates**: Duplicate expanded-name Word `val`
  attributes remain untyped and canonicalize to one value on mutation.
  Duplicate `pStyle`, `nsid`, and `tmpl` elements remain raw in schema order.
  Canonical and unmodelled-property classifications retain their reviewed
  behavior.
- **Public API contract**: The approved plan and HLD 10 still distinguish the
  additive native `rdocx` facade from intentional low-level pre-1.0
  `rdocx-oxml` exhaustive-literal breaks. Commit `4c6e1fb` adds no public
  signature or binding change.
- **OOXML and preservation**: Apart from D1 and D2, no fresh namespace-scope,
  schema-order, raw-boundary, drawing-path, document-background, row-property,
  table, or repeated-save regression was found. All earlier microscope fixes
  remain present.
- **Panics and bounds**: No new panic, unchecked arithmetic, excessive revision
  recursion, malformed revision acceptance, or depth-bound regression was
  found.
- **Tests**: Both focused final-commit regressions passed. `cargo test -p
  rdocx-oxml --quiet` passed 333 unit tests and one doctest. `cargo test -p
  rdocx --lib --quiet` passed 326 tests with three ignored. The `rdocx`
  regression binary passed 177 tests with one ignored. Workspace all-target,
  all-feature Clippy with warnings denied, prose, and both diff checks passed.
  The missing sensitivities are described in D1 and D2.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, dynamic dispatch, or unnecessary indirection was found.

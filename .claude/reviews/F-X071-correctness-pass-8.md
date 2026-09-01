# F-X071, correctness, pass 8

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through final source `f6971acfa67cbb095e9b55cf5eb37fcd3de87e78`, 22 implementation and test files plus the approved-plan contract and 4,102 changed lines (3,917 additions, 185 deletions), together with untracked correctness passes 1 through 7
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, URI-aware snapshots reject valid nested namespace-owner replay
`crates/rdocx/src/document.rs:659`
`crates/rdocx/src/document.rs:1053`
`crates/rdocx/tests/regression_test.rs:1932`
`crates/rdocx/tests/regression_test.rs:2874`

The foreign-identity repair now places every resolved namespace URI into both
the structural and semantic owner snapshots. Candidate snapshots are computed
before any retained declarations have been replayed. An outer owner can
therefore see a nested owner's foreign prefix as unresolved in the candidate,
even though the original outer snapshot saw that nested declaration in source.
The snapshots no longer have equal structure, so the legitimate outer owner is
discarded before marker or semantic fallback can identify it. This is not a
hypothetical edge. The existing legacy horizontal-rule round trip now rejects
its paragraph because the nested run's Office declaration is pending, and the
primary insert, remove, and reorder namespace-owner regression rejects its
paragraph with nested hyperlink and run declarations. `cargo test -p rdocx
--test regression_test --quiet` fails both tests with `cannot identify retained
\`p\` nested namespace owner after mutation`.

### D2, a same-URI self-contained decoy can still replace a removed owner
`crates/rdocx/src/document.rs:941`
`crates/rdocx/src/document.rs:1094`

Exact foreign URI comparison blocks the pass-7 decoy only because that decoy
uses a different URI. The precomputed ambiguity flags still require the raw
marker multisets to match. If a decoy has the same expanded name but declares
the same URI directly on its raw child, its marker bytes differ from a target
that inherited the declaration from its modeled owner. The decoy is therefore
not recorded as an alternate. After the target is removed, the new structural
and semantic fallback sees the decoy as the sole exact match and replays the
removed owner's declaration onto it instead of failing closed. For example,
`<p><x:producer xmlns:x="urn:target"/></p>` can replace the removed
`<p xmlns:x="urn:target"><x:producer/></p>`. This transfers retained owner
state solely because equivalent namespace scope used a different lexical
location. The new regression at
`crates/rdocx/tests/regression_test.rs:2925` covers only a different-URI decoy.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-7 subtree remediation**: The numbering validator now walks the full
  retained subtree, validates element and attribute QNames, applies local
  declarations before use, pushes and pops nested namespace scopes, and rejects
  undeclared descendant prefixes before mutation. Root attributes, nested
  elements, nested attributes, shadows, default namespaces, and declared fresh
  prefix collisions retain the reviewed behavior.
- **Foreign URI identity**: Bound foreign namespaces no longer collapse to
  prefix spelling, and the added different-URI removed-owner regression passes.
  The adjacent valid nested-owner and same-URI ambiguity failures are D1 and D2.
- **Numbering preservation**: Unchanged raw leaves replay byte-for-byte.
  Duplicate expanded-name Word `val` attributes remain untyped and canonicalize
  to one value on mutation. Duplicate metadata elements remain raw at their
  schema boundaries, and complete and fragment serializers retain safe output
  prefixes.
- **Public API contract**: The approved plan and HLD 10 still distinguish the
  additive native `rdocx` facade from intentional low-level pre-1.0
  `rdocx-oxml` exhaustive-literal breaks. Commit `f6971ac` adds no public
  signature or binding change.
- **OOXML and preservation**: Apart from D1 and D2, no fresh schema-order,
  raw-boundary, drawing-path, document-background, row-property, table, or
  repeated-save regression was found. All other earlier microscope fixes remain
  present.
- **Panics and bounds**: No new panic, unchecked arithmetic, excessive revision
  recursion, malformed revision acceptance, or depth-bound regression was
  found.
- **Tests**: Both new focused remediation tests pass. `cargo test -p
  rdocx-oxml --quiet` passed 334 unit tests and one doctest. `cargo test -p
  rdocx --lib --quiet` passed 326 tests with three ignored. The `rdocx`
  regression binary passed 176 tests, failed the two existing owner-replay tests
  cited in D1, and ignored one test. Workspace all-target, all-feature Clippy
  with warnings denied, prose, and both diff checks passed. The missing
  same-URI sensitivity is D2.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, dynamic dispatch, or unnecessary indirection was found.

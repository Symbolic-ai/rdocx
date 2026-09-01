# F-X071, correctness, pass 10

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through final source `d183ff1c0ac3d9694f21f95262bed4d4e48bd1a8`, 22 implementation and test files plus the approved-plan contract and 4,343 changed lines (4,151 additions, 192 deletions), together with untracked correctness passes 1 through 9
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-9 D1 remediation**: Candidate owner markers are limited to the exact
  raw-marker multiset captured from owner-dependent uses. Pre-existing
  redundant-local uses no longer inflate cardinality, while different-URI
  shadows remain excluded by the owner declaration and active-scope checks.
  The new regressions cover sibling and nested elements, namespaced attributes,
  nested table cells, repeated saves, and stabilization.
- **Owner identity and ambiguity**: Marker replay remains fail closed for
  same-URI and different-URI decoys. Exact raw bytes, namespace facts, quote
  style, spacing, multiplicity, nested owner order, redundant local bindings,
  and fixed-prefix shadows remain distinguished where they affect owner
  identity.
- **Namespace preservation**: Retained subtrees are checked across complete
  element and attribute namespace-scope stacks. Undeclared prefixes reject
  before serialization. Valid aliases, default namespaces, local shadows, and
  fresh declaration collisions remain safe in direct numbering leaves,
  abstract numbering, and complete numbering output.
- **OOXML preservation and order**: Unchanged raw leaves replay byte-for-byte.
  Duplicate expanded-name values fail closed until typed mutation safely
  canonicalizes them. Raw metadata, drawings, document backgrounds, revisions,
  tables, and row properties retain their schema boundaries, child order, and
  foreign namespace identity.
- **Correctness and public contract**: The approved contract continues to
  distinguish the additive native `rdocx` facade from intentional low-level
  pre-1.0 `rdocx-oxml` exhaustive-literal breaks. The final remediation adds no
  public signature, field, binding, or serialization-contract change.
- **Panics and bounds**: No new panic path, unchecked arithmetic, malformed XML
  acceptance, revision-depth bypass, namespace-scope underflow, or excessive
  recursion was found.
- **Tests and gates**: `cargo test -p rdocx-oxml --quiet` passed 334 unit tests
  and one doctest. `cargo test -p rdocx --lib --quiet` passed 326 tests with
  three ignored. The `rdocx` regression binary passed 180 tests with one
  ignored. Workspace all-target, all-feature Clippy with warnings denied,
  prose, both diff checks, and the 49-entry hash harness passed.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, dynamic dispatch, or unnecessary indirection was found.

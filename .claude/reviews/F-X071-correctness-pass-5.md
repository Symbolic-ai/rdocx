# F-X071, correctness, pass 5

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through final source and contract `8bb2835cc4cf3b6eb04242da5ec8805fcfd3c546`, 20 implementation files plus the approved-plan contract and 3,900 changed lines (3,725 additions, 175 deletions)
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, direct numbering fragment serializers still use a locally shadowed value prefix
`crates/rdocx-oxml/src/numbering.rs:726`
`crates/rdocx-oxml/src/numbering.rs:2486`
`crates/rdocx-oxml/src/numbering.rs:2797`

The pass-4 missing-value repair discovers declarations from the three new raw
sidecars before `CT_Numbering::to_xml` selects its output Word prefix. The
public `CT_Lvl::to_xml` and `CT_AbstractNum::to_xml` fragment serializers bypass
that discovery and still call their internal writers with the fixed prefix
`w`. When a retained `q:pStyle`, `q:nsid`, or `q:tmpl` locally binds `w` to a
foreign namespace and has no Word `val`, a typed mutation makes
`typed_leaf_start` add `w:val` inside that local shadow. The emitted attribute
therefore has the foreign expanded name, and reopening the fragment through a
correctly scoped caller loses the typed value. The new regression at
`crates/rdocx-oxml/src/numbering.rs:3646` exercises only the complete
`CT_Numbering` serializer, so both public fragment paths remain uncovered.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-4 duplicate remediation**: A leaf with duplicate expanded-name Word
  `val` attributes remains untyped and unchanged raw bytes are retained. Typed
  mutation removes the ambiguity and emits one Word value. The shared helper
  covers `pStyle`, `nsid`, and `tmpl`, including empty and nonempty elements.
- **Pass-4 API-contract remediation**: The approved plan now distinguishes the
  additive native `rdocx` facade from the intentional pre-1.0 exhaustive-literal
  breaks in low-level `rdocx-oxml`. This agrees with the numbering preservation
  contract in HLD 10. The doc-hidden sidecars remain public Rust fields, but no
  contract mismatch remains.
- **Raw preservation and namespace scope**: Apart from D1, unchanged raw typed
  leaves retain attributes, nested content, lexical prefixes, missing values,
  and duplicate occurrences. Foreign same-local elements and locally shadowed
  aliases remain opaque. Complete numbering serialization chooses a prefix that
  is safe against declarations in all three new raw sidecars.
- **OOXML and schema order**: No additional child-order, raw-boundary, duplicate
  element, table namespace, row-property, drawing-path, or document-background
  regression was found. Prior pass fixes remain present.
- **Public facts and contracts**: Canonical typed leaves do not report
  unmodelled properties. Missing values, extra attributes, nested content,
  duplicate attributes, and duplicate elements do. Default-style numbering,
  direct overrides, `numId=0`, and producer-defined formats retain their
  reviewed behavior.
- **Panics and bounds**: No new panic, unchecked arithmetic, excessive revision
  recursion, malformed XML acceptance, or depth-bound regression was found.
- **Tests**: Both new pass-4 remediation regressions passed.
  `cargo test -p rdocx-oxml --quiet` passed 332 unit tests and one doctest.
  `cargo test -p rdocx --lib --quiet` passed 326 tests with three ignored.
  The integration binary passed 129 tests with one ignored, while its known
  pinned LibreOffice test failed because `soffice` could not launch in the
  sandbox. Scoped all-target, all-feature Clippy with warnings denied and both
  diff checks passed. The remaining sensitivity gap is described in D1.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, dynamic dispatch, or unnecessary indirection was found.

# F-X071, correctness, pass 6

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through final source `74eace00b851512e73e8c93e3bc4a6c36f6b5591`, 21 implementation files plus the approved-plan contract and 3,943 changed lines (3,768 additions, 175 deletions)
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a fresh Word declaration can collide with an undeclared retained prefix
`crates/rdocx-oxml/src/numbering.rs:709`
`crates/rdocx-oxml/src/numbering.rs:741`
`crates/rdocx-oxml/src/numbering.rs:748`

The direct-fragment repair preserves every non-Word attribute, but its fresh
prefix allocator excludes only prefixes that have namespace declarations. It
does not exclude lexical prefixes already used by retained attributes or child
names. The parser accepts such undeclared names as unmodelled raw XML. For
example, a default-Word `nsid` with no typed value and an undeclared
`rdocxWord:val="producer"` remains raw. After setting the typed `nsid`, the
writer preserves that attribute, declares `rdocxWord` as WordprocessingML, and
adds another `rdocxWord:val`. This both reinterprets the retained producer name
and emits duplicate qualified attributes instead of failing closed. An
undeclared `rdocxWord:*` child is likewise reinterpreted by the new declaration.
The regression at `crates/rdocx-oxml/src/numbering.rs:3665` covers a declared
foreign `w` shadow with a safe declared `q` alias. It does not cover a default
namespace with no named Word alias or a collision between the fresh prefix and
an undeclared retained name.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-5 remediation**: Direct `CT_Lvl::to_xml` and
  `CT_AbstractNum::to_xml` no longer use a locally foreign-shadowed `w` for a
  newly added value when a safe named Word alias is in scope. With only a
  default Word namespace, the writer creates a named local Word declaration.
  Declared foreign `rdocxWord` collisions advance to an unused suffix. The
  remaining undeclared-name collision is D1.
- **Unchanged and complete paths**: Unchanged typed-leaf raw bytes are replayed
  without rewriting. The complete `CT_Numbering::to_xml` path still discovers
  declarations from `nsid_raw`, `tmpl_raw`, and `p_style_raw`, chooses safe root
  prefixes, reopens with the mutated facts, and retains the raw sidecars on a
  repeated save.
- **Duplicates and typed facts**: Duplicate expanded-name Word `val`
  attributes remain untyped and canonicalize to one value on mutation. Duplicate
  `pStyle`, `nsid`, and `tmpl` elements stay raw at their observed boundaries.
  Canonical leaves do not report unmodelled properties, while missing values,
  extra attributes, nested content, and duplicates do.
- **Public API contract**: The additive native `rdocx` facade and intentional
  low-level pre-1.0 `rdocx-oxml` exhaustive-literal breaks remain accurately
  distinguished in the approved plan and HLD 10. No new public signature or
  binding change was introduced by the fragment repair.
- **OOXML and preservation**: Apart from D1, no namespace-scope, schema-order,
  raw-boundary, drawing-path, document-background, row-property, table, or
  repeated-serialization regression was found. All earlier microscope fixes
  remain present.
- **Panics and bounds**: No new panic, unchecked arithmetic, excessive revision
  recursion, malformed revision acceptance, or depth-bound regression was
  found.
- **Tests**: The direct-fragment and complete-numbering shadow regression plus
  the duplicate expanded-value regression passed. `cargo test -p rdocx-oxml
  --quiet` passed 332 unit tests and one doctest. `cargo test -p rdocx --lib
  --quiet` passed 326 tests with three ignored. Workspace all-target, all-feature
  Clippy with warnings denied and both diff checks passed. The remaining test
  sensitivity gap is described in D1.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, dynamic dispatch, or unnecessary indirection was found.

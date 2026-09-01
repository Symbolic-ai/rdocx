# F-X071, correctness, pass 4

**Reviewed**: claim-base `f5f43008b9b2d921d84f40cfd70db9ef86f385c9` through final implementation `c72d0501a5d6b9cd12275a789b2dc597301abfcf`, 20 implementation files and 3,797 changed lines (3,625 additions, 172 deletions)
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, adding a missing value can use a locally foreign-shadowed prefix
`crates/rdocx-oxml/src/numbering.rs:704`
`crates/rdocx-oxml/src/numbering.rs:2968`

When a retained typed leaf had no Word `val`, `typed_leaf_start` adds the new
attribute with the generated document-level Word prefix. Prefix-conflict
discovery scans abstract and level extras plus `ppr_raw` and `rpr_raw`, but it
does not scan the new `nsid_raw`, `tmpl_raw`, or `p_style_raw` subtrees. For
example, parse
`<q:nsid xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w="urn:foreign"/>`,
then set `nsid` to a value. The generated output prefix remains `w`, and the
rewriter adds `w:val` inside the leaf where `w` is locally bound to
`urn:foreign`. Reopening the output reports no `nsid` value. The same failure
applies to `tmpl` and `pStyle`. The preservation test at
`crates/rdocx-oxml/src/numbering.rs:3510` starts with existing `w:val`
attributes and has no local Word-prefix shadow, so it does not exercise this
missing-value mutation path.

### D2, aliased duplicate value attributes still publish an arbitrary typed fact
`crates/rdocx-oxml/src/numbering.rs:647`
`crates/rdocx-oxml/src/numbering.rs:692`

The `c72d050` refinement counts expanded-name Word `val` attributes and retains
the raw leaf when the count is not one, but typed parsing still calls
`word_attribute_value`, which returns the first matching attribute. A leaf with
`w:val="first"` and `q:val="second"`, where both prefixes bind the
WordprocessingML namespace, therefore exposes whichever value occurs first.
On typed mutation, `typed_leaf_start` rewrites both aliases to the new value and
still emits duplicate expanded-name attributes. Namespace-ambiguous input must
reject or remain untyped rather than publish an attribute-order-dependent fact.
The duplicate regression at `crates/rdocx-oxml/src/numbering.rs:3578` covers
duplicate elements only, not duplicate expanded-name attributes on one leaf.

### D3, the numbering additions are not additive public API
`crates/rdocx-oxml/src/numbering.rs:2244`
`crates/rdocx-oxml/src/numbering.rs:2247`
`crates/rdocx-oxml/src/numbering.rs:2626`
`crates/rdocx-oxml/src/numbering.rs:2629`
`crates/rdocx-oxml/src/numbering.rs:2633`
`crates/rdocx-oxml/src/numbering.rs:2636`

`CT_Lvl` and `CT_AbstractNum` are exhaustive public structs in the published
`rdocx-oxml` crate. The full story adds the public fact fields and pass 3 adds
three more public raw-sidecar fields. Every downstream struct literal must now
supply them, so existing callers fail to compile. `#[doc(hidden)]` affects
generated documentation only and does not remove a field from Rust's public
API. The workspace had to update its own literal at
`crates/rdocx/src/epub.rs:1477`. This contradicts the approved design's
additive pre-1.0 API classification and introduces preservation internals that
the story did not request as public surface.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-3 remediation**: Existing-value mutations retain producer attributes
  and nested child XML. Unchanged raw leaves write their original bytes.
  Missing values remain raw while unchanged. Duplicate `pStyle`, `nsid`, and
  `tmpl` elements remain raw at their observed schema boundaries. Foreign
  same-local elements and bound prefix shadows do not acquire typed element
  identity. The remaining alias and shadow failures are D1 and D2.
- **Reader facts**: `has_unmodeled_properties` remains false for a canonical
  empty leaf with exactly one modeled `w:val`, and becomes true for retained
  leaf attributes, missing values, nested content, or duplicate elements.
- **Prior remediation**: Foreign document backgrounds remain untyped and
  preserved. Picture relationships require the direct schema path and fail
  closed on unknown prefixes and duplicate payloads. Raw row-property extras
  retain their boundaries and receive RTF loss diagnostics. Extreme numbering
  levels do not overflow.
- **Panics and bounds**: No new panic, unchecked arithmetic, unbounded
  recursion, malformed revision acceptance, or depth-limit regression was
  found.
- **OOXML and preservation**: Apart from D1 and D2, no issue was found in
  numbering child order, raw duplicate placement, owner namespace scope,
  retained table XML, drawing payload identity, or repeated save and reopen
  behavior.
- **Tests**: `cargo test -p rdocx-oxml --quiet` passed 330 unit tests and one
  doctest. `cargo test -p rdocx --lib --quiet` passed 326 tests with three
  ignored. All three focused pass-3 remediation tests passed. `git diff
  --check` passed. The missing sensitivity is described in D1 and D2.
- **Structure**: No new crate, module, feature flag, trait, generic parameter,
  forwarding wrapper, or dynamic dispatch violation was found. The public API
  exception is D3.

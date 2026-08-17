# F-150, all, pass 1

**Reviewed**: full working diff against `e25ef35`, 2 files, 938 additions and 2 deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, revision resolution reaches unmodelled raw placements
`crates/rdocx/src/revision.rs:546`

`push_element` attaches revision metadata to every namespace-correct `w:ins`,
`w:del`, move, or property-change element anywhere in the serialized document.
It does not verify that F-149 projected the element at one of the modeled
placements. A revision-shaped element inside an opaque raw subtree is therefore
counted and transformed by `accept_all` or `reject_all` even though
`Document::revisions` does not report it. This changes unmodelled XML that the
contract requires to remain verbatim.

### D2, unwrapping can discard namespace declarations needed by retained content
`crates/rdocx/src/revision.rs:299`

Keeping a selected wrapper renders only its inner bytes, which drops every
namespace declaration carried by the wrapper. If a retained child or opaque
descendant uses a prefix declared only on that revision element, the staged XML
contains an unbound prefix. The later quick-xml parse and serialization at
lines 149 to 150 check syntax but do not restore declarations for opaque raw
children, so the saved document can contain namespace-invalid XML.

### D3, contextual marker resolution is order-dependent
`crates/rdocx/src/revision.rs:439`

`selected_owner_marker` returns only the first selected insertion or deletion
marker. A row property set containing an insertion followed by a deletion is a
modeled `Vec<CT_Revision>` placement. Accepting all revisions keeps the row
because the insertion is found first, then removes both marker elements while
rendering. The selected deletion never applies its remove-owner semantics. The
same ambiguity affects other owner-marker placements with more than one
selected marker.

### D4, the date selector accepts malformed timestamps and can overflow
`crates/rdocx/src/revision.rs:702`

The hand-written parser does not enforce RFC 3339 field widths or nonnegative
hour, minute, and second values. For example,
`2026-08-17T-1:00:00Z` is accepted as a bound instead of returning an error.
It also accepts an `i64` year and performs unchecked calendar multiplication at
lines 779 to 785, so a very large parseable year can panic in checked builds or
wrap in optimized builds. Both outcomes violate the public contract that a
malformed bound returns an error without mutation.

### D5, the required atomicity and nesting gate is not exercised
`crates/rdocx/tests/regression_test.rs:221`

The malformed-property test compares serialized bytes and the remaining
revision count, but it does not populate or verify either layout cache as the
approved integration test requires. The added matrix also has no nested
revision case, so it cannot detect a selected outer wrapper hiding or
miscounting a separately selected descendant. These are explicit test-plan and
risk-routing obligations, and the regression gate is incomplete without them.

## Smells

No smells found.

## Nitpicks

No nitpicks found.

## Not found

No additional findings in public API shape, mutation commit ordering, deleted
text conversion, property-change replacement, package ownership, schema child
order, or structural-rule compliance.

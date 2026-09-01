# F-226, correctness, pass 2

**Reviewed**: remediated working diff, 5 tracked paths plus the design plan, 1,558 diff lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, notes placeholders bypass PresentationML fallback matching

`crates/rpptx/src/lib.rs:6810`

Overlay lookup uses exact `HashMap` key equality. PresentationML placeholder
matching is index-first with type fallback when either side omits an index, as
the existing typed contract implements at
`crates/rpptx-oxml/src/placeholder.rs:275`. A valid notes body that omits its
index while the notes master carries `idx="3"` therefore fails as unmatched.
Equivalent title and body placeholder types have the same problem. Duplicate
and ambiguity detection at `crates/rpptx/src/lib.rs:6781` also sees only exact
keys, so two distinct keys that match through the fallback rule are not
rejected before overlay. The compositor must use `PlaceholderKey::matches` and
require exactly one match on both sides.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D1 now rejects unmatched placeholders rather than dropping them. Pass-1
D2 now keeps handout master graphics behind thumbnails and has a raster
regression. Pass-1 D3 now has exact layout, clipping, three-up rules, media,
malformed graph, edge geometry, and 1.01-point sensitivity coverage. No new
API, dependency, allocation, relationship target, namespace, schema-order,
source mutation, deterministic-font, panic, or hash-harness issue was found.
The focused suite, `rpptx` suite, Clippy, prose, diff hygiene, and 49 of 49 hash
entries passed on the reviewed source.

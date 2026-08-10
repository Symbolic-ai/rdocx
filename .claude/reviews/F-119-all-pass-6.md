# F-119, all, pass 6

**Reviewed**: the remediated F-119 working diff from `87b5d92`, 3 tracked
files, 2,079 insertions and 68 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, setting a typed optional field can duplicate its preserved wrapper

`crates/rpptx-chart/src/lib.rs:604`

An unsupported `c:tx`, `c:cat`, or `c:bubbleSize` wrapper is intentionally
stored as a raw child while the corresponding public typed field remains
`None`. If a caller then sets that public field, the writer emits the preserved
wrapper and the new typed wrapper at `crates/rpptx-chart/src/lib.rs:495`. For
example, parsing a valid `c:cat` containing `c:multiLvlStrRef` and then assigning
`Series::categories` writes two `c:cat` children. The same duplicate occurs for
`name` and `bubble_size`. The result violates the schema occurrence limit even
though both the parse and the public field edit succeed. Retain the opaque
wrapper occurrence in model state and reject or replace it when the typed field
is set.

### D2, resizing a parsed cache can drop or misorder its preserved tail

`crates/rpptx-chart/src/lib.rs:1016`

Cache raw children are anchored to the original point count, but the writer
emits raw boundaries only while iterating the current public value vector at
`crates/rpptx-chart/src/lib.rs:1396`. If a parsed two-point cache has a preserved
schema-final `c:extLst` and the caller shortens `values` to one item, the writer
never emits the old trailing boundary and silently drops the extension. If the
caller grows the vector to three items, the old tail is emitted after point two
and before the new point three, putting `c:extLst` before a `c:pt`. Reconcile
raw point boundaries against the edited vector so preserved payloads remain
present and schema-final.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 5 conflicting `xmlns:c` defect is remediated for typed wrapper,
reference, cache, point, scalar, and text roots. Mixed ChartML aliases still
resolve by namespace URI, inherited foreign plot aliases remain opaque, fixed
prefixes and modelled child order are retained, sparse cache validation is
consistent with the completed plan, and the earlier shape-property namespace
findings remain fixed. No additional panic, numeric-validation, test-gate,
public-surface, or structural findings were found.

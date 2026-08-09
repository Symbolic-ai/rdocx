# F-118, all, pass 2

**Reviewed**: full remediated working-tree diff against claim base `38aec895c0060ee3da0823bd2d70b6d900b76227`, 8 files, 1,751 changed lines. This includes 85 tracked changed lines and the 1,666 untracked lines in `crates/rpptx-chart/Cargo.toml` and `crates/rpptx-chart/src/lib.rs`. The worker-local `corpus` symlink is excluded.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, nested foreign DrawingML names are still converted to typed content

`crates/rpptx-chart/src/lib.rs:1028`

`reject_foreign_drawing_children` validates only each immediate child of
`c:spPr` or `c:txPr`, then captures the complete child subtree without
examining its descendants. The delegated DrawingML parsers continue matching
nested names by local name. For example, an `a:p` containing a foreign
`x:r`, or an `a:solidFill` containing a foreign `x:srgbClr`, passes this guard
and is rewritten with fixed `a:` names. This still violates namespace-aware
reading and unmodeled XML preservation. Validate every descendant that the
concrete parser models, or pass resolved namespace bindings through the
DrawingML readers. Add nested foreign-name cases for both text and shape
properties.

### D2, the corpus preservation comparison discards schema boundaries and order

`crates/rpptx-chart/src/lib.rs:1456`

Both lists of preserved regions are sorted before comparison. The assertion
therefore proves only that the original and written parts contain the same
multiset of byte strings. It still passes if an opaque child moves to a
different schema boundary or two preserved children exchange order. The later
model equality cannot independently detect a consistently wrong raw-slot map,
because both parses apply the same map. Retain the parent, schema boundary,
and sibling order with each captured region, then compare those ordered
records without sorting away their positions.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panic safety, scalar defaults
and validation, duplicate rejection, root ChartML namespace handling, fixed
writer prefixes, schema ordering for modeled children, direct comment and
processing-instruction preservation, scalar extension preservation,
dependency direction, package metadata, HLD scope, or structure. Pass 1's
raw-slot edit defect is covered for chart and chart-space mutations. The XML
parser macro has been replaced with ordinary shared helpers and concrete type
implementations.

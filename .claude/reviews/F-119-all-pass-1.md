# F-119, all, pass 1

**Reviewed**: the F-119 working diff from `87b5d92`, 3 files, 1,832
insertions and 59 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, series shape properties can convert foreign elements into DrawingML

`crates/rpptx-chart/src/lib.rs:548`

The series parser sends `c:spPr` directly through `CT_ShapeProperties` without
the foreign-element comparison already used by the chart-space parser. A
producer namespace declared on `c:ser` can therefore contain a foreign child
whose local name matches a modelled DrawingML child. The shape parser accepts
that local name, and serialization writes it under `a:`, changing the preserved
producer payload. Carry the series namespace bindings into this branch and
reject a parse whose typed write changes foreign element regions.

### D2, a conflicting `a` binding on `c:ser` is silently rebound

`crates/rpptx-chart/src/lib.rs:378`

Series parsing validates the ChartML prefix but drops any `xmlns:a`
declaration while serialization always writes the DrawingML binding at
`crates/rpptx-chart/src/lib.rs:430`. An input that binds `a` to a producer
namespace is accepted, then its preserved `a:` children acquire DrawingML
meaning on write. Reject a conflicting `a` declaration at the series root and
cover the failure without panicking.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML ordering, cache consistency,
preservation, test-gate, or structural findings were found.

# F-X040, all, pass 1

**Reviewed**: working-tree diff against `bbd97cb371b07bf51141a1481480c536f24d8f9f`, 4 files, 1,290 insertions and 33 deletions
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, the restart fallback gate does not prove all documented unsafe states
`crates/rdocx-layout/src/engine.rs:5152`

The fallback matrix ends after tables, split paragraphs, floats, notes,
multiple sections, and backgrounds. It does not construct field, header,
footer, or mismatched-boundary cases even though the current testing contract
requires those states to use the full paginator. A regression that accidentally
admits one of those states could pass this gate.

### D2, table border buffers are not measured by retained capacity
`crates/rdocx-layout/src/engine.rs:1572`

Table and cell borders are measured using the length of a temporary debug
string. This neither measures the retained `String` capacities in border
colours nor the owned border representation itself. The table cache can
therefore retain more memory than the stated byte ceiling while its counter
still reports an in-bounds value.

## Smells

### S1, the exact table cache key depends on debug formatting
`crates/rdocx-layout/src/engine.rs:1250`

`CT_Tbl` already implements structural equality, but the cache key converts it
to a debug string. This makes exact cache identity depend on an incidental
formatting implementation instead of the typed table projection promised by
the design and HLD.

## Nitpicks

None.

## Not found

No additional correctness, contract, panic, OOXML preservation, test, or
structure findings. The diff adds no module, file, trait, generic parameter,
wrapper, dependency, or public API.

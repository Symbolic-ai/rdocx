# F-119, all, pass 2

**Reviewed**: the remediated F-119 working diff from `87b5d92`, 3
implementation files, 1,849 insertions and 59 deletions
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, cache parsing suppresses the structural warning its state shape earns

`crates/rpptx-chart/src/lib.rs:961`

`parse_cache_child` takes nine arguments and suppresses
`clippy::too_many_arguments`, while five of those arguments are mutable fields
of one cache parse operation. The adjacent series parser already uses a state
value for the same reason. Give the cache loop one concrete parse-state value
so the child transition has one owner and the lint suppression disappears.

## Nitpicks

None.

## Not found

The pass 1 namespace defects are fixed. No further correctness, contract,
panic, OOXML ordering, preservation, cache consistency, test-gate, or public
surface findings were found.

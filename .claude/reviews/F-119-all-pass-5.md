# F-119, all, pass 5

**Reviewed**: the remediated F-119 working diff from `87b5d92`, 3
implementation files, 2,081 insertions and 68 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, nested typed nodes can rebind the fixed ChartML writer prefix

`crates/rpptx-chart/src/lib.rs:1251`

The wrapper parser retains every root attribute, including a local
`xmlns:c` declaration, while `write_wrapper_start` at
`crates/rpptx-chart/src/lib.rs:1494` always changes the wrapper name to
`c:*`. A valid aliased input such as a `q:val` that locally binds `c` to a
producer namespace is therefore written as `<c:val xmlns:c="urn:producer">`.
The namespace declaration applies to the element itself, so the written value
wrapper is no longer ChartML and reparsing reports the required `c:val` as
missing. The same local-rebinding path is present in cache, point, scalar, and
text markup captured for fixed-prefix output. Reject conflicting `c`
declarations at every typed rewrite boundary, or remove them only after
proving no preserved qualified content depends on the binding, and add a
nested-rebinding regression.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 4 standalone namespace propagation, fixed series-root `c`, `a`, and
`r` checks, optional-wrapper occurrence tracking, and sparse-cache contract
correction are present. No additional cache consistency, numeric validation,
panic, schema-order, test-gate, public-surface, or structural findings were
found.

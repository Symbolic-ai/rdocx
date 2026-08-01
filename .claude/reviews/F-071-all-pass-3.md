# F-071, all, pass 3

**Reviewed**: Current F-071 implementation and contract diff, 7 files, 1,007 insertions and 20 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the namespace-order regression can pass when the fix is reverted
`crates/rpptx-oxml/src/namespace.rs:166`

The regression constructs one randomized `HashMap` and expects its returned
prefixes to be sorted. With the production sort removed, that map can still
iterate as `a`, `m`, `z` by chance, so the test can pass against the defective
implementation. It therefore does not provide the workflow's required
deterministic revert proof. Exercise sorting through a deliberately unsorted
deterministic input or otherwise make the reverted implementation fail on
every run.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 2 production defect is resolved. `NamespaceBindings::entries` sorts
all prefixed bindings before they become stored model attributes, and the
default binding has a stable final position. The two namespace-preservation
defects from pass 1 also remain resolved.

No additional correctness, matching-contract, panic, OOXML namespace or
schema-order, preservation, test-gate, or structural findings were found. The
feature still preserves raw subtrees in their slots, enforces the required
shape shell, and covers every approved placeholder matching rule. No production
input indexing, new trait, generic parameter, feature flag, crate, or
forwarding wrapper was introduced.

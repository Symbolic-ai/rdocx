# F-071, all, pass 2

**Reviewed**: Current F-071 implementation and contract diff, 7 files, 981 insertions and 20 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, inherited namespace declarations have nondeterministic order
`crates/rpptx-oxml/src/namespace.rs:66`
`crates/rpptx-oxml/src/namespace.rs:124`

`NamespaceBindings::entries` collects prefixes directly from a `HashMap`
without sorting them. The new self-contained attribute helper persists those
entries in that order, and the placeholder and shape writers later emit the
stored order unchanged. Parsing the same alternate-prefix tree in independent
maps can therefore produce different namespace-attribute order, different
derived model equality, and different serialised bytes. This violates the
repository's deterministic serialisation requirement. The inherited entries
need a stable order before they become model state.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 1 D1 is resolved. A standalone placeholder now declares every fixed
model namespace that its opaque children may use, and the preservation test
covers an opaque `a:` child. Pass 1 D2 is also resolved. Shapes and placeholders
retain non-fixed inherited bindings, and the nested alternate-prefix test
serialises and reparses both extracted models independently.

No additional matching-correctness, contract-scope, panic, OOXML schema-order,
test-gate, or structural findings were found. The gate still covers index
priority, type fallback, the absent-type default, and both equivalence classes.
No production input indexing, new trait, generic parameter, feature flag,
crate, or forwarding wrapper was introduced.

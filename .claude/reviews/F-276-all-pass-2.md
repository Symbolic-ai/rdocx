# F-276, all aspects, pass 2

**Reviewed**: Frozen working diff against `8f4631d264333ba31bddec46f53ade1d389b0c3a`, 15 files, 2967 insertions and 191 deletions. Current approved contract, complete implementation diff, previous review and added regression assertions were inspected.
**Verdict**: 2 defects, 0 smells, 0 nitpicks. D1 through D4 are resolved. D5 and D6 remain.

## Defects

### D5, fresh custom XML IDs can collide with another incoming store

`crates/rdocx/src/field.rs:2230`

The replacement store-ID allocator searches raw destination package bytes,
the selected fragment XML and previously allocated replacements. It does not
exclude the semantic IDs in the complete incoming `binding_stores` closure.
A valid source can have store A in its selected body and a different store B
referenced only by a required note or comment. Let B already have
`{F2760000-0000-4000-8000-000000000001}`, and let only A collide with the
destination. B is absent from the selected XML and the destination. The first
replacement for A can therefore allocate B's existing ID. B needs no collision
replacement and retains that same ID. The copied item properties now identify
two stores by the same semantic GUID, making binding lookup ambiguous.
An unbraced B binding in selected XML can also evade the raw braced-candidate
search after D3 introduced semantic normalization. Reserve normalized source
and destination store identities before choosing replacements, including
stores discovered in companions. Test that an incoming companion store using
the allocator's first candidate remains distinct from another remapped store.

### D6, legacy rich merge accepts external edges from comment and note companions

`crates/rdocx/src/field.rs:2038`

The comment companion external-edge branch queues an external relationship
without checking `allow_external_relationships`. The note companion branch at
`crates/rdocx/src/field.rs:2169` does the same. Legacy rich fragment merge calls
this importer with the flag false at `crates/rdocx/src/field.rs:1847`. Its later
closure check covers only internal target parts, so these direct companion
edges bypass the restriction. A rich merge fragment whose selected paragraph
has a comment containing an external hyperlink now imports that external edge,
where the previous implementation rejected it. Notes have the same bypass.
This contradicts the retained rich merge contract in
`docs/hld/04-opc-and-packaging.md:1558`. Enforce the flag for all directly queued
companion external edges. Keep explicit `DocumentFragment` imports permissive.
Test both call paths with note and comment hyperlinks, including atomic
failure for the legacy merge path.

## Resolved findings

- D1: `crates/rdocx/src/field.rs:2567` applies the complete numbering map in one pass against original attributes. Body, note and comment paths use the same helper. The public regression at `crates/rdocx/tests/regression_test.rs:46632` checks exact list associations and package determinism across 32 imports.
- D2: `crates/rdocx/src/field.rs:2613` requires the destination candidate to be relationship-free. The public regression at `crates/rdocx/tests/regression_test.rs:46730` preserves the original outgoing graph and checks that the new imported image is a separate leaf.
- D3: `crates/rdocx/src/field.rs:3265` normalizes discovered store IDs, and the replacement path uses the same semantic normalization. Lexical aliases now share one allocation and are rewritten together. D5 concerns a different missing reservation across distinct stores.
- D4: `crates/rdocx/src/field.rs:2841` patches complete numbering XML with its namespace scope. The public regression at `crates/rdocx/tests/regression_test.rs:46799` asserts both linked-style associations with canonical and producer prefixes.

## Smells

Zero smells found.

## Nitpicks

Zero nitpicks found.

## Not found

No additional correctness, contract or test findings beyond D5 and D6.
No additional panic, OOXML or structural findings were established. The repairs
add no new public API, traits, generic parameters, modules or crates.

The README and archive constants consistently record the dated measurement.
The current archive has 36 members. Normalizing its VCS member with the
documented recorder rule gives 7,573,679 member bytes, matching the recorded
inventory. Its compressed size during inspection was 1,299,015 bytes, versus
the recorded observation of 1,299,013 bytes. Concurrent publication dry-run
regeneration and VCS metadata affect the compressed artifact, so this pass
does not treat that two-byte difference as a defect or claim the pending
publication gate passed.

Worker test evidence remains supporting evidence. This independent pass used
code, contract, assertions and read-only archive inspection. No source or test
files were changed.

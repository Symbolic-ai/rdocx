# F-162, all, pass 3

**Reviewed**: complete working tree against `HEAD` (`6a60586`), 7 implementation files, 1,533 additions and 58 deletions, excluding review artifacts
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, nested source patching can still update an opaque lookalike

`crates/rdocx-oxml/src/text.rs:2421`

The package-level repair now verifies paragraph and typed ancestry, but the
recursive patch inside an owning complex field still selects the first equal
byte sequence after the previous match. A producer extension inside an earlier
outer-field run can contain the exact run sequence used by the later typed
nested field. `complex_field_events` skips that opaque subtree, so only the
later occurrence is evaluated, while this search patches the opaque occurrence
and advances past it. The real nested cache remains stale and the producer XML
is changed. Reparse validation succeeds because both fragments remain valid
XML. Nested edits need the same typed source-location identity used at the
package boundary, not raw first-match identity. The identity regression at
`crates/rdocx-oxml/src/text.rs:5054` covers new and foreign replacement values,
but not an identical opaque fragment before the original parsed child.

### D2, comments between complex-field runs break source identity

`crates/rdocx-oxml/src/text.rs:2158`

The pretty-print repair retains whitespace text between paragraph runs, but
the following catch-all still drops XML comments and processing instructions.
`complex_field_source` reconstructs the match key only from run sources and
retained boundary XML at `crates/rdocx-oxml/src/text.rs:1403`. A valid complex
field with `<!-- producer -->` between its begin and instruction runs therefore
has no contiguous source match in a header, footer, or endnote part.
`update_fields` returns a source-not-found error instead of materializing the
cache. In the main story, serialization removes the comment. The regression at
`crates/rdocx-oxml/src/text.rs:5107` proves whitespace retention only. Boundary
capture must retain all source events that may legally occur between the field
runs so the update both succeeds and preserves unmodelled XML.

## Smells

None.

## Nitpicks

None.

## Pass-2 repair verification

- D1 is closed for pretty-print whitespace. Complex source capture now includes
  inter-run whitespace without exposing it as lasting paragraph raw content,
  and the package regression updates the field in place.
- D2 is closed for the reported package lookalikes and repeated typed fields.
  Paragraph spans use expanded Word names, candidates require approved typed
  ancestry, and identical typed occurrences are consumed in source order.
- D3 is closed for public and foreign same-instruction nested replacements.
  Parsed fields carry clone-stable private source identities. A new or
  differently parsed replacement misses the source-preserving branch and is
  serialized canonically, while edits to the original parsed child retain its
  producer source.

## Checks

- `cargo test -p rdocx-oxml text::tests`, passed, 64 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 14 tests.
- `cargo test -p rdocx --test regression_test`, passed, 75 tests.
- `cargo clippy -p rdocx -p rdocx-oxml --all-targets --no-deps -- -D warnings`,
  passed.
- `cargo fmt --all -- --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/sync_agent_skills.py --check`, passed.
- Progress evidence for full crate tests, package dry-run, archive size, and the
  remaining repository gates was inspected.

## Not found

No additional defect was found in F-161 traversal order and field identity,
simple-field mutation, cache and dirty policy, atomic live-state commit, layout
invalidation, update-aware save delegation, leave-alone save APIs, settings and
property preservation, namespace alias handling, schema order, public binding
scope, panic safety, HLD scope, or structure. No smells or nitpicks were found.

# F-162, all, pass 4

**Reviewed**: complete working tree against `HEAD` (`6a60586`), 7 implementation files, 1,756 additions and 58 deletions, excluding review artifacts
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, hyperlink-local boundary events still break complex source identity

`crates/rdocx-oxml/src/text.rs:3502`

The pass-3 boundary repair handles comments and processing instructions read by
the paragraph parser, but complex fields inside an explicit `w:hyperlink` use
`parse_hyperlink_children` instead. Its catch-all still drops whitespace text,
comments, and processing instructions between hyperlink child runs. A typed
complex field inside a hyperlink therefore receives a reconstructed source
without those events. Main-story mutation removes them. A header, footer, or
endnote update cannot find that incomplete byte sequence in the original part
and returns a source-not-found error. This is within the approved typed scope,
as explicit hyperlink runs are projected and traversed. The new regression at
`crates/rdocx/tests/regression_test.rs:800` covers direct paragraph runs only.
Every parser that supplies runs to `complex_field_source` must retain the same
boundary raw events.

### D2, sibling nested fields in one physical run have overlapping source spans

`crates/rdocx-oxml/src/text.rs:2444`

The new reverse-edit logic assumes every immediate nested field begins at a
different top-level run and owns a non-overlapping source range. That is not
true for fields whose begin, instruction, separator, result, and end tokens
share one `w:r`, a supported form demonstrated at
`crates/rdocx-oxml/src/text.rs:4873`. `complex_field_source` records the whole
run when a field starts and ends at that run at
`crates/rdocx-oxml/src/text.rs:1403`. Two nested siblings in the same run thus
receive the same whole-run source and start offset. After the first sibling is
matched, `search_start` advances to the run end, so the second sibling cannot
match and serialization returns `nested field source in owning complex field`.
`update_fields` then fails atomically instead of updating the typed siblings.
The pass-3 regression at `crates/rdocx-oxml/src/text.rs:5162` places each
sibling across five distinct runs and does not exercise overlapping run-local
field spans.

## Smells

None.

## Nitpicks

None.

## Pass-3 repair verification

- D1 is closed for the reported opaque descendant and distinct-run sibling
  cases. Candidate starts are limited to top-level Word runs using expanded
  namespace bindings, typed siblings are consumed in source order, and edits
  are applied in reverse without shifting later offsets.
- D2 is closed for direct paragraph fields in both the main story and an
  aliased package-backed header. Comments and processing instructions are
  retained in the field source and survive cache mutation. D1 above identifies
  a separate hyperlink-child parsing path that did not receive the repair.

## Checks

- `cargo test -p rdocx-oxml text::tests`, passed, 66 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 14 tests.
- `cargo test -p rdocx --test regression_test`, passed, 76 tests.
- `cargo clippy -p rdocx -p rdocx-oxml --all-targets --no-deps -- -D warnings`,
  passed.
- `cargo fmt --all -- --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/sync_agent_skills.py --check`, passed.
- Progress evidence for full crate tests, package dry-run, archive size, and the
  remaining repository gates was inspected.

## Not found

No additional defect was found in package paragraph span mapping, opaque
package lookalikes, source identity for new or foreign nested replacements,
F-161 traversal order, simple-field mutation, cache and dirty policy, atomic
live-state commit, layout invalidation, update-aware save delegation,
leave-alone save APIs, settings and property preservation, schema order, public
binding scope, panic safety, HLD scope, or structure. No smells or nitpicks were
found.

# F-162, all, pass 1

**Reviewed**: complete working tree against `HEAD` (`6a60586`), 7 files, 697 additions and 19 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, nested updates move text operands ahead of nested operands

`crates/rdocx-oxml/src/text.rs:3067`

The preservation branch emits every text positional argument before it walks
`nested_order`. That does not preserve the argument vector's order. For a
parsed field such as `IF { REF target } = "x" "yes" "no"`, the argument vector
starts with the nested field, followed by the four text operands. Any explicit
update changes the nested field's cache or dirty state and reaches this writer.
The saved instruction becomes `IF = x yes no { REF target }`, so reopening
changes the IF operand positions and its result. The regression at
`crates/rdocx/tests/regression_test.rs:303` covers a switch-nested operand and a
positional nested operand with no surrounding text arguments, so it cannot
detect this semantic reorder.

### D2, updating a nested field discards the outer field's producer XML

`crates/rdocx-oxml/src/text.rs:2354`

The source-preserving rewrite is selected with full `FieldInstruction`
equality. Nested `Field` equality includes cached result and dirty state, so an
F-162 update to any nested field makes this test false even though the
instruction structure did not change. The fallback rebuilds the complete outer
complex field, and `write_nested_instruction_field` at
`crates/rdocx-oxml/src/text.rs:3185` also rebuilds each child instead of using
its source-aware writer. Producer run formatting, instruction partitioning,
attributes, and unmodelled children anywhere in the outer or nested field are
therefore lost during a cache and dirty-only update. This violates the approved
requirement that those updates rewrite only typed values while preserving
unmodelled simple and complex field content. The new round-trip test at
`crates/rdocx-oxml/src/text.rs:4910` exercises only non-nested fields and leaves
this path uncovered.

### D3, package-backed story updates rewrite or drop unrelated XML

`crates/rdocx/src/field.rs:136`

Every referenced header or footer containing a field is serialized as a whole
`CT_HdrFtr`. That model writes all typed paragraphs before all captured unknown
children at `crates/rdocx-oxml/src/header_footer.rs:153`, so an interleaved
table, text box, or producer extension moves even though it is outside the
typed F-161 boundary. The endnote path has the stronger loss at
`crates/rdocx/src/field.rs:161`. `CT_Footnotes::from_xml` skips unknown root and
note content at `crates/rdocx-oxml/src/footnotes.rs:154`, and the staged full
serialization permanently omits it. Updating one typed field must not reorder
or discard unrelated package XML. The story regression at
`crates/rdocx/tests/regression_test.rs:569` checks only that new field text is
present and that an orphan header is unchanged. It does not put unmodelled
content in a changed referenced part or compare its preserved boundaries.

## Smells

None.

## Nitpicks

None.

## Checks

- `cargo test -p rdocx-oxml text::tests`, passed, 61 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 14 tests.
- `cargo test -p rdocx --test regression_test`, passed, 71 tests.
- `cargo fmt --all -- --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/prose_check.py`, passed with zero violations before writing
  this review.
- `git diff --check HEAD`, passed before writing this review.
- Progress evidence for the scoped clippy, check, full package dry-run, and
  archive-size assertion was inspected.

## Not found

No additional defect was found in top-level F-161 story ordering, physical
header and footer deduplication, normal-note filtering, run-level or block-level
content-control traversal, raw-only instruction exclusion, outcome policy,
unsupported cache retention, XML character validation, atomic commit ordering,
layout invalidation, update-aware save delegation, leave-alone save APIs,
settings and property preservation, public binding scope, panic safety, or
structure. No smells or nitpicks were found.

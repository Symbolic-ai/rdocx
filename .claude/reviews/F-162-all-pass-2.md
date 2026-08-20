# F-162, all, pass 2

**Reviewed**: complete working tree against `HEAD` (`6a60586`), 7 implementation files, 999 additions and 37 deletions, excluding review artifacts
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, pretty-printed complex package fields cannot be patched

`crates/rdocx/src/field.rs:876`

The package-backed story patch requires a field's captured source to be an
exact contiguous byte substring of the original part. Complex field source is
reconstructed by concatenating run elements and modeled boundary XML at
`crates/rdocx-oxml/src/text.rs:1397`. Whitespace text between those runs is not
retained in that source. A valid pretty-printed header, footer, or endnote with
newlines between the begin, instruction, separator, result, and end runs
therefore has no matching substring. `update_fields` returns `package story
field source was not found in its part` instead of updating it. The package
preservation regression at `crates/rdocx/tests/regression_test.rs:636` uses only
single-element simple fields and does not exercise a formatted complex field.

### D2, first-match patching can update an opaque lookalike instead of the typed field

`crates/rdocx/src/field.rs:876`

The patcher selects the first equal byte sequence after the previous match. It
does not anchor the match to the paragraph and run location from which the
typed `Field` was projected. If an earlier raw text box or producer extension
contains the same field fragment as a later direct typed field, the opaque copy
is replaced and the typed field remains stale. Reparse validation still
succeeds, so the wrong update is committed and raw text boxes no longer remain
byte-preserved and unevaluated. The new boundary regression at
`crates/rdocx/tests/regression_test.rs:636` uses distinct `x:` children rather
than a field-shaped opaque lookalike, so it cannot distinguish the intended
typed occurrence.

### D3, a same-structure public nested replacement is silently discarded

`crates/rdocx-oxml/src/text.rs:2366`

The source-preserving branch now treats two nested fields as structurally equal
when their instructions match, without retaining their source identity. A
caller can replace a public `FieldArgument::Nested` with `Field::new` using the
same instruction but a different cached display. The outer field is changed by
normal `PartialEq`, yet it enters this branch. The new child has no parsed source,
so `update_nested_field_sources` skips it at
`crates/rdocx-oxml/src/text.rs:2408` and returns the old producer bytes. The
replacement cache and dirty state disappear on serialization. Replacing the
child with a parsed field from another owner instead produces a source-not-found
error. Public structured edits must either serialize canonically or retain a
verified association with the original nested source. The repair test at
`crates/rdocx-oxml/src/text.rs:4933` mutates the original parsed child in place
and does not cover replacement identity.

## Smells

None.

## Nitpicks

None.

## Pass-1 repair verification

- D1 is repaired for its reported nested IF case. Cache and dirty updates retain
  the nested REF before the following comparison operands, and serialize-reparse
  evaluation remains `yes`.
- D2 is repaired for in-place updates of the original parsed nested tree. The
  recursive source patch retains producer run formatting, attributes,
  instruction partitioning, aliases, and unmodelled children.
- D3 is repaired for distinct unmodelled content around simple fields. Headers,
  footers, and endnotes are patched inside their original part bytes instead of
  being fully serialized. D1 and D2 above are new identity failures in that raw
  patching mechanism.

## Checks

- `cargo test -p rdocx-oxml text::tests`, passed, 62 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 14 tests.
- `cargo test -p rdocx --test regression_test`, passed, 73 tests.
- `cargo clippy -p rdocx -p rdocx-oxml --all-targets --no-deps -- -D warnings`,
  passed.
- `cargo fmt --all -- --check`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/prose_check.py`, passed with zero violations before writing
  this review.
- `git diff --check HEAD`, passed before writing this review.
- Progress evidence for the full package dry-run and archive-size assertion was
  inspected.

## Not found

No additional defect was found in F-161 evaluation order, mutable story order,
nested result indexing, dirty outcome policy, simple or complex marker
canonicalization, unsupported cache retention, atomic live-state commit,
layout invalidation, update-aware save delegation, leave-alone save APIs,
settings and property preservation, public binding scope, panic safety, HLD
scope, or structure. No smells or nitpicks were found.

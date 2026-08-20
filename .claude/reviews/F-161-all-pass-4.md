# F-161, all, pass 4

**Reviewed**: complete feature working tree against `HEAD` (`6629639`), 12 files, 2,852 additions and 39 deletions, excluding prior review artifacts
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, raw-only instruction edits evaluate the stale structured AST

`crates/rdocx/src/field.rs:156`

The evaluator dispatches directly from `field.instruction.name`, arguments, and
switches. F-160 explicitly supports changing the public raw instruction alone,
and serialization reparses that changed raw instruction at
`crates/rdocx-oxml/src/text.rs:2936`. For example, changing a parsed `DATE`
field's raw instruction to `AUTHOR` still evaluates it as `DATE`, while saving
the same field writes `AUTHOR`. A raw edit of a complex field can likewise
leave evaluation reporting nested fields that serialization removes. The
current snapshot therefore disagrees with the effective instruction that the
package will contain. Evaluation must use the same raw-versus-structured edit
resolution as serialization.

## Smells

None.

## Nitpicks

None.

## Pass-3 repair verification

- D1: repaired for structured edits. `nested_fields_in_source_order` uses the
  original private ordering only while the structured AST is unchanged, then
  falls back to the serializer's canonical positional-then-switch ordering.
  The focused test verifies current order and serialize-reparse order agree.
  The defect above is the separate raw-only edit path.
- D2: repaired. STYLEREF treats only a nonzero `numId` as numbering, and the
  focused test covers the OOXML `numId=0` numbering-off sentinel.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo test -p rdocx-oxml text::tests`, passed, 60 tests.
- `cargo test -p rdocx-oxml settings`, passed, 4 tests.
- `cargo test -p oxml-core custom_properties`, passed, 4 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 11 tests.
- `cargo test -p rdocx --test regression_test`, passed, 67 tests including the
  pinned Word field matrix.
- `cargo clippy -p rdocx --all-targets --no-deps -- -D warnings`, passed.
- `cargo clippy -p rdocx-oxml --all-targets --no-deps -- -D warnings`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/prose_check.py`, passed with zero violations before writing
  this review.
- `git diff --check HEAD`, passed before writing this review.

## Not found

No additional defect was found in structured-edit ordering, the numbering-off
sentinel, quote validation, numeric picture formatting, wildcard behavior,
unchanged nested source order, namespace and schema preservation, settings or
custom-property byte preservation, story traversal, sequence isolation,
pagination deferral, explicit input sourcing, stable diagnostics, public
binding scope, HLD scope, or module structure. No panic path was found. No
smells or nitpicks were found.

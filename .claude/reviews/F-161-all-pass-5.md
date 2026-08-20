# F-161, all, pass 5

**Reviewed**: complete feature working tree against `HEAD` (`6629639`), 12 files, 2,904 additions and 39 deletions, excluding prior review artifacts
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the effective-instruction clone evaluates nested IF fields twice

`crates/rdocx/src/field.rs:146`

`effective_instruction` returns an owned deep clone of the instruction tree.
The eager nested walk then evaluates the original nested fields and keys their
outcomes by address at `crates/rdocx/src/field.rs:217`, while IF argument
resolution looks up the cloned nested fields by their different addresses at
`crates/rdocx/src/field.rs:293`. Every nested IF operand therefore misses the
cache and is evaluated a second time. A nested MERGEFIELD produces a duplicate
`FieldEvaluation`. More seriously, a nested `SEQ Figure` advances once during
enumeration and again during IF resolution, so the outer IF consumes `2`
instead of `1`. This breaks document-order result enumeration and sequence
semantics even for an unchanged parsed instruction.

## Smells

None.

## Nitpicks

None.

## Pass-4 repair verification

- Dispatch: repaired. Raw-only edits are parsed through the same effective
  instruction selection used by serialization before handler dispatch.
- Result instruction: repaired. `FieldEvaluation.instruction` records the
  selected effective raw instruction.
- Nested enumeration: repaired for the raw-only case. The old structured
  children are no longer reported after raw replacement. D1 above is a new
  identity regression for effective instructions that still contain nested
  fields.
- Serialize-reparse agreement: repaired. Focused coverage verifies a raw-only
  replacement produces the same field and empty nested sequence before and
  after serialization.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo test -p rdocx-oxml text::tests`, passed, 60 tests.
- `cargo test -p rdocx-oxml settings`, passed, 4 tests.
- `cargo test -p oxml-core custom_properties`, passed, 4 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 12 tests.
- `cargo test -p rdocx --test regression_test`, passed, 67 tests including the
  pinned Word field matrix.
- `cargo clippy -p rdocx --all-targets --no-deps -- -D warnings`, passed.
- `cargo clippy -p rdocx-oxml --all-targets --no-deps -- -D warnings`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/prose_check.py`, passed with zero violations before writing
  this review.
- `git diff --check HEAD`, passed before writing this review.

## Not found

No additional defect was found in raw-only dispatch and reporting,
serialize-reparse agreement, structured-edit ordering, the numbering-off
sentinel, quote validation, numeric and date picture formatting, wildcard
behavior, namespace and schema preservation, settings or custom-property byte
preservation, story traversal, pagination deferral, explicit input sourcing,
stable diagnostics, public binding scope, HLD scope, or module structure. No
panic path was found. No smells or nitpicks were found.

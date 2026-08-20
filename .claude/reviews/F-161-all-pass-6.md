# F-161, all, pass 6

**Reviewed**: complete feature working tree against `HEAD` (`6629639`), 12 files, 2,948 additions and 39 deletions, excluding prior review artifacts
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, temporary clone addresses can reuse a stale nested outcome

`crates/rdocx/src/field.rs:219`

Nested outcomes remain cached for the evaluator's full lifetime by the address
of a nested `Field`. Those addresses now come from the temporary effective
instruction clone created at `crates/rdocx/src/field.rs:146`, which is dropped
when its outer field evaluation returns. A later outer field can allocate its
own cloned nested field at the same address. The `contains_key` branch then
skips that new field, and IF resolution returns the earlier field's stale
outcome from `crates/rdocx/src/field.rs:295`. Two consecutive IF fields with
different nested operands can therefore omit the second nested
`FieldEvaluation` and make the second IF consume the first operand's value.
Nested SEQ state can likewise be skipped or reused. The pass-5 repair makes
identity consistent within one live clone, but the cache key is not stable
across outer evaluations.

## Smells

None.

## Nitpicks

None.

## Pass-5 repair verification

- Nested SEQ: repaired within one effective instruction tree. The focused test
  asserts one nested result, value `1`, and no second sequence increment.
- Effective identity: repaired within one outer evaluation. Eager enumeration
  and IF resolution now use the same cloned nested objects and reuse their
  outcomes. D1 above is the remaining lifetime boundary between outer fields.
- Source order: repaired. Unchanged parsed instructions apply the captured
  physical order to the effective clone, while changed structured instructions
  use canonical positional-then-switch order.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo test -p rdocx-oxml text::tests`, passed, 60 tests.
- `cargo test -p rdocx-oxml settings`, passed, 4 tests.
- `cargo test -p oxml-core custom_properties`, passed, 4 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 13 tests.
- `cargo test -p rdocx --test regression_test`, passed, 67 tests including the
  pinned Word field matrix.
- `cargo clippy -p rdocx --all-targets --no-deps -- -D warnings`, passed.
- `cargo clippy -p rdocx-oxml --all-targets --no-deps -- -D warnings`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/prose_check.py`, passed with zero violations before writing
  this review.
- `git diff --check HEAD`, passed before writing this review.

## Not found

No additional defect was found in per-field effective instruction identity,
nested source ordering, raw-only dispatch and reporting, serialize-reparse
agreement, the numbering-off sentinel, quote validation, numeric and date
picture formatting, wildcard behavior, namespace and schema preservation,
settings or custom-property byte preservation, story traversal, pagination
deferral, explicit input sourcing, stable diagnostics, public binding scope,
HLD scope, or module structure. No panic path was found. No smells or nitpicks
were found.

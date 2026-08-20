# F-161, all, pass 3

**Reviewed**: complete feature working tree against `HEAD` (`6629639`), 12 files, 2,763 additions and 39 deletions, excluding prior review artifacts
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, private nested order becomes stale after public structured edits

`crates/rdocx-oxml/src/text.rs:94`

`nested_fields_in_source_order` always trusts positions captured from the
original parsed instruction before appending nested fields absent from that
private list. The public `Field.instruction.arguments` and `switches` vectors
remain directly mutable, and the serializer emits a changed instruction in
canonical positional-then-switch order at
`crates/rdocx-oxml/src/text.rs:2925`. If a caller inserts a new positional
nested operand into the parsed switch-before-positional case covered by the new
regression, evaluation still visits the original switch position first and
assigns the new operand a later index. The evaluated snapshot and its eventual
serialization therefore disagree on field order. Original parsed ordering is
now correct, but changed ASTs need to invalidate or rebuild the private order.

### D2, STYLEREF treats the numbering-off sentinel as a numbered source

`crates/rdocx/src/field.rs:515`

The numbered-switch fallback tests only whether `effective.num_id` is present.
In WordprocessingML, `w:numId w:val="0"` explicitly removes numbering inherited
from a style. Such a source paragraph has no numbered source text, but
`STYLEREF ... \\n`, `\\r`, `\\t`, or `\\w` returns `KeepStored` instead of the
ordinary paragraph text. The direct-property repair correctly catches real
numbering, but the predicate must distinguish the zero sentinel.

## Smells

None.

## Nitpicks

None.

## Pass-2 repair verification

- D1: repaired. Balanced-quote validation now keeps unterminated argument and
  picture instructions on stable fallback, with focused coverage.
- D2: repaired for directly numbered paragraphs. Direct `pPr` is merged into
  effective STYLEREF properties and a direct nonzero `numId` falls back. D2
  above is the remaining numbering-off sentinel case.
- D3: repaired. Missing integer and decimal `#` slots now emit spaces, and
  embedded quoted literals count those blank slots. The focused expectations
  match the pinned Word numeric-picture behavior.
- D4: repaired. IF wildcard regexes use case-insensitive dot-all matching and
  the multiline nested REF case is covered.
- D5: repaired for an unchanged parsed instruction. Nested switch and
  positional fields retain their physical source indices. D1 above covers the
  remaining public-mutation case.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo test -p rdocx-oxml text::tests`, passed, 59 tests.
- `cargo test -p rdocx-oxml settings`, passed, 4 tests.
- `cargo test -p oxml-core custom_properties`, passed, 4 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 11 tests.
- `cargo test -p rdocx --test regression_test`, passed, 67 tests including the
  pinned Word field matrix and nested source-order regression.
- `cargo clippy -p rdocx --all-targets --no-deps -- -D warnings`, passed.
- `cargo clippy -p rdocx-oxml --all-targets --no-deps -- -D warnings`, passed.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/prose_check.py`, passed.
- `git diff --check HEAD`, passed.

## Not found

No additional defect was found in quote validation, numeric blank-slot and
literal placement, multiline wildcard behavior, unchanged nested source order,
namespace and schema preservation, settings or custom-property byte
preservation, story traversal, sequence isolation, pagination deferral,
explicit input sourcing, stable diagnostics, public binding scope, HLD scope,
or module structure. No panic path was found. No smells or nitpicks were found.

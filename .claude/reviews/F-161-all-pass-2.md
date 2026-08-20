# F-161, all, pass 2

**Reviewed**: complete feature working tree against `HEAD` (`6629639`), 11 files, 2,538 additions and 26 deletions, excluding the pass-1 review artifact
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, unterminated quoted instructions can resolve instead of keeping stored text

`crates/rdocx/src/field.rs:838`

`validate_instruction_shape` verifies operand counts and switch argument shapes,
but it has no validation for the lexical integrity of `instruction.raw`. The
F-160 lexer removes an opening quote even when no closing quote exists, so an
instruction such as `MERGEFIELD "Name` reaches this validator as one ordinary
text operand and can resolve from the merge map. The same problem applies to an
unterminated quoted picture. The approved contract requires malformed
instructions to return `KeepStored` with a stable diagnostic.

### D2, STYLEREF numbered switches miss direct paragraph numbering

`crates/rdocx/src/field.rs:510`

The numbered-source check resolves style properties but never merges the source
paragraph's direct `pPr`, unlike the SEQ heading path. A matching paragraph with
a direct `w:numPr` and an unnumbered style therefore has no `effective.num_id`,
so `STYLEREF ... \\n`, `\\r`, `\\t`, or `\\w` incorrectly resolves the paragraph
text. The plan requires every numbered source to stay on cached fallback until
numbered source text is supported.

### D3, optional numeric placeholders do not preserve Word's blank slots

`crates/rdocx/src/field.rs:1150`

The formatter counts only required `0` placeholders when padding and emits no
character for an unfilled `#` placeholder. Word's `#` placeholder emits a space
when that numeric place has no digit. For example, the approved optional-digit
picture `$###` applied to `15` must retain the leading blank slot, while this
implementation returns `$15`. This also misplaces quoted literals embedded
among optional positions because literal insertion is based only on emitted
digits.

### D4, IF wildcards do not match multiline bookmark values

`crates/rdocx/src/field.rs:917`

`wildcard_matches` builds a case-insensitive regex without dot-all mode, so the
`.` expansions for both `?` and `*` cannot consume a newline. Valid REF operands
can contain multiline bookmark text because bookmark ranges join paragraphs
with a newline at `crates/rdocx/src/comments.rs:776`. An IF equality wildcard
over such a nested REF therefore returns the wrong branch even though the
approved `?` and `*` wildcard contract has no single-paragraph restriction.

### D5, nested fields can receive indices out of source order

`crates/rdocx/src/field.rs:216`

The recursive walk reports every positional nested argument first and only then
reports nested switch arguments. The F-160 AST stores those collections
separately, but a switch and a later positional operand can appear in the
opposite order in the original instruction. For example, an outer instruction
whose nested switch argument precedes a nested positional argument is reported
in positional-then-switch order. This violates the public contract that
`field_index` is document order for the unchanged snapshot.

## Smells

None.

## Nitpicks

None.

## Pass-1 repair verification

- D1: repaired for enumeration. Nested positional and switch arguments now all
  receive outcomes, including those under an unsupported outer field. D5 above
  is the remaining ordering defect.
- D2: repaired. Header and footer parts now come from section references in
  section order, orphan relationships are excluded, and shared physical parts
  are deduplicated.
- D3: repaired for operand and switch arity. Required text switch arguments and
  extra positional operands now fall back.
- D4: repaired. SEQ increment uses checked arithmetic and returns stable
  fallback on overflow.
- D5: repaired. Weekday arithmetic promotes the year to `i64` before adjusting
  January and February.
- D6: repaired. Empty merge input omits both affixes.
- D7: repaired. Non-DATE and non-TIME date pictures parse the resolved field
  value rather than using the context clock.
- D8: repaired for single-quoted literals and embedded required-digit
  literals. D3 above is a separate optional-placeholder failure in the same
  formatter.
- D9: repaired. Custom-property expanded names and unqualified property
  attributes are checked, while foreign lookalikes remain package-preserved and
  absent from evaluation.
- D10: repaired. The focused tests now exercise the named general formats,
  single-quoted numeric literals, resolved-value date pictures, and SEQ heading
  restart.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo test -p rdocx-oxml settings`, passed, 4 tests.
- `cargo test -p oxml-core custom_properties`, passed, 4 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 10 tests.
- `cargo test -p rdocx --test regression_test`, passed, 66 tests including the
  pinned Word field matrix and existing REF and PAGEREF pagination regression.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/prose_check.py`, passed.
- `git diff --check HEAD`, passed.

## Not found

No additional defect was found in namespace alias handling, settings or custom
property byte preservation, section-driven header and footer selection, story
sequence isolation, explicit input sourcing, pagination deferral, public
binding scope, HLD file scope, or module structure. No panic path was reproduced
after the SEQ and extreme-year repairs. No smells or nitpicks were found.

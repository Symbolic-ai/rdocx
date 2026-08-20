# F-161, all, pass 1

**Reviewed**: complete working tree against `HEAD`, 10 files, 1,928 additions and 26 deletions
**Verdict**: 10 defects, 0 smells, 0 nitpicks

## Defects

### D1, nested fields are omitted unless a handler happens to resolve their argument

`crates/rdocx/src/field.rs:152`

The generic field path dispatches the outer instruction without first walking
its recursive argument and switch-argument tree. Only handlers such as `IF`
that call `resolve_argument` report nested fields. A nested field inside an
unsupported outer field, or inside a switch argument that the selected handler
does not resolve, receives no `FieldEvaluation` and no document-order index.
This violates the recursive F-160 AST contract and the promise to evaluate
every typed field.

### D2, header and footer traversal evaluates orphan parts in relationship order

`crates/rdocx/src/field.rs:67`

`related_parts` selects every header or footer relationship and orders stories
by relationship-list order. It never checks the section properties that place
those parts in the document. An unreferenced header relationship is therefore
reported as a document field, while two referenced parts can receive indices in
an order unrelated to their section placement. The existing facade resolves
header and footer stories from `CT_SectPr.header_refs` and `footer_refs`. Field
evaluation must use that source placement and deduplicate shared physical parts
without admitting orphans.

### D3, malformed switch arity is accepted as a valid instruction

`crates/rdocx/src/field.rs:168`

Validation checks only whether a switch name is allowed. It does not verify
required arguments or reject extra positional operands. For example,
`DATE \\@` silently uses the default picture, `SEQ Figure \\r` increments the
sequence, and `PAGE extra` returns `DeferredPagination`. The approved contract
requires malformed instructions to return `KeepStored` with a stable
diagnostic.

### D4, a valid SEQ reset can overflow on the next field

`crates/rdocx/src/field.rs:335`

`SEQ Figure \\r 9223372036854775807` stores `i64::MAX`. The next ordinary SEQ
adds one without checked arithmetic. Debug and test builds panic, while release
builds can wrap to a negative value. Both outcomes violate the no-panic and
stable-fallback contract for untrusted field instructions.

### D5, an accepted FieldDateTime can overflow weekday calculation

`crates/rdocx/src/field.rs:1176`

`valid_date_time` accepts any `i32` year. Formatting a January or February date
with a weekday token when `year == i32::MIN` subtracts one from that value.
Debug and test builds panic before the evaluator can return a stable fallback.
The public input must either be bounded during validation or use checked wider
arithmetic.

### D6, MERGEFIELD prefix and suffix are emitted for an empty supplied value

`crates/rdocx/src/field.rs:568`

The evaluator always appends `\\b` and `\\f` text around a present map value,
even when that value is the empty string. Word emits those fragments only when
the merge field contains data. A context containing `Name -> ""` therefore
resolves `MERGEFIELD Name \\b "Dear " \\f "!"` to `Dear !` instead of the empty
result required by the pinned Word contract.

### D7, date pictures format the context clock instead of the resolved field value

`crates/rdocx/src/field.rs:814`

The general `\\@` path discards the resolved string and formats
`FieldEvaluationContext.now`. This is correct for DATE and TIME, but wrong for
fields such as `MERGEFIELD Date \\@ "MMMM d, yyyy"` and a date-valued
DOCPROPERTY. With no clock they fall back despite having a supplied value. With
a clock they return that unrelated clock value. Date-time pictures must consume
the selected field's value, with DATE and TIME obtaining that value from the
explicit clock.

### D8, Word numeric-picture literal quoting is not implemented

`crates/rdocx/src/field.rs:1030`

Word numeric pictures use single quotes for literal text inside the picture.
`unquote_picture` recognizes only double quotes. A parsed picture such as
`$##0.00 'is sales tax'` therefore retains the quote characters, and a literal
between digit placeholders is rejected by the core-token check. The approved
numeric matrix explicitly includes quoted literals, so these inputs must
produce the Word text rather than a wrong string or `KeepStored`.

### D9, custom properties are evaluated without enforcing OOXML namespaces

`crates/rdocx/src/document.rs:497`

F-161 loads `CustomProperties::from_xml` directly, but that parser recognizes
the root, property, value elements, and attributes by local name alone at
`crates/oxml-core/src/custom_properties.rs:69`. A relationship target made of
foreign-namespace `Properties`, `property`, and `lpwstr` elements is accepted
and can resolve a DOCPROPERTY field. Prefix aliases must be accepted only when
they bind to the custom-properties and variant-types namespaces. Foreign
lookalikes must remain preserved but absent from the typed evaluation source.

### D10, the required switch tests leave contract branches unproved

`crates/rdocx/src/field.rs:1400`

The formatting test covers one capitalization result, alphabetic, Roman,
ordinal, two numeric pictures, and one date picture. It does not cover Lower,
FirstCap, Caps, Arabic, MERGEFORMAT, Charformat, Word single-quoted literals,
or date formatting of resolved property and merge values. The SEQ test at
`crates/rdocx/src/field.rs:1309` also omits heading restart `\\s`, despite its
test-plan name requiring every supported sequence switch. The pinned regression
uses only one happy-path format per family, so the defects above can pass the
declared gate.

## Smells

None.

## Nitpicks

None.

## Checks

- `cargo fmt --all --check`, passed.
- `cargo check -p rdocx --all-targets`, passed.
- `cargo test -p rdocx-oxml settings`, passed, 4 tests.
- `cargo test -p rdocx --lib field::tests`, passed, 8 tests.
- `cargo test -p rdocx --test regression_test`, passed, 64 tests including the existing REF and PAGEREF pagination regression.
- `cargo clippy -p rdocx --all-targets --no-deps -- -D warnings`, passed for the F-161 crate and targets.
- `cargo clippy -p rdocx --all-targets -- -D warnings`, stopped on the inherited F-160 findings at `crates/rdocx-oxml/src/text.rs:1205` and `crates/rdocx-oxml/src/text.rs:2695`. They are outside the F-161 diff and are not counted above.
- `python3 scripts/hash_harness.py --check`, passed, 49 entries unchanged.
- `python3 scripts/prose_check.py`, passed.
- `git diff --check HEAD`, passed.

## Not found

No additional defects were found in settings byte preservation, settings alias
handling, established PAGE, NUMPAGES, REF, and PAGEREF layout substitution,
ambient clock or filesystem access, public binding expansion, HLD file scope,
or the approved module structure. No smells or nitpicks were found.

# F-160, all, pass 6

**Reviewed**: complete working diff against `HEAD`, including staged and
unstaged changes, 11 files and 5,504 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, simple control-only caches still choose formatting the serializer discards

`crates/rdocx-oxml/src/text.rs:128`
`crates/rdocx-oxml/src/text.rs:2361`

The edited-cache projection takes properties from the first cached segment,
but the simple-field serializer does not always place replacement text in that
segment. For a valid simple field whose only stored display is a formatted
`w:tab` or `w:br`, parsing records that control's run properties. After
`cached_result` changes to `fresh`, `cached_display_segments` reports `fresh`
with those properties. Serialization suppresses the old control, never marks a
result as written, and appends a new default result run with no properties at
the field end. Layout, HTML, and Markdown therefore show formatted text before
save and unformatted text after reopen. The same mismatch occurs when an
earlier control-only segment has different properties from the later text run
that receives the replacement.

## Pass-5 closure

- Edited parsed complex caches now retain the first original result segment's
  properties in the shared display projection.
- The deterministic computed PAGE regression edits `cached_result` before
  layout and proves the computed text retains bold and italic formatting.
- The focused complex model regression proves the edited cache and first-run
  properties are returned together.

## Checks

- `cargo test -p rdocx-oxml edited_complex_cache_keeps_the_first_result_runs_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml simple_field_cache_mutation_replaces_old_result_controls --lib`, 1 passed.
- `cargo test -p rdocx-layout a_computed_complex_field_keeps_its_cached_result_run_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml --lib`, 247 passed.
- `cargo test -p rdocx-layout --lib`, 111 passed.
- `cargo test -p rdocx-html --lib`, 12 passed.
- `cargo check -p rdocx-oxml --all-targets` passed.
- `cargo check -p rdocx-layout --all-targets` passed.
- `cargo fmt --all --check` passed.
- `python3 scripts/hash_harness.py --check`, 49 entries matched.
- `git diff --check HEAD` passed.

## Not found

No additional correctness, contract, panic, OOXML namespace or child-order,
test-structure, or repository-structure findings were found. No smells or
nitpicks were found.

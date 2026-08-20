# F-160, all, pass 8

**Reviewed**: complete working diff against `HEAD`, including staged and
unstaged changes, 11 files and 6,321 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, an empty text element still selects the wrong result-run formatting

`crates/rdocx-oxml/src/text.rs:844`
`crates/rdocx-oxml/src/text.rs:2344`

Simple-field segment collection now excludes a run whose display string is
empty, but the serializer still treats any direct empty `w:t` as its first
replacement point. For a bold run containing `<w:t/>` followed by an italic
run containing `old`, changing the cache to `fresh` projects italic formatting
in memory because the first segment is excluded. Serialization writes `fresh`
into the bold run's empty text element, then reopening projects bold
formatting. The new regression covers a leading run with no display child, so
it does not exercise the empty-element replacement path. Empty text elements
are valid producer XML and must use the same segment-selection rule on both
sides of the cache mutation.

## Pass-7 closure

- A leading result run with no text or control is no longer recorded as a
  cached display segment.
- The new regression proves that a no-content bold run followed by italic text
  keeps the italic formatting in memory and after reopen.
- The control-only and mixed-control cache mutation regressions continue to
  pass.

## Checks

- `cargo test -p rdocx-oxml simple_cache_mutation_skips_empty_leading_result_runs_for_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml simple_control_only_cache_mutation_keeps_result_run_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml simple_field_cache_mutation_replaces_old_result_controls --lib`, 1 passed.
- `cargo test -p rdocx-oxml --lib`, 249 passed.
- `cargo check -p rdocx-oxml --all-targets` passed.
- `cargo fmt --all --check` passed.
- `python3 scripts/hash_harness.py --check`, 49 entries matched.
- `git diff --check HEAD` passed.

## Not found

No additional correctness, contract, panic, OOXML namespace or child-order,
test-structure, or repository-structure findings were found. No smells or
nitpicks were found.

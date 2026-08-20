# F-160, all, pass 9

**Reviewed**: complete working diff against `HEAD`, including staged and
unstaged changes, 11 files and 5,362 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, expanded empty result controls disappear from the cached display

`crates/rdocx-oxml/src/text.rs:898`
`crates/rdocx-oxml/src/text.rs:525`
`crates/rdocx-oxml/src/text.rs:534`

The new direct-display scan recognizes `w:tab` and `w:br` start elements, but
the run parser models those controls only when quick-xml reports a self-closing
empty element. XML producers may equivalently write `<w:tab></w:tab>` or
`<w:br></w:br>`. The start-element path captures either form as unknown XML,
so `cached_text_for_run` contributes no tab or break even though the display
scan records the run as a cached segment. An unchanged simple field therefore
preserves the source bytes but renders and exports an empty cached display in
place of the stored control. The regressions cover self-closing controls only.

## Pass-8 closure

- Simple-field segment collection now selects a run by its direct display
  elements instead of requiring nonempty concatenated text.
- A leading self-closing empty `w:t` now supplies the same run formatting in
  memory and after reopen.
- Direct `w:delText` is included in both display detection and cache rewriting.
- The no-display-child and control-only regressions continue to pass.

## Checks

- `cargo test -p rdocx-oxml simple_cache_mutation_skips_empty_leading_result_runs_for_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml simple_cache_mutation_uses_an_empty_text_runs_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml simple_control_only_cache_mutation_keeps_result_run_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml simple_field_cache_mutation_replaces_old_result_controls --lib`, 1 passed.
- `cargo test -p rdocx-oxml --lib`, 250 passed.
- `cargo check -p rdocx-oxml --all-targets` passed.
- `cargo fmt --all --check` passed.
- `python3 scripts/hash_harness.py --check`, 49 entries matched.
- `git diff --check HEAD` passed.

## Not found

No additional correctness, contract, panic, OOXML namespace or child-order,
test-structure, or repository-structure findings were found. No smells or
nitpicks were found.

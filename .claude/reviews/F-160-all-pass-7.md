# F-160, all, pass 7

**Reviewed**: complete working diff against `HEAD`, including staged and
unstaged changes, 11 files and 5,530 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, an empty simple-field run can still become the wrong formatting source

`crates/rdocx-oxml/src/text.rs:844`
`crates/rdocx-oxml/src/text.rs:128`

Simple-field parsing records every direct result run as a cached segment even
when that run contributes no text or control. The edited-cache projection then
takes properties from the first such segment. For a field with an empty bold
run followed by an italic run containing `old`, changing `cached_result` to
`fresh` renders bold in memory. Serialization leaves the empty run alone and
writes `fresh` into the later `w:t`, so reopening renders italic. Empty runs are
valid WordprocessingML and are common producer residue. The pass-6 repair fixes
a first segment that contains a control, but it does not align empty leading
segments with the serializer's actual replacement location.

## Pass-6 closure

- A changed simple cache whose first display segment is `w:tab` or `w:br` now
  writes replacement content inside that source run before suppressing the old
  control.
- The new control-only regression reparses the field and proves the original
  bold and italic run properties remain attached to the replacement display.
- The prior mixed-text and complex-cache regressions continue to pass.

## Checks

- `cargo test -p rdocx-oxml simple_control_only_cache_mutation_keeps_result_run_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml simple_field_cache_mutation_replaces_old_result_controls --lib`, 1 passed.
- `cargo test -p rdocx-oxml edited_complex_cache_keeps_the_first_result_runs_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml --lib`, 248 passed.
- `cargo check -p rdocx-oxml --all-targets` passed.
- `cargo fmt --all --check` passed.
- `python3 scripts/hash_harness.py --check`, 49 entries matched.
- `git diff --check HEAD` passed.

## Not found

No additional correctness, contract, panic, OOXML namespace or child-order,
test-structure, or repository-structure findings were found. No smells or
nitpicks were found.

# F-160, all, pass 5

**Reviewed**: complete working diff against `HEAD`, including staged and
unstaged changes, 11 files and 5,493 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, an in-memory cache edit still discards stored formatting

`crates/rdocx-oxml/src/text.rs:120`

`cached_display_segments` returns the original result-run properties only while
`cached_result` equals its parsed value. After a caller changes the public
cache, the fallback segment has no properties. Layout, HTML, and Markdown
therefore render a parsed bold or styled field without that formatting until
the field is saved and reopened. The serializer keeps the original first result
run and its properties, so reopening restores the formatting and makes output
depend on whether a round trip happened. This also leaves the computed-field
repair incomplete for a `PAGE`, `NUMPAGES`, resolved `REF`, or resolved
`PAGEREF` field whose cache was edited in memory. The cache-update contract says
run formatting is preserved.

## Pass-4 closure

- Changed simple-field caches now suppress old direct `w:tab` and `w:br`
  children. The new regression reparses the output and proves the public cache
  value is exact.
- Unchanged parsed computed fields now use the first cached result segment's
  properties. The new deterministic PAGE regression proves bold and italic
  formatting reaches final layout.

## Checks

- `cargo test -p rdocx-oxml simple_field_cache_mutation_replaces_old_result_controls --lib`, 1 passed.
- `cargo test -p rdocx-layout a_computed_complex_field_keeps_its_cached_result_run_formatting --lib`, 1 passed.
- `cargo test -p rdocx-oxml --lib`, 246 passed.
- `cargo test -p rdocx-layout --lib`, 111 passed.
- `cargo test -p rdocx-html --lib`, 12 passed.
- `cargo check -p rdocx-oxml --all-targets` passed.
- `cargo check -p rdocx-layout --all-targets` passed.
- `cargo check -p rdocx-html --all-targets` passed.
- `cargo check -p rdocx --all-targets` passed.
- `python3 scripts/hash_harness.py --check`, 49 entries matched.
- `git diff --check HEAD` passed.

## Not found

No additional correctness, contract, panic, OOXML namespace or child-order,
test-structure, or repository-structure findings were found. No smells or
nitpicks were found.

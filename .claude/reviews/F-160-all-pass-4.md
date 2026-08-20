# F-160, all, pass 4

**Reviewed**: complete working diff against `HEAD`, including staged and
unstaged changes, 11 files and 5,390 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, simple-field cache replacement retains old controls

`crates/rdocx-oxml/src/text.rs:2338`

The simple-field rewrite handles direct `w:t` children specially but copies
every other empty run child unchanged. If a parsed simple field contains
`w:tab` or `w:br` in its cached display and the caller replaces
`cached_result`, the writer emits the new text while retaining those old
controls. A field changed from `one\ttwo` to `fresh`, for example, reparses as
`fresh\t`. The complex-field repair removes and reconstructs these controls,
but the equivalent simple-field path still fails to serialize the public cache
value.

### D2, computed fields discard stored result-run formatting

`crates/rdocx-layout/src/engine.rs:953`

The computed-value branch creates a single segment with no stored properties.
Consequently a complex `PAGE`, `NUMPAGES`, resolved `REF`, or resolved
`PAGEREF` field whose result run is bold, italic, styled, sized, coloured, or
otherwise formatted loses that formatting during layout. Before projection,
the complex result was an ordinary formatted run. The new formatting regression
test covers only the unsupported-field branch that calls
`cached_display_segments`, so it does not exercise this path.

## Pass-3 closure

- Multi-run stored displays now retain distinct result-run properties in the
  model, HTML, Markdown, and the unsupported-field layout path.
- Complex cache and dirty rewrites reconstruct tabs and breaks, remove dirty
  attributes from non-begin markers, and add missing result text.
- A nested-only complex result is replaced rather than appended to.
- Hyperlink-local raw children inside a projected field are captured into the
  field source and remain within the field boundary.
- Generated cache text uses `xml:space="preserve"` when boundary whitespace is
  significant.
- A structured nested edit converts a simple source to complex form.

## Checks

- `cargo test -p rdocx-oxml --lib`, 245 passed.
- `cargo test -p rdocx-layout --lib`, 110 passed.
- `cargo test -p rdocx-html --lib`, 12 passed.
- `cargo check -p rdocx-oxml --all-targets` passed.
- `cargo check -p rdocx-layout --all-targets` passed.
- `cargo check -p rdocx-html --all-targets` passed.
- `cargo check -p rdocx --all-targets` passed.
- `python3 scripts/hash_harness.py --check`, 49 entries matched.
- `git diff --check HEAD` passed.

## Not found

No additional contract, panic, OOXML namespace or child-order, test-structure,
or repository-structure findings were found. No smells or nitpicks were found.

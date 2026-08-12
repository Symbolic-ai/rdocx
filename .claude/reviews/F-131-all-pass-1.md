# F-131, all, pass 1

**Reviewed**: working implementation diff from claim base `3db056b`, 16
files and 2,553 changed or newly added lines, with 13 modified tracked files
and 3 approved untracked files
**Verdict**: 4 defects, 0 smells, 1 nitpick

## Defects

### D1, Cell text replacement does not invalidate nested paths
`crates/rdocx-py/src/table.rs:613`
`crates/rdocx/src/table.rs:277`

The Python `Cell.text` setter returns without bumping the document revision,
but the facade operation clears the first paragraph's run vector and inserts a
new run, or inserts a new paragraph when none exists. A held nested Run or Font
therefore keeps the old revision and can silently resolve to the replacement
run at the same index. This recreates the index-path aliasing failure that the
F-129 revision domain must turn into `StaleElementError`. Bump the revision
after the successful replacement and add a regression that holds a nested run
or font across `cell.text = ...`.

### D2, The published underline enum change is not additive
`crates/rdocx/src/run.rs:12`
`crates/rdocx/src/lib.rs:37`

`UnderlineStyle` is a publicly re-exported exhaustive enum, and this diff adds
`DotDash` and `DotDotDash` variants. Existing downstream exhaustive matches
will stop compiling, so the change contradicts the plan's additive semver
classification. A publication dry run cannot detect this source break. Keep
the established exhaustive enum unchanged or revise the design and release
impact before treating this branch as additive.

### D3, The established first-line-indent helper changed behavior
`crates/rdocx/src/paragraph.rs:275`

The existing `set_first_line_indent` now delegates to the new Python-oriented
value setter. Negative inputs that previously populated `ind_first_line`
instead become positive `ind_hanging` values, and positive inputs now clear an
existing hanging indent. This changes the serialization behavior of a
published Rust helper even though F-131 is required to preserve old helper
semantics. Keep the existing setter behavior intact and route only the new
Python property path through the signed clearing behavior, with a regression
for the established helper.

### D4, The planned clearing regression omits paragraph tri-state values
`crates/rdocx-py/tests/test_formatting_tables.py:29`
`crates/rdocx-py/tests/test_formatting_tables.py:73`

`test_none_clears_direct_formatting` clears only font bold, italic, and
underline. The paragraph round-trip test writes explicit false and true values
but never assigns `None` to keep-with-next, keep-together,
page-break-before, or widow control. Changing any paragraph clearing setter to
write explicit false would leave the complete F-131 Python suite green, so the
test plan does not protect the paragraph half of the tri-state contract. Add
at least one paragraph boolean true-to-None round trip and assert the reopened
public property is exactly `None`.

## Smells

None.

## Nitpicks

- `crates/rdocx/src/paragraph.rs:743`, the existing public `alignment` accessor
  lost its rustdoc sentence while nearby new accessors remain documented.

## Not found

- Correctness beyond D1 and D3: Body, Row, Cell, Para, and Run resolution is
  consistent for body and nested-cell paths. Successful table and paragraph
  additions bump once, failed lookups do not bump, and value-only formatting
  setters retain the captured revision.
- Tri-state behavior beyond D4: run and paragraph readers preserve `None`,
  false, and true. Clearing removes direct values, and the established bool
  helpers continue to collapse unset to false as before.
- Lazy collections: table, row, cell, and nested paragraph collections store
  document and path state rather than wrapper vectors. Integer and negative
  indexing, slices in both directions, and iterators normalize against current
  lengths. Slice results are materialized only at the Python slice boundary.
- Python values: getters construct the F-132 public `Length`, `RGBColor`, and
  bounded `IntEnum` classes. Setters convert EMU through the canonical Rust
  `Length`, including negative first-line indentation, without a python-docx
  runtime or test dependency.
- Table behavior: style, center alignment, dxa widths, cell text, cell width,
  vertical alignment, and nested paragraph formatting round-trip through the
  public facade. Lookups are total and use no direct `rdocx-oxml` dependency.
- PyO3 safety and cache purity: no escaped Rust borrow, unsafe block, GIL-free
  Python access, or nested mutable pyclass borrow was found. Formatting and
  table reads use immutable facade accessors and do not clear layout caches.
- Panics: no new panic is reachable from Python indexing or untrusted path
  input. Collection bounds use checked normalization and facade lookup returns
  `Option`.
- OOXML: no parser, namespace, schema child-order, whitespace, or raw-subtree
  preservation code changed. The property changes stay behind facade methods.
- Contract and scope: the diff consumes the integrated F-129, F-130, and F-132
  contracts, introduces only the approved modules and test file, and takes no
  F-133 rendering or GIL-release scope.
- HLD discipline: exactly HLD03, HLD10, and HLD14 changed, matching the
  approved impact list. The updates describe current facade ownership,
  bounded Python inventory, and the real F-132 dependency without history
  prose.
- Artifacts and checks: no extension, wheel, Python cache, or compiled Python
  file is present in the worktree. `cargo check -p rdocx-py --all-targets` and
  the full `cargo test -p rdocx` suite passed in isolated review targets. The
  worker evidence records the focused wheel tests, WASM rider, publication dry
  run and archive sizes, conversion regressions, and unchanged 28-entry hash
  harness.

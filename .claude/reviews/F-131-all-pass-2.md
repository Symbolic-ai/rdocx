# F-131, all, pass 2

**Reviewed**: working implementation diff from claim base `3db056b`, 16
files and 2,657 changed or newly added lines, with 13 modified tracked files
and 3 approved untracked implementation files
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, The implementation checklist is still entirely unticked
`.claude/plans/F-131-design.md:100`

All six implementation checklist entries remain unchecked even though the
progress record says implementation and pass-1 remediation are complete. This
leaves the approved plan out of sync with the reviewed implementation and
would fail the explicit `/complete-feature` precondition that every checklist
item be ticked. Check each completed item after confirming its evidence.

### D2, Table justification cannot be represented by the Python getter
`crates/rdocx-py/src/table.rs:77`
`crates/rdocx-py/python/rdocx/enum/table.py:6`
`crates/rdocx/src/table.rs:90`

The public Rust table setter accepts `Alignment::Justify` and serializes it as
`ST_Jc::Both`, while the new Python getter maps that value to integer 3. The
approved `WD_TABLE_ALIGNMENT` enum defines only 0, 1, and 2, so constructing
the getter result raises `ValueError: 3 is not a valid WD_TABLE_ALIGNMENT`.
An installed-wheel probe reproduced this after reopening a table whose
`w:jc` value was `both`. The getter must handle every value that the public
facade can return without raising an enum-construction error.

### D3, The invalid underline-code test does not prove non-mutation
`crates/rdocx/tests/integration_test.rs:225`
`crates/rdocx/tests/integration_test.rs:227`
`crates/rdocx/src/run.rs:168`

The public method promises that an unsupported code returns false without
mutation. The regression sets code 9, calls invalid code 5, then immediately
overwrites the value with valid code 10 before observing state. It would stay
green if the invalid call cleared or otherwise changed the underline because
the later write hides that change. Assert code 9 immediately after the failed
call, then continue with the code 10 round trip.

### D4, Automatic OOXML font colour raises from the advertised getter
`crates/rdocx-py/src/formatting.rs:223`
`crates/rdocx-py/python/rdocx/shared.py:102`
`crates/rdocx-oxml/src/properties.rs:600`

The OOXML layer accepts and preserves the standard `w:color w:val="auto"`
form as a raw string. The Python `Font.color` getter sends every stored string
to `RGBColor.from_string`, which accepts exactly six hexadecimal digits. An
installed-wheel reopen probe therefore raised `ValueError` when reading
`font.color` from an otherwise accepted document with automatic colour. The
bounded colour property needs an explicit automatic-colour result rather than
letting a valid stored value escape as a conversion failure.

### D5, Nested paragraph stale errors give an unusable recovery path
`crates/rdocx-py/src/paragraph.rs:79`
`crates/rdocx-py/src/paragraph.rs:44`

`PyParagraph` uses the body-only hint `doc.paragraphs[i]` for every path, even
though the same class resolves cell paths containing table, row, and cell
segments. After a nested paragraph becomes stale, the emitted instruction
points to an unrelated body paragraph instead of the required
`doc.tables[t].rows[r].cells[c].paragraphs[i]` path. An installed-wheel probe
held a cell paragraph, added a body paragraph, and reproduced the incorrect
hint. Select the recovery guidance from the resolved path shape and cover the
nested case.

## Smells

None.

## Nitpicks

None.

## Pass-1 re-evaluation

- D1 is resolved at `crates/rdocx-py/src/table.rs:613`. A successful cell text
  replacement now bumps once at line 626, and
  `crates/rdocx-py/tests/test_formatting_tables.py:144` holds both a nested Run
  and Font and checks the exact revision transition.
- D2 is resolved at `crates/rdocx/src/run.rs:12`. The established exhaustive
  `UnderlineStyle` variants are restored, while codes 9 and 10 use the additive
  integer facade at line 168. The compatibility match at
  `crates/rdocx/tests/integration_test.rs:267` would fail to compile if another
  public enum variant were reintroduced.
- D3 is resolved at `crates/rdocx/src/paragraph.rs:275`. The legacy helper again
  writes `firstLine` directly, and the separate signed Python path begins at
  line 282. The compatibility regression at
  `crates/rdocx/tests/integration_test.rs:283` protects the negative legacy
  serialization.
- D4 is resolved at
  `crates/rdocx-py/tests/test_formatting_tables.py:29`. All four paragraph
  tri-state values are set true, cleared with `None`, reopened, and asserted
  exactly `None` at lines 58 through 61.
- The pass-1 rustdoc nitpick is resolved at
  `crates/rdocx/src/paragraph.rs:743`.

## Not found

- Revision and cache correctness beyond D5: structural document, paragraph,
  table, and cell mutations bump after success. Value-only formatting writes
  preserve handle revisions. Read paths use immutable facade accessors and do
  not clear the layout cache.
- Lazy collection defects: table, row, cell, and nested paragraph collections
  retain document and path state rather than wrapper vectors. Checked integer
  indexing, negative indexing, forward and reverse slices, and iteration use
  current validated lengths.
- Path resolution defects beyond D5: body and cell paragraph shapes resolve to
  the intended total facade accessors. Incomplete shapes return Python errors
  rather than panicking.
- Length and integer conversion defects: Python lengths route through the
  shared F-132 value types and canonical Rust `Length`, including signed first
  line indentation and truncation. Checked enum setters reject unsupported
  values before acquiring a mutable document borrow.
- Public semver defects beyond the pass-1 fixes: the restored exhaustive
  underline enum and unchanged first-line helper preserve established Rust
  behavior. New facade entry points are additive and introduce no trait,
  generic parameter, feature flag, or dynamic dispatch.
- PyO3 safety defects: no unsafe block, escaped Rust borrow, GIL-free Python
  access, nested mutable pyclass borrow, or direct OOXML dependency was found
  in the bindings.
- OOXML preservation and schema defects: no parser ownership, namespace,
  schema child order, whitespace, or raw-subtree behavior changed. New facade
  setters update existing property models.
- HLD and scope defects: exactly `docs/hld/03-architecture.md`,
  `docs/hld/10-bindings-spec.md`, and `docs/hld/14-development-backlog.md`
  changed, matching the approved impact list. No F-133 rendering or GIL-release
  scope entered the diff.
- Artifact defects: no extension module, wheel, Python cache, or compiled
  Python file is present in the worker worktree.
- Focused gate failures: independent `cargo check -p rdocx-py --all-targets`
  passed. Independent `cargo test -p rdocx` passed 63 unit, 76 integration, 17
  regression, and 1 doctest. A fresh abi3 wheel build passed all 19 binding
  tests from an isolated temporary directory. The worker also records green
  workspace clippy, formatting, WASM, unit conversion, publication dry run,
  archive-size, prose, skill-drift, diff, and unchanged 28-entry hash gates.

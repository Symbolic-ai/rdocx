# F-135, all, pass 1

**Reviewed**: complete working diff from claim base `e233385`, 6 files and
546 added plus 20 removed lines, including the approved untracked parity module
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the sixteen-ID manifest is neither complete nor exact for the tagged documentation
`crates/rdocx-py/tests/test_python_docx_parity.py:7`
`crates/rdocx-py/tests/test_python_docx_parity.py:129`

The python-docx 1.2.0 Quickstart contains the three-statement row example at
`docs/user/quickstart.rst:114` in the upstream v1.2.0 tag. It obtains
`table.rows[1]`, then assigns text through `row.cells[0]` and `row.cells[1]`.
Every referenced API belongs to the completed S33 table surface, but the exact
body is absent from both the pinned ID set and the table entries at lines 66
through 100. Running that body with only the namespace changed raises
`StaleElementError` on its second write. The first cell setter bumps the global
revision at `crates/rdocx-py/src/table.rs:601`, while the held row rejects that
revision at `crates/rdocx-py/src/table.rs:423`. This is a real documented
compatibility gap, not grounds to omit an otherwise in-scope example.

The line-spacing entry also prepends `from docx.shared import Pt`, although the
tagged Line spacing section uses `Pt(18)` without that import at
`docs/user/text.rst:226`. That import belongs to Apply character formatting at
line 302 of the upstream source. All recorded page references use the mutable
`/en/latest/` route, for example line 130, so they do not identify the tagged
source that would expose this heading mismatch. The gate therefore cannot
support the HLD claim that these are every in-scope example's unchanged
statements with only an import namespace substitution.

### D2, normalized records collapse distinct relative line spacing values
`crates/rdocx-py/tests/test_python_docx_parity.py:295`
`crates/rdocx-py/tests/test_python_docx_parity.py:331`

`ParagraphFormat.line_spacing` is either a Length or a relative float, but the
normalizer sends both through `int()`. Independent records for paragraphs with
line spacing 1.5 and 1.75 were byte-for-byte equal and both recorded the value
1. A serializer regression that changed one relative spacing to another could
therefore pass both readers and the direct writer comparison. The authoring
helper uses only `Pt(18)` at line 400, so the two-way gate never exercises the
relative branch even though the pinned manifest includes 1.75 at line 132.

### D3, table style never enters the two-writer round trip
`crates/rdocx-py/tests/test_python_docx_parity.py:340`
`crates/rdocx-py/tests/test_python_docx_parity.py:416`

The record format can observe a table style, but the shared authoring helper
creates a table and sets only alignment, cell text, cell width, and vertical
alignment before saving. The sole `LightShading-Accent1` example at lines 93
through 99 observes the live rdocx handle without saving or opening the result
through either reader. A regression that drops table style while serializing
or loading therefore leaves the manifest, both reader comparisons, and direct
writer equality green. This does not prove the promised two-way preservation
of the documented table-style surface.

## Smells

None.

## Nitpicks

None.

## Not found

No additional oracle-version, distribution-name, namespace-transform,
manifest-ID uniqueness, writer-direction, reader-direction, direct-writer
equality, paragraph, run, font, indentation, pagination, table-cell, nested
paragraph, unit, enum, package-byte, XML, binary-fixture, runtime-dependency,
new-file approval, HLD-impact, WASM isolation, formatting, prose, generated
skill, diff-hygiene, hash-expectation, or artifact issue was found.

The isolated environment reported exact `python-docx==1.2.0` and rdocx 0.4.1.
Both focused parity tests passed, and the complete binding directory passed all
33 tests. Independent mutations of the oracle version, one expected manifest
ID, and the indentation value each failed the intended gate. The current
two-writer test opens each output with both readers and directly compares the
two writer records. It uses only public object-model access and commits no ZIP,
XML, document, or generated binding artifact. The binding crate and
`rdocx-wasm` checks passed, as did formatting, prose, skill sync, diff hygiene,
and artifact checks. The worker records all 28 hash entries unchanged.

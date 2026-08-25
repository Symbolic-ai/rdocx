# F-180, all aspects, pass 1

**Reviewed**: Uncommitted F-180 working tree diff, 9 files, 1,760 additions and 24 deletions
**Verdict**: 9 defects, 2 smells, 0 nitpicks

## Defects

### D1, small positive image dimensions do not round trip
`crates/rdocx/src/odt.rs:967`

The writer formats EMU dimensions as inches with only ten decimal places through
`emu_inches` at `crates/rdocx/src/odt.rs:1370`. The reader converts those inches
back through floating point and truncating `Length::pt` at
`crates/rdocx/src/odt.rs:3364`. For example, 1 EMU becomes
`0.0000010936in`, which converts to less than 1 EMU and is rejected as
non-positive at `crates/rdocx/src/odt.rs:3369`. Other positive dimensions can
return one EMU smaller. This violates the exact truncating-EMU image boundary
and can drop a supported image entirely.

### D2, numbered paragraphs inside table cells are emitted into a reader blind spot
`crates/rdocx/src/odt.rs:860`

The writer emits consecutive numbered cell paragraphs as a nested `text:list`.
The F-179 reader only visits direct `text:p` and `text:h` children of a table
cell at `crates/rdocx/src/odt.rs:2940`. It never projects a `text:list` there.
A supported numbered paragraph in a table cell is therefore silently absent
after the declared write-read gate.

### D3, a list that starts below level zero gains an extra paragraph
`crates/rdocx/src/odt.rs:1119`

When the first source item has a level greater than the current level, the
writer creates a synthetic `text:list-item` containing an empty `text:p` at
`crates/rdocx/src/odt.rs:1124`. The reader projects every such paragraph as a
numbered Word paragraph at `crates/rdocx/src/odt.rs:2550`. A valid source whose
first list paragraph is level 1 or deeper therefore reopens with extra empty
paragraphs that were not in the source.

### D4, numbering inherited from a paragraph style is ignored
`crates/rdocx/src/odt.rs:383`

The scan resolves effective paragraph properties, but list detection later
reads only the paragraph's direct `CT_PPr` through `paragraph_numbering_odt` at
`crates/rdocx/src/odt.rs:1157`. Word paragraph styles can supply `num_id` and
`num_ilvl`. Such paragraphs are written as ordinary ODT paragraphs, with no
list and no diagnostic, despite the contract to materialize effective
formatting and preserve supported lists.

### D5, vertical-merge continuation content is silently discarded
`crates/rdocx/src/odt.rs:833`

The scan walks and allocates styles and media for every continuation cell, but
the write path replaces the complete cell with covered-cell elements and
continues at `crates/rdocx/src/odt.rs:841`. Any text in that physical cell is
lost without a diagnostic. Any inline image is copied into the ZIP and
manifest but is never referenced from `content.xml`. The writer must either
reject or diagnose non-empty continuation content before omitting it.

### D6, table and cell widths are dropped without diagnostics
`crates/rdocx/src/odt.rs:1374`

`cell_has_lossy_properties` omits `CT_TcPr::width`, and
`table_has_lossy_properties` at `crates/rdocx/src/odt.rs:1384` omits
`CT_TblPr::width`. Grid-column widths are also reduced to a count without any
loss report. Even the normal `Document::add_table` path supplies table and
grid widths. The current tests consequently accept an empty diagnostic list
while those source properties are discarded, contrary to the milestone rule
that every lossy conversion names what it dropped.

### D7, unsupported inherited formatting can disappear without a diagnostic
`crates/rdocx/src/odt.rs:401`

Style XML is generated from effective run properties, but
`scan_run_losses` at `crates/rdocx/src/odt.rs:487` examines only direct run
properties. An unsupported property such as caps, character spacing, or a
theme font inherited from a paragraph or character style is omitted by
`text_style_xml` at `crates/rdocx/src/odt.rs:1207` without any diagnostic.
The same mismatch exists between effective paragraph style generation and the
direct-only paragraph loss scan. Loss reporting must inspect the effective
projection as well as direct retained properties.

### D8, CRLF source text becomes two line breaks
`crates/rdocx/src/odt.rs:1335`

`write_odf_text` maps both carriage return and line feed independently to
`text:line-break`. A programmatically created run containing CRLF therefore
reopens with two breaks, while XML normalization of the equivalent DOCX source
produces one. Exact whitespace and line-break round trip is part of the
approved fidelity boundary.

### D9, table span arithmetic can overflow before the bound check
`crates/rdocx/src/odt.rs:1402`

The row width uses unchecked `usize` summation of public `u32` grid spans.
`row_cell_column` repeats the unchecked sum at
`crates/rdocx/src/odt.rs:1493`. Two large spans can panic in debug builds or
wrap in release builds, especially on wasm32, before the 256-column rejection
runs. Untrusted or caller-built table geometry must use checked accumulation
and return `Error::Odt`.

## Smells

### S1, the diagnostic regression does not exercise the promised loss matrix
`crates/rdocx/src/odt.rs:4014`

The test named for unsupported document content covers body raw XML and two
numbering simplifications only. It does not exercise the plan's paragraph,
run, field, note, comment, bookmark, revision, content-control, table, row,
cell, drawing, and media categories or assert stable paths for them. The broad
claim can regress while this test stays green.

### S2, the atomic-save test does not induce a staging failure against an existing file
`crates/rdocx/src/odt.rs:4050`

The test makes the destination a directory. Staging succeeds, then replacement
fails because a directory occupies the destination. It does not exercise a
serialization or staging failure while preserving existing file bytes, which
is the acceptance condition stated in the approved plan.

## Nitpicks

None.

## Not found

No additional defects were found in fixed namespace prefixes, XML attribute
escaping, ODF top-level child order, ZIP entry order, ZIP reproducibility,
manifest image ordering, source-document mutation, atomic replacement logic,
public API scope, dependency edges, HLD impact scope, or tracked prose. The
focused writer tests and the public round-trip test pass, but they do not cover
the defects above.

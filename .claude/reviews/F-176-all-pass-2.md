# F-176, all, pass 2

**Reviewed**: the complete uncommitted worker implementation diff, including the untracked RTF module, across 8 files with 2,578 added lines and 0 removed lines
**Verdict**: 14 defects, 0 smells, 0 nitpicks

## Defects

### D1, the differential gate covers only two plain text paragraphs
`crates/rdocx/tests/integration_test.rs:112`

The oracle normalizer records only paragraph text and run text, font, size, and bold state. The
oracle input at line 95 consequently exercises none of the story's table, list, image, colour,
paragraph-geometry, or diagnostic scope. Reverting any of those projections leaves the named
differential gate green. No test saves the converted RTF document and reopens it through
`Document`, either, so the design plan's generated-DOCX reopen rider remains unproved.

### D2, RTF line breaks and tabs are serialized as text characters
`crates/rdocx/src/rtf.rs:1237`

Both `\line` and `\tab` are appended to `RunData.text`. Projection passes that string to
`Paragraph::add_run`, which writes one `w:t`, rather than constructing `RunContent::Break` or
`RunContent::Tab`. A save and reopen therefore does not preserve the RTF hard line break or
tab as the corresponding Word run content. The current assertions inspect only the in-memory
text projection and cannot detect the wrong OOXML shape.

### D3, positive and automatic RTF line spacing become exact spacing
`crates/rdocx/src/rtf.rs:759`

When `\slmult0` is active, every `\slN` value is passed through `saturating_abs()` to
`set_line_spacing`, whose contract emits the `exact` rule. RTF 1.9.1 defines positive `N` as
at-least spacing, negative `N` as exact spacing, and zero as automatic spacing. Inputs such as
`\sl360\slmult0` and `\sl0` are therefore converted to different paragraph formatting. The
focused test uses only negative exact spacing and multiple spacing.

### D4, malformed Unicode alternate destinations are accepted and dropped
`crates/rdocx/src/rtf.rs:935`

The parser skips the first `\upr` child and labels the second child as a generic container, but it
never requires exactly two children or requires the second child to begin with `\*\ud`. For
example, `{\rtf1{\upr{ansi}{Unicode}}}` succeeds and drops both representations instead of
rejecting the malformed Unicode destination. This leaves invalid Unicode parser state accepted
after the basic valid `\upr` case from pass 1 was fixed.

### D5, destination changes are allowed after body content in the same group
`crates/rdocx/src/rtf.rs:1097`

RTF destination changes are legal only immediately after an opening brace, but the parser has
no group-start state and unconditionally changes destination for `\fonttbl`, `\colortbl`,
`\pict`, and the other supported destinations. An input such as
`{\rtf1 body\fonttbl trailing}` changes the root group into a font-table destination and
drops or misinterprets later content. The related `starred` bit is also retained across literal
text until the next control word, so `\*` need not immediately precede the destination it marks.

### D6, valid font character sets select a wrong or unavailable decoder
`crates/rdocx/src/rtf.rs:1985`

Charset 1 is the default charset and should fall back to the document code page, but it is forced
to Windows-1252. Charsets 2 and 130 are mapped to code pages 42 and 1361, while
`encoding_for_code_page` supports neither. Valid runs using a default font under
`\ansicpg1251` decode as Windows-1252, while Symbol and Johab font runs fail as unsupported
code pages. The pass-1 legacy header tests do not cover these font-selected paths.

### D7, ungrouped font-table entries inherit the preceding entry's charset
`crates/rdocx/src/rtf.rs:1741`

RTF 1.9.1 permits font-table entries with or without a group around each entry. At a semicolon
the parser clears only `font_name`. It leaves `font_charset` and `font_code_page` in the same
state. In an ungrouped table, a following font that omits those optional controls therefore
inherits the preceding font's decoder and its runs are decoded through the wrong code page.

### D8, a picture immediately after a table is moved before the table
`crates/rdocx/src/rtf.rs:1723`

After `\row`, completed rows remain buffered until another text paragraph flushes them.
`finish_picture` adds the following picture to `current_paragraph` without flushing the table.
At end of input, `finish_document` pushes that picture paragraph before calling `flush_table`,
so `table, picture` source order becomes `picture, table`. The pass-2 picture test checks inline
and cell ownership but the existing integration input at line 51 does not assert block order for
its picture after a table.

### D9, table geometry from every row except the first is discarded silently
`crates/rdocx/src/rtf.rs:543`

Projection creates one table grid exclusively from `rows.first().boundaries`. Later rows may
carry different `\cellxN` boundaries, and even a greater cell count, but those values neither
affect widths nor produce a diagnostic. A Word table with row-specific cell geometry is thus
silently rewritten using the first row's widths, contrary to the table-formatting and lossy
diagnostic contracts.

### D10, picture goal controls without parameters are accepted
`crates/rdocx/src/rtf.rs:1322`

`\picwgoal` and `\pichgoal` assign the optional scanner parameter directly instead of requiring
one. `{\rtf1{\pict\pngblip\picwgoal <valid PNG hex>}}` is accepted and silently falls
back to the probed native width. These are numeric picture controls in the supported subset, so
a missing parameter is malformed input and must not become an implicit default.

### D11, out-of-range list levels are silently changed to level 8
`crates/rdocx/src/rtf.rs:1232`

The parser clamps every `\ilvlN` above 8 with `min(8)`. An invalid or unsupported level is
therefore attached to a different list level without an error or diagnostic. This recreates the
silent list fallback class from pass 1 for paragraph-level list state.

### D12, the input limit still permits unbounded parser-output amplification
`crates/rdocx/src/rtf.rs:1406`

There is no bound on diagnostics, paragraphs, runs, or total projected cells. A sub-64 MiB
stream of short unknown controls can allocate tens of millions of owned diagnostic strings, and
a stream of `\par` controls can allocate millions of `ParagraphData` blocks. A table within the
declared 10,000 by 256 limits can likewise project 2.56 million cells. These paths amplify a
bounded input into gigabytes of heap state and can abort on allocation, so the hostile-input
denial of service remains even though pass 1's group-state buffer cloning was removed.

### D13, undefined font and colour references silently lose formatting
`crates/rdocx/src/rtf.rs:783`

Projection uses chained `and_then` lookups for font, foreground colour, and highlight colour,
then simply omits the property when an index is absent. Valid references are applied, but
`\f999`, `\cf999`, or `\highlight999` succeed with different formatting and no diagnostic.
Undefined list references are rejected, so the other bounded lookup tables should not silently
accept the same malformed-reference class.

### D14, a control symbol can split a UTF-16 surrogate pair
`crates/rdocx/src/rtf.rs:1586`

`append_unicode` flushes pending bytes but does not reject a pending high surrogate before
appending a control-symbol character. After its fallback byte is skipped, a sequence such as
`\u-10179?\~\u-8704?` is accepted and emits a nonbreaking space before the eventual emoji.
Literal and group-boundary paths reject the same interrupted surrogate state, so control
symbols currently bypass the malformed-Unicode check.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 default-font restoration, legacy `\mac`, `\pc`, and `\pca` decoding, malformed-minus
  rejection, and root-marker placement are fixed.
- List override starts, unsupported list-format diagnostics, missing list references, and list
  projection inside table cells are fixed apart from D11.
- Inline picture ownership, scaling, crop diagnostics, path preflight, and the specific nested
  state-buffer amplification from pass 1 are fixed apart from D8 and D12.
- Direct indexing and arithmetic panics were not found. The hostile allocation failure is D12.
- Public API shape and repository structure produced no additional finding. The new private
  module was approved, no trait, generic, feature flag, or crate was introduced, and dependency
  direction remains valid.
- Existing OOXML parser preservation and schema child ordering were not changed by this diff.

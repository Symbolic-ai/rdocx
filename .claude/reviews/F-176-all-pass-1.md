# F-176, all, pass 1

**Reviewed**: the complete uncommitted worker diff, including the untracked RTF module, across 7 files with 1,893 added lines and 0 removed lines
**Verdict**: 14 defects, 0 smells, 0 nitpicks

## Defects

### D1, the declared differential gate never compares against Microsoft Word
`crates/rdocx/tests/integration_test.rs:95`

The checked test parses one local RTF string and compares it with local constants. It does not
consume a Word-produced DOCX or a normalized expected record from the pinned oracle. The
ignored regeneration test at line 129 only compares an environment variable with a string.
A reader that returns the asserted local values while disagreeing with Word passes the story's
named test gate, so the approved differential contract and oracle-pinning rider are not proved.

### D2, the default RTF font is discarded
`crates/rdocx/src/rtf.rs:982`

The parser explicitly ignores `\deffN`, and `\plain` at line 903 resets the run to a format with
no font. A normal Word stream such as
`{\rtf1\ansi\deff0{\fonttbl{\f0 Calibri;}}\plain text}` therefore produces a run whose font
is `None` instead of `Calibri`. It also decodes that run through the document code page rather
than the default font's code page. The existing tests avoid the failure by issuing `\f0` before
all asserted text.

### D3, three accepted document character sets cannot be decoded
`crates/rdocx/src/rtf.rs:815`

The `\mac`, `\pc`, and `\pca` controls select code pages 10000, 437, and 850, but
`encoding_for_code_page` at line 1618 maps none of them. Valid text under any of those
headers fails as an unsupported code page after the parser has accepted the declaration.
The font charset table also maps charset 255 to 437 at line 1613, while RTF 1.9.1 defines
charset 254 as PC 437 and charset 255 as OEM 850.

### D4, Unicode alternate destinations select the ANSI branch
`crates/rdocx/src/rtf.rs:799`

Neither `\upr` nor `\ud` is a supported destination. The parser treats `\upr` as an unknown
ordinary control, appends its ANSI child to the body, then treats the starred `\ud` child as an
unsupported destination and skips it. A Unicode-aware reader is required to select the `\ud`
representation. Valid RTF with an ANSI fallback can therefore return the fallback text rather
than the Unicode text.

### D5, malformed optional numeric parameters become document text
`crates/rdocx/src/rtf.rs:165`

When a minus sign is not followed by a digit, the scanner rewinds to the minus sign and reports
that the control has no parameter. A stream containing `\b-foo` is accepted as bold text
`-foo` instead of being rejected as a malformed numeric control. This contradicts the approved
plan's explicit malformed-control rejection requirement.

### D6, the RTF version marker is accepted anywhere in the group tree
`crates/rdocx/src/rtf.rs:651`

Final validation checks only that some consumed control set `seen_rtf`. It does not require
`\rtf1` immediately after the root opening brace. Input such as
`{\ansi body{\rtf1}}` is accepted and projected even though it is not an RTF file under the
declared grammar. A nested or late marker can therefore turn arbitrary grouped control text
into accepted input.

### D7, core Word paragraph formatting is outside the implemented model
`crates/rdocx/src/rtf.rs:262`

`ParagraphFormat` contains only alignment and list identity. Word-emitted controls for left,
right, first-line, and hanging indents, space before and after, and line spacing all fall through
as unsupported controls even though the facade already exposes the corresponding setters.
The story promises the Word-written formatting subset and cites Paragraph Formatting
Properties, so common formatted RTF is converted without the paragraph formatting in scope.

### D8, table cell boundaries are reduced to a count
`crates/rdocx/src/rtf.rs:927`

The `\cellxN` handler increments `expected_cells` but discards every boundary value. A row
whose cells have unequal Word-authored widths is projected as the facade's default table with
no corresponding column widths. The integration test checks only cell text, so reverting or
omitting table geometry cannot fail the stated structural gate.

### D9, numbering on paragraphs inside table cells is silently removed
`crates/rdocx/src/rtf.rs:449`

Table projection passes `None` as the list definition to `apply_paragraph_format` for every
cell paragraph. A valid `\lsN\ilvlN` paragraph inside a table is parsed with numbering state
but emitted as an ordinary paragraph. No diagnostic reports the loss, despite the plan
requiring tables, lists, and a diagnostic for every lossy conversion.

### D10, unsupported list formats and start overrides silently become different lists
`crates/rdocx/src/rtf.rs:1645`

Every `\levelnfcN` value outside the seven locally modeled formats is coerced to decimal.
Controls such as `\listoverridestartat` are also explicitly ignored at line 1022. Cardinal-text,
ordinal-text, East Asian, and per-override starts therefore produce different numbering with no
diagnostic. Missing list or override references similarly synthesize a decimal list at line 510
instead of rejecting or reporting the malformed reference.

### D11, pictures are reordered and cannot remain in their source container
`crates/rdocx/src/rtf.rs:1506`

Finishing a picture pushes a top-level `Block::Picture` without first committing the current
paragraph or attaching the picture to the current table cell. For
`before{\pict ...}after\par`, the picture block is emitted before the later paragraph containing
both text fragments. A picture inside a cell is likewise emitted as a body picture before the
eventual table. This violates the approved source-order and typed ownership-tree contract.

### D12, Word picture scaling and cropping are silently ignored
`crates/rdocx/src/rtf.rs:1023`

The parser recognizes `\picscalex`, `\picscaley`, and all four crop controls only to discard
them. A scaled or cropped Word picture is emitted at an unscaled size with its full pixels, and
no diagnostic names the dropped transform. This violates the end-of-milestone diagnostic
gate even if those picture properties remain outside the supported projection.

### D13, the path API allocates an unbounded file before enforcing the input limit
`crates/rdocx/src/rtf.rs:39`

`open_rtf` uses `std::fs::read` and only calls the bounded byte parser after the entire file has
been allocated. A hostile multi-gigabyte file can exhaust memory before the 64 MiB limit is
checked. The public path entry point therefore does not preserve the bounded-parser claim.

### D14, nested groups multiply the largest parser buffers
`crates/rdocx/src/rtf.rs:324`

Every opening group clones the complete `State`, including accumulated picture bytes,
font-name bytes, and list-marker bytes. An input below the 64 MiB file limit can place tens of
megabytes of picture hex before a deeply nested group sequence. Each level retains another
copy up to depth 256, causing several gigabytes of allocation while every declared size limit
still passes. This is a hostile-input denial of service in the bounded scanner.

## Smells

None.

## Nitpicks

None.

## Not found

- Public API shape: no unrelated API expansion or binding-surface change was found.
- Direct panics: no reachable `unwrap`, `expect`, indexing, slicing, or arithmetic panic was
  found beyond the hostile allocation defects above.
- OOXML preservation and ordering: no edit to an existing OOXML parser or serializer was
  present in this diff. Generated DOCX reopen coverage is missing as described in D1.
- Repository structure: the new private module was explicitly approved, no new trait or crate
  was introduced, and the dependency direction remains valid.

# F-176, all, pass 4

**Reviewed**: the complete uncommitted worker implementation diff, including the untracked RTF module, across 8 files with 3,347 added lines and 0 removed lines
**Verdict**: 7 defects, 0 smells, 0 nitpicks

## Defects

### D1, Symbol-font bytes are decoded as Windows-1252
`crates/rdocx/src/rtf.rs:2387`

RTF charset 2 selects code page 42, but the decoder aliases code page 42 to
Windows-1252. Those are not the same character mapping. For example, a byte in
a `\fcharset2` Symbol run is returned as its Windows-1252 character instead of
the corresponding Symbol character. The pass-2 regression only uses ASCII
`a`, which cannot distinguish the two mappings, so valid Word Symbol runs still
silently produce wrong text.

### D2, character-set declarations are still accepted after header tables
`crates/rdocx/src/rtf.rs:2260`

The header check rejects a character-set control only after `body_started`, but
opening and parsing a font, colour, or list table does not set that flag. Input
such as `{@rtf1{@fonttbl{@f0 Arial;}}@ansicpg1251 text}`, with each `@` replaced
by a backslash, is therefore accepted even though the cited header grammar
requires the character set and code page before any table control word. This is
the table-position branch of pass 3 D9 and remains unverified by the regression,
which checks only text, row, and nested-group positions.

### D3, a cell boundary outside a table row is accepted
`crates/rdocx/src/rtf.rs:1399`

The `\cellxN` arm updates the pending boundary list without checking
`in_table`. `{@rtf1@cellx1440 text}`, with `@` replaced by a backslash, succeeds
as an ordinary paragraph and silently discards the stray boundary. The cited
RTF grammar calls this strongly illegal table data, and the approved plan
requires malformed controls to be rejected.

### D4, the valid no-list value fails as an undefined override
`crates/rdocx/src/rtf.rs:1374`

RTF defines paragraph `\ls0` as no list. The parser stores every `\lsN`,
including zero, as an override reference. Projection then looks up override 0
and returns `RTF list override 0 is undefined`. A Word stream that explicitly
clears numbering with `\ls0` therefore fails instead of producing an
unnumbered paragraph.

### D5, a binary control can interrupt a Unicode surrogate pair
`crates/rdocx/src/rtf.rs:230`

The scanner converts `\binN` directly into a `Binary` token, so it bypasses the
non-`u` control-word surrogate guard. For example,
`{@rtf1@uc0@u-10179@bin1X@u-8704}`, with `@` replaced by a backslash, is
accepted and emits the emoji while merely diagnosing the binary payload. A
binary control between the UTF-16 halves is malformed in the same way as the
control words fixed after pass 3, so that earlier defect is not fully closed.

### D6, visible document formatting is dropped without diagnostics
`crates/rdocx/src/rtf.rs:1569`

The explicit no-op arm silently consumes `\paperw`, `\paperh`, all four page
margins, and `\gutter`. These controls change the converted document's page
geometry, but the result neither represents them nor reports what was dropped.
This violates the milestone gate requiring every lossy conversion to name its
loss, independently of whether page geometry is added to the supported
projection.

### D7, the differential record omits much of the implemented formatting surface
`crates/rdocx/tests/integration_test.rs:98`

The pinned-oracle run record compares only font, size, bold, colour, and image
presence. It omits italic, underline, strike, highlight, caps, small caps,
hidden text, vertical position, breaks, and tabs. The paragraph record likewise
omits right and first-line indents, space before, and every line-spacing mode.
Reverting any of those implemented projections leaves the named differential
gate green. Focused Rust assertions against local expectations do not replace
the approved structural comparison against the pinned Word conversion.

## Smells

None.

## Nitpicks

None.

## Not found

- Passes 1 through 3 picture ownership and ordering, picture transform
  diagnostics, row-cardinality checks, list-property diagnostics, override
  finalization, and ordinary control-word and control-symbol surrogate checks
  are fixed apart from D5.
- Input, group, lookup, diagnostic, block, run, cell, picture, and retained-byte
  bounds produced no additional finding.
- No reachable indexing, slicing, arithmetic, `unwrap`, `expect`, or explicit
  panic defect was found.
- The generated DOCX is reopened and compared through the same normalizer. Its
  incomplete field coverage is D7.
- Public API shape and repository structure produced no finding. The private
  module was approved, no trait, generic, feature flag, or crate was introduced,
  and dependency direction remains valid.
- Existing OOXML preservation and schema child ordering were not changed by
  this diff.

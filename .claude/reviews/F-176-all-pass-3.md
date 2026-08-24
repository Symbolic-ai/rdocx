# F-176, all, pass 3

**Reviewed**: the complete uncommitted worker implementation diff, including the untracked RTF module, across 8 files with 3,148 added lines and 0 removed lines
**Verdict**: 9 defects, 0 smells, 0 nitpicks

## Defects

### D1, conforming list override levels are discarded
`crates/rdocx/src/rtf.rs:1337`

The parser snapshots and inserts an override when it encounters `\lsN`. RTF 1.9.1 places the optional `\lfolevel` after `\lsN`, so the later level data only changes `active_override_levels` and never reaches the stored override. A conforming start-at or format override therefore projects the base list instead. The focused test avoids this by placing `\lfolevel` before `\lsN`, contrary to the cited grammar.

### D2, Word special-character controls lose the character
`crates/rdocx/src/rtf.rs:1531`

Text controls such as `\emdash`, `\endash`, `\bullet`, `\lquote`, `\rquote`, `\ldblquote`, and `\rdblquote` fall into the unsupported-control arm. The reader emits a diagnostic but appends no character. These are defined RTF text controls written by Word, so input such as `one\emdash two` becomes `onetwo` instead of preserving the scoped text.

### D3, a control word can still split a UTF-16 surrogate pair
`crates/rdocx/src/rtf.rs:1147`

The pass-2 guard covers control symbols only. `control_word` does not reject a pending high surrogate before handling a non-`u` word. After its fallback is skipped, `\u-10179?\b\u-8704?` is accepted and emits the emoji with formatting changed between its UTF-16 halves. The same malformed pair interrupted by text or a control symbol is rejected.

### D4, malformed table rows are padded or projected with unmatched cells
`crates/rdocx/src/rtf.rs:1862`

RTF requires the number of `\cellx` definitions to match the number of `\cell` terminators in a row. The parser pads a short row with empty cells and does not reject a row with more cells than boundaries. Malformed input is therefore accepted, and the latter case produces default geometry for cells with no declared boundary.

### D5, malformed and unknown font charset controls silently select another decoder
`crates/rdocx/src/rtf.rs:1256`

`\fcharset` stores its optional parameter without requiring one. At font completion, both a missing parameter and an unmapped value such as `\fcharset999` become `None`, which makes runs fall back to the document code page. A malformed or unsupported font declaration therefore succeeds with incorrectly decoded text instead of returning the promised RTF error.

### D6, lossy list properties are explicitly ignored without diagnostics
`crates/rdocx/src/rtf.rs:1512`

The no-op arm includes `\leveljc`, `\levelfollow`, `\levelspace`, and `\levelindent`. These affect visible list alignment, suffix spacing, and indentation, but the projected list model drops them without any diagnostic. This violates the required diagnostic for every lossy conversion even if those properties remain outside the supported projection.

### D7, the newer list-format control does not take priority
`crates/rdocx/src/rtf.rs:1407`

RTF 1.9.1 requires `\levelnfcnN` to take priority over `\levelnfcN` when both occur. The parser handles both identically and lets whichever control appears last overwrite the level. A level containing `\levelnfcn23\levelnfc0` is therefore converted from a bullet to decimal numbering.

### D8, Unicode list markers are decoded a second time through the ANSI code page
`crates/rdocx/src/rtf.rs:1740`

Unicode text in `\listtext` or `\pntext` is appended as UTF-8 bytes to `active_list_marker`. `finish_list_marker` later decodes that buffer as the current ANSI code page. A marker such as `\u8226?` becomes mojibake and is inferred as a decimal list instead of a bullet list.

### D9, document character-set declarations are accepted after body text
`crates/rdocx/src/rtf.rs:1191`

The RTF header grammar requires `\ansi`, `\mac`, `\pc`, `\pca`, and `\ansicpgN` before plain text or table controls. The parser permits them in any body group. Malformed input can therefore switch decoding midway through body text instead of being rejected by the bounded grammar.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 default-font restoration, legacy header decoding, malformed-minus rejection, root-marker placement, paragraph formatting, table-cell list projection, picture ownership and ordering, picture scaling and crop diagnostics, path preflight, and group-state buffer bounds are fixed.
- Pass-2 typed break and tab projection, line-spacing rules, strict Unicode alternate destinations, destination placement, ungrouped font reset, row-boundary diagnostics, required picture parameters, list-level bounds, missing lookup-reference rejection, and output collection bounds are fixed apart from the defects above.
- The expanded differential fixture covers body order, paragraphs, runs, colours, lists, tables, images, diagnostics, and generated-DOCX reopen. No additional oracle or reopen defect was found beyond its invalid list-override ordering noted in D1.
- No reachable indexing, slicing, arithmetic, `unwrap`, `expect`, or explicit panic defect was found.
- Public API shape and repository structure produced no finding. The private module was approved, no trait, generic, feature flag, or crate was introduced, and dependency direction remains valid.
- Existing OOXML preservation and schema child ordering were not changed by this diff.

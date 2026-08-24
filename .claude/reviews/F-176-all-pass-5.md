# F-176, all, pass 5

**Reviewed**: the complete worker diff from base `d73dc2b`, including the untracked RTF module, across 8 files with 3,665 added lines and 0 removed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, visible formatting controls are still silently dropped
`crates/rdocx/src/rtf.rs:1591`

The explicit no-op arm still consumes Word-visible controls such as `\widowctrl`, `\nowidctlpar`, `\hyphauto`, `\headery`, `\footery`, and `\endnhere` without either projecting them or adding a diagnostic. An input such as `{\rtf1\hyphauto text}` therefore returns an apparently lossless conversion even though automatic hyphenation was dropped. This violates the M18 gate and the design plan requirement that every safe lossy skip produce a stable diagnostic naming the loss.

### D2, EOF picture-only paragraphs lose their paragraph formatting
`crates/rdocx/src/rtf.rs:2194`

`finish_picture` appends the picture item but never copies the current paragraph state onto `current_paragraph`. That is only repaired when a later text, break, tab, or explicit `\par` path runs. A valid final picture paragraph such as `{\rtf1\qc{\pict\pngblip ...}}` reaches EOF with content, then is pushed by `finish_document` with the default paragraph format instead of centered alignment. Picture paragraphs are part of the approved image and formatting scope, so this drops supported formatting without a diagnostic.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-4 D1 through D5 are fixed: Symbol bytes use a dedicated decoder, root character-set declarations fail after header tables, stray cell boundaries fail, `\ls0` clears numbering, and binary controls cannot interrupt surrogate pairs.
- Pass-4 D6 is fixed for the listed page size, margin, and gutter controls. D1 above is the remaining silent-drop class for other visible controls in the same no-op arm.
- Pass-4 D7 is materially fixed: the pinned structure now covers body order, run and paragraph formatting, tables, lists, images, diagnostics, generated-DOCX reopen, and XML markers for break, tab, caps, small caps, and hidden text.
- Public API shape is additive and matches the design. No unrelated binding or facade expansion was found.
- Parser bounds now cover input bytes, group depth, lookup tables, diagnostics, blocks, runs, table cells, picture bytes, and retained output bytes. No additional panic, indexing, slicing, or arithmetic overflow defect was found.
- Existing OOXML parser preservation and schema child ordering were not changed by this diff.
- Repository structure produced no additional finding. The private module was approved, no new trait, generic parameter, feature flag, or crate was introduced, and dependency direction remains valid.

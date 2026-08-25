# F-180, all aspects, recovery pass 3

**Reviewed**: Entire uncommitted F-180 implementation diff, 9 files, 6,690 additions and 2,026 deletions, plus all original and recovery reviews, the reviewer correction, and the complete progress record
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, exact automatic line-height percentages can still reopen one twip low
`crates/rdocx/src/odt.rs:2025`

The recovery rounds upward only while converting the exact rational percentage
to ten decimal places. Some exact terminating percentages still evaluate below
their mathematical value when the reader divides the parsed `f64` by 100 and
the paragraph setter multiplies by 240 at
`crates/rdocx/src/odt.rs:4095`. A retained automatic line height of 123 twips
is emitted as `51.25%`, then reopens as 122 twips because the intermediate
floating-point product is approximately `122.99999999999999` before the pinned
truncation. The same defect occurs at 246, 492, 507, and 132 other values in
the accepted 1 through 24,000 range. The current boundary regression covers
2, 119, 359, and 24,000, so it remains green.

### D2, a drawing with no inline or anchor payload disappears silently
`crates/rdocx/src/odt.rs:833`

`scan_drawing` returns success when both `CT_Drawing::inline` and
`CT_Drawing::anchor` are absent. The write path then emits nothing. This state
is reachable from a retained malformed drawing or through the public OOXML
model, and it receives no path-aware diagnostic. The approved contract names
malformed pictures among the items that must be diagnosed and omitted, and the
HLD promises a diagnostic for unsupported Word content.

### D3, distributed paragraph alignment is simplified without a diagnostic
`crates/rdocx/src/odt.rs:1810`

The paragraph projection maps both `ST_Jc::Both` and `ST_Jc::Distribute` to the
same ODF `justify` value. `Distribute` is a retained parsed OOXML value at
`crates/rdocx-oxml/src/shared.rs:27`, but the unsupported-property scan at
`crates/rdocx/src/odt.rs:2053` does not classify this simplification as loss.
The result reopens as ordinary justification with no diagnostic, contrary to
the effective-formatting and complete lossy-diagnostic contract.

## Smells

None.

## Nitpicks

None.

## Recovery findings verified

- Recovery pass-2 D2 is fixed. Inclusive positive paragraph margins and
  indents, the negative first-line boundary, and exact line height use a
  boundary-safe point representation. D1 above is limited to automatic
  percentage line height.
- Recovery pass-2 D3 is fixed. The inclusive 12,700,000,000-EMU image boundary
  reopens exactly, while the next EMU is rejected before package creation.
- Recovery pass-2 D4 is fixed. Numeric `HeadingN` style IDs are accepted only
  for 1 through 9, including parse-overflowing suffixes.
- Recovery pass-2 D5 is fixed. Every synthesized empty paragraph in an emitted
  non-continuation table cell is charged against the reader's block ceiling.
- Recovery pass-2 D6 is fixed. Selected font names that XML attribute
  normalization or reader trimming would change are rejected before output.
- Recovery pass-2 D7 is fixed. Explicit `baseline` and malformed retained
  vertical alignment receive the stable `rPr/vertAlign` diagnostic.
- Recovery pass-2 D8 is fixed in the required direction. The F-179 backlog
  entry now describes the current private two-way facade boundary rather than
  retaining the stale one-way statement. The withdrawn table-name finding did
  not result in a source or test change.
- All earlier original and recovery remediations remain present for numbering
  cancellation and level fallback, relationship type and target mode, inline
  anchoring, image and font ceilings, document stories, output block, row,
  cell, run, and XML-node ceilings, list continuation, Unicode whitespace,
  table-span validation, diagnostics, atomic save, and the approved public API.

## Not found

- **Packaging and determinism**: no defect was found in the stored first
  `mimetype`, local-header extras, fixed ZIP metadata, content and manifest
  order, MIME agreement, media encounter order, repeated-write bytes, XML
  prefixes, or ODF child order.
- **Tables, lists, and media**: no additional defect was found in nested-list
  structure, supported list kind and level, horizontal or vertical spans,
  covered-cell placement, cell paragraphs, supported image bytes, exact image
  dimensions, or malformed span rejection.
- **Bounds and panics**: no additional reachable panic or unchecked arithmetic
  was found in table geometry, media resolution, ZIP construction, style
  allocation, or generated XML validation. Writer `expect` sites rely on
  immutable facts established by the completed scan.
- **Diagnostics and ownership**: apart from D2 and D3, no additional silent
  unsupported-property or source-content loss was found. Serialization does
  not mutate the source package or retained XML, and failed staging preserves
  the destination.
- **Tests**: the source-built public round-trip gate covers body order,
  effective paragraph and run formatting, per-level list kind, cell paragraph
  structure, spans, image bytes, and dimensions. All 27 focused writer unit
  tests pass. D1 identifies the remaining unrepresented arithmetic class.
- **API and structure**: the only additive native surface is
  `OdtWriteResult`, `Document::to_odt_bytes`, and `Document::save_odt`. No new
  crate, module, source file, dependency, trait, generic, feature, wrapper-only
  abstraction, Python, WASM, or CLI surface was added.
- **HLD and file scope**: the six modified HLD files exactly match the approved
  impact list and describe current behavior. There are no `rdocx-py` edits or
  other out-of-scope tracked files in the implementation diff.
- **Verification evidence**: `cargo test -p rdocx odt_writer_ --lib` passes all
  27 selected tests. `git diff --check` and the existing tracked review prose
  check pass before this review artifact is added.

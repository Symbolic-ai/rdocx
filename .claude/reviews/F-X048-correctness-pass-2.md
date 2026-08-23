# F-X048, correctness, pass 2

**Reviewed**: working-tree diff against exact claim base
`fa3dacad97a58de7faf317eedc294f25bf95dfd9`, 15 files and 2,494 changed
lines, with 2,251 additions and 243 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, a conditional region mutation returns the preserved old attribute

`crates/rdocx-oxml/src/styles.rs:820`

`CT_TblStylePr::region` is a public typed projection of `w:tblStylePr/@w:type`,
but the preserved-subtree fast path compares only the paragraph, table, and
cell property projections. A caller that changes only `region` therefore
receives `raw_xml` at line 825 with the original region. The canonical writer
at line 834 is reached only when another typed property also changes. This
violates the package-integrity contract that modeled table-style attributes
round trip through typed mutations.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass 1 D1 through D7**: exact-row content is clipped, the terminal merge
  edge is painted, merge-only minimum rows grow at the final eligible row,
  conditional shading and `cnfStyle` participate in the cascade, direct mark
  metrics are not replaced by the legacy line, character-relative cell
  anchors include paragraph indent, and conditional property mutations are
  serialized.
- **Pass 1 S1**: the explicit non-drop handle release and obfuscated test
  conditionals are removed. The recorded strict affected-crate Clippy run is
  green.
- **Correctness and contract**: no additional wrong logic, unhandled case, or
  plan expansion was found in the bounded diff.
- **Panics**: no new panic path on document-controlled input was found.
- **OOXML**: no additional expanded-name, schema-order, or unmodelled-child
  preservation defect was found.
- **Tests**: the focused regressions cover recursive cell blocks, exact merge
  spans, clipping, conditional styles, anchors, outer `nil`, mark inheritance,
  transactional cache state, provenance, and the deterministic one-page
  golden. Microsoft Word 16.104 is unavailable on this host, so the external
  observation remains honestly unperformed and no headless result is treated
  as Word evidence.
- **Structure and semantics**: recursive cache accounting and provenance walk
  the complete nested payload. Nested tables retain the Table, TR, TH or TD,
  paragraph, and Figure ownership hierarchy used by the tagged-PDF path.
- **Hash, HLD, API, WASM, and packaging**: the 49-entry baseline differs from
  the exact claim base only at `feature_showcase:pdf/bytes` and
  `feature_showcase:pdf/pages`, plus its review reason. The four planned HLD
  files describe the implementation. Recorded evidence covers both WASM
  targets and verified sub-10 MiB packages for `rdocx-oxml`, `rdocx-layout`,
  and `rdocx`.
- **Uncited findings**: none.

# F-145, all, pass 2

**Reviewed**: uncommitted `work/f-145-codex` implementation, 3 files and 343
changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, field-only titles are still emitted twice

`crates/rpptx-cli/src/commands.rs:399`

The replacement identity check can recognize the same title shape only through
a direct default-run-property address, a non-empty regular-run text address, or
a direct regular-run-property address. A valid title paragraph containing only
an `a:fld` has none of those exposed values. `slide.title()` still supplies its
visible field text for the heading, but `same_shape_identity` returns false and
the same paragraph is emitted again as a bullet. Empty regular-run-only titles
have the same identity gap, although their empty bullet happens to be omitted.
Use a total identity that also covers field-only and property-free text bodies,
and add a field-only title regression that asserts the visible title appears
once.

## Smells

None.

## Nitpicks

None.

## Pass 1 re-evaluation

- **D1, collapsed placeholder-index title suppression**: partially resolved.
  The crafted unindexed body before an unindexed regular-run title is now
  correct, but the borrowed-address identity is not total as described above.
- **D2, DrawingML line-break control output**: resolved. Carriage returns,
  line feeds, and U+000B are normalized to printable spaces for both titles and
  paragraph bullets. The strengthened exact-output gate covers the U+000B
  case and retains paragraph-level indentation.

## Not found

- **Thumbnail size and aspect ratio**: derived DPI yields exactly 320 pixels in
  width under the rasterizer's ceiling rule. The nonstandard portrait case
  proves proportional height rather than stretching.
- **Thumbnail resources and errors**: zero slides and invalid rendered width
  fail before output. Derived dimensions pass through the checked 8,000,000-
  pixel budget before raster allocation, and raster failure propagates.
- **Thumbnail paths and determinism**: omitted output uses the shared helper,
  explicit output wins, and rendering uses the deterministic facade path.
- **Outline traversal**: apart from incomplete title identity, ordinary shape
  paragraphs, row-major unspanned table cells, recursive group children, and
  selected alternate-content fallback children retain document order. Empty
  paragraphs are omitted and levels control two-space indentation.
- **Tests and sensitivity**: all 13 CLI integration tests passed with the
  verified 50-deck corpus, and clippy passed with warnings denied. Replacing
  borrowed identity with collapsed index comparison and removing U+000B
  normalization each failed the strengthened gate before recorded
  byte-identical restoration. No current fixture uses a field-only title.
- **Contract, OOXML, and structure**: both commands remain within the existing
  source and integration files and use public facade accessors. No dependency,
  feature, module, test binary, raw production XML access, schema mutation, or
  public facade API was added. Every F-144 command remains dispatched.
- **Hygiene**: prose validation, generated-skill drift, and `git diff --check`
  passed during review.

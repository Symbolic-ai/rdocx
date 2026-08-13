# F-145, all, pass 1

**Reviewed**: uncommitted `work/f-145-codex` implementation, 3 files and 275
changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, outline can skip body text and duplicate the title

`crates/rpptx-cli/src/commands.rs:351`

Title suppression compares only `placeholder_idx()`. That facade accessor maps
an absent `idx` to zero, so an unindexed body placeholder and an unindexed title
placeholder both report `Some(0)` even though their placeholder types make them
distinct. If the body precedes the title in z-order, outline skips the body as
the first apparent match, then emits the actual title again as a bullet. This
is valid PresentationML because absent indices match by placeholder type and do
not violate the duplicate-index invariant. Suppress the exact title shape
identity rather than its collapsed index, or carry enough public identity to
distinguish the title. Add a regression with an unindexed body before an
unindexed title and assert both retained body text and a single title.

### D2, outline emits DrawingML line-break control characters

`crates/rpptx-cli/src/commands.rs:393`

`TextParagraphRef::text()` represents an `a:br` as U+000B, but the outline path
trims and prints that string unchanged. A single paragraph such as `First`,
break, `Second` therefore places a vertical-tab control character inside the
bullet instead of producing stable textual outline content. The title path
already normalizes the same character. Normalize paragraph line breaks to a
defined printable form while retaining one paragraph-level bullet, and add a
regression that checks the exact output bytes and indentation.

## Smells

None.

## Nitpicks

None.

## Not found

- **Thumbnail size and aspect ratio**: DPI is derived from the rendered slide
  width, and the rasterizer's ceiling rule yields exactly 320 pixels. The
  portrait regression proves proportional height rather than stretching.
- **Thumbnail resources and errors**: zero slides and invalid rendered width
  fail before output. Derived dimensions pass through the existing checked
  8,000,000-pixel budget before raster allocation, and raster failure propagates
  without a success message.
- **Thumbnail paths and determinism**: omitted output uses the shared helper,
  explicit output wins, and rendering uses the deterministic facade path.
- **Outline traversal**: apart from D1 and D2, ordinary shape paragraphs,
  row-major unspanned table cells, recursive group children, and selected
  alternate-content fallback children retain document order. Empty paragraphs
  are omitted and paragraph level controls two-space indentation.
- **Contract and scope**: thumbnail and outline are added to the existing
  command and integration files. No new file, module, dependency, feature, raw
  PresentationML access in production, or public facade API was introduced.
  Every F-144 command remains dispatched.
- **Tests and sensitivity**: all 13 CLI integration tests passed with the
  verified 50-deck corpus. Clippy passed with warnings denied. The recorded
  320-to-321 width mutation and indentation-removal mutation are relevant and
  independently sensitive, but the current outline fixtures do not exercise
  either defect above.
- **OOXML and preservation**: production code is read-only and uses facade
  accessors. It performs no package mutation, schema-order change, or raw XML
  interpretation.
- **Hygiene**: prose validation, generated-skill drift, and `git diff --check`
  passed during review.

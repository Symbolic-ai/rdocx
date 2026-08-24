# F-183, all aspects, pass 2

**Reviewed**: cumulative working tree diff from `a6d98f4`, 21 files, 1765 insertions and 126 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Word CLI default image selection is based on the wrong layout

`crates/rdocx-cli/src/commands.rs:163`
`crates/rdocx-cli/src/commands.rs:166`
`crates/rdocx-cli/src/commands.rs:346`
`crates/rdocx-cli/src/commands.rs:355`
`crates/rdocx-cli/src/commands.rs:427`
`crates/rdocx-cli/Cargo.toml:28`

The Word CLI image paths call `selected_zero_based_pages` before rendering.
That helper counts pages with `doc.layout()`, which is the normal layout path.
The same commands then render with `render_pages_deterministic`. The binary
enables `rdocx/system-fonts`, so the normal layout can use installed system
fonts while the deterministic renderer uses bundled fonts only. A DOCX near a
page boundary can therefore paginate differently between selection and render.
With no `--pages` argument, `rdocx render input.docx --format png` and
`rdocx convert input.docx --to png` can omit deterministic pages when the
normal layout has fewer pages, or fail when the normal layout has more pages.
That regresses the legacy deterministic all-page output contract and makes the
new default image range depend on ambient system fonts.

## Smells

None.

## Nitpicks

None.

## Checks

- `git diff --check a6d98f4` passed.
- `python3 scripts/prose_check.py .claude/reviews/F-183-all-pass-1.md .claude/plans/F-183-design.md` passed.
- `python3 scripts/sync_agent_skills.py --check` passed.
- Focused Cargo tests were not run in this review pass. The sandbox rejected
  running them in the worker worktree because Cargo would write build artifacts
  during an audit-only task.

## Not found

- D1 from pass 1 was not found. The central raster regression now decodes PNG,
  JPEG and TIFF for selection `[2, 0]`, checks two outputs, dimensions and
  center pixels, and checks the TIFF directory count.
- D2 from pass 1 was not found. Both CLIs render PNG and JPEG output with a
  one-page helper inside the selected-page loop rather than retaining one
  `Vec<Vec<u8>>` of every encoded page.
- S1 from pass 1 was not found. Both CLIs preflight multi-file paths, stage
  bytes through the shared `StagedOutputSet`, reject duplicate and pre-existing
  targets, remove temporary files on success and failure, and roll back
  published files when a later publication fails.
- Shared raster API and backend behavior were otherwise not found defective.
  The API is additive, validates DPI, empty selections, duplicates,
  out-of-range pages and JPEG quality before encoding, preserves caller order,
  keeps old opaque PNG wrappers byte-identical, composites JPEG and TIFF on
  white, and keeps transparent PNG pixels clear unless an authored page
  background paints them.
- Native and Python APIs were otherwise not found defective. Native selected
  pages are zero-based, revision-view variants are present, Python uses
  keyword-only image options, releases the GIL for render work, returns
  `list[bytes]` for separate images, returns `bytes` for TIFF, and maps raster
  errors through the existing binding error path.
- Presentation CLI behavior was not found defective. Image ranges use the
  shared one-based parser, TIFF stays one stream, PNG and JPEG are staged
  one selected slide at a time, the 320-pixel thumbnail remains fixed PNG, and
  the direct `rpptx-cli -> oxml-layout` dependency is a format CLI depending
  downward on a shared crate rather than a shared crate depending on a format
  crate.
- No new production `unwrap`, `expect`, unchecked indexing, arithmetic overflow
  on F-183 inputs, schema-order risk, unmodelled XML loss, or deterministic
  hash-harness baseline edit was found in the reviewed diff.

# F-183, all aspects, pass 3

**Reviewed**: cumulative working tree diff from `a6d98f4`, 22 files, 1817 insertions and 126 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Word CLI no longer accepts the legacy zero-based render page flag

`crates/rdocx-cli/src/main.rs:96`
`crates/rdocx-cli/tests/integration.rs:330`
`docs/hld/10-bindings-spec.md:717`

The binding spec says the `rdocx-cli` flags and zero-based `render --page`
compatibility contract do not change. The render command now exposes only
the one-based `--pages` range flag, and the existing deterministic render
test was changed from `--page 0` to `--pages 1`. A caller that still runs
`rdocx render input.docx --page 0` now gets a clap unknown-argument failure
instead of the selected deterministic PNG. F-183 can add the shared
one-based range grammar, but it must retain the old single-page flag or an
equivalent compatibility path.

## Smells

None.

## Nitpicks

None.

## Checks

- `git diff --check a6d98f4` passed.
- `python3 scripts/prose_check.py .claude/reviews/F-183-all-pass-1.md .claude/reviews/F-183-all-pass-2.md .claude/plans/F-183-design.md .claude/scratch/F-183-progress.md` passed.
- `python3 scripts/sync_agent_skills.py --check` passed.
- Focused Cargo tests were not run in this audit pass because they write build artifacts during a review-only task.

## Not found

- D1 from pass 1 was not found. The central raster regression now decodes PNG, JPEG and TIFF for selection `[2, 0]`, checks two outputs, dimensions and center pixels, and checks the TIFF directory count.
- D2 from pass 1 was not found. Both CLIs render PNG and JPEG output with a one-page helper inside the selected-page loop rather than retaining one `Vec<Vec<u8>>` of every encoded page.
- S1 from pass 1 was not found. Both CLIs preflight multi-file paths, stage bytes through `StagedOutputSet`, reject duplicate and pre-existing targets, remove temporary files on success and failure, and roll back published files when a later publication fails.
- D1 from pass 2 was not found. Word CLI image conversion and render now compute one deterministic layout snapshot, use that snapshot page count for defaults and ranges, and pass the same `layout.layout` to `oxml_pdf::render_pages`.
- Shared raster behavior was not found defective. The API is additive, validates DPI, empty selections, duplicates, out-of-range pages and JPEG quality before encoding, preserves caller order, keeps old opaque PNG wrappers byte-identical, composites JPEG and TIFF on white, and keeps transparent PNG pixels clear unless an authored page background paints them.
- Native and Python APIs were otherwise not found defective. Native selected pages are zero-based, revision-view variants are present, Python uses keyword-only image options, releases the GIL for render work, returns `list[bytes]` for separate images, returns `bytes` for TIFF, and maps raster errors through `LayoutError`.
- Presentation CLI behavior was not found defective. Image ranges use the shared one-based parser, TIFF stays one stream, PNG and JPEG are staged one selected slide at a time, and the 320-pixel thumbnail remains fixed PNG.
- Dependency, WASM and packaging risks were not found defective in the diff. The new encoder dependencies are direct `oxml-pdf` dependencies with default features disabled, no shared crate depends on a format crate, and the progress notes record green dependency policy, WASM target checks, package dry runs, and archive size checks.
- Test strength was otherwise not found defective. The regression gate covers declared formats, exact selected pages, transparent PNG behavior, JPEG quality validation, old PNG wrapper equivalence, CLI index conventions, pre-existing output preservation, and Python keyword-only rendering.
- No new production `unwrap`, `expect`, unchecked indexing, arithmetic overflow on F-183 inputs, schema-order risk, unmodelled XML loss, deterministic hash-harness baseline edit, or golden PNG baseline edit was found in the reviewed diff.

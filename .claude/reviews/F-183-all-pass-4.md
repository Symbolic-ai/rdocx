# F-183, all aspects, pass 4

**Reviewed**: cumulative working tree diff from `a6d98f4`, 22 files, 1941 insertions and 121 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Checks

- `git diff --check a6d98f4` passed.
- `python3 scripts/prose_check.py .claude/plans/F-183-design.md .claude/reviews/F-183-all-pass-1.md .claude/reviews/F-183-all-pass-2.md .claude/reviews/F-183-all-pass-3.md .claude/scratch/F-183-progress.md` passed before this file was written.
- `python3 scripts/prose_check.py .claude/reviews/F-183-all-pass-4.md` passed.
- `python3 scripts/sync_agent_skills.py --check` passed.
- Focused Cargo tests were not run in this audit pass because they write build artifacts during a review-only task.

## Prior findings

- D1 from pass 1 was not found. The central raster regression selects `[2, 0]`, decodes PNG and JPEG output, checks the selected-page order and center pixels, and verifies that TIFF contains two directories with the same selected order at `crates/oxml-pdf/src/raster.rs:1213`, `crates/oxml-pdf/src/raster.rs:1227`, `crates/oxml-pdf/src/raster.rs:1247`, and `crates/oxml-pdf/src/raster.rs:1267`.
- D2 from pass 1 was not found. Word and presentation CLI PNG and JPEG paths render one selected page at a time through `render_one_raster_page`, stage each page, and do not branch on one retained all-page image vector at `crates/rdocx-cli/src/commands.rs:194`, `crates/rdocx-cli/src/commands.rs:491`, `crates/rpptx-cli/src/commands.rs:181`, and `crates/rpptx-cli/src/commands.rs:515`.
- S1 from pass 1 was not found. Multi-file CLI publication now preflights target paths, stages bytes beside the final path, removes staged temp files, and rolls back already published outputs on a later failure at `crates/oxml-cli-support/src/lib.rs:111`, `crates/oxml-cli-support/src/lib.rs:146`, `crates/oxml-cli-support/src/lib.rs:175`, and `crates/oxml-cli-support/src/lib.rs:183`.
- D1 from pass 2 was not found. Word CLI image conversion and render now compute one deterministic layout snapshot, select against that snapshot's page count, and render from the same snapshot at `crates/rdocx-cli/src/commands.rs:171`, `crates/rdocx-cli/src/commands.rs:172`, `crates/rdocx-cli/src/commands.rs:354`, and `crates/rdocx-cli/src/commands.rs:355`.
- D1 from pass 3 was not found. `rdocx render --page` is restored as a zero-based flag, conflicts with the new `--pages` range, keeps the legacy default PNG path and single-line stdout for one selected page, and rejects out-of-range legacy selections before creating the output directory at `crates/rdocx-cli/src/main.rs:96`, `crates/rdocx-cli/src/main.rs:97`, `crates/rdocx-cli/src/commands.rs:453`, `crates/rdocx-cli/src/commands.rs:461`, `crates/rdocx-cli/src/commands.rs:385`, and `crates/rdocx-cli/tests/integration.rs:373`.

## Not found

- Shared raster behavior was not found defective. The additive surface is exported through `oxml-pdf`, rejects invalid DPI, empty selections, duplicate pages, out-of-range pages and invalid JPEG quality before encoding, preserves caller order, keeps old opaque PNG wrappers byte-identical, composites JPEG and TIFF over white, and keeps transparent PNG pixels clear unless an authored page background paints them at `crates/oxml-pdf/src/lib.rs:52`, `crates/oxml-pdf/src/raster.rs:134`, `crates/oxml-pdf/src/raster.rs:174`, `crates/oxml-pdf/src/raster.rs:220`, `crates/oxml-pdf/src/raster.rs:231`, `crates/oxml-pdf/src/raster.rs:1283`, and `crates/oxml-pdf/src/raster.rs:1387`.
- Word CLI page encoding and output behavior was not found defective. The legacy `--page` path remains zero-based and default-PNG, the new `--pages` path remains one-based through the shared parser, both are mutually exclusive, and rejected ranges do not create the requested output directory at `crates/rdocx-cli/src/main.rs:96`, `crates/rdocx-cli/src/main.rs:99`, `crates/rdocx-cli/src/commands.rs:442`, `crates/rdocx-cli/src/commands.rs:453`, and `crates/rdocx-cli/tests/integration.rs:389`.
- Word `convert` behavior was not found regressed. PDF, HTML and Markdown stay on their existing output paths, image conversion uses deterministic layout and the shared one-based page range, TIFF stays one stream, and separate PNG or JPEG pages use existing default path semantics for one page and numbered files for multiple pages at `crates/rdocx-cli/src/commands.rs:147`, `crates/rdocx-cli/src/commands.rs:169`, `crates/rdocx-cli/src/commands.rs:171`, `crates/rdocx-cli/src/commands.rs:173`, and `crates/rdocx-cli/src/commands.rs:473`.
- Native Word and Python surfaces were not found defective. Native selected pages are zero-based, deterministic and revision-view variants are present, Python `render_pages` is keyword-only, accepts zero-based pages, releases the GIL for render work, returns `list[bytes]` for PNG or JPEG and `bytes` for TIFF, and maps raster failures to `LayoutError` at `crates/rdocx/src/document.rs:3837`, `crates/rdocx/src/document.rs:3861`, `crates/rdocx/src/document.rs:3874`, `crates/rdocx-py/src/document.rs:93`, `crates/rdocx-py/src/document.rs:103`, `crates/rdocx-py/src/document.rs:115`, `crates/rdocx-py/src/lib.rs:66`, and `crates/rdocx-py/tests/test_rendering_threads.py:215`.
- Presentation CLI behavior was not found defective. Image ranges use the shared one-based parser, deterministic render output is checked against the slide count, TIFF stays one stream, separate PNG and JPEG pages are staged one selected slide at a time, invalid quality and out-of-range slide selections leave no output, and the 320-pixel thumbnail remains fixed PNG at `crates/rpptx-cli/src/commands.rs:151`, `crates/rpptx-cli/src/commands.rs:160`, `crates/rpptx-cli/src/commands.rs:167`, `crates/rpptx-cli/src/commands.rs:176`, `crates/rpptx-cli/src/commands.rs:370`, and `crates/rpptx-cli/tests/integration.rs:560`.
- Dependency, WASM and packaging risks were not found defective in the diff. The new encoder dependencies are direct `oxml-pdf` dependencies with default features disabled, the CLI crates depend downward on shared crates, and the progress notes record green dependency policy, WASM target checks, package dry runs and archive size checks at `Cargo.toml:123`, `Cargo.toml:124`, `crates/oxml-pdf/Cargo.toml:30`, `crates/oxml-pdf/Cargo.toml:31`, `crates/rdocx-cli/Cargo.toml:30`, `crates/rpptx-cli/Cargo.toml:27`, and `.claude/scratch/F-183-progress.md:58`.
- Test strength was not found defective. Coverage includes declared formats and exact page selection, transparent PNG, JPEG quality, old PNG wrapper equivalence, native zero-based selection, Python keyword-only selection, Word CLI legacy `--page`, Word CLI one-based `--pages`, no convert regression, and presentation CLI image export at `crates/oxml-pdf/src/raster.rs:1181`, `crates/oxml-pdf/src/raster.rs:1283`, `crates/oxml-pdf/src/raster.rs:1340`, `crates/oxml-pdf/src/raster.rs:1387`, `crates/rdocx/tests/regression_test.rs:1295`, `crates/rdocx-py/tests/test_rendering_threads.py:215`, `crates/rdocx-cli/tests/integration.rs:373`, `crates/rdocx-cli/tests/integration.rs:466`, and `crates/rpptx-cli/tests/integration.rs:589`.
- No new production `unwrap`, `expect`, unchecked indexing, arithmetic overflow on F-183 inputs, schema-order risk, unmodelled XML loss, deterministic hash-harness baseline edit, golden PNG baseline edit, new feature flag, new trait, or new generic parameter was found in the reviewed diff.

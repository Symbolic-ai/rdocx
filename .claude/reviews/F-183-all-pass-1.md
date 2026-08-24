# F-183, all aspects, pass 1

**Reviewed**: working tree diff from `a6d98f4`, 19 files, 1269 insertions and 126 deletions
**Verdict**: 2 defects, 1 smell, 0 nitpicks

## Defects

### D1, the central raster regression does not prove exact selected pages

`crates/oxml-pdf/src/raster.rs:1178`
`crates/oxml-pdf/src/raster.rs:1200`
`crates/oxml-pdf/src/raster.rs:1219`

The design test gate says selected pages 0 and 2 must be exported in order, with TIFF page count, dimensions and pixels checked. The test selects `[2, 0]`, but the PNG and JPEG branches only check signatures, length and inequality. The TIFF branch only checks the file signature. A broken implementation that sorted the selection to `[0, 2]`, reversed it, or wrote a one-directory TIFF would still satisfy these assertions. That leaves the story's exact page order and TIFF cardinality contract unproved.

### D2, PNG and JPEG CLI exports now retain every encoded page

`crates/rdocx-cli/src/commands.rs:305`
`crates/rpptx-cli/src/commands.rs:164`
`crates/oxml-pdf/src/raster.rs:145`

The design keeps separate PNG and JPEG pages on the one-page-at-a-time memory path, and the existing presentation CLI regression explicitly says convert must not retain every encoded PNG. Both CLIs now call `render_pages` for multi-page image output, and the PNG and JPEG arms collect the whole selected set into `Vec<Vec<u8>>` before any write. A large selected range therefore holds every encoded PNG or JPEG at once, which is the memory shape the existing streaming path was meant to avoid.

## Smells

### S1, multi-file CLI output is not staged before publication

`crates/rdocx-cli/src/commands.rs:387`
`crates/rpptx-cli/src/commands.rs:456`

Invalid ranges and invalid quality values are rejected before output, but successful multi-page writes are still published one file at a time with direct `std::fs::write` calls. If a later write fails after earlier files were created, the command leaves a partial output set. That falls short of the atomic no-partial-output requirement for the CLI image paths.

## Nitpicks

None.

## Not found

- API shape: shared raster format, options, output and facade error wiring are additive.
- Validation: DPI, empty native page selections, duplicate native page selections, out-of-range pages and JPEG quality are rejected before raster encoding.
- Raster pixels: transparent PNG starts clear, authored backgrounds still paint, and JPEG and TIFF use opaque white composition.
- Native facade: normal, deterministic and revision-view variants are present for selected pages.
- Python binding: `render_pages` is keyword-only, releases the GIL during render work, returns `list[bytes]` for separate pages and `bytes` for TIFF, and maps raster failures to `LayoutError`.
- CLI indexing: native and Python selections are zero-based, and both CLIs use the shared one-based range parser.
- Thumbnail: presentation thumbnail remains fixed PNG at 320 pixels wide.
- Dependency and packaging policy: the new encoder dependencies are direct `oxml-pdf` dependencies with default features disabled, and no format-family dependency edge was added.
- Panics and overflow: no new `unwrap`, `expect`, unchecked indexing or arithmetic panic on untrusted F-183 inputs was found in the production paths reviewed.

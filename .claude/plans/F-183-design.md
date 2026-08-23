# F-183, Image export options

**Status**: approved
**Sprint**: S54
**Size**: S
**Depends on**: none

## Problem

Shared raster output is PNG-only and always paints white before an authored
background (`crates/oxml-pdf/src/raster.rs:21` and
`crates/oxml-pdf/src/raster.rs:49`). The shared public facade exposes only
single-page and all-page PNG functions (`crates/oxml-pdf/src/lib.rs:36`). Word,
Python, and both CLIs mirror those restrictions. The all-page paths cannot
select an exact page range, JPEG has no quality control, transparent PNG is
impossible, and there is no multi-page TIFF container.

The current `rdocx::RenderOptions` selects a layout projection and remains a
small `Copy + Eq` value (`crates/rdocx/src/document.rs:46`). Encoding controls
belong at the shared raster backend, not in that layout type.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule" and "Why these seams".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap", "The
  rasteriser", and "Performance".
- `docs/hld/10-bindings-spec.md`, "Python API shape", "Native Word facade
  stability", and "CLIs".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness",
  "The golden-PNG gate", and "Binding tests".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering",
  "Packaging", and "Dependency policy".
- `docs/hld/14-development-backlog.md`, "F-183, Image export options".

## Approach

Keep pixel production and encoding in the existing `oxml-pdf` raster files.
Add concrete shared values rather than a trait or generic:

```rust
pub enum RasterFormat {
    Png { transparent_background: bool },
    Jpeg { quality: u8 },
    Tiff,
}

pub struct RasterOptions {
    pub dpi: f64,
    pub format: RasterFormat,
}

pub enum RasterOutput {
    SeparatePages(Vec<Vec<u8>>),
    MultiPageTiff(Vec<u8>),
}

pub fn render_pages(
    layout: &LayoutResult,
    page_indices: &[usize],
    options: RasterOptions,
) -> Result<RasterOutput, RasterError>;
```

Preserve caller order. Reject duplicate, empty, or out-of-range page
selections, non-finite or non-positive DPI, and JPEG quality outside 1 through
100 before encoding. Native and Python page indices remain zero-based. CLI
ranges stay one-based and reuse `oxml_cli_support::parse_range`.

Existing PNG APIs remain source-compatible wrappers over opaque white PNG
defaults. New Word methods retain the current revision-view variants. Python
uses keyword-only format, quality, transparency, and pages arguments. General
`convert` and `render` commands in both CLIs expose the same formats and shared
range grammar. The specialized 320-pixel presentation thumbnail stays PNG.

Use focused direct encoder dependencies with default features disabled. JPEG
composites alpha over white. Transparent PNG begins with transparent black,
while an authored `PageFrame.background` still paints normally. TIFF is one
opaque, deterministic, lossless multi-page stream with no compression option.
Separate PNG and JPEG pages retain one-page-at-a-time rendering and output.

## Rejected alternatives

- Extending existing public signatures would break published callers.
- Adding image fields to `rdocx::RenderOptions` conflates layout and encoding.
- Encoding separately in every facade duplicates format behavior.
- Returning a one-element page vector for TIFF hides container cardinality.
- A new raster module or test binary is unnecessary.
- System-font baselines and baseline re-recording are not valid opt-in tests.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `image_export_options_produce_the_declared_formats_and_exact_pages` | A three-page in-code layout exports selected pages 0 and 2 in order as PNG, JPEG, and one two-directory TIFF with correct signatures, dimensions, and pixels. |
| unit | `transparent_png_keeps_clear_pixels_but_authored_background_paints` | Clear pixels keep zero alpha only when requested, the default remains white, and authored backgrounds remain visible. |
| unit | `jpeg_quality_is_validated_and_changes_the_encoded_result` | Values outside 1 through 100 fail and different accepted qualities produce distinct decodable output. |
| regression | `existing_png_entry_points_equal_opaque_option_defaults` | Every old PNG wrapper remains byte-identical to its new default path. |
| integration | `image_export_ranges_share_declared_index_conventions` | Native and Python use zero-based pages, both CLIs use one-based ranges, extensions are correct, and invalid options leave no partial output. |

The **test gate** is regression. Each option produces the declared output and a
page range selects exactly the requested pages. All fixtures are constructed in
code and pixel assertions use deterministic fonts.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout and raster rendering**. Read `docs/hld/08-rendering-spec.md`. Use
  bundled deterministic fonts for every pixel assertion, run the golden PNG
  gate, and never record a system-font baseline.
- **Public API of a published crate**. The changes are additive. Run verified
  package dry runs and archive size assertions for every affected crate.
- **WASM or PyO3 bindings**. Read `docs/hld/10-bindings-spec.md`. Run both WASM
  target checks. Workspace tests keep the required `rdocx-py` and `rpptx-py`
  excludes.
- **Crate dependency graph**. JPEG and TIFF encoders are direct third-party
  dependencies of `oxml-pdf`, their named consumer. Run dependency-policy,
  packaging, and WASM checks. No format-family edge changes.

## Hash harness

Expected unchanged, 49 of 49. Existing sample generation uses opaque
deterministic PNG wrappers. New formats and selection are opt-in. Do not edit
`scripts/hash_baseline.json` or `scripts/golden_pixel_manifest.json`.

## Implementation checklist

- [ ] Add concrete raster format, options, output, and error values.
- [ ] Split pixmap creation from encoding and make initial background explicit.
- [ ] Encode separate PNG and JPEG pages and one multi-page TIFF.
- [ ] Validate quality, DPI, selection, and output atomically.
- [ ] Preserve old PNG APIs as byte-identical wrappers.
- [ ] Thread options through Word, Python, and both general CLI export paths.
- [ ] Add in-code deterministic regression and integration coverage.
- [ ] Run raster, binding, CLI, WASM, package, and unchanged-harness checks.

## Open questions

- Resolved. Presentation `thumbnail` remains fixed PNG. Python uses keyword
  arguments. TIFF is opaque, lossless, and multi-page, with no public
  compression or alpha control.

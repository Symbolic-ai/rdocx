# F-225, PDF page content import

**Status**: completed
**Sprint**: S64
**Size**: L
**Depends on**: F-109, F-110, F-111

## Problem

`Presentation` opens OPC packages at `crates/rpptx/src/lib.rs:826` and converts
bounded ODP through `crates/rpptx/src/odp.rs:69`, but it has no PDF import
boundary. The facade can synthesize slides, text boxes, preset shapes, and
pictures through `crates/rpptx/src/lib.rs:2353`,
`crates/rpptx/src/lib.rs:2461`, and `crates/rpptx/src/lib.rs:4807`. It cannot
yet parse PDF page geometry, content streams, resources, or annotations.

F-225 requires two explicit projections. Preserved mode inserts one full-page
graphic per PDF page. Editable mode retains the declared text, raster image,
path, and URI-link subset as ordinary slide content. Unsupported operators and
font substitutions must remain visible as stable diagnostics, and a late error
must publish no partial presentation.

## Spec reference

- `docs/hld/01-glossary.md`, "Units".
- `docs/hld/02-scope-and-non-goals.md`, "Beyond v1" and "Still non-goals, and still permanent".
- `docs/hld/03-architecture.md`, "Three families, one workspace", "The dependency rule", "Why these seams", and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Media" and "Package integrity".
- `docs/hld/06-presentationml-model.md`, "Public facade", "The shape tree", "Preservation strategy", "Adding a slide", and "Validation".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap", "The recursion hazard", "The rasteriser", "Text in a shape", and "The renderer's input".
- `docs/hld/10-bindings-spec.md`, the native PowerPoint facade sections and "Packaging".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness", "The golden-PNG gate", and "What CI runs".
- `docs/hld/14-development-backlog.md`, "Milestone 21, Presentation depth" and "F-225, PDF page content import".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags" and the new-dependency policy.

## Approach

Add one private `crates/rpptx/src/pdf.rs` module behind the existing `render`
feature. Add optional workspace dependency `lopdf` 0.44.0 with default features
disabled. `lopdf` owns bounded PDF syntax, object, page-tree, resource, stream,
and content decoding. It does not render or enter an `oxml-*` crate.

Expose an additive native facade:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PdfImportMode {
    PreservedGraphic { dpi: f64 },
    Editable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PdfImportLimits {
    pub max_input_bytes: u64,
    pub max_pages: usize,
    pub max_objects: usize,
    pub max_decompressed_bytes: usize,
    pub max_operations_per_page: usize,
    pub max_pixels_per_page: u64,
    pub max_shapes_per_page: usize,
    pub max_diagnostics: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PdfImportDiagnostic {
    pub path: String,
    pub message: String,
}

pub struct PdfImportResult {
    pub presentation: Presentation,
    pub diagnostics: Vec<PdfImportDiagnostic>,
}

impl Presentation {
    #[cfg(feature = "render")]
    pub fn from_pdf_bytes(
        bytes: &[u8],
        mode: PdfImportMode,
    ) -> Result<PdfImportResult>;

    #[cfg(feature = "render")]
    pub fn from_pdf_bytes_with_limits(
        bytes: &[u8],
        mode: PdfImportMode,
        limits: PdfImportLimits,
    ) -> Result<PdfImportResult>;

    #[cfg(all(feature = "render", not(target_arch = "wasm32")))]
    pub fn open_pdf<P: AsRef<Path>>(
        path: P,
        mode: PdfImportMode,
    ) -> Result<PdfImportResult>;
}
```

Add `Error::PdfImport { page, offset, message }`, distinct from the existing
PDF output conformance error. Python, WASM, and CLI methods remain unchanged.

Load PDF bytes in strict mode with finite decompression limits. Parse the page
tree, inherited `MediaBox`, `CropBox`, rotation, resources, content streams,
embedded fonts, image XObjects, and URI link annotations. Reject encryption,
cyclic page or resource graphs, JavaScript, non-URI actions, malformed state,
non-finite numbers, and every declared limit.

The editable subset supports balanced graphics state, affine transforms,
move, line, cubic curve, rectangle, close, nonzero fill and stroke, line width,
cap, join, strictly positive dash arrays at phase zero or an exactly
representable dash boundary, DeviceGray and DeviceRGB solid colours, basic text
state and show operators, JPEG and bounded 8-bit gray or RGB Flate images, and
URI link annotations. A zero member or interior phase produces one stable
diagnostic and omits affected strokes until a valid dash state or graphics-state
restore. A positive member that would convert to a zero DrawingML dash stop is
handled through the same fail-closed boundary. Unsupported safe operators,
colour spaces, font encodings, image
forms, masks, transparency, blend modes, shadings, patterns, forms, and actions
produce stable ordered operation-path diagnostics. Unsupported state cannot
leak into later supported operators.

Normalize each page into existing `oxml-layout` values. Convert the effective
page box and rotation from PDF bottom-left points to top-left points. One PDF
point is exactly 12,700 EMU. Use checked arithmetic and truncate fractional EMU
values toward zero. Embedded supported TrueType fonts retain their decoded
Unicode mapping. Missing or unsupported fonts select a deterministic bundled
substitute and record the requested family or encoding.

For `PreservedGraphic`, lower the complete supported normalized page to one
`LayoutResult`, rasterize it through `oxml_pdf::render_page_to_png`, then add
one full-slide PNG picture. For `Editable`, add text boxes and runs, pictures,
and importer-private canonical custom-geometry shapes to the ordinary shape
tree. Add transparent canonical shape overlays for URI annotation rectangles
and allocate their external slide relationships. These private construction
paths avoid an unrelated general public path or hyperlink mutation API.

Build a fresh `Presentation` in local staged state. Require every page to have
the same effective box size after rotation, set that as the presentation slide
size, add all pages in source order, serialize, reopen, validate, and return the
result only when every step passes. The foreign PDF object graph is never
retained as a second public model.

## Rejected alternatives

- PDFium, MuPDF, Poppler, browser, or LibreOffice in production would add a
  system engine and weaken deterministic publication.
- Writing the complete object, cross-reference, and filter parser locally would
  duplicate a hardened syntax layer and enlarge the security surface.
- Adding a new crate or public PDF object model would create another ownership
  boundary without a second consumer.
- Rasterizing through a separate product renderer would make preserved and
  editable modes disagree before they reach the shared slide renderer.
- General public custom-path and hyperlink APIs are outside this import story.
  Importer-private canonical typed construction is sufficient.
- Silently flattening unsupported content would contradict the diagnostic and
  bounded-compatibility contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `pdf_points_crop_rotation_and_fractional_coordinates_map_to_truncating_emu` | Page boxes, rotation, 12,700 EMU per point, and fractional truncation are exact. |
| unit | `pdf_import_rejects_malformed_encrypted_cyclic_or_unbounded_input_before_publication` | Strict parsing and every input, page, object, stream, operation, pixel, shape, and diagnostic cap fail closed. |
| regression | `unsupported_pdf_operators_are_diagnosed_without_losing_supported_siblings` | Exact ordered paths and messages, retained siblings, and no graphics-state leakage. |
| regression | `pdf_font_substitution_is_explicit_and_deterministic` | Supported embedded fonts need no substitution, while missing or unsupported encodings name the deterministic replacement. |
| regression | `pdf_content_comments_cannot_silently_truncate_operator_decoding` | Comment and indentation syntax cannot silently drop later content operations. |
| integration | `both_pdf_import_modes_publish_valid_reopenable_presentations` | Both modes validate, save, reopen, and retain slide order and bounds. |
| round-trip | `editable_pdf_text_images_paths_and_links_survive_save_and_reopen` | Text, link target and rectangle, path geometry and paint, and image bytes and dimensions remain editable and exact. |
| differential | `pdf_page_import_matches_pinned_poppler_geometry_pixels_text_and_links` | Exact page geometry, source-render similarity, and editable text and link mappings match the pinned oracle. |
| regression | `pdf_import_differential_rejects_geometry_text_link_and_pixel_perturbations` | An unchanged source passes, while a 1.01-point shift, one-pixel imported geometry shift at 150 DPI, text or link mutation, and calibrated pixel mutation fail the final predicate. |

The **test gate** is the backlog's differential gate: pinned PDF pages preserve
page geometry and match the source render, while the editable subset retains
text and link mappings.

The oracle is Poppler 26.01.0. The gate verifies `pdfinfo` and `pdftoppm`
identity, builds the PDF and embedded image and font input in source, and
rasterizes at 150 DPI. Effective dimensions must match exactly. Preserved mode
and the normalized supported editable projection must reach raw full-image
luminance SSIM of at least 0.995. Pixel-aligned representative geometry
includes a 38.4-point styled square so renderer-only antialiasing does not
weaken the metric. Editable text and URI link target and rectangle are exact.
A 1.01-point shift, one-pixel imported geometry shift, and
calibrated pixel sensitivity check prove the final predicate detects the
intended regressions. No binary fixture is added.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Unit conversion. Assert exact point-to-EMU conversion, CropBox and rotation,
  preserve fractional truncation, and declare the harness result.
- Layout, text shaping, and rendering. Use deterministic bundled fonts, pin
  Poppler and DPI, state the SSIM floor, and prove geometry and pixel
  sensitivity. Record no system-font baseline.
- Parser and serialiser. Use strict bounded PDF parsing, schema-ordered OOXML
  construction, candidate save and reopen, relationship ownership checks, and
  sibling preservation diagnostics. Generated OOXML retains unrelated template
  XML byte for byte. Foreign PDF bytes are not claimed to round-trip.
- Crate dependency graph. Keep no-default `lopdf` optional and direct only in
  `rpptx` under `render`, then verify no `oxml-*` family edge changes.
- Public API of a published crate. State the additive pre-1.0 impact, run
  rustdoc with warnings denied, run `cargo publish --dry-run`, and assert the
  package archive remains below 10 MiB.
- WASM or PyO3 bindings. Add no binding API, check the default and render
  wasm32 graphs, and keep both binding exclusions on workspace tests.
- New module or file. Obtain explicit approval for `pdf.rs`. Add no trait,
  generic parameter, crate, builder, wrapper, or feature flag.
- External oracle. Pin and verify Poppler, use source-built input, deterministic
  fonts, exact geometry, a stated 150 DPI tolerance, and sensitivity checks.

## Hash harness

Expected unchanged, 49 of 49. Existing sample generation and rendering do not
call the importer. Any delta blocks the story, and the baseline is not
re-recorded.

## Implementation checklist

- [x] Add the approved private module and optional no-default `lopdf` edge.
- [x] Add the gated native types, error, methods, and default limits.
- [x] Implement strict bounded page, resource, content, image, font, and annotation parsing.
- [x] Normalize page geometry, transforms, paths, text, images, links, and diagnostics.
- [x] Implement both preserved and editable transactional projections.
- [x] Serialize, validate, reopen, and return only the complete candidate.
- [x] Add unit tests in the module and public coverage to the existing `rpptx` integration binary.
- [x] Run the pinned Poppler differential and perturbation checks.
- [x] Run every routed package, WASM, dependency, deterministic-render, and full verification gate.
- [x] Update exactly the listed HLD files.

## Open questions

Resolved for S64. The private module, optional dependency, native-only API,
mixed-page-size rejection, pinned Poppler oracle, and declared comparison and
sensitivity thresholds are approved.

# F-224, HTML slide content import

**Status**: completed
**Sprint**: S64
**Size**: L
**Depends on**: F-110, F-112

## Problem

The Presentation facade can author editable tables, text boxes, shapes, and
pictures through `crates/rpptx/src/lib.rs:2461` and
`crates/rpptx/src/lib.rs:4781`, but it has no HTML import boundary. Its existing
foreign-format importer is the private ODP module declared at
`crates/rpptx/src/lib.rs:117`. F-224 requires a bounded HTML5 and CSS projection
into those existing owners, with stable source-path diagnostics and no partial
presentation on failure.

The missing work is browser-grade tree repair, an explicit CSS layout contract,
safe caller-owned image resolution, hyperlink relationship authoring, and a
save-and-reopen publication check. The importer must not imply support for
arbitrary browser layout or add a second slide rendering model.

## Spec reference

- `docs/hld/01-glossary.md`, "Units" and "Geometry and text".
- `docs/hld/02-scope-and-non-goals.md`, "Beyond v1" and "Still non-goals, and still permanent".
- `docs/hld/03-architecture.md`, "Why these seams" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Media" and "Package integrity".
- `docs/hld/05-drawingml-model.md`, "Text".
- `docs/hld/06-presentationml-model.md`, "Public facade", "The shape tree", "Adding a slide", and "Validation".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap", "Text in a shape", and "Tables".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and the native PowerPoint facade sections.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The hash harness", and "The render fidelity gate".
- `docs/hld/14-development-backlog.md`, "Milestone 21, Presentation depth" and "F-224, HTML slide content import".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags" and the dependency policy around `scraper`.

## Approach

Add one private `crates/rpptx/src/html.rs` module behind the existing
`default-template` feature. Reuse the pinned workspace `scraper` 0.27 dependency
as an optional direct dependency of `rpptx`. The module parses an HTML document
or fragment, builds one fresh `Presentation`, serializes and reopens the
candidate, validates it, and returns it only after every step succeeds.

Expose an additive native-only facade:

```rust
#[derive(Clone, Copy, Debug)]
pub struct HtmlImageResource<'a> {
    pub source: &'a str,
    pub bytes: &'a [u8],
    pub filename: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDiagnostic {
    pub path: String,
    pub property: Option<String>,
    pub message: String,
}

pub struct HtmlReadResult {
    pub presentation: Presentation,
    pub diagnostics: Vec<HtmlDiagnostic>,
}

impl Presentation {
    pub fn from_html(
        html: &str,
        images: &[HtmlImageResource<'_>],
    ) -> Result<HtmlReadResult>;

    pub fn open_html<P: AsRef<Path>>(
        path: P,
        images: &[HtmlImageResource<'_>],
    ) -> Result<HtmlReadResult>;
}
```

Add `Error::Html { path, message }`. Python, WASM, and CLI surfaces remain
unchanged. This is an additive pre-1.0 native API.

The bounded input contract is:

- One slide is created for each top-level `section[data-slide]`. A document
  with no such section projects its `body` into one slide.
- The default slide size is the viewport. One CSS pixel is exactly 9,525 EMU.
  `px`, `pt`, `in`, `cm`, `mm`, and zero are supported with checked arithmetic
  and the repository's truncating conversion rule.
- Layout requires explicit `position: absolute` with `left`, `top`, `width`,
  and `height`. Nested positioned boxes accumulate parent offsets. Implicit
  flow, flexbox, grid, transforms, and browser intrinsic sizing are diagnosed.
- Supported structure is `section`, `div`, paragraphs, headings, `span`,
  semantic bold and italic, line breaks, tables, images, and anchors.
- Supported selectors are type, class, id, descendant, and child selectors.
  Inline declarations, specificity, and source order form the bounded cascade.
- Supported style covers background and border colour, border width, font
  family and size, bold, italic, underline, strike, text colour, text alignment,
  and explicit geometry.
- Rectangles, text, tables, and images project through existing facade methods.
  Links use slide-scoped external relationships and typed run hyperlink data.
- Images resolve only through the caller-provided resource slice. Duplicate
  source names fail. The importer performs no network or filesystem fetch.
- Relative targets and `http`, `https`, and `mailto` links are retained. Unsafe
  schemes are diagnosed while their visible text remains.
- Input bytes, DOM depth and nodes, text, CSS rules, projected objects, table
  dimensions, link count, image bytes, diagnostics, and aggregate selector
  match attempts all have hard bounds. Exceeding a bound fails before
  publication.

The parser records HTML5 repair notices and unsupported content in document
order with stable DOM paths. Unsupported positioned elements, semantic control
or media elements, and elements carrying visual or semantic attributes remain
diagnostic even when they contain no text. The candidate uses existing shape IDs, media
deduplication, relationship ownership, text mutation, table mutation, and
schema-ordered serialization. No public intermediate HTML or layout model is
retained.

## Rejected alternatives

- Reusing `rdocx::html` would introduce a Presentation-to-Word production edge
  and couple the importer to the wrong document model.
- Moving HTML parsing into an `oxml-*` crate would put format conversion policy
  below the facade and create a second public model.
- Implementing flow, flexbox, grid, or resource fetching would create browser
  semantics that the story does not authorize.
- Adding a new feature flag was rejected because there is no separate named
  consumer. The importer follows the existing template-backed construction
  boundary.
- Treating a screenshot as the only oracle was rejected because pixel similarity
  cannot prove editable structure, text, images, or hyperlink ownership.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `html_slide_import_converts_explicit_css_boxes_to_exact_emu` | Absolute units, nested offsets, finite bounds, and truncation are exact. |
| unit | `html_slide_import_applies_the_bounded_cascade_to_shapes_and_text` | Selector specificity, inline priority, source order, fills, lines, paragraphs, and runs are exact. |
| unit | `html_slide_import_rejects_every_declared_resource_limit` | Every input, DOM, CSS, object, table, image, link, and diagnostic cap fails closed. |
| regression | `unsupported_html_layout_and_styles_are_diagnosed_without_losing_supported_siblings` | Stable paths and property names accompany every safe loss. |
| regression | `unsupported_empty_semantic_elements_are_diagnosed` | Positioned and semantic video, canvas, and input nodes remain diagnostic without text. |
| regression | `html_diagnostics_follow_document_order_across_collection_phases` | Stylesheet, resource, and projection diagnostics publish in exact DOM order rather than collection-phase order. |
| integration | `html_slide_import_projects_editable_shapes_tables_images_and_links_after_reopen` | Shape order, text, cells, media bytes, relationships, validation, save, and reopen agree. |
| round-trip | `html_slide_import_preserves_unmodelled_template_xml_and_schema_order` | Opaque template content remains byte-preserved and generated XML reparses without repair. |
| differential | `source_built_html_matches_pinned_chrome_after_save_and_reopen` | Exact structure and text, one-pixel geometry tolerance, and the declared image threshold match Chrome. |
| regression | `html_browser_differential_rejects_geometry_text_and_pixel_perturbations` | One-pixel movement, text mutation, and calibrated pixel mutation fail the intended boundary. |

The **test gate** is the backlog's differential gate: source-built HTML matches
the browser reference at the declared shape, text, and pixel boundary after
save and reopen.

The oracle is Google Chrome 152.0.7977.65. The gate verifies the executable
identity, uses an isolated headless profile with network disabled, fixes the
viewport at 1280 by 720 CSS pixels, and supplies the same source-built HTML,
PNG, and bundled font bytes to both sides. It compares exact shape kind,
z-order, text, links, and reopened relationships. Geometry may differ by at
most one CSS pixel. Visible text and selected run formatting are exact. The
full image luminance SSIM floor is 0.95 at 96 DPI. No binary fixture is added.

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

- Unit conversion. Assert exact CSS pixel and point conversion to EMU, preserve
  truncation, and declare the harness result.
- Layout, line breaking, and text shaping. Use deterministic bundled fonts for
  Rust output and explicit font bytes in Chrome. Record no system-font baseline.
- Parser and serialiser. Save and reopen the candidate, prove schema order and
  fixed OOXML prefixes, and prove opaque template XML remains byte-preserved.
- Crate dependency graph. Keep `scraper` direct only in `rpptx` under
  `default-template`, then verify no `oxml-*` family edge changes.
- Public API of a published crate. State the additive pre-1.0 impact, run
  rustdoc with warnings denied, run `cargo publish --dry-run`, and assert the
  package archive remains below 10 MiB.
- WASM or PyO3 bindings. Add no binding API, run the wasm32 checks, and keep
  both binding crates excluded from the workspace test command.
- New module or file. Obtain explicit approval for `html.rs`. Add no trait,
  generic parameter, crate, integration test binary, builder, or feature flag.
- External oracle. Pin and verify Chrome, use one common source, state every
  tolerance, and prove the gate detects structural, geometry, text, and pixel
  perturbations.

## Hash harness

Expected unchanged, 49 of 49. Existing sample generation and rendering do not
call the new importer. Any delta blocks the story, and the baseline is not
re-recorded.

## Implementation checklist

- [x] Add the approved private module and optional direct `scraper` edge.
- [x] Add the native result, diagnostic, image resource, error, and constructor surfaces.
- [x] Bound HTML parsing, CSS parsing, resource lookup, projection, and diagnostics.
- [x] Implement explicit CSS geometry and the declared cascade.
- [x] Project shapes, formatted text, tables, images, and hyperlinks through existing owners.
- [x] Serialize, validate, reopen, and return only the complete candidate.
- [x] Add unit tests in the module and public coverage to the existing `rpptx` integration binary.
- [x] Run the pinned Chrome differential and perturbation checks.
- [x] Run every routed package, WASM, dependency, deterministic-render, and full verification gate.
- [x] Update exactly the listed HLD files.
- [x] Bound aggregate selector-match work independently of node and CSS rule caps.
- [x] Diagnose empty visual and semantic unsupported elements.
- [x] Publish diagnostics in deterministic document order across collection phases.

## Open questions

Resolved for S64. The private module, optional dependency, native-only API,
explicit absolute-position boundary, excluded browser features, pinned Chrome
oracle, and declared comparison thresholds are approved.

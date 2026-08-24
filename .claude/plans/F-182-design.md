# F-182, SVG page export

**Status**: approved
**Sprint**: S56
**Size**: M
**Depends on**: none

## Problem

The shared rendering boundary already exposes immutable pages, fonts,
diagnostics, and recursive positioned elements through `LayoutResult` and
`PageFrame` at `crates/oxml-layout/src/output.rs:328`. PDF and raster consume
that contract at `crates/oxml-pdf/src/lib.rs:18`, but the Word facade stops at
PDF and raster APIs in `crates/rdocx/src/document.rs:3631`. There is no
searchable, scalable fixed-page export.

The backend cannot flatten the page to a bitmap or glyph outlines. `GlyphRun`
retains Unicode text and shaped advances, while the recursive model also
carries paths, images, links, groups, clips, opacity, effects, marked content,
and page background. SVG must preserve supported siblings and return explicit
diagnostics for every lossy lowering (`docs/sprints/CURRENT_SPRINT.md:62`).

## Spec reference

- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap",
  "Extending PositionedElement", "The recursion hazard", "The PDF backend",
  and "The rasteriser".
- `docs/hld/03-architecture.md`, "The dependency rule", "Why these seams", and
  "Facade conventions".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and the
  recursive `MarkedContent` backend rule.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and deterministic pixel
  gates.
- `docs/hld/14-development-backlog.md`, "F-182, SVG page export".
- `docs/hld/15-build-and-toolchain.md`, deterministic fonts, dependency policy,
  packaging, and published-family boundaries.

## Approach

Add one private `crates/rdocx/src/svg.rs` renderer. It consumes an immutable
`LayoutResult` and one zero-based page index, recursively emits SVG, and does
not mutate or re-layout the document. It belongs in stable `rdocx` for S56.
Putting a new production API in already-published `oxml-pdf` 0.5.0 would make
stable 0.10.0 depend on an unpublished incubating change, while F-X055 keeps
that family unchanged.

Expose additive native-only values and facade methods:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgDiagnostic {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SvgRenderResult {
    pub svg: String,
    pub diagnostics: Vec<SvgDiagnostic>,
}

impl Document {
    pub fn render_page_to_svg(
        &self,
        page_index: usize,
    ) -> Result<Option<SvgRenderResult>>;

    pub fn render_page_to_svg_with_options(
        &self,
        page_index: usize,
        options: RenderOptions,
    ) -> Result<Option<SvgRenderResult>>;

    pub fn render_page_to_svg_deterministic(
        &self,
        page_index: usize,
    ) -> Result<Option<SvgRenderResult>>;

    pub fn render_page_to_svg_deterministic_with_options(
        &self,
        page_index: usize,
        options: RenderOptions,
    ) -> Result<Option<SvgRenderResult>>;
}
```

Reuse the normal and deterministic cached layout paths, matching the PNG
facade. Out-of-range pages return `Ok(None)`. Emit a self-contained SVG root
whose point dimensions and view box preserve the top-left, y-down `PageFrame`
coordinates. Definitions receive deterministic depth-first IDs and include
only used resources.

- Text remains `<text>`. Embed each used font as a data URL under a stable
  family keyed by `FontId`. Preserve Unicode and use cumulative shaped advances
  as scalar positions when glyph and scalar counts agree. When complex shaping
  prevents one-to-one placement, retain searchable text, constrain its total
  advance, and emit one stable diagnostic.
- Rectangles, lines, and paths map directly to SVG geometry with alpha, dashes,
  caps, joins, fill rules, and solid or gradient paint. Diagnose tile paint,
  whose media bytes are unavailable from its carrier.
- Images remain embedded data URLs, use sniffed PNG or JPEG MIME, occupy the
  exact rectangle, and never reference external resources.
- Groups recurse as nested `<g>` elements with affine transforms, deterministic
  clip paths, opacity, and child order. `MarkedContent` recurses without visible
  geometry.
- Lower `OuterShadow` to one deterministic filter. Any future unsupported
  effect produces one path-specific diagnostic while children still render.
- Preserve safe link rectangles as SVG anchors. Permit relative targets and
  reviewed `http`, `https`, and `mailto` schemes. Diagnose and omit active or
  unsupported schemes.
- Escape all text, attribute, URL, and font-family content. Emit no scripts,
  remote fonts, remote images, or ambient filesystem references.
- Return existing layout diagnostics first, then SVG lowering diagnostics in
  traversal order.

Add `base64` as a direct `rdocx` dependency for self-contained resources. Add
exact `resvg` 0.48.1 only as a development dependency for the pixel gate. Its
font database receives only the same explicit `LayoutResult` fonts, never
system fonts.

## Rejected alternatives

- Public SVG in `oxml-pdf` is the ideal later ownership seam, but it requires a
  separate incubating version and release decision absent from S56.
- Visible glyph outlines plus hidden text violates the requirement that text
  stays text and creates two divergent representations.
- Browser or system-font rasterization makes the golden machine-dependent.
- External font or image references are not self-contained and can change.
- A new SVG crate creates a publication boundary for one current consumer.
- Python, WASM, CLI, and Presentation APIs exceed the native Word scope.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden, gate | `svg_page_rasterises_like_the_png_backend` | A source-built deterministic SVG raster at 150 dpi has exact dimensions and global luminance SSIM of at least 0.99 against PNG. A one-point perturbation fails the threshold. |
| integration | `svg_export_preserves_searchable_text_geometry_fonts_images_links_and_clips` | The facade returns parseable, self-contained SVG with exact text, geometry, used resources, safe links, and no external reference. |
| regression | `svg_export_recurses_without_dropping_supported_siblings` | Three-deep groups and marked content preserve order, transforms, clip, opacity, and siblings around diagnosed unsupported paint or effects. |
| regression | `svg_export_escapes_untrusted_content_and_rejects_active_links` | XML metacharacters cannot inject markup, active links are omitted with diagnostics, and safe links remain. |
| unit | `svg_export_is_deterministic_and_emits_only_used_definitions` | Two exports are byte-identical and definitions follow stable first-use order. |
| regression | `svg_complex_text_stays_text_and_reports_positioning_approximation` | Complex shaping stays searchable and reports the declared placement approximation. |

The **test gate** is golden. `svg_page_rasterises_like_the_png_backend` uses
deterministic fonts, exact dimensions, pinned resvg 0.48.1, and the recorded
0.99 SSIM threshold. All fixtures are source-built. Unit tests stay in the
private module, facade regressions use `regression_test.rs`, and cross-surface
coverage uses `integration_test.rs`. No test binary is added.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Layout, pagination, line breaking, text shaping**. Read
  `docs/hld/08-rendering-spec.md`. All raster comparisons use deterministic
  layout and the same explicit font bytes. Run the named SVG golden and the
  existing golden PNG harness without recording a system-font baseline.
- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add parseability, namespace, stable
  order, escaping, and source-document preservation regressions.
- **Public API of a published crate**. This additive `rdocx` surface enters
  v0.10.0. Run documentation warnings, the patched 22-package dry run, archive
  size checks, and confirm no binding or Presentation surface changed.
- **Crate dependency graph**. `base64` has one named runtime consumer and
  resvg is development-only. Run metadata, dependency tree, WASM, and supply
  chain checks. No reverse format-family edge is introduced.
- **A new module or file**. `svg.rs` keeps one complete recursive backend out
  of `document.rs`. It requires consolidated approval. No trait, generic,
  feature, crate, or test binary is added.
- **External oracle comparison**. Pin resvg 0.48.1, record its identity and
  SSIM metric, feed it explicit fonts only, and keep it out of runtime and
  package inventories.
- **WASM or PyO3 bindings**. Although no binding API is added, run both wasm32
  checks and retain the workspace Python test exclusions.

## Hash harness

Expected unchanged across all 49 entries. Samples do not call SVG export, and
the PDF and PNG paths are not modified. Any delta blocks integration.

## Implementation checklist

- [ ] Add the approved private SVG module and additive native results and APIs.
- [ ] Emit deterministic geometry, definitions, escaping, fonts, and images.
- [ ] Recurse through every page element, marked content, transform, clip, opacity, background, paint, effect, and link.
- [ ] Keep text as text with exact placement when representable and diagnostics otherwise.
- [ ] Preserve siblings and merge layout and lowering diagnostics in stable order.
- [ ] Add approved base64 and pinned resvg dependencies.
- [ ] Add source-built unit, integration, security, determinism, and calibrated golden coverage in existing binaries.
- [ ] Run deterministic-font, oracle, WASM, package, supply-chain, full verify, and unchanged-harness gates.

## Open questions

Resolved. F-182 may add private `crates/rdocx/src/svg.rs`, `base64` at runtime,
and exact resvg 0.48.1 for development-only validation. SVG remains native to
stable `rdocx` 0.10.0. Text uses exact scalar positions when shaped glyph and
scalar counts agree, then retains searchable text with a stable diagnostic
otherwise. The deterministic gate rasterises at 150 dpi and requires SSIM of
at least 0.99.

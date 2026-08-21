# F-X032, Expose complete Word layout results

**Status**: approved
**Sprint**: S51
**Size**: S
**Depends on**: F-009, F-151

## Problem

`Document` caches the complete normal and deterministic layouts as
`Arc<oxml_layout::LayoutResult>` values, but the native facade exposes only a
cloned `PageFrame` through `layout_page`. Third-party renderers therefore see
positioned glyph ids without the `FontData` bytes and `FontId` mapping required
to resolve those glyphs.

The caller-font path has the same gap. `to_pdf_with_fonts_and_options` already
builds an uncached layout with caller-provided font bytes, then immediately
consumes it in the built-in PDF backend. Browser and other external renderers
cannot obtain that intermediate result.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put" and facade ownership.
- `docs/hld/08-rendering-spec.md`, "Performance" and "Word revision views".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and
  `Document` threading.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The hash harness".
- `docs/hld/14-development-backlog.md`, "F-X032, Expose complete Word layout
  results".

## Approach

Expose the existing layout paths through four additive native methods:

```rust
pub fn layout(&self) -> Result<Arc<oxml_layout::LayoutResult>>;

pub fn layout_with_options(
    &self,
    options: RenderOptions,
) -> Result<Arc<oxml_layout::LayoutResult>>;

pub fn layout_with_fonts(
    &self,
    font_files: &[(&str, &[u8])],
) -> Result<oxml_layout::LayoutResult>;

pub fn layout_with_fonts_and_options(
    &self,
    font_files: &[(&str, &[u8])],
    options: RenderOptions,
) -> Result<oxml_layout::LayoutResult>;
```

Rename the current private two-argument `layout_with_options` helper to
`layout_for_options` because Rust does not overload methods. `layout` and
`layout_with_options` share the existing accepted-view cache and retain the
existing uncached tracked-view behavior. Caller-font methods return owned,
uncached results because arbitrary borrowed font sets have no stable cache key.
Route `to_pdf_with_fonts_and_options` through the new caller-font accessor so
there is one implementation path.

Return `Arc` for cached layouts. Cloning `LayoutResult` would duplicate pages
and every font byte merely to hide a standard-library ownership type. Do not
re-export low-level layout types, add another cache, or add a binding surface.

## Rejected alternatives

- Return a cloned cached result. This duplicates potentially large font files
  and defeats the cache's sharing contract.
- Cache caller-provided fonts. A stable content key and eviction policy are not
  part of this requested accessor.
- Expose pages and fonts through separate methods. That can separate a page
  from the exact font mapping used to shape it.
- Add an SVG backend. Issue 37 asks for the integration surface, and the
  reporter already has an external backend.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `full_layout_exposes_resolvable_font_data_and_reuses_the_cache` | Every glyph-run font id resolves to returned `FontData`, repeated accepted calls share one cache invocation, and PDF uses the same layout |
| regression | `layout_with_fonts_returns_the_caller_font_mapping_without_caching` | Caller font bytes and family appear in the owned result and repeated calls perform separate layouts |
| regression | `layout_options_keep_tracked_and_accepted_cache_ownership_separate` | Tracked output honors `RenderOptions` without populating or replacing the accepted cache |
| integration | public API compile coverage | A downstream-style call can traverse pages, glyph runs, fonts, and raw font bytes through public types |
| boundary | WASM target check | Caller-provided fonts compile without system-font discovery |

The **test gate** is regression. Every emitted glyph-run font id resolves to
returned font data, repeated default calls share the accepted layout cache,
caller-provided fonts appear in the owned result, and tracked layout does not
populate the accepted cache.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, text shaping**. Read HLD 08. Use bundled
  deterministic fonts for any visual assertion, and treat an output baseline
  delta as blocking.
- **Public API of a published crate**. Read HLD 10 and the structural rules.
  The native API is additive. Run the workspace package dry run, enforce the
  10 MiB archive ceiling, and retain the WASM target check.

## Hash harness

Expected unchanged across all 49 entries. Existing samples consume the same
layout paths and do not call the new accessors.

## Implementation checklist

- [ ] Rename the private options helper without changing cache behavior.
- [ ] Add the four native layout accessors in the existing facade file.
- [ ] Route caller-font PDF rendering through the owned layout accessor.
- [ ] Prove font-id resolution, cache sharing, caller-font ownership, options,
  and WASM compilation.
- [ ] Run the public-package and archive-size rider.
- [ ] Run full verification and the unchanged hash harness.
- [ ] Update exactly the HLD files listed above.
- [ ] Comment on issue 37 with the reviewed API and release target after
  integration.

## Open questions

None. The user approved F-X032 for S51. Cache-backed access uses `Arc`,
caller-font access stays uncached, and `RenderOptions` variants are included
from the first public version.

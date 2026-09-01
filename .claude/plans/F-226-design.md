# F-226, Notes and handout export

**Status**: completed
**Sprint**: S63
**Size**: M
**Depends on**: F-217

## Problem

The native presentation facade renders slides, but it cannot produce the
speaker-facing notes pages or audience-facing handouts already described by
the relationship-resolved notes and handout masters. Callers therefore cannot
export slide thumbnails, speaker notes, slide numbers, stored dates, headers,
or footers through the deterministic PDF and raster backends.

## Spec reference

- `docs/hld/03-architecture.md`, package assembly and shared rendering stages.
- `docs/hld/04-opc-and-packaging.md`, relationship-owned notes and handout
  masters, themes, and media.
- `docs/hld/06-presentationml-model.md`, notes slides, notes masters, handout
  masters, placeholder ownership, and header-footer policy.
- `docs/hld/07-inheritance-and-resolution.md`, placeholder and master
  inheritance.
- `docs/hld/08-rendering-spec.md`, fixed-size page frames and deterministic PDF
  and raster output.
- `docs/hld/10-bindings-spec.md`, additive native-only export API.
- `docs/hld/12-testing-strategy.md`, deterministic golden and sensitivity
  requirements.
- `docs/hld/15-build-and-toolchain.md`, public API and package riders.

## Approach

Add one concrete native handout layout enum with the six PowerPoint audience
arrangements:

```rust
pub enum HandoutLayout {
    One,
    Two,
    Three,
    Four,
    Six,
    Nine,
}
```

Add four native deterministic export methods:

```rust
Presentation::to_notes_pdf_deterministic(&self) -> Result<Vec<u8>>;
Presentation::notes_page_pngs_deterministic(&self, dpi: f64) -> Result<Vec<Vec<u8>>>;
Presentation::to_handout_pdf_deterministic(&self, layout: HandoutLayout) -> Result<Vec<u8>>;
Presentation::handout_page_pngs_deterministic(&self, layout: HandoutLayout, dpi: f64) -> Result<Vec<Vec<u8>>>;
```

Both paths stage the current package once and reuse the existing deterministic
slide layout, font manager, DrawingML resolver, PDF writer, and PNG rasterizer.
Notes pages use `p:notesSz`, overlay notes-slide placeholder content onto its
relationship-resolved notes master, apply the master header-footer flags, and
replace the slide-image placeholder with the already rendered source slide.
Slides without a notes part still produce one notes page with an empty body.

Handouts use the same `p:notesSz` paper, render the relationship-resolved
handout master once per output page, and place source slides into the selected
one, two, three, four, six, or nine-up grid in presentation order. Three-up
adds the established audience note-rule area. Every thumbnail is aspect-fit,
clipped, bordered, and labelled with its source slide number. Stored master
date, header, footer, and page-number field text is deterministic. No current
clock or system font is consulted.

Missing, external, wrong-type, duplicate, or malformed master and theme
relationships fail closed. Invalid, non-finite, zero, or excessive raster DPI
fails before output. Notes and handout export does not mutate the presentation
and includes hidden slides in source order, matching the existing complete
presentation export policy.

The implementation stays in `crates/rpptx/src/lib.rs`. No new crate, module,
file, dependency, trait, generic, feature, binding method, or renderer API is
added. The native additions are additive in the pre-1.0 API and require release
review before publication.

## Rejected alternatives

- Rasterize slide thumbnails before composition. Keeping them as ordinary
  positioned elements preserves vector geometry and text in PDF output.
- Add notes-specific behavior to `rpptx-layout` or `rpptx-render`. Package
  relationships and master ownership belong to the facade assembly stage.
- Read filename conventions for notes, handout, or theme parts. Every part is
  relationship-resolved from its owning source.
- Use the current date. Deterministic output renders only stored field text.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden | `notes_pages_follow_master_geometry_text_metadata_and_slide_order` | Source-built notes pages use notes size, master geometry, source thumbnails, speaker text, stored date, footer, and exact slide numbers in order. |
| golden | `handouts_follow_master_metadata_and_all_six_audience_layouts` | All six layouts have exact page counts, thumbnail order, bounds, labels, repeated master metadata, and three-up note rules. |
| backend | `notes_and_handouts_export_pdf_and_png_with_deterministic_dimensions` | Repeated PDF and PNG output is byte-identical and every raster has the declared dimensions. |
| hierarchy | `notes_and_handout_export_resolve_noncanonical_master_theme_and_media_targets` | Custom part names and owner-local relationships resolve without conventional paths. |
| rejection | `notes_and_handout_export_fail_closed_for_broken_hierarchy_and_invalid_dpi` | Duplicate, external, wrong-type, missing, malformed, and invalid-DPI cases publish no output. |
| preservation | `notes_and_handout_export_leave_the_opened_package_byte_identical` | Every export method leaves ordinary serialization unchanged and the hash harness stays stable. |

The required gate is
`notes_pages_follow_master_geometry_text_metadata_and_slide_order`.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Package graph and parsers**: validate exact owner relationship types,
  internal targets, cardinality, and namespace-aware typed roots. Run package
  preservation and malformed-graph riders.
- **Rendering and differential output**: use deterministic fonts, compare page
  geometry and text order, prove PDF and raster dimensions, and retain a
  sensitivity case that rejects a one-point placement change.
- **Public API**: run rustdoc with warnings denied, README inventories, patched
  package dry-runs, archive-size checks, WASM checks, and release review.

## Hash harness

Expected to remain unchanged. The new entry points are opt-in and do not alter
ordinary slide rendering.

## Implementation checklist

- [x] Record the 49-entry baseline and add the six source-built regressions.
- [x] Add the native layout enum and deterministic PDF and PNG methods.
- [x] Resolve notes and handout master, theme, media, placeholder, and metadata
  ownership without conventional paths.
- [x] Compose ordered vector slide thumbnails and notes text through the shared
  layout and backend paths.
- [x] Update exactly the listed HLD files and pass the routed public-rendering
  riders.
- [x] Complete with a zero-defect, zero-smell microscope pass.

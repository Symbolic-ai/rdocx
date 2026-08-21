# F-168, Watermarks

**Status**: approved
**Sprint**: S51
**Size**: S
**Depends on**: none

## Problem

The run parser types `w:drawing` but leaves `w:pict` as opaque run XML
(`crates/rdocx-oxml/src/text.rs:604`). That preserves existing VML, but layout
only consumes `RunContent::Drawing` and `alt_drawings`
(`crates/rdocx-layout/src/engine.rs:1487`), so VML-only watermarks are not
rendered.

Header paragraphs already enter layout, but header image relationship ids are
part-scoped and cannot be resolved through the main document relationship map.
The facade can install raw VML headers, yet has no typed text or image
watermark API.

## Spec reference

- `docs/hld/03-architecture.md`, "What stays put".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Media", and
  "Package integrity".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap" and "Word
  revision views" as the neighboring Word rendering boundary.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "The golden-PNG gate".
- `docs/hld/14-development-backlog.md`, "F-168, Watermarks".

## Approach

In existing files, add a conservative layout-only VML projection while keeping
captured `w:pict` bytes as the sole serialization source for opened documents:

```rust
pub enum VmlWatermark {
    Text {
        text: String,
        width_pt: f64,
        height_pt: f64,
        rotation_degrees: f64,
        color: String,
        font_family: Option<String>,
        opacity: f64,
    },
    Image {
        relationship_id: String,
        width_pt: f64,
        height_pt: f64,
        rotation_degrees: f64,
        opacity: f64,
    },
}
```

Project only header `w:pict/v:shape` values containing `v:textpath@string` or
`v:imagedata@r:id`. Leave every other VML subtree opaque. Generated watermarks
use fixed `w`, `v`, `o`, and `r` prefixes and the required VML child order.

Add two additive native methods with Word-like fixed text defaults:

```rust
pub fn set_text_watermark(&mut self, text: &str) -> Result<()>;
pub fn set_image_watermark(
    &mut self,
    image_data: &[u8],
    image_filename: &str,
    width: Length,
    height: Length,
) -> Result<()>;
```

Each setter replaces one API-owned watermark in every active default, first,
and even header variant across all sections while preserving ordinary header
content and unrelated VML. Image relationships belong to their header part and
use the existing collision-safe media allocator.

Carry header-local image data through layout keyed by header relationship and
then local image relationship id. Lower recognized text and image watermarks to
backend-neutral group, text, and image elements with deterministic fonts. Clone
the selected group onto each applicable page before normal header and body
elements, which keeps it behind body text. Add even-header selection needed to
make the every-page contract true.

## Rejected alternatives

- Render every `w:pict`. That would misrender unrelated VML and violate the
  parse-only-what-you-render rule.
- Convert opened VML to DrawingML on save. That would discard verbatim producer
  bytes.
- Put the watermark in the body through `add_background_image`. That does not
  satisfy the header `w:pict` contract or header-local relationships.
- Add backend-specific watermark code. Shared layout elements already express
  the required drawing.
- Add a module, builder, or options object. Fixed defaults keep this S story
  bounded.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| golden, gate | `watermark_renders_behind_body_text_on_every_page` | Bundled-font multi-page output has exact pixels, a watermark on every page, and body content above it |
| unit | `word_vml_watermarks_parse_and_preserve_source_bytes` | Aliased text and image VML project typed values while source XML and unmodelled siblings survive byte-identical |
| unit | `generated_watermarks_write_fixed_prefixes_and_vml_child_order` | Authored text and image VML use canonical prefixes and order, then reparse identically |
| integration | `text_and_image_watermarks_round_trip_through_header_relationships` | Save and reopen retain header refs, header-local image relationships, media bytes, and typed projections |
| regression | `header_image_relationship_ids_are_scoped_per_part` | Body and multiple headers may each use `rId1` without resolving the wrong image |
| regression | `non_watermark_w_pict_remains_opaque` | Unsupported VML remains preserved and is not rendered |
| unit | `watermark_group_precedes_body_elements_on_every_page` | Default, first, and even pages emit the selected group before header and body content |

The **test gate**, from the backlog, is golden. A watermark renders behind body
text on every page.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Unit conversion**. Read HLD 01 units and the deliberately truncating
  constructor note in `CLAUDE.md`. Keep existing truncation and add exact
  point and EMU conversion coverage.
- **Layout, pagination, line breaking, and text shaping**. Read HLD 08. Use
  bundled deterministic fonts, check repetition and z-order on multiple pages,
  and update no baseline incidentally.
- **Any parser or serialiser**. Read HLD 04 and HLD 06. Check namespace aliases,
  fixed-prefix generated XML, VML child order, header-local relationships, and
  byte preservation of unmodelled subtrees.
- **Public API of published crates**. Read HLD 10 and the structural rules.
  State the additive high-level API and pre-1.0 low-level field additions, run
  the full package dry-run, and assert every archive stays below 10 MiB.

## Hash harness

Expected unchanged across all 49 entries. Existing samples do not author or
contain watermarks. Any delta is unrelated and blocks integration.

## Implementation checklist

- [ ] Parse recognized header VML text and image watermark projections while retaining raw bytes.
- [ ] Write canonical generated VML and preserve unrelated header content.
- [ ] Add native setters with header-local media relationships and atomic cache invalidation.
- [ ] Resolve header-local image ids without cross-part collisions.
- [ ] Lower text and image watermarks to deterministic neutral layout groups.
- [ ] Repeat the selected group behind body content on every default, first, and even page.
- [ ] Add preservation, scope, round-trip, z-order, and multi-page golden tests in existing files.
- [ ] Run scoped checks, risk riders, the full gate, and the unchanged hash harness.
- [ ] Update exactly HLD 03, HLD 04, HLD 08, HLD 10, and HLD 12.

## Open questions

None. Each active header carries one typed watermark, ordinary header content
and unrelated VML stay preserved, every section and header variant is covered,
and text authoring uses fixed Word-like defaults without an options object.

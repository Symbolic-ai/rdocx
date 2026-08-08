# F-111, add_picture

**Status**: completed
**Sprint**: S27
**Size**: M
**Depends on**: F-106, F-026

## Problem

F-106 left a content-hash `MediaStore` in the facade, including a private
insertion path at `crates/rpptx/src/lib.rs:211`, but no public operation can
place a picture on a slide. `CT_Picture` at
`crates/rpptx-oxml/src/picture.rs` can parse and write an existing picture but
cannot construct the required non-visual, blip-fill, and shape-property shell.

The operation must coordinate image sniffing, native-size calculation,
content-hash deduplication, slide-scoped relationships, tree-wide shape ids,
and schema-ordered picture XML. Native dimensions must use the completed F-026
intrinsic sizing behavior rather than the unrelated F-028 renderer work.

## Spec reference

- `docs/hld/01-glossary.md`, "Units" and "Relationship scope".
- `docs/hld/04-opc-and-packaging.md`, "Relationships", "Content types", and
  "Media".
- `docs/hld/06-presentationml-model.md`, "The shape tree" and "Shape ids".
- `docs/hld/14-development-backlog.md`, "F-111, add_picture".

## Approach

Keep F-111 independent of the F-109 handle lifecycle by adding one operation
to the owning facade:

```rust
impl Presentation {
    pub fn add_picture(
        &mut self,
        slide_index: usize,
        image_data: &[u8],
        image_filename: &str,
        left: Emu,
        top: Emu,
        width: Option<Emu>,
        height: Option<Emu>,
    ) -> Result<ShapeRef<'_>>;
}
```

Resolve and validate the target slide before mutating the package. Probe the
bytes with `oxml_media::probe`. When neither dimension is supplied, use
`native_size(72.0)`. When exactly one dimension is supplied, infer the other
from pixel aspect ratio with truncation toward zero. When both are supplied,
use them directly and do not require probe dimensions. Reject an unsupported
image or unavailable native size only when inference needs it, and leave the
presentation unchanged on every error path.

Insert bytes through `MediaStore` so equal bytes reuse one media part even when
called on different slides. Create or reuse a slide-scoped image relationship
to that part, allocate a tree-wide shape id immediately before append, and add
a picture at top z-order. Add a narrow `CT_Picture` constructor in the existing
`picture.rs` file. It emits canonical non-visual picture properties, a
relationship-backed `a:blipFill`, and typed position and extent in the existing
writer's schema order.

## Rejected alternatives

- Depend on F-109 and add only `SlideMut::add_picture`. The corrected story
  depends on F-106 and F-026 and can be implemented independently.
- Trust the filename extension. Existing media infrastructure sniffs bytes so
  misleading names cannot create an invalid MIME registration.
- Default to 96 DPI. The existing intrinsic-size contract and python-pptx
  parity use 72 DPI when metadata is absent.
- Duplicate the media part per slide. Relationship ids are slide-scoped, but
  media bytes are package-wide and F-106 requires content-hash deduplication.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `picture_without_explicit_size_uses_native_dimensions` | An in-code PNG with no explicit size reopens with exact 72-DPI native EMU dimensions |
| unit | `picture_constructor_round_trips_in_schema_order` | Id, name, relationship id, transform, and blip-fill serialize with fixed prefixes and reparse structurally |
| regression | `picture_one_dimension_preserves_aspect_ratio_with_truncation` | Width-only and height-only calls infer the other dimension toward zero |
| integration | `duplicate_picture_bytes_share_one_media_part_across_slides` | Equal bytes produce one media part and valid independent slide relationships |
| regression | `picture_sniffs_bytes_when_extension_is_misleading` | The stored extension and MIME follow the actual image format |
| negative | `invalid_picture_input_does_not_mutate_the_presentation` | Invalid slide, unsupported bytes, and missing native dimensions return errors with byte-identical package state |
| integration | `added_picture_validates_and_opens_without_repair` | `validate()` is empty, save plus reopen preserves bounds, and pinned PowerPoint reports no repair |

The backlog test gate is named explicitly: a picture added with no explicit
size uses its native dimensions.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/14-development-backlog.md`

Document the media insertion and slide-relationship behavior, the public
picture operation and sizing contract, and the corrected F-026 dependency.

## Risk routing

- Unit conversion and `Emu`: read `docs/hld/01-glossary.md`, "Units", and
  `CLAUDE.md`, "Things that are deliberately wrong". Assert exact 72-DPI EMU
  sizes and aspect-ratio truncation, and declare the hash harness unchanged.
- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add fixed-prefix, schema-order,
  relationship-target, reparse, and raw-subtree preservation checks.
- External oracle comparison: read `.claude/skills/differential-testing.md`.
  Pin python-pptx to `1.0.2` for intrinsic-size parity and PowerPoint to
  `16.104.25121423` for native acceptance, then record both results.

The existing `oxml-media` dependency and unpublished facade mean no new
dependency or published-public-API rider is added.

## Hash harness

Expected to be unchanged. Picture insertion is confined to unpublished
PresentationML packages and does not alter Word rendering output.

## Implementation checklist

- [x] Add the owning-facade `add_picture` operation and contextual errors.
- [x] Probe native size at 72 DPI and infer one missing dimension with pinned
  truncation.
- [x] Reuse `MediaStore`, create slide-scoped relationships, and allocate a
  tree-wide shape id.
- [x] Add the narrow picture constructor in the existing OOXML file.
- [x] Add native-size, aspect, deduplication, MIME, no-mutation, and reopen
  tests.
- [x] Run and record pinned python-pptx and PowerPoint comparisons.
- [x] Update exactly the three listed HLD files.

## Open questions

None. The approved scope uses an owning-facade method, optional dimensions,
72-DPI fallback, truncating aspect inference, and content-hash deduplication.

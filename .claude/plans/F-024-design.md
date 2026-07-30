# F-024, Image probing and DPI

**Status**: approved
**Sprint**: S05
**Size**: L
**Depends on**: F-023

## Problem

The only reusable header reader is the private JPEG dimension walk at
`crates/rdocx-pdf/src/image.rs:51`, and it reports dimensions alone. The shared
media contract at `docs/hld/04-opc-and-packaging.md:137` also requires format,
pixel dimensions, DPI, bit depth, channel count, and alpha metadata across PNG,
JPEG, GIF, BMP, and WebP. Every malformed or truncated header must return
safely.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, "Media".
- `docs/hld/12-testing-strategy.md`, "oxml-media".
- `docs/hld/14-development-backlog.md`, "F-024, Image probing and DPI".

## Approach

Extend `oxml-media/src/lib.rs` with the public `ImageInfo` record and
`probe(data: &[u8]) -> Option<ImageInfo>`. Dispatch through F-023 magic-byte
sniffing, then use bounds-checked format readers for PNG, JPEG, GIF, BMP, and
WebP.

PNG reads IHDR and optional `pHYs`, preserving unit 0 as unspecified DPI and
converting unit 1 pixels-per-metre to DPI. JPEG preserves the existing safe
marker walk, recognizes baseline and progressive SOF markers, reads JFIF units
1 and 2, and continues past APP and EXIF segments before SOF. GIF reads the
logical screen descriptor. BMP reads supported DIB headers and pixels-per-metre
metadata. WebP reads VP8, VP8L, and VP8X dimensions and alpha flags. Unsupported
or inconsistent headers return `None`.

Fixtures are constructed as byte vectors in the existing crate-root test
module. No binary fixture or decoding dependency is added.

## Rejected alternatives

- Use the PDF decoder as the shared API. It decodes pixels, depends on PDF
  concerns, and cannot report all required metadata.
- Accept partially parsed zero-valued records. `Option` keeps malformed input
  distinct from valid metadata.
- Add separate format modules. One bounds-checked reader per format remains
  locally understandable in the existing crate root.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `png_dimensions_and_phys_units_are_probed` | IHDR plus `pHYs` unit 0 and unit 1 produce the expected metadata. |
| unit | `jpeg_jfif_density_units_are_probed` | JFIF units 1 and 2 convert to the expected DPI. |
| regression | `jpeg_exif_before_progressive_sof_preserves_dimensions` | APP1 before SOF2 does not stop dimension discovery. |
| unit | `gif_bmp_and_webp_dimensions_are_probed` | Minimal in-code headers cover each required format and WebP layout. |
| regression | `every_truncated_supported_header_returns_without_panicking` | Every prefix of every fixture is safe and returns no invalid record. |

The test gate is dimension and DPI assertions per format, plus a truncation
loop `for n in 0..data.len()` that panics nowhere.

## HLD impact

None. The implementation follows the existing media parser contract.

## Risk routing

- Binary parser. Read `docs/hld/04-opc-and-packaging.md`, "Media", and
  `docs/hld/06-presentationml-model.md`, "Pictures". XML child-order, prefix,
  and raw-subtree obligations do not apply to binary headers. The equivalent
  preservation gate is a truncation loop for every format and explicit bounds
  checks before every indexed field.
- Public API of a reserved crate. Keep the API to the planned `ImageInfo` and
  `probe`, run the local package and size check, and leave publication disabled.

## Hash harness

Expected to remain unchanged. Only the isolated unpublished crate changes.

## Implementation checklist

- [ ] Add `ImageInfo` and format dispatch through F-023.
- [ ] Implement PNG dimensions and `pHYs` DPI semantics.
- [ ] Lift and extend the safe JPEG marker walk for JFIF, EXIF ordering, and
      progressive SOF.
- [ ] Implement GIF, BMP, and WebP metadata readers.
- [ ] Add in-code fixtures and every-prefix truncation loops.
- [ ] Run focused parser tests, package riders, and the unchanged hash gate.

## Open questions

None.

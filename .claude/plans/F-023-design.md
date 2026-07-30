# F-023, oxml-media format sniffing

**Status**: approved
**Sprint**: S05
**Size**: M
**Depends on**: none

## Problem

Image type handling is split across extension-only helpers in
`crates/rdocx/src/document.rs:2777` and the private JPEG detector in
`crates/rdocx-pdf/src/image.rs:23`. Trusting the filename means image bytes and
their declared MIME type can disagree. The staged format-neutral crate named by
`docs/hld/04-opc-and-packaging.md`, "Media", does not exist yet.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule" and "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, "Media".
- `docs/hld/11-migration-plan.md`, "Order of operations".
- `docs/hld/12-testing-strategy.md`, "oxml-media".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Create the explicitly planned `oxml-media` workspace crate at version 0.0.0
with `publish = false`. Keep the implementation and unit tests in `src/lib.rs`
so this story introduces only the manifest and crate root required by the new
crate.

Add the public `ImageFormat` variants `Png`, `Jpeg`, `Gif`, `Bmp`, `Tiff`,
`Webp`, `Svg`, `Emf`, and `Wmf`. Implement magic-byte sniffing, case-insensitive
extension lookup, canonical extension and content-type mappings, and
`resolve(data, filename)` with sniff-first, extension-second, PNG-default
precedence. Extension aliases such as `jpg`, `tif`, and `wmz` map to their
canonical formats without changing the canonical extension returned by the
enum.

Do not change any released rdocx crate, call site, dependency, or publication
workflow.

## Rejected alternatives

- Rewire the existing rdocx helpers now. F-027 owns that deferred consumer
  cutover and its intentional hash delta.
- Add a third-party image crate. Signature classification is small, and the
  architecture requires the staged crate to remain a cheap leaf.
- Split each format into a module. The current stories fit in one crate root,
  and extra files would add navigation without reducing cases.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `every_supported_format_sniffs_from_magic_bytes` | Every enum variant is identified from a minimal in-code signature. |
| unit | `extensions_and_content_types_are_canonical` | Aliases resolve while every format returns its canonical extension and MIME type. |
| regression | `sniffed_jpeg_overrides_a_misleading_png_extension` | JPEG bytes named `.png` resolve to JPEG. |
| unit | `unknown_image_defaults_to_png` | Unknown bytes and extension use the documented default. |
| regression | `every_signature_prefix_returns_without_panicking` | Every truncated signature is safe. |

The test gate is every supported format sniffs from magic bytes, and a `.png`
that is really a JPEG resolves to JPEG.

## HLD impact

None. The implementation realizes the existing media and staging contracts.

## Risk routing

- New crate, module, and files. F-023 explicitly authorizes `oxml-media` and
  its required manifest and crate root. Add no trait, generic parameter,
  feature flag, forwarding wrapper, or extra source file.
- Crate dependency graph. Inspect `cargo tree -p oxml-media --edges normal`
  and confirm the initial crate has no dependencies and no `rdocx-*` or
  `rpptx*` edge.
- Public API of a reserved crate. Treat the API as additive while version 0.0.0
  and unpublished. Run `cargo package -p oxml-media`, assert the archive is
  below 10 MiB, and confirm the seven-package release allowlist is unchanged.
- Version strings. Inspect the root manifest, `crates/oxml-media/Cargo.toml`,
  `Cargo.lock`, and `.github/workflows/publish.yml`. Confirm `publish = false`
  and do not tag or publish.

## Hash harness

Expected to remain unchanged. The crate is isolated and has no released rdocx
consumer. Any output delta blocks the sprint.

## Implementation checklist

- [ ] Add the unpublished 0.0.0 crate to the workspace.
- [ ] Add `ImageFormat` and the canonical mappings.
- [ ] Add safe magic-byte sniffing for every supported format.
- [ ] Add sniff-first `resolve` behavior and the PNG fallback.
- [ ] Run focused tests, dependency and package riders, and the unchanged hash
      gate.

## Open questions

None.

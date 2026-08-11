# F-028, add_picture_auto

**Status**: completed
**Sprint**: S32.2
**Size**: S
**Depends on**: F-026, F-027

## Problem

`Document::add_picture` at `crates/rdocx/src/document.rs:582` requires callers
to supply both dimensions even though `oxml_media::probe` and
`ImageInfo::native_size` already expose intrinsic size. Changing the existing
signature would break current callers, while silently inventing a size for
malformed or unsupported bytes would hide an input error.

## Spec reference

- `docs/hld/01-glossary.md`, EMU units and truncation.
- `docs/hld/04-opc-and-packaging.md`, native media size and 72 DPI parity.
- `docs/hld/11-migration-plan.md`, behavior-preserving consumer cutover.
- `docs/hld/14-development-backlog.md`, "F-028, add_picture_auto".

## Approach

Add `Document::add_picture_auto(&mut self, image_data: &[u8],
image_filename: &str) -> Result<Paragraph<'_>>`. Probe and calculate
`native_size(72.0)` before mutating the document, convert the returned EMUs
with `Length::emu`, then delegate to the existing `add_picture` method.

Add one concrete error variant carrying the filename when dimensions are
unavailable. A malformed or unsupported image returns that error before a
relationship, part, drawing identifier, or paragraph is added. F-027 supplies
the shared dependency and storage path first.

## Rejected alternatives

- Change `add_picture`. Existing callers require the explicit-size API.
- Default malformed input to a fixed size. That would create undocumented
  geometry and make failure non-obvious.
- Return `Option<Paragraph>`. Image probing failure is actionable input error
  information already represented by the crate result type.
- Duplicate probing in rdocx. `oxml-media` is the single owner.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `add_picture_auto_uses_native_size_at_72_dpi` | An in-code image with known pixels produces the exact EMU extent before and after round-trip |
| unit conversion | existing `oxml-media` native-size cases | Declared DPI precedence, 72 DPI fallback, and truncation remain pinned |
| negative | malformed image input | A typed error is returned and the document has no new part, relationship, drawing, or paragraph |
| regression | existing explicit-size picture tests | `add_picture` behavior and signature remain unchanged |
| packaging | rdocx dry-run and size check | The additive API packages against registry shared dependencies |

The backlog gate is that a picture without an explicit size matches its native
dimensions at a caller default of 72 DPI.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/14-development-backlog.md`

Record the public convenience method, atomic failure behavior, and the F-027
consumer dependency.

## Risk routing

- Public API of a published crate. State the additive semver effect and run
  rdocx package verification plus archive-size checks.
- Unit conversion. Use the shared EMU result directly, preserve truncation
  toward zero, and run declared and fallback DPI cases.
- Crate dependency graph. Confirm F-027's `rdocx -> oxml-media` edge remains
  one-way.
- Layout and rendering. Use deterministic fonts for any render evidence and
  require the existing harness to remain unchanged.

## Hash harness

Expected unchanged. Existing samples continue calling the explicit-size API.

## Implementation checklist

- [x] Add the additive auto-size method.
- [x] Add one typed unavailable-dimensions error.
- [x] Probe and size before any document mutation.
- [x] Delegate successful calls to the existing picture path.
- [x] Add exact extent, round-trip, and atomic-failure coverage.
- [x] Run media, rdocx, packaging, workspace, and hash gates.
- [x] Update exactly the two listed HLD files.

## Open questions

None. The caller default is fixed at 72 DPI by the approved compatibility
contract.

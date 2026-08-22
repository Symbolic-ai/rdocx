# F-X039, Share layout payloads and transfer reusable engines

**Status**: approved
**Sprint**: S52
**Size**: M
**Depends on**: F-X032, F-X037, F-X038

## Problem

`FontData::data` owns a `Vec<u8>` at
`crates/oxml-layout/src/output.rs:299`, and `LayoutResult::pages` owns
`Vec<PageFrame>` at `crates/oxml-layout/src/output.rs:345`. Cloning a complete
layout therefore deep-copies every used font and every positioned page.

`Document` retains its reusable normal engine privately at
`crates/rdocx/src/document.rs:115`, but `clone_for_staging` deliberately drops
it at `crates/rdocx/src/document.rs:455`. An editor that rebuilds one document
as another has no checked way to transfer safe reusable work.

## Spec reference

- `docs/hld/03-architecture.md`, "Why these seams".
- `docs/hld/08-rendering-spec.md`, "The seam that makes this cheap" and
  "Performance".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "The hash harness".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and
  "Packaging".

## Approach

Change `FontData::data` to `Arc<[u8]>` and `LayoutResult::pages` to
`Vec<Arc<PageFrame>>`. Update font loading, canonicalization, PDF, raster,
Word, and Presentation consumers to borrow through `Arc`. Constructors keep
accepting the new exact field types, with local `Arc::from` conversions at
ownership boundaries. This is an intentional pre-1.0 low-level API break with
no new wrapper or parallel result type.

Expand the reusable Word engine's context identity from the current partial
styles and theme key to an exact private identity for every layout input that
can affect retained work. Add one crate-hidden checked take operation on
`Engine`. Expose
`Document::transfer_reusable_layout_from(&mut self, source: &mut Document) -> bool`.
It builds the receiver context first, takes the source engine only when the
exact context is compatible, preserves the receiver's existing engine on
failure, and clears neither document's completed result cache. Self-transfer
is impossible because the method requires two mutable borrows.

## Rejected alternatives

- A second shared-result wrapper would make every caller choose between two
  representations and increase indirection.
- `Arc<LayoutResult>` alone does not share payloads when an owned result must
  be subdivided or a page tail retained.
- An unchecked public engine getter exposes cache mutation and stale-context
  hazards outside the facade.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `cloned_layout_shares_font_bytes_and_page_frames` | `Arc::ptr_eq` holds for every cloned font payload and page while all public values remain equal. |
| regression | `all_backends_accept_shared_layout_payloads_without_output_change` | PDF, raster, page access, caller-font layout, diagnostics, outlines, and Word provenance match the pre-change behavior. |
| regression | `compatible_document_transfer_reuses_normal_layout_work` | A rebuilt document with the same complete context takes the engine and records safe cache hits. |
| regression | `incompatible_or_failed_transfer_preserves_both_engines` | Changes to styles, theme, fonts, numbering, notes, headers, footers, media, fields, revision view, and poison state reject or recover without stale output or replacement. |
| regression | `transferred_warm_layout_equals_fresh_layout` | Pages, fonts, diagnostics, provenance, numbering, notes, fields, and outlines are exactly equal. |

The test gate is **regression**. Cloning complete layout results shares font
bytes and page frames, while PDF, raster, provenance, diagnostics, and visible
output remain identical. A transferred engine reuses safe work for the same
complete context, rejects or invalidates stale context, remains bounded and
poison-safe, and leaves deterministic and caller-font paths isolated. Both
WASM targets, package dry runs, and the hash harness pass unchanged.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`

## Risk routing

- Layout, pagination, and text shaping: re-read
  `docs/hld/08-rendering-spec.md`. Run every render baseline only in
  deterministic font mode and require byte-identical hashes.
- Public API of published crates: document the intentional pre-1.0 low-level
  break, run dry-run packaging for `oxml-layout`, `oxml-pdf`, `rdocx-layout`,
  `rdocx`, `rpptx-render`, and `rpptx`, then check archive sizes.
- WASM bindings: run both locked WASM checks and keep the shared ownership
  types free of host-only dependencies.

## Hash harness

Expected to be unchanged. Shared ownership changes allocation only, not output.

## Implementation checklist

- [ ] Change font bytes and pages to shared immutable ownership.
- [ ] Update every producer and consumer without adding wrapper types.
- [ ] Define the complete private reusable-engine context identity.
- [ ] Add checked engine take and the single native facade transfer method.
- [ ] Add pointer-sharing, compatibility, poison, warm-cold, and backend tests.
- [ ] Run focused layout, rendering, WASM, package, and hash checks.

## Open questions

None. The backlog fixes both ownership types and requires the smallest checked
facade transfer surface.

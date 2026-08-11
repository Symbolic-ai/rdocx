# F-046, rdocx layout and PDF cutover

**Status**: approved
**Sprint**: S32.2
**Size**: M
**Depends on**: F-030, F-037, F-047 through F-050, F-X005

## Problem

`rdocx-layout` still duplicates the format-neutral error, font, line, output,
and bundled-font implementations, while its flow engine constructs the old
types throughout `crates/rdocx-layout/src/engine.rs` and
`crates/rdocx-layout/src/paginator.rs`. `rdocx-pdf` remains a second backend
implementation tied to those duplicate types, and `rdocx::Error::Layout`
still wraps the old error at `crates/rdocx/src/error.rs:17`.

The shared contract has evolved to owned line parameters, content-addressed
media identifiers, non-exhaustive page and result structures, groups, paths,
backgrounds, and diagnostics. The cutover must adapt Word flow inputs without
changing pagination or rendered output.

## Spec reference

- `docs/hld/01-glossary.md`, unit conversions.
- `docs/hld/03-architecture.md`, format boundary and retained flow model.
- `docs/hld/08-rendering-spec.md`, shared layout and backend contract.
- `docs/hld/11-migration-plan.md`, line conversion boundary and cutover order.
- `docs/hld/14-development-backlog.md`, "F-046, rdocx layout and PDF cutover".
- `docs/hld/15-build-and-toolchain.md`, font modes, WASM, and packaging.

## Approach

After F-X005 publishes `oxml-layout` and `oxml-pdf` 0.1.2, make
`rdocx-layout` retain only the Word flow input, engine, paginator, blocks,
tables, and style resolver. Replace its duplicate neutral modules and bundled
font assets with direct `oxml-layout` types and font services. Re-export the
shared types needed by the high-level rdocx surface, while documenting public
module paths that necessarily change at this breaking cutover.

Add `crates/rdocx-layout/src/convert.rs` as the one Word conversion boundary.
Use concrete functions, not a trait or foreign inherent implementation, to map
tab stops, alignment, underline, line spacing, wrap behavior, and twips to the
owned `oxml-layout` line types. Convert input font files at the engine boundary.
Construct `PageFrame` and `LayoutResult` through their shared constructors.

Resolve DOCX relationship identifiers inside the retained flow engine and
emit images with bytes, content type, and `MediaId::from_bytes`. Remove the old
post-pagination `embed_id` fill pass. Preserve draw order, geometry, metadata,
outlines, deterministic fonts, and every existing flow-model decision.

Reduce `rdocx-pdf` to documented `pub use oxml_pdf::*`, set its description to
`deprecated: moved to oxml-pdf`, delete the duplicate backend modules, and
depend only on `oxml-pdf`. Point rdocx rendering directly through the shared
layout result and backend. Change `rdocx::Error::Layout` to wrap
`oxml_layout::LayoutError`.

## Rejected alternatives

- Move the Word flow model into `oxml-layout`. Slides do not paginate, and the
  architecture keeps that model format-specific.
- Keep two output or backend implementations. They would continue to drift.
- Add an adapter trait. There is one Word conversion implementation today.
- Preserve `embed_id` in the shared output. It assumes one global relationship
  scope and is invalid for PresentationML.
- Correct layout behavior during the cutover. Any such change needs its own
  labelled feature and reviewed hash delta.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | conversion table cases | Every Word alignment, tab, leader, underline, spacing, wrap, and fractional twip input maps exactly |
| integration, gate | workspace compile | The retained flow engine consumes only shared neutral types and every rdocx caller compiles |
| integration, gate | shared layout error conversion | `rdocx::Error::Layout` contains `oxml_layout::LayoutError` |
| regression | image and field rendering tests | Media IDs, bytes, links, fields, metadata, outlines, and draw order survive the boundary |
| regression | deterministic PDF and PNG corpus | All existing Word sample hashes remain identical |
| dependency | cargo-tree assertions | `rdocx-layout -> oxml-layout` and `rdocx-pdf -> oxml-pdf`, with no reverse format-family edge |
| packaging | affected archive dry-runs | Registry 0.1.2 dependencies resolve, required assets remain present, and archives stay below 10 MiB |
| WASM | binding-safe target checks | Disabling system fonts continues to compile where required |

The backlog gate is a compiling workspace, the shared layout error type, and
an unchanged hash harness.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/11-migration-plan.md`
- `docs/hld/15-build-and-toolchain.md`

Convert staged and future-tense descriptions to the completed shared boundary,
record the concrete converter function, and remove stale duplicate-font and
backend ownership.

## Risk routing

- Crate dependency graph and cross-family uses. Inspect the full affected
  graph and preserve the documented inward edges.
- Layout, text shaping, and unit conversion. Run exact conversion cases and all
  deterministic render evidence without recording system-font baselines.
- Public API of published crates. Record the shared type identity,
  non-exhaustive output types, changed public line-module surface, and PDF shim
  as an intentional breaking cutover.
- File move and duplicate deletion. Account for every retained flow file,
  deleted neutral file, and deleted backend file. Require unchanged output.
- Bundled fonts and packaging. Inspect both layout archives, complete font and
  legal inventories, registry dependencies, and archive sizes.
- WASM. Run the repository's binding-safe target checks and the shared
  no-default-features layout test.

## Hash harness

Expected unchanged across all 28 entries. Any Word package or render delta is
a defect and blocks integration.

## Implementation checklist

- [ ] Install shared layout types and font services behind the retained flow model.
- [ ] Add the concrete Word-to-shared conversion module.
- [ ] Replace relationship-scoped image placeholders with resolved media IDs.
- [ ] Use shared constructors for pages and layout results.
- [ ] Delete duplicate neutral layout sources and bundled assets.
- [ ] Replace `rdocx-pdf` with the exact shared backend shim.
- [ ] Move rdocx rendering and layout error paths to shared types.
- [ ] Run conversion, layout, PDF, dependency, WASM, package, and hash gates.
- [ ] Update exactly the four listed HLD files.

## Open questions

None. The shared output contract and retained Word flow boundary are already
fixed by the completed extraction stories.

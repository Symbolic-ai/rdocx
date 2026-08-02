# F-092, rpptx-render skeleton and RenderInput

**Status**: approved
**Sprint**: S22
**Size**: M
**Depends on**: F-087, F-036

## Problem

The architecture names `rpptx-render`, but the workspace member list at
`Cargo.toml:3` has no such crate. The renderer input described at
`docs/hld/08-rendering-spec.md:318` also predates the frozen F-087 contract. It
stores raw slide parts directly in `RenderInput`, while
`docs/hld/07-inheritance-and-resolution.md:45` requires the rendering stage to
consume owned `ResolvedSlide` values and no PresentationML or DrawingML model
types.

Relationship IDs are scoped per source part. Without an explicit slide,
layout, and master map, three different `rId2` values can alias and attach the
wrong image. Media also lacks a deck-level byte store keyed by F-036's
content-addressed `MediaId`.

## Spec reference

- `docs/hld/03-architecture.md`, "The dependency rule" and "Why these seams".
- `docs/hld/07-inheritance-and-resolution.md`, "The output contract".
- `docs/hld/08-rendering-spec.md`, "The renderer's input".
- `docs/hld/14-development-backlog.md`, "F-092, rpptx-render skeleton and
  RenderInput".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Create the explicitly planned `crates/rpptx-render` workspace crate at version
0.0.0 with `publish = false`. Its normal dependency direction is
`rpptx-render -> rpptx-layout`, `rpptx-oxml`, `oxml-layout`, and `oxml-media`.
No `oxml-*` crate gains a reverse dependency.

Separate assembly from rendering in one concrete module. `SlideBundle` holds
the raw slide, shared layout, shared master, shared theme, optional notes,
hidden flag, and `RelScopes` needed to construct the frozen resolver output.
`RelScopes` owns three maps and requires an explicit `RelScope` selector for
lookup. `ResolvedRel` records the resolved package target and relationship
type. It never performs a global relationship-ID lookup.

`MediaData` owns bytes and content type. A concrete media resolver takes the
selected scope and relationship ID, reads the resolved target bytes, derives
`MediaId::from_bytes`, and inserts once into the deck media map. Equal bytes
deduplicate, while equal relationship IDs in different scopes remain distinct.

The actual renderer boundary is:

```rust
pub struct RenderInput {
    pub slides: Vec<ResolvedSlide>,
    pub media: HashMap<MediaId, MediaData>,
    pub fonts: Vec<FontFile>,
    pub metadata: Option<DocumentMetadata>,
}
```

This preserves F-087. `SlideBundle` is the upstream assembly type that later
facade work resolves before constructing `RenderInput`. F-092 supplies the
types and media-resolution seam, not shape-to-page rendering from F-093.
Tests remain in the crate's existing `lib.rs` test module so no additional
integration binary is added.

## Rejected alternatives

- Put raw OOXML parts directly in `RenderInput`. That contradicts the frozen
  M9 renderer boundary and lets later rendering duplicate inheritance logic.
- Merge the three relationship maps. Relationship IDs are local to their
  source parts and collisions are valid.
- Store media by relationship ID or part name. Neither key deduplicates shared
  bytes across a deck.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `same_relationship_id_resolves_independently_in_all_three_scopes` | Slide, layout, and master `rId2` values select their own targets and media IDs |
| unit | `equal_media_bytes_deduplicate_to_one_media_entry` | Content-addressed `MediaId` collapses a shared logo |
| unit | `missing_relationship_reports_scope_and_id` | Lookup failure retains enough source context to diagnose |
| unit | `render_input_contains_only_resolved_slides` | The rendering boundary consumes F-087 output rather than raw OOXML |
| integration | `rpptx_render_dependency_direction_is_one_way` | The new crate depends inward and no `oxml-*` crate depends on it |

The backlog test gate is
`same_relationship_id_resolves_independently_in_all_three_scopes`.

## HLD impact

- `docs/hld/08-rendering-spec.md`

## Risk routing

- Crate dependency graph and new uses across families. Read
  `docs/hld/03-architecture.md`. Run
  `cargo tree -p rpptx-render --edges normal` and inspect that every edge points
  inward with no `oxml-* -> rpptx-*` dependency.
- A new crate, module, and files. `CLAUDE.md` requires explicit approval. F-092
  explicitly names the `rpptx-render` skeleton, and the invoked sprint
  authorises that planned crate. Keep the crate to `Cargo.toml` and one
  `src/lib.rs` file.
- Public API in unpublished crates. Record that `rpptx-render` remains version
  0.0.0 with publication disabled. Inspect the manifest and lockfile diff and
  run the full publication dry-run gate without publishing.

## Hash harness

Expected to be unchanged. The new unpublished skeleton is not connected to the
released Word render path.

## Implementation checklist

- [ ] Add the unpublished workspace crate and one-way dependencies.
- [ ] Define `RelScope`, `RelScopes`, and contextual relationship errors.
- [ ] Define `SlideBundle` as the upstream assembly boundary.
- [ ] Resolve scoped media targets to content-addressed `MediaId` entries.
- [ ] Define `RenderInput` over frozen `ResolvedSlide` values.
- [ ] Reconcile the HLD input example with the frozen M9 contract.

## Open questions

None. F-087 and the lower-level architecture contract require resolved slides
at the rendering boundary. `SlideBundle` remains the explicitly requested
assembly type rather than weakening that boundary.

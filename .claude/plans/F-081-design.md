# F-081, ResolveCtx skeleton and placeholder chain

**Status**: completed
**Sprint**: S20
**Size**: M
**Depends on**: F-071

## Problem

The PresentationML model already exposes slide, layout, and master shape trees
at `crates/rpptx-oxml/src/slide_parts.rs:38`, but no crate owns the hierarchy
walk that turns them into one inheritance context. Ordinary shapes expose a
placeholder at `crates/rpptx-oxml/src/shape_tree.rs:169`, and
`PlaceholderKey::matches` already implements index-first and type-fallback
matching at `crates/rpptx-oxml/src/placeholder.rs:265`. Callers currently have
to assemble those pieces themselves.

## Spec reference

- `docs/hld/06-presentationml-model.md`, "Placeholders".
- `docs/hld/07-inheritance-and-resolution.md`, "The chains" and "The
  resolver".
- `docs/hld/14-development-backlog.md`, "F-081, ResolveCtx skeleton and
  placeholder chain".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Create the architecture-defined, unpublished `rpptx-layout` crate at version
`0.0.0`. Add workspace dependencies on `oxml-drawing` and `rpptx-oxml`, with no
reverse dependency from an `oxml-*` crate.

Add `ResolveCtx<'a>` in `src/context.rs`. Its constructor accepts concrete
references to `CT_OfficeStyleSheet`, `ColorMap`, `CT_SlideMaster`,
`CT_SlideLayout`, `CT_Slide`, and `CT_TextListStyle`. Keep these as concrete
types because there is no second implementation for a trait or generic
abstraction today.

Implement a crate-visible placeholder chain returning the matching layout and
master `CT_Shape` references. Walk ordinary shapes recursively through groups
and the selected `mc:Fallback` branch. Match the slide placeholder to the
layout first, then match the layout placeholder key to the master. When no
layout match exists, return no master match rather than skipping a level.

The crate layout reserves focused modules for the current sprint's named
consumers: `context.rs` for F-081 and F-082, `text.rs` for F-083, `style.rs`
for F-084, and `font.rs` for F-085. A module is added to `lib.rs` only when its
story implements it.

## Rejected alternatives

- Put resolution in `rpptx-oxml`. Resolution walks part relationships and is
  not an XML model responsibility.
- Add a shape trait covering every shape-tree variant. Only ordinary
  `CT_Shape` has a current resolver consumer, so a second implementer does not
  exist today.
- Match a master placeholder directly from the slide when layout matching
  fails. That skips the documented hierarchy and can select a different
  equivalence class.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `slide_placeholder_resolves_to_layout_and_master_counterparts` | A slide placeholder resolves to the matching layout placeholder and then its master counterpart |
| unit | `master_match_uses_the_layout_placeholder_key` | The second hop uses the matched layout identity rather than rematching the slide directly |
| unit | `placeholder_lookup_walks_groups_and_selected_fallbacks` | Nested ordinary shapes remain eligible for inheritance lookup |
| unit | `missing_or_non_placeholder_shape_has_no_chain` | Missing layout matches and non-placeholder shapes return no inherited shapes |

The backlog test gate is named explicitly:
`slide_placeholder_resolves_to_layout_and_master_counterparts`.

## HLD impact

None. The architecture and resolver HLD already assign this boundary to
`rpptx-layout`.

## Risk routing

- Crate dependency graph. Run
  `cargo tree -p rpptx-layout --edges normal` and confirm the direction is
  `rpptx-layout -> rpptx-oxml -> oxml-drawing`, with no new reverse edge.
- A new crate, module, or file. Obtain explicit approval for
  `crates/rpptx-layout/Cargo.toml`, `src/lib.rs`, `src/context.rs`,
  `src/text.rs`, `src/style.rs`, and `src/font.rs` before implementation. The
  latter three files are justified by the existing F-083, F-084, and F-085
  consumers in this sprint.
- Version strings. Inspect the root manifest, new crate manifest, and lockfile.
  Require `version = "0.0.0"` and `publish = false`. No release action is part
  of this sprint.

## Hash harness

Expected to be unchanged. The resolver does not change Word rendering.

## Implementation checklist

- [x] Add `rpptx-layout` to the workspace and workspace dependency table.
- [x] Create its unpublished `0.0.0` manifest and focused source layout.
- [x] Add the concrete `ResolveCtx<'a>` constructor and stored inputs.
- [x] Walk nested ordinary shapes and selected fallback branches.
- [x] Resolve slide to layout to master through existing placeholder keys.
- [x] Run focused tests and the dependency-direction rider.

## Open questions

None. The user approved creating the `rpptx-layout` crate and the six named
files on 2026-08-02.

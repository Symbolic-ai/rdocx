# F-052, Create oxml-drawing and namespace constants

**Status**: approved
**Sprint**: S12
**Size**: S
**Depends on**: none

## Problem

The architecture assigns DrawingML colour, transforms, geometry, fills, lines,
effects, themes, and text bodies to `oxml-drawing` at
`docs/hld/03-architecture.md:12`, but the workspace member list currently jumps
from `oxml-core` to `oxml-layout` at `Cargo.toml:3`. There is no crate in which
the S12 DrawingML model or its canonical namespace constants can compile.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace", "The
  dependency rule", and "Versioning".
- `docs/hld/05-drawingml-model.md`, "Modules" and "Preservation".
- `docs/hld/14-development-backlog.md`, "F-052, Create oxml-drawing and
  namespace constants".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Create `crates/oxml-drawing` as an unpublished development crate at version
0.0.0. Add only the files the usable skeleton needs today: `Cargo.toml`,
`src/lib.rs`, and `src/namespace.rs`. Register it as a workspace member and
workspace dependency.

Expose fixed DrawingML main and picture namespace constants with their `a` and
`pic` prefixes. Do not add parser dependencies, an error wrapper, a feature
flag, a trait, or a generic parameter until a current story uses one. The new
crate starts dependency-free, so the required family direction holds by
construction.

## Rejected alternatives

- Reuse the constants inside `rdocx-oxml`. That would make the shared crate
  depend on the Word family and violate the dependency rule.
- Add all future DrawingML modules as empty files. Empty forwarding modules
  increase the places a reader must inspect and have no current implementer.
- Add PresentationML or ChartML namespace constants. Their owning crates are
  later stories and they are not part of the DrawingML main or picture model.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `drawingml_namespace_uris_match_the_specification` | The `a` and `pic` namespace URIs and fixed prefixes equal the ECMA-376 values. |
| integration | `oxml_drawing_is_an_unpublished_workspace_member` | Cargo metadata reports version 0.0.0, `publish = false`, and no normal dependency. |

The **test gate** is: crate compiles, namespace URIs match the spec.

## HLD impact

None. The architecture and build documents already describe this crate and its
development publication state.

## Risk routing

- **Crate dependency graph and a new crate or files**: read
  `docs/hld/03-architecture.md` and the structural rules in `CLAUDE.md`. The
  extra checks are `cargo check -p oxml-drawing --all-targets`, Cargo metadata
  inspection, and a dependency scan proving no `rdocx-*` or `rpptx-*` edge.
  The explicit S12 F-052 invocation authorises the crate and its three required
  files.
- **Version strings**: read `.claude/commands/release.md`. Inspect the root
  manifest, new crate manifest, lockfile, and README diff. Confirm version
  0.0.0, `publish = false`, no release allowlist change, no tag, and no
  publication.
- **Packaging**: run
  `cargo package -p oxml-drawing --allow-dirty --no-verify`, inspect the file
  list, and confirm the archive is under 10 MiB.

## Hash harness

Expected to be unchanged. The new unpublished crate is not consumed by the
released Word path.

## Implementation checklist

- [ ] Add the workspace member and workspace dependency.
- [ ] Add the minimal unpublished crate manifest.
- [ ] Add the crate root and namespace module.
- [ ] Define and test the `a` and `pic` namespace constants and prefixes.
- [ ] Prove the crate has no forbidden or unused dependency.

## Open questions

None. The story explicitly authorises the new crate, and the minimal three-file
skeleton avoids speculative modules.

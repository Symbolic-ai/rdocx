# F-022, rdocx-opc deprecation shim

**Status**: approved
**Sprint**: S04
**Size**: S
**Depends on**: F-018

## Problem

After F-018 stages the format-neutral implementation, `rdocx-opc` would still
own the public modules and types at `crates/rdocx-opc/src/lib.rs:9`, while the
high-level error keeps wrapping `rdocx_opc::OpcError` at
`crates/rdocx/src/error.rs:8`. Direct consumers also retain the old crate in
`crates/rdocx/Cargo.toml:16`, `crates/rdocx-cli/Cargo.toml:25`, and
`crates/rdocx-wasm/Cargo.toml:19`.

The requested shim and consumer switch create a release-boundary conflict.
`rdocx-opc`, `rdocx`, and `rdocx-cli` are published packages, while the real
`oxml-opc` implementation must remain at 0.0.0 with `publish = false` until
PowerPoint development is complete. Landing the switch would make those
published packages depend on an unpublished implementation, and the mandatory
package dry-runs could resolve only the dependency-free placeholder on
crates.io.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace", "The
  dependency rule", and "Versioning".
- `docs/hld/10-bindings-spec.md`, "WASM" and "CLIs".
- `docs/hld/11-migration-plan.md`, "Order of operations" and "What happens to
  the published crates".
- `docs/hld/14-development-backlog.md`, "F-022, rdocx-opc deprecation shim".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".

## Approach

Once the publication gate permits the dependency, reduce `rdocx-opc` to a
deprecated `pub use oxml_opc::*` compatibility surface and set its package
description to `deprecated: moved to oxml-opc`. Keep the legacy crate name as a
type-identical path for downstream source compatibility.

Add the direct `oxml-opc` workspace dependency to `rdocx`, `rdocx-cli`, and
`rdocx-wasm`, then replace their Rust imports and qualified paths with
`oxml_opc`. Change `rdocx::Error::Opc` to wrap `oxml_opc::OpcError`. Remove
their direct `rdocx-opc` dependencies only after an exhaustive path search is
clean. Do not alter runtime package behavior or add a publication path for any
development crate.

The underlying error type remains identical because `rdocx-opc` re-exports it.
The planned semver effect is the intentional deprecation warning on the legacy
crate surface, with existing `rdocx_opc::*` type paths remaining usable.

## Rejected alternatives

- Keep duplicate OPC implementations. They would drift and violate the
  one-owner architecture.
- Publish the real `oxml-opc` implementation to make the shim packageable now.
  The user has prohibited `oxml-*` and `rpptx*` development-crate publication
  until PowerPoint development is complete.
- Land only the re-export and leave direct consumers on `rdocx-opc`. The story
  explicitly requires the consumers and `rdocx::Error::Opc` to move to the new
  type path.
- Remove `rdocx-opc`. Existing downstream users need the compatibility path,
  and the migration plan explicitly retains the deprecated crate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `rdocx_error_opc_wraps_the_shared_error_type` | An `oxml_opc::OpcError` converts to `rdocx::Error::Opc` and the variant contains the shared type |
| integration | workspace compile gate | `rdocx`, `rdocx-cli`, and `rdocx-wasm` compile after all direct imports move to `oxml_opc` |
| regression | legacy shim compile assertion | A downstream-shaped use of `rdocx_opc::OpcPackage` still resolves to the shared implementation |
| integration | package dry-runs and archive inspection | Every still-publishable rdocx package resolves the real dependency, and each archive remains below 10 MiB |

The backlog test gate is that the workspace compiles and
`rdocx::Error::Opc` wraps the new type.

## HLD impact

- `docs/hld/11-migration-plan.md`
- `docs/hld/15-build-and-toolchain.md`

The updates replace stale pre-0.5 shim timing and describe the point at which a
published rdocx package may begin depending on the real shared implementation.

## Risk routing

- Crate dependency graph and new uses across families. Confirm every new edge
  points from `rdocx-*` to `oxml-opc`, run `cargo tree -p oxml-opc`, and prove
  no `oxml-*` crate depends on either format family.
- Public API of published crates. State the intentional deprecation and
  type-identity impact, run `cargo publish --dry-run` for every affected
  publishable package, and assert every `.crate` archive remains below 10 MiB.
- Version string and publication boundary. Inspect all manifest and lockfile
  changes, confirm `oxml-opc` remains 0.0.0 with `publish = false`, and require
  a clean full gate. Do not create a tag or start publication.

## Hash harness

Expected to remain unchanged. The shim and direct import switch must not change
package bytes or rendered output.

## Implementation checklist

- [ ] Wait for F-018 to establish the shared implementation.
- [ ] Resolve the publication gate before changing any published package
      dependency.
- [ ] Convert `rdocx-opc` to the deprecated re-export surface and update its
      package description.
- [ ] Flip every direct consumer manifest and Rust path to `oxml-opc`.
- [ ] Change `rdocx::Error::Opc` and prove the legacy and shared paths have the
      same underlying error type.
- [ ] Inspect the complete manifest and lockfile diff and run the mandatory
      package dry-runs, archive-size checks, workspace gate, and hash harness.
- [ ] Update exactly the listed HLD files to the approved publication timing.

## Open questions

None. Carry F-022 until the real `oxml-opc` implementation may be published
after PowerPoint development is complete. The rdocx 0.5.0 boundary protects
that release but does not make the required dependency available to later
rdocx package dry-runs.

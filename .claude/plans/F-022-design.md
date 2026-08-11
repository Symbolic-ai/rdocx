# F-022, rdocx-opc deprecation shim

**Status**: approved
**Sprint**: S32.2
**Size**: S
**Depends on**: F-018, F-X005

## Problem

`rdocx-opc` still owns duplicate public modules at
`crates/rdocx-opc/src/lib.rs:9`, and `rdocx::Error::Opc` still wraps the old
crate path at `crates/rdocx/src/error.rs:8`. The high-level library, CLI, and
WASM packages also retain direct `rdocx-opc` dependencies and imports.

The exact re-export shim removes two Word-specific inherent constructors that
cannot exist on the foreign shared type. Current callers of
`OpcPackage::new_docx()` therefore need an explicit format-specific setup at
the consumer boundary rather than a wrapper or a new shared constructor.

## Spec reference

- `docs/hld/03-architecture.md`, dependency direction and crate ownership.
- `docs/hld/04-opc-and-packaging.md`, generic OPC construction and content
  types.
- `docs/hld/10-bindings-spec.md`, WASM package construction.
- `docs/hld/11-migration-plan.md`, cutover order and deprecated crates.
- `docs/hld/14-development-backlog.md`, "F-022, rdocx-opc deprecation shim".
- `docs/hld/15-build-and-toolchain.md`, publication and release process.

## Approach

After F-X005 publishes `oxml-opc` 0.1.1, reduce `rdocx-opc` to crate docs and
`pub use oxml_opc::*`. Set its package description exactly to
`deprecated: moved to oxml-opc`, remove its four obsolete implementation files
and direct implementation dependencies, and depend only on `oxml-opc`.

Move `rdocx`, `rdocx-cli`, and `rdocx-wasm` manifests and Rust paths directly
to `oxml-opc`. Change `rdocx::Error::Opc` to the shared error type. Replace the
two `new_docx()` call sites with `OpcPackage::with_main_part` plus the Word main
part content type and the styles override previously installed by
`ContentTypes::new_docx()`. Keep that Word setup in the two existing consumer
files. Do not add a Word constructor to `oxml-opc` and do not wrap its types.

The legacy crate path remains type-identical for retained APIs such as
`rdocx_opc::OpcPackage`. The removed `new_docx` and `ContentTypes::new_docx`
methods are an intentional breaking surface documented by F-051. Cargo exposes
the package description and crate documentation as the whole-crate deprecation
signal, not a compiler warning on every re-export.

## Rejected alternatives

- Keep duplicate implementations. They would drift and violate one-owner
  architecture.
- Add `new_docx` to `oxml-opc`. A format-neutral leaf must not know Word setup.
- Wrap `OpcPackage` in the shim. That would break type identity and create a
  forwarding-only construct.
- Leave direct consumers on `rdocx-opc`. The story requires them to use the
  shared implementation directly.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `rdocx_error_opc_wraps_the_shared_error_type` | An `oxml_opc::OpcError` converts to the high-level error and remains the shared type |
| integration | new-document and WASM construction tests | Explicit generic OPC setup reproduces the Word main part, content types, relationships, and styles part |
| regression | legacy shim compile assertion | A retained `rdocx_opc::OpcPackage` path is the shared type |
| integration | workspace compile with binding exclusions | Every direct consumer compiles after the path switch |
| WASM | `cargo check --target wasm32-unknown-unknown -p rdocx-wasm` | The consumer-side Word setup remains target-safe |
| packaging | affected package dry-runs | Registry `oxml-opc` 0.1.1 resolves and every archive verifies below 10 MiB |

The backlog gate is a compiling workspace and an `rdocx::Error::Opc` variant
that wraps the shared type.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/11-migration-plan.md`
- `docs/hld/15-build-and-toolchain.md`

Replace the stale `new_docx` WASM path and future publication wording with the
explicit consumer setup and published 0.1.1 shared boundary.

## Risk routing

- Crate dependency graph and cross-family uses. Confirm all edges point from
  `rdocx-*` to `oxml-opc`, and that `oxml-opc` has no format-family dependency.
- Parser and serializer ownership cutover. Run existing OPC relationship,
  content-type, raw-preservation, and round-trip tests without changing shared
  parser or serializer behavior.
- Public API of published crates. Record the removed Word-specific constructors
  as breaking, preserve retained type paths, and run affected package dry-runs.
- WASM. Run the dedicated `rdocx-wasm` target check and the normal binding-safe
  workspace gate.
- Version strings and publication boundary. Verify every selected manifest
  resolves registry version 0.1.1 and make no tag or publication mutation here.

## Hash harness

Expected unchanged. Generic constructor use must reproduce the same Word
package bytes and renders.

## Implementation checklist

- [ ] Replace the duplicate crate with the exact shared re-export shim.
- [ ] Remove four obsolete modules and implementation dependencies.
- [ ] Move the library, CLI, and WASM consumers directly to `oxml-opc`.
- [ ] Rebuild Word package setup from generic constructors in existing files.
- [ ] Change and test the high-level OPC error type.
- [ ] Prove retained legacy type identity and document removed constructors.
- [ ] Run OPC, workspace, WASM, packaging, dependency, and hash gates.
- [ ] Update exactly the three listed HLD files.

## Open questions

None. F-X005 publishes `oxml-opc` 0.1.1 before this consumer cutover.

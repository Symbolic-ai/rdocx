# F-146, npm publication

**Status**: completed
**Sprint**: S36
**Size**: S
**Depends on**: F-140, F-142

## Problem

Both WASM crates build and run under Node, but neither produces a reviewed npm
tarball that a clean consumer can install. The sprint contract intentionally
stops at local packaging. It does not authorize registry publication, tags,
credentials, OIDC, or any other external mutation.

The current wasm-pack optimizer also lacks the WebAssembly feature flags
required by these generated modules. A release package needs the reviewed
`-Oz --enable-bulk-memory --enable-nontrapping-float-to-int` optimization.
The third flag is required because Rust 1.93 and later emit nontrapping
float-to-int operations in the linked standard library.

## Spec reference

- `docs/hld/10-bindings-spec.md`, WASM package names and profiles.
- `docs/hld/12-testing-strategy.md`, WASM CI and installation gates.
- `docs/hld/14-development-backlog.md`, "F-146, npm publication".
- `docs/hld/15-build-and-toolchain.md`, npm packaging and release boundaries.

## Approach

Add identical wasm-pack release metadata to both WASM manifests. Exact wasm-opt
125 runs `-Oz`, `--enable-bulk-memory`, and
`--enable-nontrapping-float-to-int`. Extend the existing CI WASM job, retaining
exact Node 24.11.1 and wasm-pack 0.15.0, to install the checksum-pinned optimizer
and build both crates with target `bundler`, scope `tensorbee`, release mode,
and locked Cargo dependencies.

For each generated exact scoped package, run `npm pack` into an isolated
temporary directory. Install its tarball into a fresh temporary consumer with
an isolated cache and scripts, audit, and funding disabled. Assert exact name
and version plus packaged WASM, JavaScript glue, and TypeScript declarations.

Add structured workflow regressions for the two-package set, exact target,
scope, pins, optimization, pack, and clean install. Explicitly reject any
publish command, registry authentication, token, OIDC grant, or release tag.

## Rejected alternatives

- Publish to npm now. The user and sprint contract explicitly defer registry
  publication.
- Add a publication workflow. Local pack and install are the complete gate.
- Use `--no-opt`. It avoids the tool issue by shipping materially larger
  artifacts and diverges from the reviewed F-142 optimizer path.
- Use the Node-only target. `bundler` is the npm/browser consumer surface.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| packaging, gate | `npm pack` for both scoped packages | One installable tarball is produced for each exact package |
| integration | fresh local consumer install | Package metadata, WASM, JavaScript glue, declarations, and imports survive clean installation |
| regression | structured CI contract | Exact pins, scope, target, optimization, package set, and install steps hold, with no publication authority |

Sensitivity removes either optimizer feature flag, removes one package,
changes scope or target, omits clean install, and adds a publish or
authentication step. The same structured contract must reject every mutation
before byte-identical restoration.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- WASM binding and release-profile metadata. Run both locked wasm32 checks,
  both Node suites, exact package inventories, and dependency trees.
- Version strings and packaging. Inspect both manifests and generated package
  versions. No release command or final publication approval is triggered
  because this story performs no external publication.

## Hash harness

Expected unchanged. npm packaging does not affect native sample generation.

## Implementation checklist

- [x] Add the reviewed optimizer metadata to both WASM manifests.
- [x] Add exact two-package bundler pack/install steps to existing CI.
- [x] Add structured positive and mutation-sensitive workflow regressions.
- [x] Run local tarball installation, WASM, dependency, and hash riders.
- [x] Confirm no publication authority or generated package artifact remains.

## Open questions

None. `bundler` is approved as the npm target, with identical exact wasm-opt
`-Oz --enable-bulk-memory --enable-nontrapping-float-to-int` release metadata
in both existing manifests. The third flag is approved from real Rust 1.93 and
1.97 toolchain evidence after the two-flag optimizer rejected the generated
`rdocx-wasm` module. No npm publication authority is granted.

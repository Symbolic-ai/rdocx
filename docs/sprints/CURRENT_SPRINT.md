# Current Sprint, S32.2

**Milestone**: M6 Shared publication and rdocx cutover.

**Goal**: After the real shared crates are published through their approved
release plan, move released rdocx consumers onto them and document the cutover.
Preserve existing call sites through facades and shims, admit only the declared
F-027 package metadata change, and prove released packages against registry
versions.

## Spec references

- `docs/hld/03-architecture.md`, for the final shared, WordprocessingML, and
  rendering crate boundaries plus the permitted dependency directions.
- `docs/hld/04-opc-and-packaging.md`, for shared OPC ownership, media naming,
  byte-sniffed content types, image probing, and intrinsic EMU sizing.
- `docs/hld/08-rendering-spec.md`, for the shared layout result and PDF backend
  contract that F-046 installs behind the retained rdocx flow model.
- `docs/hld/11-migration-plan.md`, for facade mechanics, cutover order,
  deprecation shims, behaviour preservation, and the approved release boundary.
- `docs/hld/12-testing-strategy.md`, for workspace, archive, deterministic
  rendering, focused package regressions, and hash-harness evidence.
- `docs/hld/14-development-backlog.md`, for F-015, F-016, F-022, F-027, F-028,
  F-046, and F-051 dependencies and test gates.
- `docs/hld/15-build-and-toolchain.md`, for registry publication order,
  package verification, split release families, and CI enforcement.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X005 | Tag rpptx-v0.1.2 | S | done | - |
| F-015 | rdocx-oxml becomes a facade | S | done | - |
| F-016 | Length re-export | S | done | - |
| F-022 | rdocx-opc deprecation shim | S | done | - |
| F-027 | rdocx adopts oxml-media | M | done | - |
| F-028 | add_picture_auto | S | done | - |
| F-046 | rdocx layout and PDF cutover | M | done | - |
| F-051 | CHANGELOG and migration notes | S | done | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order. F-X005 repairs the
registry metadata rejected during the immutable partial 0.1.0 publication and
the CI-only tool dependency exposed before upload by the immutable 0.1.1 run,
then prepares and publishes the incubating family at 0.1.2. Its verified
registry result is the common prerequisite for all consumer cutovers. F-028
follows F-027, and the remaining cutovers are ordered by file overlap during
implementation. F-051 follows the integrated cutovers so its migration notes
describe the final released surface rather than an intermediate state.

## Definition of done for this sprint

- The approved shared-crate versions exist in the registry, and a clean
  consumer resolves them without local patches.
- `rdocx-oxml` becomes the specified facade and `rdocx::Length` re-exports the
  shared type without call-site churn.
- `rdocx-opc` and `rdocx-pdf` are deprecated shims, direct consumers use the
  shared crates, and public error variants wrap the shared types.
- Released rdocx media handling uses `oxml-media`, and `add_picture_auto`
  produces intrinsic dimensions at 72 dpi.
- `rdocx-layout` retains its flow model while consuming shared layout types,
  and the PDF path uses the published shared backend through the approved
  conversion boundary.
- CHANGELOG and migration notes name every moved or deprecated crate and the
  breaking cutover surface.
- Released rdocx packages pass archive verification, the F-027 package
  regression proves sniffed media metadata, the full workspace gate passes,
  and all 28 hash entries remain unchanged.

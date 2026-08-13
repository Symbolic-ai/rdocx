# Current Sprint, S37

**Milestone**: M13 Bindings and tooling.

**Goal**: Prepare one fresh common version for the complete 14-package
incubating Rust family, then publish it only through the reviewed release
workflow after separate final approval. Preserve the immutable 0.1.2 release
and keep npm publication outside this sprint.

## Spec references

- `docs/hld/03-architecture.md`, for the split version trains, immutable
  0.1.2 release, and fresh-version requirement for the expanded family.
- `docs/hld/14-development-backlog.md`, for F-X006 dependencies and its exact
  registry and GitHub release acceptance gate.
- `docs/hld/15-build-and-toolchain.md`, for the 14-package allowlist,
  dependency order, archive checks, tag namespace, and separate final approval
  before publication.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X006 | Tag the expanded rpptx family | S | pending | - |

## Sequencing note

F-X006 is the only implementation story. Its dependencies F-143, F-144, and
F-145 are complete. Version preparation, full verification, and a clean sprint
review must finish before `/release rpptx-vX.Y.Z` asks for separate final
approval at the external mutation boundary.

## Definition of done for this sprint

- One fresh common version above 0.1.2 covers exactly the 14 incubating Rust
  packages and their workspace pins.
- The full verification gate and exact 21-package dry-run union pass with all
  archives below 10 MiB and all 28 deterministic hashes unchanged.
- The release command receives separate final approval at the reviewed SHA
  before creating or pushing the fresh `rpptx-v*` tag.
- All 14 incubating packages resolve from crates.io at the fresh version with
  the expected owner, and the GitHub release targets the reviewed sprint SHA.
- The immutable `rpptx-v0.1.2` release remains unchanged, and no npm package is
  published.

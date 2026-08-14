# Current Sprint, S40

**Milestone**: X Cross-cutting.

**Goal**: restore a green hosted CI baseline after runner and package-manager
updates exposed unpinned or incorrectly validated external tools. Preserve the
reviewed Poppler 26.01.0 rendering oracle and Binaryen 125 optimizer boundary
without changing product output or recorded rendering baselines.

## Spec references

- `docs/hld/12-testing-strategy.md`, for the exact Poppler rendering oracle and
  output-stability gates.
- `docs/hld/14-development-backlog.md`, for the F-X012 scope and hosted CI
  acceptance gate.
- `docs/hld/15-build-and-toolchain.md`, for deterministic external-tool
  installation and CI job ownership.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X012 | Restore pinned CI toolchains | M | pending | - |

## Sequencing note

There is one CI restoration story. It first pins and validates the shared
external tools, then runs the complete hosted workflow so platform-specific
failures are covered before sprint closure.

## Definition of done for this sprint

- Every CI job that executes a Poppler-dependent gate installs and verifies
  Poppler 26.01.0 from the checksum-pinned source.
- The WASM job accepts only the exact official Binaryen 125 Linux version
  identity after verifying the reviewed archive checksum.
- The workflow contract rejects missing pins, omitted jobs, weakened version
  checks, and package-manager drift.
- `/verify --full` passes with all 28 deterministic hashes unchanged.
- A hosted pull-request CI run at the reviewed SHA completes every job
  successfully.
- No crate, release version, package publication, or rendering baseline changes.

# Current Sprint, S39

**Milestone**: X Cross-cutting.

**Goal**: every Cargo workspace package has documentation at its package
boundary, and every crates.io-eligible package publishes that documentation at
the next minor version through its exact release family.

## Spec references

- `docs/hld/12-testing-strategy.md`, for README compilation and package
  inventory gates.
- `docs/hld/11-migration-plan.md`, for the stable family release boundary.
- `docs/hld/03-architecture.md`, for the incubating family version boundary.
- `docs/hld/14-development-backlog.md`, for the exact F-X009, F-X010, and
  F-X011 acceptance gates.
- `docs/hld/15-build-and-toolchain.md`, for package metadata and publication
  boundaries.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X009 | README coverage for every workspace crate | L | done | |
| F-X010 | Tag v0.6.0 | S | done | |
| F-X011 | Tag rpptx-v0.2.0 | S | pending | |

## Sequencing note

The documentation story completes first. The stable train then moves to 0.6.0
and publishes seven crates. After that release is verified, the incubating
train moves to 0.2.0 and publishes fourteen crates. The five `publish = false`
packages move with their local train but are not published.

## Definition of done for this sprint

- All 26 workspace packages declare a README.
- Every README states purpose, audience, package relationships, and a concrete
  usage example.
- Rust examples compile where applicable, and CLI, Python, and JavaScript
  examples pass exact syntax and package-name contracts.
- Every publishable archive contains exactly one intended README.
- `/verify --full` passes with all 28 deterministic hashes unchanged.
- All seven stable crates publish at 0.6.0 and render their README on crates.io.
- All fourteen incubating crates publish at 0.2.0 and render their README on
  crates.io.
- Python, WASM, npm, and PyPI publication authority remains unchanged.

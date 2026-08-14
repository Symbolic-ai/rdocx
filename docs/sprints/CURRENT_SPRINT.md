# Current Sprint, S38

**Milestone**: X Cross-cutting.

**Goal**: integrate PR 25 with its contributor credit intact, harden its new
Word composition APIs, and publish useful documentation for every stable
crate. After the integrated result passes full verification and sprint review,
prepare and release the complete stable family at 0.4.2 through the dedicated
release workflow.

## Spec references

- `docs/hld/04-opc-and-packaging.md`, for relationship ownership and
  package-preserving document mutation.
- `docs/hld/10-bindings-spec.md`, for the public facade and downstream binding
  compatibility boundary.
- `docs/hld/12-testing-strategy.md`, for round-trip, README, hash, and external
  contribution gates.
- `docs/hld/14-development-backlog.md`, for the two story contracts and their
  exact acceptance gates.
- `docs/hld/15-build-and-toolchain.md`, for stable package documentation,
  archive inventory, version preparation, and release authority.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X007 | Integrate PR 25 and stable crate documentation | L | in-progress | codex |
| F-X008 | Tag the updated stable rdocx family | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-X007 merges and hardens the contributor change before the stable version is
prepared. F-X008 depends on that reviewed implementation and requests separate
final approval immediately before the first release mutation.

## Definition of done for this sprint

- PR 25 is merged through GitHub into `sprint/s38` with a merge note that
  explains the value of the fix and credits Jon Stokes as `@jonstokes`.
- The new custom-list, numbering, hyperlink, hard-break, and fixed-table APIs
  preserve package state and pass their focused regression and round-trip
  gates.
- Every stable crate has current package documentation and a README that tells
  users when and how to use it. All Rust examples compile.
- `/verify --full` passes with all 28 deterministic hashes unchanged and every
  stable archive below 10 MiB.
- `/release v0.4.2` receives separate final approval at the reviewed SHA before
  creating or pushing the tag.
- All seven stable packages resolve from crates.io at 0.4.2 with the expected
  owner, and no incubating, WASM, Python, or npm package is published.

# Current Sprint, S32.1

**Milestone**: M6 Shared publication and rdocx cutover.

**Goal**: Make the completed shared crates packageable, fully gated, and ready
for an explicitly approved publication without publishing from this sprint.
Prepare deterministic package contents, split-family release preparation, the
expanded publication workflow, and the missing CI jobs as one reviewable wave.

## Spec references

- `docs/hld/11-migration-plan.md`, for the deferred shared-crate publication
  boundary, dependency-ordered release tooling, and later rdocx cutover.
- `docs/hld/12-testing-strategy.md`, for the package dry-run, archive-size,
  no-default-features, wasm, prose, and unchanged hash-harness gates.
- `docs/hld/14-development-backlog.md`, for F-047 through F-050 dependencies,
  focused test gates, and the M6 end-of-milestone gate.
- `docs/hld/15-build-and-toolchain.md`, for package include contents, bundled
  font licences, archive verification, dependency order, tag namespaces,
  release preparation, and the CI job matrix.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-047 | Packaging include and size gate | M | in-progress | codex |
| F-048 | Automate split-family release preparation | M | in-progress | codex |
| F-050 | CI matrix additions | S | in-progress | codex |
| F-049 | Extend publish.yml to the extracted workspace | M | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order. F-047 depends only on the
completed F-037 package boundary, while F-048 and F-050 may also begin
independently. F-049 starts after F-048 has established the stable and
incubating release preparation contract, then extends publication over the
full dependency graph using both tag namespaces.

## Definition of done for this sprint

- `cargo package --list` includes every required `oxml-layout` TTF and licence
  file, archive verification is enabled, and every candidate archive is under
  the crates.io 10 MiB limit.
- A dry-run release preparation updates `[workspace.package]`, every internal
  `[workspace.dependencies]` version pin, and the lockfile without rewriting
  README prose.
- The publication workflow supports the stable and incubating tag namespaces
  and dry-runs the expanded workspace graph in dependency order.
- CI exercises `oxml-layout` without default features, checks the supported
  wasm targets, and enforces the tracked Markdown prose rules on a clean tree.
- The full workspace gate passes and all 28 deterministic hashes remain
  unchanged unless a design plan declares and reviews a behavioural delta.
- No crate is published during S32.1. S32.2 remains blocked until an explicitly
  approved release places the shared versions in the registry and a clean
  consumer resolves them.

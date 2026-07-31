# Current Sprint, S11

**Milestone**: M6 deferred shared publication and rdocx cutover.

**Goal**: Verify the isolated shared crates and continue PowerPoint development
without publishing them or changing released rdocx dependencies. S11 is the
staged extraction validation boundary before DrawingML construction begins.

**Validation-only**: yes

## Spec references

- `docs/hld/03-architecture.md`, for the shared-crate dependency direction and
  the rule that implemented development crates stay at version 0.0.0 with
  publication disabled.
- `docs/hld/11-migration-plan.md`, for keeping released rdocx packages on their
  existing dependency graph until PowerPoint development and a separate shared
  publication plan are complete.
- `docs/hld/12-testing-strategy.md`, for the workspace, 28-entry hash, exact
  golden-PNG, packaging, and supply-chain validation gates.
- `docs/hld/14-development-backlog.md`, for the deferred M6 publication and
  consumer-cutover boundary and the M7 DrawingML work that follows it.
- `docs/hld/15-build-and-toolchain.md`, for deterministic rendering, archive
  verification, and the prohibition on publishing `oxml-*` or `rpptx*`
  development crates.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|

S11 has no implementation F-IDs. It is an explicit validation boundary.

## Sequencing note

There is no implementation order in S11. F-046 through F-051 remain deferred
to S32.1 and S32.2 after PowerPoint development and the separately approved
shared-crate publication. S12 begins M7 DrawingML construction only after this
staged boundary is confirmed.

## Definition of done for this sprint

- The full workspace, no-default-features, WASM, documentation, packaging, and
  supply-chain gates pass from a clean tree.
- All seven golden page-one buffers match exactly, the existing 28-entry hash
  harness remains unchanged, and the injected one-pixel proof still fails
  precisely.
- Each implemented `oxml-*` development crate remains at version 0.0.0 with
  `publish = false`, and no `oxml-*` crate depends on an `rdocx-*` or `rpptx-*`
  crate beyond the documented future Theme adapter exception.
- Released rdocx manifests and dependency edges remain unchanged.
- No crate is published and no consumer cutover runs.

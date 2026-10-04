# Current Sprint, S87

**Milestone**: X, cross-cutting release repair.

**Goal**: refresh the two CLI archive measurements made stale by the S86
Windows stack correction. Verify the resulting exact source through the full
local gate, hosted CI and a clean sprint review before both Issue 266 releases.

## Spec references

- `docs/hld/14-development-backlog.md`, F-X175, for the exact repair and its
  release regression test gate.
- `docs/hld/15-build-and-toolchain.md`, for package archives, release versions,
  the hosted gate and the unified publication boundary.
- `docs/hld/12-testing-strategy.md`, for the full integrated verification and
  unchanged hash harness.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X175 | Refresh CLI archive evidence after release hardening | S | done | - |

## Sequencing note

F-X175 depends on the completed S86 release preparations. It is the only S87
story. The previous related-story wave moved to S88 so both family releases
can use a new reviewed main merge after this repair.

## Definition of done for this sprint

- Fresh macOS and Linux source archives match the two CLI README measurements
  and the enforced inventory.
- The full local gate, 49 matching hash entries, hosted Docs and Release
  regressions, and a clean sprint review pass on the final S87 tree.
- `/close-sprint S87` merges only the reviewed result. Separate `/release`
  approvals publish `rpptx-v0.13.0` and `v0.15.0` from that merge SHA.

# Current Sprint, S88

**Milestone**: X, cross-cutting release repair.

**Goal**: repair the tag-only package inventory failure that stopped the first
PowerPoint release before publication. Prepare fresh PowerPoint and Word family
releases from one reviewed main merge, with a build-only rehearsal and the full
local and hosted gates before separate publication approvals.

## Spec references

- `docs/hld/14-development-backlog.md`, F-X176, for the exact repair and its
  release regression test gate.
- `docs/hld/15-build-and-toolchain.md`, for the unified package inventory,
  family versions, hosted workflow and publication boundary.
- `docs/hld/12-testing-strategy.md`, for the full integrated verification and
  unchanged hash harness.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X176 | Repair unified release inventory and respin PowerPoint | M | done | - |

## Sequencing note

F-X176 follows the completed S87 release repair and is the only S88 story.
The related-story wave moves to S89 so both family releases can use a new
reviewed main merge after this repair.

## Definition of done for this sprint

- A clean-target dry run yields exactly the selected unpacked package set for
  each family, with no missing or extra package directories.
- Fresh PowerPoint version carriers, the stable Word family contract, and both
  rendered release notes agree with the planned tags.
- The build-only artifacts, full local gate, hosted CI and a clean sprint
  review pass on the final S88 tree, with 49 unchanged hash entries.
- `/close-sprint S88` merges only the reviewed result. Separate `/release`
  approvals publish the two families from that merge SHA.

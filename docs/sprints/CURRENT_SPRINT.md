# Current Sprint, S42

**Milestone**: X Cross-cutting.

**Goal**: take the outstanding semver-compatible dependency updates and measure
what they do to rendered output. None of them is a security fix, since the
advisory scan is already clean, so the value is in keeping the lockfile from
drifting far enough that a later refresh becomes a large delta nobody can
attribute.

## Spec references

- `docs/hld/12-testing-strategy.md`, "The hash harness", for the rule that an
  intentional delta lands as its own labelled commit with the expected change
  stated. That rule is what this sprint turns on.
- `docs/hld/15-build-and-toolchain.md`, for the pinned toolchain the refresh
  must keep working against.
- `docs/hld/14-development-backlog.md`, for the F-X020 scope and its gate.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X020 | Refresh the dependency lockfile | S | done | - |

## Sequencing note

One story, so there is no order to explain. It is alone in the sprint on
purpose: a lockfile refresh that also moves a rendering baseline should not
share a sprint with unrelated work, because the two would compete to explain the
same delta.

## Definition of done for this sprint

- Every semver-compatible update outstanding at the sprint's start is taken, or
  the ones held back are named with a reason.
- `cargo audit` reports zero vulnerabilities and `cargo deny check` passes,
  with the `ttf-parser` unmaintained advisory still the single documented
  exception.
- The hash harness is either unchanged, or its delta names the dependency that
  caused it and was reviewed before the baseline was re-recorded.
- The pinned toolchain and MSRV still build the workspace.

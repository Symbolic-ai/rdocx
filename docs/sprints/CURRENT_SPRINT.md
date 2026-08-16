# Current Sprint, S44

**Milestone**: X Cross-cutting.

**Goal**: finish the job S43 started. S43 closed two gaps in the gates and found,
in passing, that the records describing those gates had drifted from them. This
sprint puts the two remaining gates where CI can see them and repairs the
documentation that tells every future session what is true.

## Spec references

- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate", for what F-X027 has
  to wire in, and "What CI runs" for the job table both gate stories extend.
- `docs/hld/15-build-and-toolchain.md`, the `publish.yml` paragraph, for the
  release preflights F-X026 brings under CI, and the pinned Poppler build
  F-X027 has to account for.
- `docs/hld/10-bindings-spec.md`, the wheel-building traps, which carry one of
  the `bundled-fonts` claims F-X028 corrects.
- `docs/hld/14-development-backlog.md`, for the three story definitions.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X026 | CI must run the release regressions too | S | in-progress | codex |
| F-X027 | Wire the golden-PNG gate into something | S | in-progress | codex |
| F-X028 | Repair the agent-facing documentation drift | M | pending | - |
| F-X029 | Path-filtered CI jobs | M | in-progress | codex |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

There is no hard dependency between these four, so the order is a preference
rather than a constraint and any of them may be claimed first. The one soft
coupling is that F-X026 and F-X029 both edit `ci.yml`, so whichever lands second
rebases on the first.

F-X026 leads because it is the narrower half of a gap S43 half-closed. `/verify`
runs the release preflights now and CI still does not, so a contributor who
skips the local gate can move a version carrier and see a green pull request.

F-X027 follows because the golden-PNG gate is fully specified and wired into
nothing, and deciding where it belongs needs a judgement about the pinned
Poppler build that F-X026 does not need.

F-X029 and F-X030 came out of a monorepo-versus-split review and are the two
concrete improvements that survived it. F-X029 pairs naturally with F-X026,
since both edit `ci.yml`, and whichever lands second rebases on the first.
F-X030 is independent of everything.

F-X028 lands last, and is the largest, because it is the only one that rewrites
`CLAUDE.md`. A story that changes the file every other session reads first
should land against a tree the other three have already settled.

Every implementation milestone is closed, so this sprint carries no feature
work. Three of the four stories exist because S43 went looking at the
instruments rather than the product. The fourth, F-X029, came out of a review of
whether the workspace should be split into separate repositories. The conclusion
was that it should not.

That review also produced F-X030, which was archived before the sprint started
once the WASM packages turned out to be deliberately unpublished, so its stated
problem does not exist. `docs/hld/02-scope-and-non-goals.md` records the
position.

## Definition of done for this sprint

- A stale version literal fails a named CI job, not only `/verify --full`, and
  not first at publication time.
- The golden-PNG gate runs somewhere that fails when nobody remembers it, and
  the spec set says where.
- Every path, version and feature name `CLAUDE.md` and
  `.claude/commands/verify.md` cite resolves against the workspace, and a test
  asserts it, so the next stale claim fails a gate rather than surviving 40
  sprints.
- `CLAUDE.md` no longer tells a reader that a false font licence notice ships
  today, that the family is on crates.io at 0.2.0, that the bundled fonts live
  under `rdocx-layout`, or that a `bundled-fonts` feature exists.
- A docs-only change reports every required check without running the workspace
  suite, the MSRV suite, the WASM targets or the fidelity job, and each filtered
  job has an asserted must-trigger and must-not-trigger path.
- The hash harness stays at 49 of 49. No story here touches rendering.

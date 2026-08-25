# Current Sprint, S58

**Milestone**: M20 Fidelity at scale.

**Goal**: close M20 and finish every planned non-spreadsheet capability before
the advanced spreadsheet programme begins. Use the corpus, SSIM, and
large-document gates established in S57 to measure language-aware line
breaking, complex-script and directional layout, and bounded incremental
relayout. Finish by making the stable aggregate CI check a required repository
protection at the reviewed sprint SHA.

## Spec references

- `docs/hld/03-architecture.md`, for dependency direction, shared line-breaking
  ownership, Word pagination, and the facade and engine cache boundary that
  incremental layout must preserve.
- `docs/hld/08-rendering-spec.md`, for Unicode line-break discovery, exact
  shaping and source-span behavior, vertical-text lowering, deterministic
  layout, and bounded paragraph and shaping reuse.
- `docs/hld/12-testing-strategy.md`, for external-oracle discipline, golden and
  SSIM evidence, deliberate render sensitivity, performance regression gates,
  and the always-reporting `ci-gate` contract.
- `docs/hld/14-development-backlog.md`, for the exact F-198, F-199, F-200,
  F-202, and F-X031 dependencies and acceptance gates.
- `docs/hld/15-build-and-toolchain.md`, for bundled-font deterministic output,
  cache ceilings, pinned corpus and oracle runtimes, the CI matrix, and the
  separation between tracked workflow state and repository protection state.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-198 | Hyphenation | L | pending | - |
| F-199 | Complex script shaping | L | pending | - |
| F-202 | Incremental layout | L | pending | - |
| F-200 | Vertical and bidirectional text | M | pending | - |
| F-X031 | Require the CI gate in branch protection | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-198, F-199, and F-202 may begin independently because their S57
prerequisites are complete. F-200 follows F-199 so directional layout builds on
the completed complex-script shaping contract. F-X031 is the final operational
step because the stable `ci-gate`, reviewed workflow, and sprint SHA must have
settled before repository protection is changed. F-198 is expected to move
rendered output, so its hash delta must be isolated and declared.

## Definition of done for this sprint

- Language-specific hyphenation produces the reviewed oracle line breaks and
  carries a declared deterministic hash delta.
- Arabic, Indic, Thai, and CJK text follow their shaping and line-breaking
  rules within the recorded corpus threshold.
- Mixed-direction runs and supported vertical text render in the correct visual
  order without losing preserved source content.
- Editing one paragraph of the thousand-page document re-lays out a bounded
  number of pages while the established memory and throughput limits remain
  green.
- The exact stable `ci-gate` becomes required for the protected branch without
  removing existing protections. A documentation-only pull request succeeds
  with expensive jobs skipped, and a selected failing job makes the aggregate
  gate fail. Evidence names the repository, branch pattern, protection
  identifier, and reviewed sprint SHA.
- The full workspace, corpus, fidelity, performance, package, and deterministic
  hash gates pass with only reviewed and declared output changes.

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
  F-202, F-X031, and F-X058 through F-X063 dependencies and acceptance gates.
- `docs/hld/15-build-and-toolchain.md`, for bundled-font deterministic output,
  cache ceilings, pinned corpus and oracle runtimes, the CI matrix, and the
  separation between tracked workflow state and repository protection state.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-X061 | Support staged dependency checkpoints in run-sprint | S | in-progress | codex |
| F-X062 | Reuse restart pagination with notes and headers | M | pending | - |
| F-X063 | Avoid duplicate caller-font byte comparisons | S | pending | - |
| F-X058 | Shared multilingual text substrate | L | in-progress | codex |
| F-X059 | Tag rpptx-v0.7.0 | S | pending | - |
| F-198 | Hyphenation | L | in-progress | codex |
| F-199 | Complex script shaping | L | pending | - |
| F-202 | Incremental layout | L | in-progress | codex |
| F-200 | Vertical and bidirectional text | M | pending | - |
| F-X060 | Tag v0.11.0 | S | pending | - |
| F-X031 | Require the CI gate in branch protection | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

F-X061 first makes ordinary and release dependency checkpoints resumable.
F-X062 and F-X063 then close the two reported retained-layout cliffs. F-X058
establishes the complete shared text contract, and F-X059 publishes it as
the incubating 0.7.0 family. F-198, F-199, and then F-200 consume that published
boundary. F-202 is already independently reviewed. F-X060 publishes the
settled stable family at 0.11.0. F-X031 is the final operational step
because the stable `ci-gate`, reviewed workflow, releases, and sprint SHA must
have settled before repository protection is changed. F-198 is expected to
move rendered output, so its hash delta must be isolated and declared.

## Definition of done for this sprint

- Language-specific hyphenation produces the reviewed oracle line breaks and
  carries a declared deterministic hash delta.
- Arabic, Indic, Thai, and CJK text follow their shaping and line-breaking
  rules within the recorded corpus threshold.
- Mixed-direction runs and supported vertical text render in the correct visual
  order without losing preserved source content.
- The complete incubating 0.7.0 and stable 0.11.0 crate families publish from
  their separately approved reviewed SHAs, and their registry graphs verify.
- Each dependency release checkpoint records full verification and clean review
  at its release SHA, then resumes later waves in the same sprint state.
- Unchanged notes, endnotes, headers, and footers retain safe restart
  pagination, and changed related stories invalidate it exactly.
- Warm caller-font relayout avoids the redundant retained-context byte pass
  while exact changed-font and transfer checks remain authoritative.
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

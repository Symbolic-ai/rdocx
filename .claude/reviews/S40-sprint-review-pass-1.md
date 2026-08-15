# S40 sprint review, pass 1

**Reviewed**: `sprint/s40` at
`204de165fc790f9273f0b5365ef8fe9f65c5bcbd` against merge base
`499b070da8c83f3268018b2f63cc27fd7c0ca8d7`, 22 files and 2,748 changed
lines, 2,693 additions and 55 deletions, crates: none
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Sprint contract and interaction

- S40 contains one completed story, so there is no cross-feature ordering or
  helper interaction to reconcile. The sprint goal, wave, and definition of
  done consistently require restored hosted CI, exact Poppler 26.01.0 and
  Binaryen 125 boundaries, unchanged output, and no package or baseline change
  (`docs/sprints/CURRENT_SPRINT.md:5`,
  `docs/sprints/CURRENT_SPRINT.md:23`,
  `docs/sprints/CURRENT_SPRINT.md:31`,
  `docs/sprints/CURRENT_SPRINT.md:42`). The roadmap carries the same single
  story and boundary (`docs/sprints/SPRINT_PLAN.md:617`,
  `docs/sprints/SPRINT_PLAN.md:632`).
- Test, MSRV, both Python binding rows, and Presentation fidelity use the one
  shared Poppler installer before their dependent work
  (`.github/workflows/ci.yml:31`, `.github/workflows/ci.yml:69`,
  `.github/workflows/ci.yml:235`, `.github/workflows/ci.yml:384`). The installer
  owns exact source provenance, bounded streaming extraction, an absent prefix,
  the three required binaries, and their runtime identities
  (`scripts/install_pinned_poppler.py:22`,
  `scripts/install_pinned_poppler.py:24`,
  `docs/hld/15-build-and-toolchain.md:345`,
  `docs/hld/15-build-and-toolchain.md:356`).
- Test and MSRV additionally pin uv, scope the runner-temporary cache and 8 MiB
  stack to the broad suite, install the exact Ubuntu LibreOffice oracle, and
  propagate failures (`.github/workflows/ci.yml:26`,
  `.github/workflows/ci.yml:40`, `.github/workflows/ci.yml:44`,
  `.github/workflows/ci.yml:379`, `.github/workflows/ci.yml:393`,
  `.github/workflows/ci.yml:397`). The WASM job verifies the reviewed Binaryen
  checksum before requiring the exact official Linux identity
  (`.github/workflows/ci.yml:127`, `.github/workflows/ci.yml:137`).
- The central regression suite rejects missing, conditional,
  failure-tolerant, and successfully short-circuited Poppler consumers, plus uv,
  cache, stack, Binaryen, LibreOffice, archive, provenance, identity, and
  orchestration mutations (`scripts/test_sprint_workflow.py:813`,
  `scripts/test_sprint_workflow.py:850`,
  `scripts/test_sprint_workflow.py:852`,
  `scripts/test_sprint_workflow.py:930`,
  `scripts/test_sprint_workflow.py:932`,
  `scripts/test_sprint_workflow.py:1473`). All 46 workflow regressions pass in
  this review.

## Milestone gate

F-X012's HLD gate requires behavioral regressions for every source, resource,
runtime, and prefix guard, mutation-sensitive consumer and tool pins, full
verification, a successful hosted pull-request run at the reviewed SHA, and
all 28 hashes unchanged (`docs/hld/14-development-backlog.md:1310`,
`docs/hld/14-development-backlog.md:1317`). It holds.

- GitHub Actions run `31853529961` is a completed successful pull-request run
  at `e96217f88b9dfd4612913787bc736f3627f73092`. All 14 expanded jobs succeeded.
  The Test and MSRV logs show pinned Poppler and LibreOffice installation before
  their complete workspace suites. The fidelity job covered all 421 slides.
  The completion ledger binds that run and SHA to the delivered story
  (`docs/sprints/AS_BUILT.md:5778`,
  `docs/sprints/AS_BUILT.md:5782`).
- The only changes after that hosted implementation SHA are the completed plan
  and sprint delivery records. No workflow, installer, test, HLD, crate, or
  manifest changed afterward. The sprint state records canonical full
  verification at the exact review HEAD with `passed: true` and
  `harness: unchanged` (`.claude/scratch/S40-run.json:20`,
  `.claude/scratch/S40-run.json:25`). A fresh local check also passes all 46
  workflow regressions and confirms 28 of 28 hashes.
- The exact-HLD impact is complete. HLD12 describes the pinned render oracle
  and all Poppler consumers, HLD14 owns the story gate, and HLD15 owns both
  deterministic installers and the hosted runtime
  (`docs/hld/12-testing-strategy.md:211`,
  `docs/hld/12-testing-strategy.md:218`,
  `docs/hld/12-testing-strategy.md:487`,
  `docs/hld/12-testing-strategy.md:496`,
  `docs/hld/14-development-backlog.md:1290`,
  `docs/hld/15-build-and-toolchain.md:345`,
  `docs/hld/15-build-and-toolchain.md:376`). These are exactly the three HLD
  files named by the completed design plan
  (`.claude/plans/F-X012-design.md:108`,
  `.claude/plans/F-X012-design.md:112`).

## Delivery records and release boundary

- CURRENT_SPRINT and BACKLOG mark F-X012 done. The generated backlog arithmetic
  is 166 total, 166 done, zero in progress, and zero pending
  (`docs/sprints/CURRENT_SPRINT.md:23`, `docs/sprints/BACKLOG.md:32`,
  `docs/sprints/BACKLOG.md:33`, `docs/sprints/BACKLOG.md:299`). The tracker and
  AS_BUILT agree on S40, size M, two estimated days, one actual day, and the
  delivered toolchain set (`docs/sprints/SPRINT_TRACKER.md:227`,
  `docs/sprints/AS_BUILT.md:5743`, `docs/sprints/AS_BUILT.md:5754`). The run
  state is correctly in review with its sole feature completed
  (`.claude/scratch/S40-run.json:8`, `.claude/scratch/S40-run.json:16`).
- The sprint delta has no crate source, Cargo manifest, lockfile, release
  workflow, public API, dependency, package version, published artifact, or
  rendering baseline change. No tag points at the sprint HEAD, no `s40` tag
  exists before `/close-sprint`, and no release or publication action belongs
  to this sprint. This matches the explicit sprint boundary
  (`docs/sprints/CURRENT_SPRINT.md:42`,
  `docs/sprints/SPRINT_PLAN.md:631`,
  `docs/hld/14-development-backlog.md:1298`).
- The temporary validation PR is closed without merge and its remote branch is
  deleted, matching the completion record
  (`docs/sprints/AS_BUILT.md:5786`, `docs/sprints/AS_BUILT.md:5790`). Prose,
  generated-skill synchronization, CI YAML parsing, and sprint-diff hygiene are
  clean.

## Not found

No blocking gate failure, incomplete hosted job, unverified post-hosted source
change, hash delta, rendering-baseline change, crate interaction, duplicate
helper, dependency-direction violation, new dependency, unrequested public
surface, package-version drift, release or publication leakage, ledger count or
status contradiction, HLD omission, plan contradiction, workflow syntax error,
process-state mismatch, prose violation, should-fix issue, or nice-to-have issue
was found.

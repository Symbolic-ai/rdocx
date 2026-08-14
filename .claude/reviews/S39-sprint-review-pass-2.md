# S39 sprint review, pass 2

**Reviewed**: `sprint/s39` at
`7e7637e79e420d1a2f62f7ca46e686dae8d4c9dd` against merge base
`302ce2a4ece215227d1b1bf0338e266a58a41dbd`, 80 files and 2,614 changed
lines, crates: all 26 workspace packages
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Integrated delta and prior disposition

- Pass 1 was clean at the stable 0.6.0 preparation boundary and explicitly
  required a later review after release work changed HEAD
  (`.claude/reviews/S39-sprint-review-pass-1.md:3`,
  `.claude/reviews/S39-sprint-review-pass-1.md:115`). The stable release is now
  completed and recorded with its exact seven-package allowlist, reviewed tag
  peel, successful workflow jobs, registry versions, owners, and rendered
  READMEs (`docs/sprints/AS_BUILT.md:5657`,
  `docs/sprints/AS_BUILT.md:5663`, `docs/sprints/AS_BUILT.md:5669`,
  `docs/sprints/AS_BUILT.md:5685`). F-X011 therefore satisfies its declared
  F-X010 dependency (`.claude/plans/F-X011-design.md:6`).
- F-X009 remains internally consistent with both release stories. Every one of
  the 26 workspace packages has a distinct declared README and the fresh runner
  compiled 26 Rust examples, validated all 26 README sources, and checked all
  21 publishable archive inventories. The runner enforces those exact package
  counts and validates the same local source identities before packaging
  (`scripts/readme_doctests.py:273`, `scripts/readme_doctests.py:296`,
  `scripts/readme_doctests.py:443`, `scripts/readme_doctests.py:450`,
  `scripts/readme_doctests.py:457`). This matches the delivered F-X009 record
  (`docs/sprints/AS_BUILT.md:5618`, `docs/sprints/AS_BUILT.md:5642`).
- F-X011 implements the exact release boundary in its approved plan. Fresh
  Cargo metadata reports 15 incubating preparation members at 0.2.0, exactly
  14 of them publishable, and unpublished `rpptx-wasm` as the fifteenth. The
  root has the 14 matching internal pins, while the workspace version and all
  eleven stable-train members remain at 0.6.0 with exactly seven publishable
  crates (`Cargo.toml:32`, `Cargo.toml:54`, `Cargo.toml:68`). The named
  regression independently enumerates the 14-package crates.io family, adds
  the WASM preparation member, checks all lock entries and pins, and rejects
  publication of `rpptx-wasm`
  (`scripts/test_sprint_workflow.py:3102`,
  `scripts/test_sprint_workflow.py:3119`,
  `scripts/test_sprint_workflow.py:3131`,
  `scripts/test_sprint_workflow.py:3144`).
- The exact F-X011 HLD impact is 03, 14, and 15
  (`.claude/plans/F-X011-design.md:68`). HLD03 now distinguishes the immutable
  published 0.1.3 family from the prepared, unpublished 0.2.0 family and keeps
  publication behind a separately approved `/release`
  (`docs/hld/03-architecture.md:123`, `docs/hld/03-architecture.md:127`). HLD14
  states the 15-prepared, 14-publishable contract, stable 0.6.0 isolation, and
  pending external gate (`docs/hld/14-development-backlog.md:1270`,
  `docs/hld/14-development-backlog.md:1275`,
  `docs/hld/14-development-backlog.md:1281`). HLD15 names the exact 14-package
  allowlist, 0.2.0 metadata regression, preparation group, and release authority
  (`docs/hld/15-build-and-toolchain.md:159`,
  `docs/hld/15-build-and-toolchain.md:167`,
  `docs/hld/15-build-and-toolchain.md:226`,
  `docs/hld/15-build-and-toolchain.md:239`). No HLD outside the plan's exact
  impact list needed an F-X011 current-intent change.
- The three stories interact without leaking publication authority. The
  workflow runs both stable 0.6.0 and incubating 0.2.0 metadata checks before
  the exact 21-package patched dry run, then selects only the requested
  dependency-ordered allowlist (`.github/workflows/publish.yml:20`,
  `.github/workflows/publish.yml:23`, `.github/workflows/publish.yml:26`,
  `.github/workflows/publish.yml:51`, `.github/workflows/publish.yml:68`). Cargo
  metadata exposes no prohibited reverse dependency. The only `oxml-*` edge to
  an `rdocx-*` or `rpptx-*` crate remains the documented
  `oxml-drawing -> rdocx-oxml` exception
  (`docs/hld/03-architecture.md:55`).

## Verification and release boundary

- The sprint authority records F-X011 as reviewed at the requested exact HEAD
  and the run in review phase (`.claude/scratch/S39-run.json:16`,
  `.claude/scratch/S39-run.json:26`). It also records successful full
  verification at that same SHA with the harness unchanged
  (`.claude/scratch/S39-run.json:45`,
  `.claude/scratch/S39-run.json:47`,
  `.claude/scratch/S39-run.json:48`). This is a later exact-SHA verification
  than the retained stable-release evidence, not a stale reference.
- Fresh focused checks passed all 38 workflow tests, the 26-source and
  21-archive README runner, both locked WASM target checks, all 28 deterministic
  hashes, formatting, prose, generated-skill sync, sprint-delta diff hygiene,
  and the normal dependency tree. Every current publishable archive is below
  10 MiB. `oxml-layout` contains 20 TTF files and four legal files,
  `rdocx-layout` contains no duplicate font payload, and `rpptx` contains
  `assets/default.pptx`, matching the package contract
  (`docs/hld/15-build-and-toolchain.md:187`,
  `docs/hld/15-build-and-toolchain.md:188`,
  `docs/hld/15-build-and-toolchain.md:189`).
- Live registry inspection found every incubating crate still resolves at
  0.1.3 as its newest crates.io release and every exact 0.2.0 lookup is absent.
  All seven stable crates resolve at 0.6.0. Local and remote
  `rpptx-v0.2.0`, its GitHub release, the npm 0.2.0 package, and both PyPI 0.2.0
  projects are absent. This is the required pre-release state, not an unmet
  implementation gate. The plan expressly forbids npm, PyPI, Python, WASM, and
  stable publication in F-X011 (`.claude/plans/F-X011-design.md:34`,
  `.claude/plans/F-X011-design.md:48`).
- The remaining separate `/release rpptx-v0.2.0` approval is a required human
  action, not a review finding. The reviewed plan requires immediate approval
  before the first branch push or tag mutation
  (`.claude/plans/F-X011-design.md:36`,
  `.claude/plans/F-X011-design.md:109`), and HLD15 applies the same boundary to
  both release namespaces (`docs/hld/15-build-and-toolchain.md:246`,
  `docs/hld/15-build-and-toolchain.md:249`). No tag, push, registry upload, npm
  publication, or PyPI publication occurred during this review.

## Milestone gate

The S39 gate is intentionally one external release short of completion. The
26-package documentation, examples, exact archive inventory, full
verification, unchanged 28-entry harness, stable seven-package 0.6.0 release,
and publication-exclusion requirements hold
(`docs/sprints/CURRENT_SPRINT.md:37`,
`docs/sprints/CURRENT_SPRINT.md:40`,
`docs/sprints/CURRENT_SPRINT.md:42`,
`docs/sprints/CURRENT_SPRINT.md:43`,
`docs/sprints/CURRENT_SPRINT.md:44`,
`docs/sprints/CURRENT_SPRINT.md:47`). The fourteen incubating 0.2.0 crates.io
publications do not yet hold (`docs/sprints/CURRENT_SPRINT.md:45`). Their
absence is deliberate because the sprint plan gives F-X011 its own exact-SHA
review and immediate approval boundary
(`docs/sprints/SPRINT_PLAN.md:613`,
`docs/sprints/SPRINT_PLAN.md:615`).

The delivery ledgers describe this boundary honestly. F-X009 and F-X010 are
done, F-X011 remains in progress, and only the two completed stories appear in
the as-built and tracker records (`docs/sprints/CURRENT_SPRINT.md:24`,
`docs/sprints/CURRENT_SPRINT.md:25`,
`docs/sprints/CURRENT_SPRINT.md:26`,
`docs/sprints/BACKLOG.md:298`,
`docs/sprints/SPRINT_TRACKER.md:223`,
`docs/sprints/SPRINT_TRACKER.md:224`). This pass is clean for the separate
F-X011 release approval boundary. It does not claim S39 is ready to close.
Publication, external verification, F-X011 finalization, and a review of the
resulting final sprint HEAD still remain.

## Not found

No cross-feature interaction defect, duplicate helper, prohibited dependency
edge, undeclared hash delta, false milestone claim, HLD scope conflict, version
or lock mismatch, stable-family regression, incubating publication-set error,
README count or archive error, stale version-sensitive example, WASM authority
leak, archive ceiling or asset failure, unrequested public surface,
unauthorized tag or publication, ledger drift, run-state mismatch, prose
violation, or release-boundary bypass was found.

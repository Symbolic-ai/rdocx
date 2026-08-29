# S58 sprint review, pass 25

**Reviewed**: `sprint/s58` at
`d425174aa439041b98252484100b5eda4c523bf8` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 215 files, 25,616 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-py`, `rpptx-render`, and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This twenty-fifth pass is the explicitly requested pre-mutation review for
F-X031. Since pass 24, the tracked boundary adds the clean pass-24 review and
changes only the F-X031 status and owner rows in the two sprint status files.
It changes no product code, workflow, manifest, dependency, HLD, public API,
delivery completion record, or render baseline.

## Blocking

None. 0 blocking findings.

## Should-fix

None. 0 should-fix findings.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this pre-mutation checkpoint. F-X031 is in progress at
`docs/sprints/CURRENT_SPRINT.md:50`, and its final operational acceptance
conditions remain open at `docs/sprints/CURRENT_SPRINT.md:105` through
`docs/sprints/CURRENT_SPRINT.md:109`. The external ruleset, successful
documentation-only probe, failed selected-job probe, merge-block evidence, and
cleanup evidence do not exist yet.

The exact reviewed subject SHA has a clean full verification record with 49 of
49 hashes unchanged at `.claude/scratch/S58-run.json:686` through
`.claude/scratch/S58-run.json:690`. This establishes the approved
pre-mutation boundary. It does not claim the final M20 or S58 end gate.

## Not found

- **Claim and worktree ownership, 0 findings**: the claim commit changes only
  F-X031 from pending to in progress in `docs/sprints/BACKLOG.md:486` and from
  pending with no owner to in progress with owner `codex` in
  `docs/sprints/CURRENT_SPRINT.md:50`. The run state records one running worker,
  branch `work/f-x031-codex`, worktree `/private/tmp/rdocx-fx031-codex`, and
  base and head at the reviewed SHA at `.claude/scratch/S58-run.json:55`
  through `.claude/scratch/S58-run.json:64`. Direct inspection found both the
  canonical worktree and worker worktree clean at that SHA.
- **Approved plan and dependency readiness, 0 findings**: the approved plan
  depends exactly on completed F-X029 and F-X070 at
  `.claude/plans/F-X031-design.md:3` through
  `.claude/plans/F-X031-design.md:6`. It requires the repository setting to be
  tied to the reviewed sprint SHA and proved with real pull requests at
  `.claude/plans/F-X031-design.md:19` through
  `.claude/plans/F-X031-design.md:22`. Both dependency rows are complete at
  `docs/sprints/BACKLOG.md:484` and `docs/sprints/BACKLOG.md:525`.
- **Exact CI gate identity and fan-in, 0 findings**: the reviewed workflow has
  exactly one job id `ci-gate` and one check name `CI gate` at
  `.github/workflows/ci.yml:633` through `.github/workflows/ci.yml:635`. It has
  `if: always()` and depends on change detection plus every filtered job at
  `.github/workflows/ci.yml:635` through `.github/workflows/ci.yml:647`. Its
  result validator accepts only selected success or unselected skip at
  `.github/workflows/ci.yml:672` through `.github/workflows/ci.yml:684`. The
  existing regression pins that exact fan-in and rejects failed, cancelled, or
  unexpectedly skipped selected jobs at
  `scripts/test_sprint_workflow.py:389` through
  `scripts/test_sprint_workflow.py:476`.
- **Narrow bypass and close interaction, 0 findings**: the approved plan grants
  an `always` bypass only to the repository-administrator role and explicitly
  excludes broader repository roles, teams, apps, and users at
  `.claude/plans/F-X031-design.md:38` through
  `.claude/plans/F-X031-design.md:45`. That exception is justified by the
  reviewed close workflow, which creates a local no-fast-forward merge and
  pushes `main` at `.claude/commands/close-sprint.md:48` through
  `.claude/commands/close-sprint.md:55`. The approved payload therefore permits
  exactly one `RepositoryRole` administrator bypass actor, role id `5`, with
  mode `always`. An organization administrator, broader repository role, team,
  app, or user would violate the plan. The mutation remains limited to one
  active default-branch ruleset requiring exact check `CI gate` at
  `.claude/plans/F-X031-design.md:99` through
  `.claude/plans/F-X031-design.md:104`.
- **Clean-worker red evidence, 0 findings**: the worker progress record binds
  its branch, worktree, base, and head to the reviewed SHA at
  `.claude/scratch/F-X031-progress.md:5` through
  `.claude/scratch/F-X031-progress.md:12`. It records the unchanged 49 of 49
  baseline and the external-only test plan at
  `.claude/scratch/F-X031-progress.md:14` through
  `.claude/scratch/F-X031-progress.md:23`. Fresh read-only GitHub inspection
  agrees with its red evidence at `.claude/scratch/F-X031-progress.md:25`
  through `.claude/scratch/F-X031-progress.md:42`: repository
  `tensorbee/rdocx` has default branch `main` and authenticated administrator
  access, repository and inherited branch rules are empty, classic `main`
  protection returns 404, and no `proof/f-x031-*` remote branch or pull request
  exists. No external mutation has occurred.
- **Reviewed-SHA binding, 0 findings**: pass 24 is recorded clean at review
  commit `58483de8df21f89e968e2ecabeac0b7d5dd02d2d` in
  `.claude/scratch/S58-run.json:387` through
  `.claude/scratch/S58-run.json:393`. Full verification then passed again at the
  claim SHA with 49 of 49 unchanged at `.claude/scratch/S58-run.json:686`
  through `.claude/scratch/S58-run.json:690`. The exact CI identity, approved
  plan, clean worker, red external state, and verified claim therefore make
  `d425174aa439041b98252484100b5eda4c523bf8` safe to bind the approved
  external mutation to after one immediate read-only protection recheck. The
  required recheck and exact evidence inventory remain mandatory at
  `.claude/plans/F-X031-design.md:32` through
  `.claude/plans/F-X031-design.md:53`.
- **Interaction, duplication, layering, dependencies, surface, harness, docs,
  and structure, 0 findings**: the post-pass-24 claim adds no implementation,
  helper, dependency edge, feature flag, crate, module, public API, HLD claim,
  or baseline change. The prior integrated sprint audit remains clean at
  `.claude/reviews/S58-sprint-review-pass-24.md:25` through
  `.claude/reviews/S58-sprint-review-pass-24.md:35`, and the exact-head full
  verification is current.

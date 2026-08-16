# F-X026, CI must run the release regressions too

**Status**: completed
**Sprint**: S44
**Size**: S
**Depends on**: F-X025

## Problem

The local gate runs the complete release regression module, but pull-request CI
does not. `.claude/commands/verify.md:34` invokes
`python3 -m unittest scripts.test_sprint_workflow`, while the lightweight job in
`.github/workflows/ci.yml:206` runs only the prose and generated-skill checks.
A contributor who skips the local gate can therefore move a version carrier and
receive a green pull request before the tag-time publication preflights expose
the drift.

F-X025 is complete in `docs/sprints/BACKLOG.md:424`. The remaining CI omission
is the defect recorded by `.claude/reviews/S43-sprint-review-pass-1.md`, finding
N1, and specified by `docs/hld/14-development-backlog.md`, "F-X026, CI must run
the release regressions too".

## Spec reference

- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "CI job matrix".
- `docs/hld/14-development-backlog.md`, "F-X026, CI must run the release
  regressions too".

## Approach

Add a dedicated, unconditional `release-regressions` job to
`.github/workflows/ci.yml`. It will use a lightweight Ubuntu runner, check out
the repository, and run the exact whole-module command
`python3 -m unittest scripts.test_sprint_workflow`. The job will not narrow the
module to the two current publication preflights, suppress failures, or hide
behind a condition.

Extend the existing workflow-contract tests in
`scripts/test_sprint_workflow.py`. Reuse its YAML helpers to assert the job
identity, checkout ordering, exact command, unconditional execution, and
ordinary failure propagation. Mutation-sensitive cases will reject a removed
or narrowed command, a job condition, `continue-on-error`, and a successful
fallback. Existing stable and incubating version mutation tests remain the
evidence that a stale literal fails the module CI now invokes.

## Rejected alternatives

- Add the command to `prose`. F-X029 relies on that job selecting tracked
  Markdown, while release regressions inspect version carriers across the
  repository.
- Add the command to the heavyweight workspace job. That hides a four-second
  standard-library gate behind toolchain, corpus, and workspace setup.
- Run only the two dotted publication preflights. Whole-module execution also
  covers future release-contract regressions and matches the backlog gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_ci_runs_release_regressions_in_a_named_job` | A dedicated unconditional CI job runs the exact whole-module command after checkout with failures unsuppressed |
| regression | `test_ci_release_regression_job_rejects_wiring_mutations` | Removing, narrowing, bypassing, or conditionally skipping the command fails the contract |
| regression | Existing stable and incubating version mutation tests | A stale manifest, lockfile, README, Python, or CI version carrier fails the same module |

The backlog test gate is **regression**: the module runs in a named CI job,
asserted the way the other job contracts are, and a stale version literal fails
that job.

## HLD impact

- `docs/hld/12-testing-strategy.md`, "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "CI job matrix".

## Risk routing

- **Release scripting, version strings**. Read `.claude/commands/release.md`
  and `docs/hld/15-build-and-toolchain.md`. Inspect every manifest, lockfile,
  and README version diff, and require a clean full gate. The expected carrier
  diff is empty. Separate final approval before tagging remains mandatory if
  scope expands to a release, but this story creates no tag or publication
  authority.

## Hash harness

Expected unchanged at 49 of 49. This story changes CI wiring, regression tests,
and HLD prose only. Any delta is unexplained and blocks completion.

## Implementation checklist

- [x] Add failing workflow-contract test stubs in the existing Python module.
- [x] Add the dedicated unconditional release-regressions CI job.
- [x] Make positive and mutation-sensitive workflow tests pass.
- [x] Run the focused tests, both publication preflights, and the whole module.
- [x] Update only the listed HLD sections.
- [x] Confirm that manifests, the lockfile, READMEs, and version carriers did
  not change.
- [x] Run microscope and contribute the full gate evidence to the integrated
  sprint verification.

## Open questions

None.

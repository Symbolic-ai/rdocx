# F-X029, Path-filtered CI jobs

**Status**: approved
**Sprint**: S44
**Size**: M
**Depends on**: none

## Problem

`.github/workflows/ci.yml:3` triggers the whole workflow for every pull request
and main push. Its thirteen jobs have no change routing or fan-in gate, so a
docs-only change schedules the workspace suite, MSRV suite, both WASM targets,
Python bindings, packaging work, hash harness, and pinned-render fidelity job.

The required-status trap and fail-safe requirement are specified at
`docs/hld/14-development-backlog.md:2110`. A required job skipped by a native
path filter never reports, while a filter that is too narrow silently stops a
gate. The design must keep one stable reporting check and test both sides of
every filter.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".
- `docs/hld/14-development-backlog.md`, "F-X029, Path-filtered CI jobs".

## Approach

Add one `changes` job to `.github/workflows/ci.yml` using a maintained
path-filter action pinned to an immutable full SHA. Grant pull-request read
permission only where the action needs it. Keep the filter definitions inline
to avoid a new file. Every filter includes `ci.yml` so a routing edit cannot
suppress the jobs it governs.

Expose a Boolean output for each filtered expensive job. At minimum route
`test`, `msrv`, `wasm`, `python-bindings`, `presentation-fidelity`,
`hash-harness`, `supply-chain`, and `prose`. Each filter will cover the job's
complete transitive workspace closure and relevant manifests, toolchain files,
scripts, fixtures, examples, corpus inputs, or legal inputs. Scheduled
supply-chain execution remains unconditional. Leave cheap jobs unfiltered
unless their closure can be stated and covered by the same regressions.

Add an always-running `ci-gate` fan-in with `if: always()`. It depends on change
detection and every filtered job, rejects a failed or cancelled selected job,
rejects an unexpectedly skipped selected job, and accepts a skipped job only
when its filter output is false. A docs-only HLD change selects prose, skips the
expensive product jobs, and still produces a successful stable gate.

F-X029 delivers the repository-side gate only. Making `ci-gate` a required
GitHub branch-protection check is parked as F-X031 in S62, the final planned
sprint. This story performs no external repository-settings mutation.

Extend `scripts/test_sprint_workflow.py` with inline-filter parsing and
mutation-sensitive contract tests. Assert a must-trigger and must-not-trigger
path for every filtered job, the docs-only route, scheduled supply-chain
behavior, immutable action pinning, least privilege, and strict fan-in result
handling.

## Rejected alternatives

- Use top-level `paths` or `paths-ignore`. A skipped workflow produces no
  stable required check.
- Condition expensive jobs without a fan-in. Branch protection then depends on
  a changing collection of skipped job names.
- Write custom changed-file shell logic. That duplicates matching semantics
  and would require a new file or a large workflow-local program.
- Filter only leaf crate directories. A shared or transitive dependency change
  would silently suppress a required gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_each_filtered_ci_job_has_a_must_trigger_and_must_not_trigger_path` | Every filtered job selects a representative relevant path and rejects an unrelated path, with narrowing mutations rejected |
| regression | `test_docs_only_changes_skip_expensive_jobs_and_still_report_the_ci_gate` | A docs HLD change selects prose, skips the workspace, MSRV, WASM, binding, hash, and fidelity jobs, and leaves the fan-in operative |
| regression | `test_ci_gate_rejects_failed_selected_jobs_and_accepts_unselected_skips` | The gate uses `always()`, covers every filtered dependency, rejects failure, cancellation, and unexpected skip, and accepts only filter-authorized skips |
| integration | Hosted pull request with a docs-only change | GitHub reports the stable gate while expensive jobs are skipped, without changing branch-protection settings |

The backlog test gate is **regression**: a test asserts, for each filtered job,
a changed path that must trigger it and a changed path that must not, so
narrowing a filter by mistake fails the suite. A docs-only change reports every
required check.

## HLD impact

- `docs/hld/12-testing-strategy.md`, "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".

## Risk routing

none.

## Hash harness

Expected unchanged at 49 of 49. This story changes CI scheduling, standard
library regressions, and HLD prose only.

## Implementation checklist

- [ ] Add inline fail-safe change filters and the immutable action pin.
- [ ] Scope pull-request read permission to change detection.
- [ ] Route each selected expensive job through its output.
- [ ] Preserve scheduled supply-chain execution.
- [ ] Add the always-running fan-in and strict result validation.
- [ ] Add positive, negative, and mutation-sensitive path tests.
- [ ] Demonstrate the docs-only route.
- [ ] Update only the listed HLD sections.
- [ ] Run focused regressions and contribute the full integrated gate.
- [ ] Confirm the hash harness remains 49 of 49.

## Open questions

None. Branch-protection configuration is parked as F-X031 in S62.

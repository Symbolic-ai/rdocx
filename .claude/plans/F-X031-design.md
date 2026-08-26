# F-X031, Require the CI gate in branch protection

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-X029, F-X060

## Problem

The workflow exposes one stable aggregate `CI gate` check at
`.github/workflows/ci.yml:631`, and the regression contract at
`scripts/test_sprint_workflow.py:389` proves that the fan-in rejects failed or
unexpectedly skipped selected jobs. GitHub currently reports no repository
rulesets and no classic protection for `main`, so a pull request can merge
without that aggregate result.

The tracked workflow and the external protection setting must remain separate.
The protection change must be tied to the final reviewed S58 SHA, must preserve
any protections found immediately before mutation, and must be demonstrated by
real pull requests rather than inferred from the workflow text.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".
- `docs/hld/14-development-backlog.md`, "F-X031, Require the CI gate in branch protection".

## Approach

After the integrated S58 implementation has passed full verification and a
clean sprint review, inspect `.github/workflows/ci.yml` at that exact SHA and
confirm that job id `ci-gate` still reports check name `CI gate`. Query both
repository rulesets and classic `main` protection again immediately before the
write.

Create an active repository ruleset targeting the default branch and requiring
the exact `CI gate` status check. Preserve every existing rule and protection.
Use a ruleset because it supplies a stable numeric identifier for the required
evidence and avoids replacing a classic-protection document wholesale.

Prove the setting with two disposable pull requests based on the current
default branch. One changes tracked Markdown only and must report a successful
required `CI gate` while filtered product jobs remain skipped. The other makes
a selected CI input deliberately invalid and must report a failed `CI gate`
with merge blocked. Close both pull requests and delete only their disposable
remote branches after recording the pull request, check-run, ruleset, branch
pattern, repository, and reviewed S58 SHA evidence.

No tracked source file changes are expected. Completion records the external
evidence in the listed HLD sections and sprint ledgers.

## Rejected alternatives

- Require every filtered job separately. Unselected jobs are intentionally
  skipped, so those required checks would never report.
- Replace classic branch protection with one complete document. A later
  protection added before execution could be removed accidentally.
- Treat the workflow regression as proof of repository protection. It proves
  fan-in semantics, not the external GitHub setting or merge enforcement.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | Docs-only pull request against the protected default branch | Required `CI gate` succeeds while filtered expensive jobs are skipped |
| integration | Deliberately failing selected-job pull request against the protected default branch | `CI gate` fails and the pull request is blocked from merge |
| integration | GitHub ruleset readback | The active default-branch ruleset requires exact check `CI gate` and evidence names its identifier and the reviewed S58 SHA |

The backlog test gate is **integration**: a docs-only pull request reports a
successful required `ci-gate` while the filtered expensive jobs stay skipped,
and a selected failing job makes the required gate fail.

## HLD impact

- `docs/hld/12-testing-strategy.md`, "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".

## Risk routing

none.

## Hash harness

Expected unchanged. This story changes external repository protection and
tracked delivery evidence, not rendered output.

## Implementation checklist

- [ ] Bind the inspected `ci-gate` job id and `CI gate` check name to the final reviewed S58 SHA.
- [ ] Re-query rulesets and classic protection immediately before mutation.
- [ ] Add one active default-branch ruleset that requires exact check `CI gate` without removing existing protections.
- [ ] Run the docs-only pull request and record successful required-check evidence with expensive filtered jobs skipped.
- [ ] Run the selected-failure pull request and record failed-gate and merge-block evidence.
- [ ] Close the disposable pull requests and remove only their disposable remote branches.
- [ ] Update only the listed HLD sections and sprint delivery records.
- [ ] Confirm the deterministic hash harness remains unchanged.

## Open questions

None. F-X031 uses the approved post-review operational sequence. It remains in
progress through the first clean sprint review, executes and records the
external gate at that reviewed SHA, then repeats full verification and sprint
review over the final evidence commit.

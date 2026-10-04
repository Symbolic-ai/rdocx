---
description: Close a sprint. The only command that merges to main and creates an sNN sprint tag.
---

# /close-sprint SNN [--next SMM]

Merge the sprint branch to `main`, tag it, push both, and open the next sprint.
A prepared release follows through `/release` at this exact `main` merge SHA.

**This is the only command in the repository that may touch `main` or create an
`sNN` sprint tag.** `/release` alone owns `v*` release tags and crates.io
publication.

## Steps

1. **Pre-flight.** `python3 scripts/sprint_workflow.py close-preflight SNN`, plus:
   - Every F-ID in `CURRENT_SPRINT.md` is `done`, or explicitly carried with a
     stated reason.
   - The working tree is clean.
   - The current branch is `sprint/sNN`.
   - `main` has no commits the sprint branch lacks. If it does, rebase or merge
     `main` in first and re-verify.
   - No worker work is stranded. `git worktree list` and
     `git branch --list 'work/*'` against the run state. `close-preflight`
     already refuses an unconsumed `.claude/handoffs/F-XXX-ready.md`. Record
     the completed workers that become cleanup targets after the push. A
     carried worker, an untracked branch or a worktree absent from the run
     state is not a cleanup target.

2. **`/verify --full`.** Everything, workspace-wide, including packaging and the
   supply-chain check. Not `--fast`, not `--scoped`, and not the changed-crate
   subset.

3. **`/sprint-review SNN`.** Run the bounded review loop to completion. Blocking
   findings are fixed and the loop repeats, at most three passes by default. A
   fourth pass means the sprint is not ready.

4. **Confirm the milestone gate** if this sprint closes one. Find the
   "End-of-milestone gate" in `docs/hld/14-development-backlog.md` and check it
   explicitly. Some gates are manual, such as opening corpus decks in
   PowerPoint. **Do not mark a manual gate as met without performing it.**

5. **Update the tracker.** Append the per-sprint summary row to
   `docs/sprints/SPRINT_TRACKER.md` with planned, done, carried, estimated and
   actual days, and recalculate the velocity table. A validation-only sprint
   records zero planned, done, carried and estimated implementation days. Its
   actual days record the elapsed validation work, and its velocity is 0.00
   stories per week.

6. **Merge.** An explicit `--no-ff` merge commit on `main`:
   `Merge sprint/sNN into main`.

7. **Tag.** An annotated tag `sNN` at the merge commit, whose message lists the
   completed F-IDs. For a validation-only sprint with no completed F-IDs, the
   message says `Validation-only sprint, no F-IDs`.

8. **Push** `main` and the tag. Record the reviewed sprint HEAD, the merge
   commit SHA, and the tree comparison needed by `/release`. A release tag is
   created later at this merge SHA, after its own full gate and separate
   approval.

9. **Clean completed workers.** Only after both pushes succeed, inspect every
   cleanup target recorded in the run state:
   - Confirm the F-ID is `completed`, its handoff was consumed and its recorded
     integration commit is an ancestor of `main`.
   - If the path is still registered, confirm it belongs to the recorded
     worker branch and `git -C <worktree-path> status --porcelain` is empty.
     Run `git worktree remove <worktree-path>` without `--force`. A worktree
     safely removed after local integration needs no second removal.
   - Delete the recorded local worker branch. A squash integration means this
     requires `git branch -D <worker-branch>`, so perform it only after all
     preceding checks pass.
   - Never remove a carried worker, a dirty worktree, a remote branch or an
     unrelated worktree. Leave any failed target intact and report the exact
     reason.

   A validation-only sprint has no cleanup targets.

10. **Reconcile contributed PRs and issues.** After the integrated `main` push,
    compare each PR and issue assigned to this sprint with its full acceptance
    criteria and the verified integrated evidence. Close each superseded PR with
    a human-written comment that thanks its contributor, links the integrated
    work and explains any remaining scope. Close an issue only when every
    acceptance criterion has evidence on `main`. Thank the reporter and
    contributors in a human-written closure comment that cites that evidence.
    Leave partially addressed issues open, comment with the completed work and
    remaining criteria, and keep their follow-up F-IDs in the backlog. Record
    every closure or open-item decision in the sprint report. Do not use a
    generic generated comment or infer issue completion from a PR merge.

11. **Open the next sprint.** If `--next SMM` was given, run `/sync-sprint SMM`.

12. **Report** what merged, the sprint tag, the exact `main` merge SHA, the
    reviewed sprint SHA and tree comparison, the velocity for this sprint,
    every worker cleanup outcome, and the PR and issue reconciliation. Name
    each prepared family release and its subsequent `/release` command. A
    velocity variance over 30 percent is an escalation trigger.

## Carrying a story

A story that is not done may be carried, but not silently:

- Set it back to `pending` in `docs/sprints/BACKLOG.md`.
- Move it to its next eligible sprint in `docs/sprints/SPRINT_PLAN.md` and set
  that exact sprint in `BACKLOG.md`. A story blocked by an explicit release or
  publication boundary may skip intervening sprints. If no later target is
  already planned, use the immediate next sprint.
- Record the reason in the `SPRINT_TRACKER.md` summary row.

**Three carries of the same F-ID is an escalation trigger.** The design plan was
wrong. Reset to `/design`.

## Refused situations

- **Any pre-flight check fails.** Name it and stop.
- **`/verify --full` fails.** Nothing merges on a red gate.
- **A blocking sprint-review finding is unresolved.**
- **A milestone gate is unmet**, including the manual ones.
- **A release tag is requested.** Run `/release vX.Y.Z`. This command creates
  the sprint tag only.

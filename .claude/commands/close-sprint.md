---
description: Close a sprint. The only command that merges to main and creates a tag.
---

# /close-sprint SNN [--next SMM]

Merge the sprint branch to `main`, tag it, push both, and open the next sprint.

**This is the only command in the repository that may touch `main` or create a
tag.** Nothing else does so, implicitly or otherwise.

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
     already refuses an unconsumed `.claude/handoffs/F-XXX-ready.md`. Report
     the retained branches and worktrees so the human can remove them after the
     merge. **Do not remove them yourself.**

2. **`/verify --full`.** Everything, workspace-wide, including packaging and the
   supply-chain check. Not `--fast`, not the changed-crate subset.

3. **`/sprint-review SNN`.** Run the bounded review loop to completion. Blocking
   findings are fixed and the loop repeats, at most three passes by default. A
   fourth pass means the sprint is not ready.

4. **Confirm the milestone gate** if this sprint closes one. Find the
   "End-of-milestone gate" in `docs/hld/14-development-backlog.md` and check it
   explicitly. Some gates are manual, such as opening corpus decks in
   PowerPoint. **Do not mark a manual gate as met without performing it.**

5. **Update the tracker.** Append the per-sprint summary row to
   `docs/sprints/SPRINT_TRACKER.md` with planned, done, carried, estimated and
   actual days, and recalculate the velocity table.

6. **Merge.** An explicit `--no-ff` merge commit on `main`:
   `Merge sprint/sNN into main`.

7. **Tag.** An annotated tag `sNN` at the merge commit, whose message lists the
   completed F-IDs.

8. **Push** `main` and the tag.

9. **Open the next sprint.** If `--next SMM` was given, run `/sync-sprint SMM`.

10. **Report** what merged, the tag, the velocity for this sprint, and whether
    it diverged from the plan by more than 30 percent, which is an escalation
    trigger.

## Carrying a story

A story that is not done may be carried, but not silently:

- Set it back to `pending` in `docs/sprints/BACKLOG.md`.
- Move it to the next sprint in `docs/sprints/SPRINT_PLAN.md`.
- Record the reason in the `SPRINT_TRACKER.md` summary row.

**Three carries of the same F-ID is an escalation trigger.** The design plan was
wrong. Reset to `/design`.

## Refused situations

- **Any pre-flight check fails.** Name it and stop.
- **`/verify --full` fails.** Nothing merges on a red gate.
- **A blocking sprint-review finding is unresolved.**
- **A milestone gate is unmet**, including the manual ones.
- **A release tag is requested.** Releases are `cargo release` plus
  `publish.yml`, not this command. This creates the sprint tag only.

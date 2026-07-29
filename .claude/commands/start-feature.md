---
description: Begin work on a designed feature. Marks it in-progress and creates test stubs.
---

# /start-feature F-XXX [--claimed]

Transition an approved design into an in-progress feature with failing test
stubs. **No implementation in this command.**

`--claimed` is what a worker runs first in a worktree that `/claim-feature`
created. The claim already set the trackers and cut the branch, so this becomes
the stub-creation half of the command.

## Steps

1. **Confirm the branch.** The current branch must be `sprint/sNN` matching the
   sprint in `docs/sprints/CURRENT_SPRINT.md`. If it is `main`, create the
   sprint branch first through `/sync-sprint`.

   Under `--claimed`, the branch must instead be `work/<fid-lower>-<agent>`,
   in the worktree the claim created, with F-XXX already `in-progress` and
   owned by that agent. If the branch is `sprint/sNN`, you are in the canonical
   worktree and the claim did not happen. Stop.

2. **Read the design plan.** `.claude/plans/F-XXX-design.md` must exist and its
   status must be `approved`.

3. **Validate the spec reference.** The plan's `## Spec reference` must name
   specific sections. **Refuse a whole-document citation** such as
   "see docs/hld/". This check exists because a vague reference is how a story
   silently drifts from its specification.

4. **Check the dependencies** are `done` in `docs/sprints/BACKLOG.md`.

5. **Create the test stubs** from the plan's `## Test plan`. Each is a real test
   function that **fails for the right reason**, not `todo!()`. A stub that
   passes vacuously is worse than no stub.

6. **Update the trackers.** Set `in-progress` in both
   `docs/sprints/BACKLOG.md` and `docs/sprints/CURRENT_SPRINT.md`, and
   regenerate the AUTOGEN counts.

   **Skip this under `--claimed`.** The claim commit already did it, on the
   sprint branch. Doing it again here puts a tracker edit on the worker branch,
   which then conflicts with every other worker at integration time.

7. **Record the baseline.** For a story in M1 through M6, run the hash harness
   and note the current digest in `.claude/scratch/F-XXX-progress.md`, so the
   delta at completion is attributable to this story alone.

8. **Report** the files created and the first checklist item to implement.

## Refused situations

- **No design plan**, or its status is `draft`. Run `/design F-XXX` first.
- **The spec reference names a whole document.** Fix the plan.
- **A dependency is not `done`.** Name it and stop.
- **The F-ID is already `in-progress` or `done`.** If resuming, read
  `.claude/scratch/F-XXX-progress.md` and continue rather than restarting.
  Under `--claimed`, `in-progress` is the expected state, not a refusal.
- **On `main`.** Never start a feature on the default branch.
- **`--claimed` from the canonical sprint worktree.** Run `/claim-feature`
  first, from there, and then this from the new worktree.

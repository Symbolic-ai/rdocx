---
description: Audit consistency across the trackers, the HLD and the code. Reports, does not fix.
---

# /sync-status [--fix]

Cross-check every place a fact is recorded twice. Reports drift by default.
`--fix` repairs only the mechanically derivable cases listed below.

## Checks

1. **Backlog against the HLD.** Every F-ID in `docs/sprints/BACKLOG.md` exists
   in `docs/hld/14-development-backlog.md`, and the reverse. Sizes match.

2. **Backlog against the sprint plan.** Every F-ID's `Sprint` column matches the
   sprint it appears under in `docs/sprints/SPRINT_PLAN.md`.

3. **Backlog against the current sprint.** Every F-ID in `CURRENT_SPRINT.md` has
   the same status in `BACKLOG.md`. This is the pair most likely to drift,
   because `/complete-feature` writes both.

4. **AUTOGEN counts.** Recompute every summary count from the rows. Mismatches
   are always a bug in a previous command, never a hand edit.

5. **Design plans.** Every `in-progress` or `done` F-ID has a plan. Every plan's
   status agrees with the backlog: `approved` for in-progress, `completed` for
   done.

6. **AS_BUILT coverage.** Every `done` F-ID has an entry. Every entry names a
   real F-ID.

7. **Review coverage.** Every `done` F-ID has at least one
   `.claude/reviews/F-XXX-*` file whose final pass reports zero defects and zero
   smells.

8. **Dangling references.** Every F-ID mentioned anywhere in `docs/` or
   `.claude/` exists in the backlog.

9. **Spec references resolve.** Every `docs/hld/` path cited in a design plan or
   in `CURRENT_SPRINT.md` exists.

10. **Dependencies are acyclic**, and no `done` story depends on a `pending`
    one, which would mean it was completed out of order.

11. **Tracker arithmetic.** `SPRINT_TRACKER.md` has one row per `done` F-ID.

## What `--fix` may repair

Only these, because only these are mechanically derivable:

- AUTOGEN summary counts (check 4).
- A `CURRENT_SPRINT.md` status that disagrees with `BACKLOG.md`, taking the
  backlog as authoritative (check 3).
- The `Sprint` column in `BACKLOG.md`, taking `SPRINT_PLAN.md` as authoritative
  (check 2).

Everything else is reported for a human to resolve. A missing AS_BUILT entry or
an absent review cannot be synthesised, and inventing one would destroy the
value of the record.

## Reporting

Report each check as pass, or as a list of specific discrepancies with the
conflicting values and their file locations. **Say "0 discrepancies" explicitly**
rather than staying silent, so a clean run is distinguishable from a run that
did not happen.

## Refused situations

- **Fixing anything outside the three derivable cases.**
- **Creating an AS_BUILT entry or a review file to satisfy a check.** The gap is
  the finding.

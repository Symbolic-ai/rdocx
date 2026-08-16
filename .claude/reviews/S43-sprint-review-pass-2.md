# S43 sprint review, pass 2

**Reviewed**: `20ba76f..6060ff8`, 1 file, 3 insertions, 0 deletions.
`docs/sprints/SPRINT_TRACKER.md` only.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Why this pass exists

Pass 1 was clean, and a confirmation pass after a clean pass is refused. This is
not one. `close-preflight` refused the close because the review covered
`20ba76f` while HEAD had moved to `6060ff8`, and it was right to: the close
figures are content no review had seen. The rule that both the review and the
full verification must cover the final HEAD is what stops a sprint merging at a
SHA nobody looked at.

The scope is therefore the close-figures commit alone. Pass 1's findings and its
milestone-gate evidence stand unchanged, because no file it reviewed moved.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## The delta, checked against the record

Three rows, all arithmetic or prose about work already reviewed.

**Summary row.** `| S43 | X | 5 | 5 | 0 | 8 | 1 |`. Five planned matches the five
rows in `CURRENT_SPRINT.md`. Five done matches five `completed` in the run state
and five `done` in `BACKLOG.md`. Zero carried matches: nothing was set back to
`pending` and `SPRINT_PLAN.md` gained no moved story. Eight estimated days is
S plus M plus M plus L plus S read against the sizing key at the top of the
file, `S = 1d`, `M = 2-3d`, `L = 4-5d`, taking the low end of each range, which
is how S41's 12 and S42's 5 were derived. One actual day matches the completion
dates in all four AS_BUILT entries and F-X018's, all 2026-08-16.

The prose claim "no crate, package version or published artifact changed" is
checkable and holds: `Cargo.toml` and `Cargo.lock` are byte-identical to the
sprint base, which pass 1 recorded under `deps`.

**Velocity row.** `| S43 | 5 | 1 | 25.00 |`. Five stories divided by one day
times five working days is 25.00, which is the formula the section states. The
number is an outlier against every other row's 5.00 and is not smoothed, which
is correct for a log.

**Escalation row.** Eight estimated against one actual is an 87 percent
variance, well past the 30 percent trigger, so the row is required rather than
optional. It follows the format of the S39, S40 and S41 rows above it and states
the cause and the response. It also states that 25.00 stories per week is not
sustainable and is not carried into a forecast, which is the honest reading: the
rate reflects five stories that arrived with their causes already written up by
the sprints that filed them, not a change in capacity.

By contrast S42 recorded no escalation row, and that was correct rather than an
omission. Its commit `4f91f01` states the reasoning: four actual against five
estimated is 20 percent, inside the threshold.

## Milestone gate

Unchanged from pass 1, which recorded all six items of the sprint definition of
done with evidence, including the two performed by hand and reverted. This
commit touches no code, no test and no specification.

## Not found

- **interaction**, **duplication**, **layering**, **deps**, **surface**,
  **docs**, **gate**. Not applicable to a three-line tracker append, and all
  seven were checked in full at pass 1 over the code delta they apply to.
- **harness**. Re-established at this exact SHA: 49 of 49, with the sprint delta
  still 21 added, 0 changed, 0 removed. Recorded in the run state at `6060ff8`.

## Pass verdict

Zero blocking findings. Clean, and no further pass follows.

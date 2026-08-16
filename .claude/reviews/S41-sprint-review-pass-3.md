# S41 sprint review, pass 3

**Reviewed**: `sprint/s41` at `c853c27`, which is `a419e78` from pass 2 plus two
commits. The incremental delta is 6 files, 103 insertions and 9 deletions, all
of it ledgers, one spec heading and the pass 2 review file. **No product code
changed**: `git diff --name-only a419e78..HEAD` returns nothing under `crates/`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

This pass exists because `close-preflight` requires the latest sprint review to
cover the exact HEAD being merged, and two bookkeeping commits landed after pass
2. It is a real review of that delta rather than a rubber stamp, but the delta is
small and carries no behaviour.

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## What the two commits did

**`ef6a64e`, the F-X013 umbrella.** A story split into children gets its own row
alongside them, marked done, in `BACKLOG.md`, `SPRINT_PLAN.md` and
`SPRINT_TRACKER.md`. `F-064` and `F-098` both do. `F-X013` had children in all
three and no parent row anywhere, so the umbrella carrying the external PR 2
footnote work was absent from the delivery record. Its HLD heading also read
`(parent)` rather than the house `(M, split at design)`.

**`c853c27`, releasing the claims.** The `owner` field records an active claim,
not authorship. A completed feature has no holder, which is why S40 closed with
`done | -`. All six features carried `claude` in both the run state and
`CURRENT_SPRINT.md`, and `close-preflight` refused on all twelve. Both are now
`-`.

Neither was cosmetic. The first left the sprint's largest story invisible in the
ledgers, and the second was a real contract violation that the state machine
caught rather than a naming preference.

## Ledger consistency

Audited after the changes, since that is what these commits touch:

- The X family has 22 rows, 19 done and 3 pending. The summary row reads
  `22 | 19 | 0 | 3`. Consistent.
- The whole backlog has 176 F-ID rows. The total row reads `176`, with 173 done
  and 3 pending, which sums correctly.
- `CURRENT_SPRINT.md` carries the six implemented stories, all `done` with owner
  `-`. `SPRINT_TRACKER.md` carries seven S41 rows, the six stories plus the
  F-X013 umbrella at zero estimated days.
- The three pending rows are F-X017, F-X018 and F-X019, all filed during this
  sprint with test gates and no sprint assignment, which is the correct state
  for opportunistic cross-cutting work.

## Milestone gate

S41 is an X cross-cutting sprint and closes no milestone, so
`docs/hld/14-development-backlog.md` has no end-of-milestone gate to check for
it. No manual gate was required and none is claimed.

The sprint's own definition of done was verified in pass 1 and re-verified in
pass 2, clause by clause with named evidence. Nothing in this delta touches
product behaviour, so those verdicts stand unchanged.

`/verify --full` was run at this exact HEAD and passed all eleven steps,
including the two that only `--full` reaches: the patched 21-package workspace
publish dry run with all 55 archives under 10 MiB, and `cargo deny check`
reporting advisories, bans, licenses and sources ok. Recorded in the run state
against `c853c27`.

## Not found

Aspects checked that produced nothing:

- **interaction**, **duplication**, **layering**, **deps**, **surface**. No
  product code changed, so none of these can have moved. No `Cargo.toml` changed
  at any point in the sprint.
- **harness**. 28 of 28, unchanged, consistent with all six AS_BUILT entries and
  all six commit messages.
- **docs**. The spec change in this delta is the F-X013 heading matching house
  style. The substantive spec correction, the `oxml-layout` neutrality claim,
  landed in the pass 1 remediation and was reviewed in pass 2.

## Exit

Zero blocking across three passes, within the default bound. The sprint is ready
to merge.

# F-X061, contract, pass 1

**Reviewed**: working diff, 7 files, 166 additions and 26 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, checkpoint reviews exhaust the global sprint-review bound

`.claude/commands/run-sprint.md:164`

Each dependency-release checkpoint requires clean reviews after prefix records,
release preparation, and publication evidence. Two checkpoints plus final
closure therefore require at least seven clean review passes. The state records
one global pass number and refuses a pass above the default bound of three at
`scripts/sprint_workflow.py:931`, while the review command also treats a fourth
pass as an explicit extension at `.claude/commands/sprint-review.md:45`. The new
route does not define how checkpoint-local bounded loops use that global
counter, so S58 stops after the first release even when every review is clean.
The contract must preserve one bounded remediation loop per distinct reviewed
HEAD while explicitly recording why scheduled later-boundary pass numbers may
use the existing extension mechanism.

## Smells

None.

## Nitpicks

None.

## Not found

No release-authority weakening, dependency-order violation, stale-HEAD reuse,
unlisted HLD edit, generated-adapter drift, new module, or unrelated change was
found.

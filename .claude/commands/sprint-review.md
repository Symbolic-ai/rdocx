---
description: Review a whole sprint's delta before it merges. Bounded loop, changes no code.
---

# /sprint-review SNN [--pass N]

Review the sprint's integrated diff, which is not the same as the sum of its
feature reviews. `/microscope` sees one F-ID. This sees what they did to each
other.

**This command changes no code.** Fixes are normal F-IDs or amendments made by
the implementing session between passes.

## Steps

1. **Establish the diff.** `sprint/sNN` against the merge base with `main`.
   Report the file count, line count and the crates touched.

2. **Read the sprint contract.** `docs/sprints/CURRENT_SPRINT.md`, its goal, its
   definition of done, and the milestone gate from
   `docs/hld/14-development-backlog.md`.

3. **Review the integrated delta.** The aspects that only appear at sprint
   scope:

   | Aspect | Looks for |
   |---|---|
   | `interaction` | Two F-IDs that are individually correct and jointly wrong |
   | `duplication` | The same helper written twice in one sprint under different names |
   | `layering` | An `oxml-*` crate that gained a dependency on `rdocx-*` or `rpptx-*` |
   | `harness` | Every hash-harness delta declared, justified and consistent with its AS_BUILT entry |
   | `gate` | Does the milestone gate actually hold, tested rather than asserted |
   | `docs` | HLD sections the sprint contradicted but did not update |
   | `deps` | New dependencies, each with a named consumer |
   | `surface` | Public API added that no story called for |

4. **Classify.** `blocking`, `should-fix`, `nice-to-have`. Every finding carries
   `path:line`.

5. **Write** `.claude/reviews/SNN-sprint-review-pass-N.md`.

6. **Hand back.** The implementing session fixes blocking findings, then pass
   `N+1` runs.

## The bound

**At most three passes by default.** A fourth means the sprint is not ready and
the right response is to carry work rather than keep reviewing.

Exit condition: zero blocking findings. Should-fix findings are either fixed or
filed as F-IDs with their review file cited. Nice-to-have findings are recorded
and left.

## Template

```markdown
# SNN sprint review, pass N

**Reviewed**: sprint/sNN against <base>, N files, N lines, crates: ...
**Verdict**: N blocking, N should-fix, N nice-to-have

## Blocking

### B1, one-line summary
`path/to/file.rs:123`

What is wrong, what it breaks, and what a fix must establish.

## Should-fix

## Nice-to-have

## Milestone gate

The gate, quoted from the backlog, and whether it holds. **Evidence, not
assertion**: name the test or the observation.

## Not found

Aspects checked that produced nothing, by name.
```

## Refused situations

- **Modifying any file outside `.claude/reviews/`.**
- **A fourth pass** without an explicit decision to extend the bound, recorded
  in the review file.
- **Marking the milestone gate met without evidence.** Some gates are manual,
  such as opening corpus decks in PowerPoint. Say it was performed, or say it
  was not.
- **Manufacturing findings.** Zero in a category is a valid result.

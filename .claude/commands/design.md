---
description: Design a feature before implementing. Writes a design plan, no code changes.
---

# /design F-XXX [--revise] [--draft]

Write `.claude/plans/F-XXX-design.md`. **No code changes in this command.**

The plan is a machine-consumed contract: `/start-feature` reads its test plan
and refuses a vague spec reference, and `/complete-feature` executes against its
HLD impact list. Write it as something that will be acted on, not as prose.

`--draft` is the mode `/run-sprint` uses to plan a whole sprint before
implementing any of it. It runs every step except 4 and 6, records open
questions instead of asking them, and leaves the status at `draft`. A plan left
at `draft` cannot be started.

## Steps

1. **Read the story.** Find `### F-XXX` in `docs/hld/14-development-backlog.md`.
   Take its size, dependencies and **test gate** verbatim. If the F-ID is not
   there, stop and say so.

2. **Read the cited spec sections.** Follow the story into `docs/hld/` and read
   the specific sections it implements. Read the actual code it will touch. Do
   not design from the backlog entry alone.

3. **Check dependencies.** Every F-ID in the story's `Depends on` must be `done`
   in `docs/sprints/BACKLOG.md`. If one is not, say which and stop.

4. **Ask before writing.** Collect every open question and ask them in one
   round using AskUserQuestion. Questions where a wrong guess would mean
   rewriting the story are blocking. Everything else gets a stated assumption.

   Under `--draft`, skip this. Write the questions into `## Open questions` and
   carry on, so a sprint's worth of planning produces one consolidated round of
   questions rather than one per story.

5. **Route the risk.** Apply `.claude/skills/risk-routing.md` to the diff you
   are about to describe. Record the matched rows and the exact extra checks
   each one adds in `## Risk routing`. `none` is a valid and common answer.

6. **Write the plan** using the template below.

7. **Set status to `approved`** once the questions are answered. A plan left at
   `draft` cannot be started. Under `--draft`, leave it at `draft`.

## Template

```markdown
# F-XXX, Short title

**Status**: draft | approved
**Sprint**: SNN
**Size**: S | M | L
**Depends on**: F-YYY, F-ZZZ | none

## Problem

What is wrong or missing today. Cite files and lines. One or two paragraphs.

## Spec reference

Specific sections, never a whole document. For example
`docs/hld/05-drawingml-model.md`, "Colour, the part everyone gets wrong".
`/start-feature` refuses "see docs/hld/".

## Approach

What will be built. Type signatures for anything new. Enough that a reader can
predict the diff.

## Rejected alternatives

What else was considered and why not. One line each. If nothing was considered,
say so and justify it.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `name_of_test` | ... |

The **test gate** from the backlog, named explicitly. Plus anything else.

## HLD impact

The `docs/hld/` files this story changes, as a list. `/complete-feature` updates
exactly these. "None" is valid and common.

## Risk routing

The rows of `.claude/skills/risk-routing.md` this diff triggers, and the exact
extra check each one adds. `none` is valid. `/run-sprint` takes the union
across the sprint when it builds the consolidated gate.

## Hash harness

Expected to be unchanged, or the expected delta and its justification.
**Mandatory for M1 through M6.**

## Implementation checklist

- [ ] ...

## Open questions

Answered before the status becomes approved.
```

## Refused situations

- **The F-ID is not in the backlog.** Add it there first, with a size and a test
  gate.
- **A dependency is not done.** Name it and stop.
- **The story has no test gate.** Every story has exactly one. Fix the backlog
  entry first.
- **`--revise` on a plan whose F-ID is already `done`.** Write a new F-ID
  instead. AS_BUILT entries are not rewritten.
- **Approving a plan with an unanswered blocking question.** Under `--draft`,
  the questions are deferred to `/run-sprint`, not waived.

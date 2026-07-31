---
description: Open a new sprint. Regenerates CURRENT_SPRINT.md and creates the sprint branch.
---

# /sync-sprint SNN

Open sprint `SNN`. Regenerates `docs/sprints/CURRENT_SPRINT.md` from
`SPRINT_PLAN.md` and creates the sprint branch.

## Steps

1. **Check the previous sprint is closed.** Every F-ID in the outgoing
   `CURRENT_SPRINT.md` is `done` or explicitly carried. If any is still
   `in-progress`, stop and name it.

2. **Read the plan.** Find `#### Sprint SNN` in `docs/sprints/SPRINT_PLAN.md`
   and take its goal and F-ID table. If the sprint is not there, stop. A sprint
   may omit the F-ID table only when its plan explicitly defines it as a
   validation-only boundary.

3. **Create the branch.** `sprint/sNN` off the latest `main`. Fetch first. If it
   exists already, check it out and say so rather than recreating it.

4. **Regenerate `CURRENT_SPRINT.md`** using the template below. Every row starts
   `pending` with owner `-`. For a plan-defined validation-only boundary, add
   `**Validation-only**: yes`, retain the empty wave table, and state that the
   sprint has no implementation F-IDs. Never infer validation-only status from
   a missing or malformed table.

5. **Collect the spec references.** For each F-ID, read its entry in
   `docs/hld/14-development-backlog.md` and the sections it cites, and list the
   distinct `docs/hld/` documents with one line on what each contributes. For a
   validation-only sprint, collect the documents that define the boundary and
   its gates. This section is what makes the sprint file useful rather than a
   duplicate of the plan.

6. **Write the sequencing note.** Order the rows by dependency, not by F-ID, and
   explain any ordering a reader would not predict. For a validation-only
   sprint, state that there is no implementation order and name the next work
   the boundary protects.

7. **Initialise the run state.** `python3 scripts/sprint_workflow.py init SNN`.

8. **Report** the branch, the F-ID count and the estimated days. A
   validation-only sprint reports zero F-IDs and zero estimated implementation
   days.

## Template

```markdown
# Current Sprint, SNN

**Milestone**: MN <title>.

**Goal**: <from SPRINT_PLAN, expanded to two or three sentences>

<!-- Add only for a plan-defined validation boundary. -->
**Validation-only**: yes

## Spec references

- `docs/hld/NN-topic.md`, for <what it settles for this sprint>.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-XXX | ... | S | pending | - |

## Sequencing note

Rows are listed in dependency order, not F-ID order.

<what blocks what, and why>

## Definition of done for this sprint

- <the milestone gate from SPRINT_PLAN, made concrete>
```

## Refused situations

- **The previous sprint has an `in-progress` F-ID.** Close or carry it first.
- **`SNN` is not in `SPRINT_PLAN.md`.** Add it there first.
- **No F-ID rows and no explicit validation-only boundary in the plan.** Treat
  this as a malformed sprint, not an empty sprint.
- **Uncommitted changes.** Commit or stash before branching.
- **Merging or tagging.** This command creates a branch. Only `/close-sprint`
  merges.

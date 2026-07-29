---
description: How an F-ID updates docs/hld/. The spec set describes current intent, never a change history, and the design plan's HLD impact list is the work list.
---

# Skill: hld-discipline

This repository has one authoritative document set, `docs/hld/00-vision.md`
through `15-build-and-toolchain.md`. There is no LLD tree. The spec set carries
intent and mechanism, the code carries current reality, and `docs/sprints/`
carries the execution record.

Keeping the three from diverging is the whole reason the design plan has an
`## HLD impact` section.

## The contract

`/design` writes `## HLD impact` as a list of `docs/hld/` files. That list is
consumed twice:

1. `/complete-feature` step 3 updates **exactly those files**, no others.
2. `/microscope` and `/sprint-review` look for the inverse failure, a spec
   section the implementation contradicted that the plan never listed.

`**HLD impact**: none` is valid and common. Most stories implement what the set
already says. Write `none` explicitly rather than omitting the section.

## What goes in the set

| Document | Owns |
|---|---|
| `00-vision.md`, `02-scope-and-non-goals.md` | Whether a thing is in v1 at all |
| `01-glossary.md` | OOXML vocabulary, units, the placeholder triangle |
| `03-architecture.md` | Which crate owns what, and the dependency DAG |
| `04` through `10` | How each subsystem works |
| `11-migration-plan.md` | The extraction order and its safety net |
| `12-testing-strategy.md` | The harnesses, the corpus, the gates |
| `13-risks-and-open-questions.md` | What is still undecided |
| `14-development-backlog.md` | Every story, its size, its one test gate |
| `15-build-and-toolchain.md` | Toolchain pinning, determinism, CI, packaging |

Precedence is in `docs/hld/README.md`. The lower number wins on scope and
intent, the higher number wins on mechanism.

## The update procedure

For each file the plan listed:

1. **Replace stale prose with current reality.** The set describes what is true
   now. Do not write "F-042 changed this from X to Y". Describe Y. The history
   lives in `git log` and `docs/sprints/AS_BUILT.md`.

2. **Follow the precedence rule.** If a story changes which crate owns
   something, `03-architecture.md` changes, and the mechanism document changes
   to match. Updating only one of them is the drift this skill exists to stop.

3. **Keep a story's backlog entry consistent.** If the story's shape changed,
   `14-development-backlog.md` changes too, including its size and its test
   gate. `/design` reads that entry verbatim on the next story.

4. **Do not add aspirations.** A future intention belongs in
   `13-risks-and-open-questions.md` or in a backlog story, not in the mechanism
   documents as though it were built.

5. **Do not add a changelog section.** Every document in the set is current
   state. A "what changed" heading is the first sign the set is rotting into a
   history.

## When the implementation contradicts an unlisted section

Stop. This is a refused situation in `/complete-feature`, not a paperwork
problem. Exactly one of two things is true:

- The spec was wrong, and the design plan should have said so. Revise the plan
  with `/design F-XXX --revise`, add the file to `## HLD impact`, and continue.
- The implementation drifted from the approved design. Fix the code.

Papering over it by editing an unlisted file silently is how the set stops
being trustworthy.

## The deliberately wrong entries

`CLAUDE.md` lists behaviour that is wrong and stays wrong until a named story
changes it, such as `apply_tint_shade` and the truncating unit constructors.
When you touch a document that describes one of these, describe the current
behaviour and cite why it is held. Do not describe the spec-correct behaviour
as though it shipped.

## Related

- `docs/hld/README.md`, the precedence rule and the set index.
- `.claude/commands/design.md`, which writes the impact list.
- `.claude/commands/complete-feature.md`, step 3, which executes it.
- `.claude/commands/realign-docs.md`, the bulk repair when drift has already
  accumulated.

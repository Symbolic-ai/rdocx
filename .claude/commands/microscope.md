---
description: Adversarially review one feature's diff. Records findings, changes no code.
---

# /microscope F-XXX [--working] [--aspect NAME]

Review the implementation diff for one F-ID. **This command changes no code.**
It is required before `/complete-feature` and it is not skippable.

`--working` reviews the uncommitted working tree, which is the normal mode.
Without it, review the F-ID's commits on the sprint branch.

## Steps

1. **Establish the diff.** `git diff` for `--working`, otherwise the F-ID's
   commit range against the sprint base. State the file and line count so the
   scope is on the record.

2. **Read the contract first.** `.claude/plans/F-XXX-design.md`, then the cited
   `docs/hld/` sections. A review that does not know the intent can only find
   typos.

3. **Review by aspect.** Default aspects, or one named by `--aspect`:

   | Aspect | Looks for |
   |---|---|
   | `correctness` | Wrong logic, off-by-one, unhandled cases, wrong operator precedence |
   | `contract` | Does it do what the design plan said, and nothing more |
   | `panics` | `unwrap`, `expect`, indexing, slicing, arithmetic overflow on untrusted input |
   | `ooxml` | Schema child order, namespace prefixes, whitespace preservation, unmodelled subtrees dropped rather than captured |
   | `tests` | Does the test gate actually prove the story. Would it fail if the code were reverted |
   | `structure` | The rules in `AGENTS.md`. New traits, generics, wrappers without a second implementer today |

4. **Classify every finding.**

   - **Defect**, it is wrong. Must be fixed before completion.
   - **Smell**, it will be wrong later. Fix or file an F-ID.
   - **Nitpick**, taste. Record and move on.

5. **Cite everything.** Every finding carries `path:line`. **A finding that
   cannot be cited is deleted, not softened.**

6. **Write** `.claude/reviews/F-XXX-<aspect>-pass-N.md`, incrementing `N`.

7. **Iterate.** The implementing session fixes defects and smells, then
   `/microscope` runs again as pass `N+1`. **The exit condition is zero defects
   and zero smells.** Nitpicks may remain.

## Template

```markdown
# F-XXX, <aspect>, pass N

**Reviewed**: <what diff, how many files, how many lines>
**Verdict**: N defects, N smells, N nitpicks

## Defects

### D1, one-line summary
`path/to/file.rs:123`

What is wrong. What input triggers it. What happens instead of the correct
behaviour.

## Smells

### S1, one-line summary
`path/to/file.rs:456`

## Nitpicks

- `path:line`, one line each.

## Not found

Aspects checked that produced nothing. Say so explicitly, by name.
```

## Refused situations

- **Modifying any file outside `.claude/reviews/`.** No source edits, no fixes,
  no formatting, no incidental cleanups. If you find a bug, record it and move
  on. A review that patched the code cannot report what state the code was in.
- **Manufacturing findings to fill a section.** Zero findings in a category is a
  valid and expected result. Report it as such.
- **Reviewing an F-ID that is not `in-progress`.**
- **Beginning remediation.** Stop when the review file is written.

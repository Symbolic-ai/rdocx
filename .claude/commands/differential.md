---
description: Run the differential corpus against python-docx and python-pptx. Structural comparison, pinned oracle, no inline allowlist.
---

# /differential [--oracle docx|pptx|all] [--fixture NAME]

Compare this workspace against the library it is a port of. Read
`.claude/skills/differential-testing.md` first. It holds the rules, this holds
the procedure.

Defaults to `--oracle all`.

## Steps

1. **Confirm the harness exists.** If the differential harness has not shipped
   yet, say which story owns it, cite the backlog entry, and stop. Do not
   improvise a comparison inline. A one-off comparison that is not the harness
   produces a result nobody can reproduce.

2. **Confirm the oracle is installed and pinned.** Report the resolved version
   of `python-docx` or `python-pptx` before running anything. An unpinned oracle
   turns its upgrade into your regression, at a time you did not choose, so a
   drift between the pin and the resolved version stops the run.

3. **Enumerate the corpus.** Every fixture is constructed in code. There are no
   binary fixture files, per `docs/hld/12-testing-strategy.md`. The deck corpus
   is the one exception, it is fetched rather than committed, and a missing
   corpus is a skip to report rather than a failure to hide.

4. **Run both sides per fixture**, bounded by CPU count.

5. **Compare the parsed tree, never the bytes.** Attribute order, namespace
   prefix and whitespace are ours to decide, and `docs/hld/04-opc-and-packaging.md`
   sets that contract. A byte diff against a Python writer fails on decisions
   that were made deliberately.

   For a render comparison, rasterise both sides at a stated DPI and compare
   with a stated metric and threshold. **Deterministic font mode is mandatory
   on our side.**

6. **Report per fixture**, with the disagreeing path in the tree:

   ```text
   Differential corpus, oracle python-pptx 1.0.2: 41/42 pass, 1 fail.

   FAIL  inherited_title_colour
     slide.shapes[0].text_frame.paragraphs[0].runs[0].font.color.rgb
       python-pptx: 1F497D
       rpptx:       000000
     Resolution chain stopped at the layout. See
     docs/hld/07-inheritance-and-resolution.md, "Walking to the master".
   ```

   Exit non-zero on any failure.

7. **Triage every disagreement into one of three**, and say which. This is the
   step that makes the command worth running:

   | Verdict | What happens |
   |---|---|
   | We are wrong | Fix the code. The fixture stays |
   | The oracle is wrong, or is doing something out of scope | Record the divergence with its citation, and **assert the divergence**, so a later change cannot silently drop it |
   | Both readings of ECMA-376 are defensible | Decide, write the decision into the relevant `docs/hld/` document, then assert our side |

## No inline allowlist

There is no "known differences" list. It becomes a place bugs hide, because a
line in it looks identical whether it was reasoned about last year or copied
last week.

A divergence we chose is an **assertion** that we diverge, in the test, citing
the document that holds the decision. A fixture whose expectation turned out to
be wrong is retired with its rationale, not muted.

## Relationship to the hash harness

Different questions. The harness asks whether our own output changed, and gates
every PR in M1 through M6. This asks whether our output is right, and runs where
a design plan declares it.

A harness delta plus a newly green differential test means output moved toward
the oracle. That is still a delta and it still needs declaring in the story's
`## Hash harness` section.

## When to run

- Before `/complete-feature` on any story whose test plan names `differential`
  as a category.
- On anything touching inheritance resolution, unit conversion at a boundary,
  or preset geometry. See the `.claude/skills/risk-routing.md` rows.
- After changing the oracle pin, which is the only time you should expect a
  broad result change.

## Refused situations

- **Running against an unpinned oracle.**
- **Byte-comparing against a Python writer.**
- **Adding a fixture to an allowlist instead of triaging it.**
- **Reporting a pass for a fixture that was skipped.** Say it was skipped and
  why.
- **Making an oracle a dependency of anything under `crates/*/src`.** It is
  test infrastructure.

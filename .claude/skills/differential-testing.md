---
description: How to compare this workspace against an external oracle. python-docx and python-pptx for the object model, LibreOffice for renders, and the rules that keep an oracle from becoming a second source of bugs.
---

# Skill: differential-testing

`differential` is one of the six test categories in
`docs/hld/12-testing-strategy.md`. It means comparing this workspace against an
external oracle rather than against a recorded expectation of our own.

Use it when the question is "does this match the thing we are a port of", and
use `golden` or `round-trip` when the question is "does this match what we
decided last week".

## The oracles

| Oracle | Answers | Comparison |
|---|---|---|
| `python-docx` | Does the WordprocessingML object model agree | Structural, over the parsed tree |
| `python-pptx` | Does the PresentationML object model agree, including inheritance resolution | Structural, over the parsed tree |
| LibreOffice headless | Does the render agree | Rasterise both and compare within a stated tolerance |
| PowerPoint or Word | Does the file open at all | Manual, recorded as performed or not performed |

The first two are the ones that matter most. This workspace is a port, so the
oracle is the thing being ported.

## Rules

1. **Pin the oracle version and record it.** An unpinned oracle turns its
   upgrade into your regression, at a time you did not choose. The pin belongs
   in the test harness, not in a comment.

2. **Compare the tree, not the bytes, against `python-docx` and
   `python-pptx`.** Attribute order, prefix choice and whitespace are ours to
   decide. `docs/hld/04-opc-and-packaging.md` sets the serialisation contract,
   and a byte diff against a Python writer would fail on decisions we made
   deliberately.

3. **A render comparison needs a tolerance and a reason for it.** Rasterise
   both sides at a stated DPI, compare with a stated metric, and state why the
   threshold is where it is. "Close enough" is not a gate.

4. **Deterministic font mode is mandatory on our side of a render
   comparison.** System fonts differ between machines. See
   `docs/hld/15-build-and-toolchain.md`.

5. **A disagreement is not automatically our bug.** Triage it into one of
   three, and say which in the test or the finding:
   - We are wrong. Fix the code.
   - The oracle is wrong or is doing something out of scope. Record the
     divergence as intentional with a citation, and assert the divergence so a
     later change cannot silently drop it.
   - Both are defensible readings of ECMA-376. Decide, write the decision into
     the relevant `docs/hld/` document, and assert our side.

   The third case is the reason a differential test never just prints a diff.
   An unexplained known-difference list becomes a place bugs hide.

6. **Never make the oracle a build dependency of a published crate.** It is
   test infrastructure. Nothing under `crates/*/src` may reach for it.

7. **No binary fixture files.** `docs/hld/12-testing-strategy.md` holds this
   rule for the whole workspace. Construct inputs in code, including image
   headers with precomputed CRCs. The deck corpus is the one exception, it is
   fetched rather than committed, and it lives outside the published crates.

## Where it applies

Differential tests are worth their cost where a port can be subtly wrong rather
than obviously broken:

- Inheritance resolution from slide to layout to master to theme. See
  `docs/hld/07-inheritance-and-resolution.md`. A wrong answer here is a
  plausible-looking wrong colour, not a crash.
- Unit conversion at boundaries, where our constructors truncate deliberately.
- Preset geometry, against the checked-in table `tools/gen-presets/` produced.
- Placeholder inheritance and the empty-placeholder rules.

They are not worth it where we have already decided to differ, such as the
serialisation details in rule 2.

## Relationship to the hash harness

They answer different questions and neither substitutes for the other.

- The **hash harness** asks "did our own output change". It gates every PR in
  M1 through M6 and it has no opinion about correctness.
- A **differential test** asks "is our output right". It runs where it is
  declared in a design plan's test plan.

A harness delta plus a green differential test means output moved toward the
oracle, which is still a delta and still needs declaring.

## Related

- `docs/hld/12-testing-strategy.md`, which defines the six categories.
- `.claude/commands/differential.md`, which runs the corpus.
- `.claude/skills/risk-routing.md`, the oracle-pinning row.

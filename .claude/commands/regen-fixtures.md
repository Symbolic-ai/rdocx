---
description: Regenerate the workspace's generated artefacts. Samples, the preset table, the deck corpus. Re-recording a hash baseline is deliberate and separate.
---

# /regen-fixtures [--samples] [--presets] [--corpus] [--baselines]

Rebuild the things in this workspace that are produced by a generator rather
than written by hand. With no flag, does `--samples` only, which is the safe
default.

**The four targets have very different consequences.** Read the target's
section before running it.

## `--samples`, safe

`samples/` holds generated output. It is gitignored, and it is not a fixture
source. Nothing asserts against it, so regenerating it cannot break a test.

```bash
cargo run --example <the sample generator>
```

Use it to eyeball a change by opening the result. If a sample file is what
convinced you a change is correct, that belief is not yet a test.

## `--presets`, checked in and diffable

`tools/gen-presets/` is an offline generator for the preset shape table, and
its output is checked in so a build never needs the generator.

1. Run the generator. If it has not shipped yet, name the story that owns it
   and stop. Do not hand-produce a preset table.
2. **Read the diff.** An unexpected change in the preset table is the same
   class of event as an unexpected hash-harness delta, since it feeds geometry
   for every rendered shape.
3. If it is empty, say so and stop. That is the expected result unless the
   generator or its input changed.
4. If it is not empty, the change is its own labelled commit stating what moved
   and why. Never fold it into a feature commit.

## `--corpus`, fetched, not committed

The deck corpus lives outside the published crates and is fetched rather than
committed, which is why `/corpus/` is gitignored.

1. Fetch it.
2. **Report the resolved version or manifest hash.** A corpus round-trip
   failure with no recent parser change usually means the corpus moved, and
   `.claude/WORKFLOW.md` lists that as an escalation trigger. You cannot tell
   which happened without knowing which corpus you have.
3. Never let a corpus refresh and a parser change land in one commit.

## `--baselines`, the one that needs a reason

Hash-harness baselines are **not regenerated to make a failure go away.** That
is a refused situation in `/verify`, and repeating it here does not make it
allowed.

Re-record only when all of these hold:

1. A design plan's `## Hash harness` section declared this exact delta, in
   advance, with its justification.
2. `--deterministic-fonts` is on. A baseline recorded against system fonts does
   not reproduce on another machine, and a baseline that does not reproduce is
   worse than none, because its failures are indistinguishable from real
   regressions.
3. The delta you observe matches the delta that was declared. A delta that is
   real but larger than declared is a stop, not an update.
4. The re-record is its own commit, labelled, with the expected delta in the
   message.

Report the before and after, the file count that moved, and the story that
declared it. Then say plainly that the baseline changed.

## Reporting

For each target: the command, whether the output changed, and the diff summary.
**Never report a regeneration you did not run.** If a generator is missing or a
fetch failed, say so and which target was skipped.

## Refused situations

- **Re-recording a baseline to turn `/verify` step 5 green.**
- **Re-recording a baseline against system fonts.**
- **Committing a preset-table or corpus change inside a feature commit.**
- **Treating `samples/` output as evidence.** It is not a fixture source and
  nothing asserts against it.
- **Adding a binary fixture file.** `docs/hld/12-testing-strategy.md` holds
  that rule for the whole workspace. Fixtures are constructed in code,
  including image headers with precomputed CRCs.

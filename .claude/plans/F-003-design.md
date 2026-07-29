# F-003, Output-stability hash harness

**Status**: approved
**Sprint**: S01
**Size**: L
**Depends on**: F-001

## Problem

The sample generator exercises broad library behavior, but no checked-in gate
records the serialized OOXML parts or rendered pixels. The migration described
in `docs/hld/11-migration-plan.md` can therefore change output without failing
the structural test suite.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "The hash harness".
- `docs/hld/11-migration-plan.md`, "The safety net comes first".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".

## Approach

Add `scripts/hash_harness.py` with `--check` and `--update --reason <text>`
modes. It runs the existing `generate_all_samples` example, renders page one at
150 dpi through F-001's deterministic API, and computes SHA-256 values for
`word/document.xml`, `word/styles.xml`, `word/numbering.xml`, and the PNG for
each of the seven named samples. An absent optional XML part is represented
explicitly rather than silently skipped.

Store one deterministic, sorted JSON manifest at
`scripts/hash_baseline.json`. Check mode compares generated values and reports
added, removed, and changed entries without modifying the baseline. Update mode
refuses an empty reason. Keep generated sample outputs under the existing
gitignored `samples/` directory and add PNGs to that ignore rule.

## Rejected alternatives

- Checking generated DOCX ZIP bytes was rejected because ZIP metadata would
  obscure which semantic part changed.
- Storing generated DOCX and PNG binaries was rejected by the no-binary-fixture
  rule and would make review harder than a digest manifest.
- Allowing update without a reason was rejected because it would turn a gate
  failure into an unaudited baseline rewrite.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | harness comparison tests | Missing, added, and changed digests produce precise failures, and update requires a non-empty reason. |
| golden | `python3 scripts/hash_harness.py --check` | All seven samples reproduce the checked-in 28-entry manifest. |
| regression | deliberate writer whitespace injection | Check mode fails on a byte-only XML change that the existing round-trip suite accepts. |

The **test gate** is the deliberate writer whitespace injection: the harness
passes on the unmodified tree and fails after the injected writer change.

## HLD impact

- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout and text shaping. Read `docs/hld/08-rendering-spec.md`. Render every
  PNG through F-001's deterministic path at exactly 150 dpi.
- New files. The structural rules in `CLAUDE.md` require explicit approval for
  `scripts/hash_harness.py` and `scripts/hash_baseline.json`.
- Hash baseline exclusive resource. Run `--update` only for the declared
  initial baseline, then prove `--check` is read-only and catches an injected
  XML-byte delta.

## Hash harness

Expected initial delta: add one baseline manifest containing seven samples and
four states per sample, for 28 sorted entries. This is the first baseline, not
a re-record of an existing one.

## Implementation checklist

- [ ] Add deterministic page-one PNG output to the existing sample generator.
- [ ] Add the check and reason-gated update script.
- [ ] Add unit coverage for comparison and update refusal behavior.
- [ ] Record the initial deterministic baseline.
- [ ] Prove an injected writer whitespace change fails check mode, then restore it.

## Open questions

None. `scripts/hash_harness.py` and `scripts/hash_baseline.json` are approved,
with generated PNGs remaining ignored under `samples/`.

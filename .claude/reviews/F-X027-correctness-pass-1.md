# F-X027, correctness, pass 1

**Reviewed**: the F-X027 working diff on `work/f-x027-codex`, 5 files, 71
insertions and 8 deletions. The approved design checklist, CI workflow,
workflow-contract regression, and the two listed HLD sections.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- **correctness**. `.github/workflows/ci.yml:51` defines one named golden-PNG
  step after the full workspace suite. It invokes check mode directly without
  a condition, fallback, or failure suppression.
- **contract**. `.github/workflows/ci.yml:31` installs the reviewed Poppler
  26.01.0 oracle before the new gate, and `.github/workflows/ci.yml:44` places
  the workspace suite before it. Only the two HLD files named by the approved
  plan changed.
- **panics**. `scripts/test_sprint_workflow.py:4035` uses the existing
  assertion-based YAML readers and adds no unchecked indexing, slicing, or
  arithmetic.
- **ooxml**. No parser, serialiser, namespace, XML child order, or preserved
  subtree changed.
- **tests**. `scripts/test_sprint_workflow.py:4050` exercises the real workflow
  and `scripts/test_sprint_workflow.py:4060` rejects command removal, ordering
  before Poppler, omission of `--check`, and a successful fallback. The full
  51-test module, four-test harness self-test, clean seven-sample oracle run,
  and expected failing one-pixel mutation all behaved as specified.
- **structure**. No trait, generic, wrapper, module, crate, feature flag, or
  production abstraction was added. The test-only helper centralises the
  positive and mutation assertions in one file.

## Exit condition

Zero defects and zero smells.

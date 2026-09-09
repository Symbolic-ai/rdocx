# F-X089, all, pass 2

**Reviewed**: bounded pass 1 remediation in 11 named implementation and HLD
files within the current 32-file diff against base `509940cb`, whose tracked
scope has 1,332 changed lines comprising 901 additions and 431 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, comparison validation still accepts an extra short table row
`scripts/readme_doctests.py:669`
`scripts/readme_doctests.py:674`
`scripts/test_sprint_workflow.py:5750`
`docs/hld/14-development-backlog.md:4661`

The remediation collects only table lines with exactly nine parsed cells and
silently skips every other line. A Markdown table row may omit trailing cells,
so adding `| Extra | ND | ND | ND | ND | ND | ND | ND |` to the comparison
produces another rendered project row but still returns true from
`validate_comparison_evidence`. The mutation test covers only an extra row with
all nine cells. The gate therefore does not yet satisfy the HLD statement that
it rejects extra comparison rows. Treat any non-header table line with the
wrong cell count as invalid before checking the exact project sequence.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass 1 D1**: the Python binding boundary sentence is now uninterrupted, and
  the already-run full README gate reaches a clean result.
- **Pass 1 D2, covered cases**: exact URL multiplicity, exact expected product
  order and cells, and the exact date-bounded uniqueness conclusion are now
  checked. The focused tests reject a full-width extra row, duplicate evidence,
  and a broadened conclusion.
- **Pass 1 D3**: both WASM commands now include `--scope tensorbee`, matching
  their `@tensorbee/...` imports and the required-text checks.
- **Pass 1 D4**: both facade READMEs label encryption and signing as opt-in,
  show both named features, verify those features are default-off in metadata,
  and bind the root capability matrix to DOCX-002.
- **Pass 1 N1 and N2**: both Python READMEs state unpublished Cargo-package
  status only once, in the relationship section.
- **HLD scope**: the remediation remains confined to the three HLD files listed
  by the approved design plan, and their security, WASM, evidence-count, and
  uniqueness descriptions match the corresponding checks apart from D1.
- **Mechanical checks**: `git diff --check`, `scripts/prose_check.py`, and the
  three focused capability, security, and comparison test methods passed. Full
  README and repository verification were not repeated as directed.

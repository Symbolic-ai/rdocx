# F-X171, all aspects, pass 1

**Reviewed**: `work/f-x171-codex` working diff against `c8a4a7ff`, 5 files, 96 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness, contract, panics, OOXML, tests and structure produced no finding. The macOS step at `.github/workflows/ci.yml:237` runs only in the presentation matrix cell, verifies the reviewed DMG digest before mounting, checks the exact executable build, and exposes it before pytest. The full workflow contract at `scripts/test_sprint_workflow.py:905` rejects removal, wrong matrix selection, failure swallowing, early success and an absent or commented-out digest check. The new assertion failed before the CI step and passes with it. The exact image passed a clean local mount and identity check, all 77 presentation Python cases passed with `RPPTX_PINNED_SOFFICE`, and the 49-entry hash harness is unchanged. The HLD CI row matches the job. Hosted CI remains the S85 close boundary after a reviewed main push.

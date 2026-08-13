# F-137, all, pass 9

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 1,927 added plus 12 removed lines, the approved contract and HLD, all
eight earlier reviews, progress evidence, and all pass-8 remediation
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the exact-byte attestation hashes newline-normalized text
`scripts/test_sprint_workflow.py:731`
`scripts/test_sprint_workflow.py:113`

The workflow is loaded with `Path.read_text()` and then encoded again before
hashing. Python text mode performs universal-newline translation, so a physical
CRLF workflow is normalized to LF and produces the approved digest. An
independent probe changed every LF byte pair to CRLF. Its raw SHA-256 became
`773677700cf561368e3ca6abe17ab0ad50a0d668bbe995033c4c922353ec3f93`, but
reading and re-encoding it as the test does still produced the accepted
`db89119b10d04baee6011f21513f9e7a191e39bb8b2f7715857a52217f6325ac`.
The current LF file itself has the approved raw digest and is correct, but the
implementation does not enforce the plan's requirement that any byte change
fail closed at `.claude/plans/F-137-design.md:45`.

## Smells

None.

## Nitpicks

None.

## Not found

Both pass-8 findings are rejected by the revised contract. The native pytest
and typing environment mutations plus ref or repository inputs on each
checkout are explicit non-vacuous cases at
`scripts/test_sprint_workflow.py:875`. The unchanged workflow contains neither
an environment override nor checkout inputs at
`.github/workflows/wheels.yml:29`, `.github/workflows/wheels.yml:58`, and
`.github/workflows/wheels.yml:128`.

The digest comparison is the first operation in the shared contract at
`scripts/test_sprint_workflow.py:111`. The current raw workflow digest exactly
matches the recorded value. Both positive tests pass that check and continue
through the existing structural semantic assertions at
`scripts/test_sprint_workflow.py:116`. The digest therefore fail-closes changes
without making the unchanged positive path vacuous.

The negative corpus contains 99 explicit mutations, 15 generated shortcut
mutations, and 40 generated control-flow mutations, exactly 154 in total. An
independent instrumented run observed 155 contract calls, consisting of the
unchanged positive plus 154 unique mutations. Every mutation differs from the
workflow and is rejected at `scripts/test_sprint_workflow.py:1699`.

All earlier trigger, matrix, package, platform, action, setup-python,
permission, environment, script, upload, artifact, dependency, publication,
and maturin findings remain resolved in the current workflow. No fresh YAML,
GitHub expression, runner, PyO3 abi3, action revision, OIDC isolation, secret,
artifact count, release metadata, HLD scope, risk-routing, formatting, prose,
hash-harness, or generated-artifact finding was found. The focused two-test
gate and complete 26-test workflow module passed. Prose, generated-skill, and
diff checks also passed. The full-workspace LibreOffice rider remains the
cleanly stopped environmental attempt recorded in progress and is not a
focused feature-gate failure.

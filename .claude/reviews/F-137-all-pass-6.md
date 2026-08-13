# F-137, all, pass 6

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 1,378 added plus 12 removed lines, the approved contract and HLD, all
five earlier reviews, progress evidence, and all pass-5 remediation
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, required shell gates can return success before executing their assertions
`scripts/test_sprint_workflow.py:244`
`scripts/test_sprint_workflow.py:436`

The helper checks native and musllinux step conditions, selected command
substrings, and the publication validator's glob and assertion lines, but it
does not reject earlier shell control flow. Adding `exit 0` as the first run
line independently to the wheel metadata validator, native runtime gate,
typing gate, or musllinux install all passed
`assert_wheels_workflow_contract`. Adding it before the publication validator's
Python heredoc also passed while the parsed glob and count lines remained
unchanged below it. Each mutated workflow reports success without performing
the promised check. This leaves every compatible artifact installation claim
at `docs/hld/10-bindings-spec.md:235`, the musllinux proof at
`docs/hld/12-testing-strategy.md:418`, and the pre-publication count check at
`docs/hld/10-bindings-spec.md:240` insensitive. The current shell bodies at
`.github/workflows/wheels.yml:43`, `.github/workflows/wheels.yml:58`,
`.github/workflows/wheels.yml:85`, `.github/workflows/wheels.yml:105`, and
`.github/workflows/wheels.yml:162` contain no early exit and are correct.

### D2, maturin inputs can ignore the exact matrix while expected text survives elsewhere
`scripts/test_sprint_workflow.py:305`
`scripts/test_sprint_workflow.py:332`

The action identity and peeled SHA are now exact, but neither maturin step's
`with` map is structurally compared. Changing the wheel action's target to a
fixed x86_64 target while retaining `${{ matrix.platform.target }}` in an
inline comment passed the contract. So did setting `manylinux: off` with the
matrix expression in a comment, and dropping `--locked --compatibility pypi`
from the operative arguments while preserving that phrase in a comment.
Changing only the sdist action's working directory to `crates/rdocx-py` also
passed, causing both sdist matrix rows to build rdocx. These mutations break
the exact platform or package product even though the matrix declarations stay
unchanged. The current wheel inputs at `.github/workflows/wheels.yml:36` and
sdist inputs at `.github/workflows/wheels.yml:134` correctly consume their
matrix values.

## Smells

None.

## Nitpicks

None.

## Not found

Both pass-5 findings are resolved. The helper extracts every operative action
from structured step blocks and compares the exact ordered eleven-action set,
including each reviewed peeled SHA and step position, at
`scripts/test_sprint_workflow.py:476`. Inline comments cannot satisfy an action
pin. The publish header requires exactly both build dependencies, and wheel and
sdist upload input maps require exact names, paths, and error policies at
`scripts/test_sprint_workflow.py:282` and `scripts/test_sprint_workflow.py:335`.

All earlier trigger, matrix, package, platform, permission, environment,
condition, failure-suppression, artifact, publication-order, publication-input,
action-count, and job-set findings remain resolved. The mutation corpus contains
exactly 70 non-vacuous cases and rejects every one at
`scripts/test_sprint_workflow.py:564`. The unmodified workflow passes.

No fresh current-YAML, expression, runner, action SHA, OIDC, secret, artifact
name, artifact path, dependency, release metadata, HLD, scope, formatting,
prose, hash, or generated-artifact finding was found. The complete 26-test
workflow module, Python compilation, prose, generated-skill, and diff checks
passed. The full-workspace LibreOffice rider remains the cleanly stopped
environmental attempt recorded in progress and is not a focused feature-gate
failure.

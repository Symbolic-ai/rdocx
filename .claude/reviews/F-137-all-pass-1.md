# F-137, all, pass 1

**Reviewed**: complete working diff from claim base `6ade43a`, 7 files and
403 added plus 12 removed lines, including the approved untracked wheel workflow
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the workflow contract accepts material matrix, install, and publication weakening
`scripts/test_sprint_workflow.py:77`
`scripts/test_sprint_workflow.py:93`
`scripts/test_sprint_workflow.py:109`
`scripts/test_sprint_workflow.py:138`

The helper searches the YAML text for required substrings rather than checking
the exact effective matrix, step conditions, and job predicate. Three
independent in-memory mutations all passed
`assert_wheels_workflow_contract`: adding a `python: [3.9, 3.12]` matrix axis
doubled the promised twelve wheel jobs, restricting every native install and
test step to `rdocx` skipped all compatible `rpptx` validation, and appending
`|| github.event_name == 'workflow_dispatch'` to the publish predicate allowed
manual publication. The five negative mutations at lines 171 through 180 do
not cover any of these equivalent weakenings. This contradicts the HLD claims
that the suite parses the exact product and proves clean-install and tag-only
OIDC sensitivity at `docs/hld/12-testing-strategy.md:418`, and it leaves the
manual-dispatch prohibition at `docs/hld/10-bindings-spec.md:240` without a
sensitive local gate. The current workflow is correct, but the acceptance gate
can remain green after regressions in all three central guarantees.

## Smells

None.

## Nitpicks

None.

## Not found

No additional YAML, GitHub expression, Bash, Windows path, package target,
maturin input, cp39-abi3, native install, musllinux install, source-distribution
build, package test, oracle pin, typing, stubtest, artifact upload or collection,
OIDC permission, environment, secret, action pin, release metadata, HLD impact,
scope, formatting, prose, hash, or generated-artifact finding was found.

The workflow currently defines the exact two-package by six-platform product at
`.github/workflows/wheels.yml:17`, builds through exact maturin 1.13.3 inputs at
`.github/workflows/wheels.yml:34`, validates cp39-abi3 metadata at
`.github/workflows/wheels.yml:43`, clean-installs and tests native wheels at
`.github/workflows/wheels.yml:58`, clean-installs musllinux wheels in Python 3.9
Alpine at `.github/workflows/wheels.yml:105`, builds both source distributions
at `.github/workflows/wheels.yml:118`, and collects exactly twelve wheels plus
two source distributions at `.github/workflows/wheels.yml:146`. Only that final
job receives `id-token: write`, has the `pypi` environment, and is gated to a
matching push tag. It contains no checkout, secret, tag creation, push, or
source execution.

All six action references resolve to the exact commit for their stated release,
including the peeled commit for each annotated tag. The two focused workflow
tests passed, as did prose, generated-skill, and diff checks. Preserved local
evidence contains exact maturin 1.13.3 cp39-abi3 macOS arm64 wheels and source
distributions for both packages, and clean Python 3.9 wheel and source installs
imported `rdocx==0.4.1` and `rpptx==0.4.1`. The recorded full-workspace
LibreOffice rider was environmental and stopped cleanly after permission and
duration limits. It is not reported as a feature-gate failure because the
focused F-137 gates are green and the hash record remains unchanged.

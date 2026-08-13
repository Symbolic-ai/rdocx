# F-137, all, pass 8

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 1,865 added plus 12 removed lines, the approved contract and HLD, all
seven earlier reviews, progress evidence, and all pass-7 remediation
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, step environment can turn the exact pytest command into collection only
`scripts/test_sprint_workflow.py:311`
`scripts/test_sprint_workflow.py:338`

The contract now compares every operative command in the native installation
step exactly, but it does not constrain the step's `env` mapping. Adding
`PYTEST_ADDOPTS: --collect-only` to that step passed
`assert_wheels_workflow_contract` unchanged. Pytest consumes that environment
option even when invoked through the exact retained `python -m pytest` lines,
so every listed suite can be collected without executing a test while the step
returns success. This defeats the installed-wheel pytest guarantee at
`docs/hld/10-bindings-spec.md:235` and the mutation-sensitivity claim at
`docs/hld/12-testing-strategy.md:419`. The current step at
`.github/workflows/wheels.yml:58` has no such environment and executes the
required tests correctly.

### D2, checkout action inputs can redirect both artifact builds
`scripts/test_sprint_workflow.py:643`
`scripts/test_sprint_workflow.py:653`

The action multiset binds each checkout to its position and reviewed action
revision, but it does not require an empty checkout input map. Adding
`with: { ref: main }` to either checkout passed the contract, as did adding an
alternate `repository` with that ref. A tag-triggered run could therefore build
wheels or source distributions from a branch or different repository while
retaining the exact approved checkout action tuple. The publication job would
still collect and publish those products. This violates the reviewed `py-v*`
artifact path at `docs/hld/15-build-and-toolchain.md:173`. The current wheel and
source-distribution checkouts at `.github/workflows/wheels.yml:29` and
`.github/workflows/wheels.yml:128` correctly use the event revision by default.

## Smells

None.

## Nitpicks

None.

## Not found

Both pass-7 defects are resolved for their reported forms. The five critical
run bodies now require exact normalized operative-line tuples at
`scripts/test_sprint_workflow.py:338`. False and true wrappers, `set +e`, OR
success, semicolon success, and trailing no-op forms are rejected, while the
comment-only affirmative control passes at
`scripts/test_sprint_workflow.py:1629`.

The cp39, cp312, and source-distribution setup-python steps require exact
comment-insensitive version maps, reviewed conditions, identities, action
revisions, job membership, and positions at
`scripts/test_sprint_workflow.py:226`. Wrong and comment-smuggled versions,
renamed IDs, altered conditions, and moved steps are rejected at
`scripts/test_sprint_workflow.py:869`.

All earlier matrix, package, platform, trigger, action revision, permission,
publication environment, condition, upload, artifact, dependency,
publication-order, and maturin-input findings remain resolved. The negative
corpus contains 93 explicit mutations plus 15 generated shortcut mutations
and 40 generated control-flow mutations, exactly 148 in total. Every mutation
is non-vacuous and rejected at `scripts/test_sprint_workflow.py:1623`.

No fresh current-YAML syntax, matrix execution, setup-python, PyO3 abi3,
maturin, action revision, OIDC isolation, secret, artifact count, release
metadata, HLD scope, formatting, prose, hash, or generated-artifact finding was
found. The focused two-test gate and complete 26-test workflow module passed.
Prose, generated-skill, and diff checks also passed. The first full-suite run
was blocked only by the review sandbox denying its temporary directory inside
the worker. The approved rerun passed all 26 tests. The full-workspace
LibreOffice rider remains the cleanly stopped environmental attempt recorded
in progress and is not a focused feature-gate failure.

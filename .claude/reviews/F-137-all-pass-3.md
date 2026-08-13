# F-137, all, pass 3

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 809 added plus 12 removed lines, the approved contract and HLD, both
earlier reviews, progress evidence, and all pass-2 remediation
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, publish-job permission and environment values are not exact
`scripts/test_sprint_workflow.py:103`
`scripts/test_sprint_workflow.py:316`

The root permission block is now compared exactly and both build jobs are
forbidden from overriding it, but the publish job is checked only for the
substrings `environment: pypi` and `id-token: write`. Two independent
in-memory mutations still passed `assert_wheels_workflow_contract`: changing
the publish job's `contents: read` to `contents: write`, and changing its
environment from `pypi` to `pypi-staging`. The first unnecessarily expands the
OIDC-bearing job's repository authority. The second detaches it from the exact
reviewed environment while preserving the accepted substring. This leaves the
job-level half of the exact permission and environment boundary at
`.github/workflows/wheels.yml:151` insensitive despite the HLD contract at
`docs/hld/10-bindings-spec.md:240`.

### D2, required musl and publication steps may still be conditionally skipped
`scripts/test_sprint_workflow.py:197`
`scripts/test_sprint_workflow.py:321`

The helper proves exact direct conditions for the two native steps and exact
unconditional wheel and source-distribution uploads, but it does not assert the
condition on `Install musllinux wheel` or require the publication validator and
trusted-publishing action to be unconditional. Adding `if: false` independently
to each of those three steps passed the contract. Skipping musllinux installation
removes the only clean Alpine proof promised at
`docs/hld/12-testing-strategy.md:418`. Skipping validation can publish an
unchecked artifact set, while skipping the action lets a release-tag workflow
report success without publishing. The current steps at
`.github/workflows/wheels.yml:105`, `.github/workflows/wheels.yml:162`, and
`.github/workflows/wheels.yml:173` are correct, but their execution remains
mutation-insensitive.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-2 D2 is resolved: the sdist matrix is isolated, has exactly the package
axis, and has the exact `rdocx` and `rpptx` rows at
`scripts/test_sprint_workflow.py:262`. Pass-2 D1 is resolved for its reported
forms: exact root permissions, the exact three-job set and sole publication
job, no build-job event condition or permission override, global failure
suppression rejection, and exact unconditional wheel and sdist uploads are all
enforced. The mutation corpus contains exactly 29 cases, every replacement is
proved non-vacuous, and all 29 are rejected at
`scripts/test_sprint_workflow.py:352`.

No fresh current-YAML, matrix product, package target, runner, maturin,
cp39-abi3, native install, sdist build, pytest, oracle, typing, stubtest,
artifact naming, artifact path, dependency graph, count validation, tag
predicate, action pin, secret, release metadata, HLD, scope, formatting, prose,
hash, or artifact finding was found. The current workflow continues to define
the intended exact twelve wheels and two source distributions, with no tag
creation or publication performed by the story. The complete 26-test workflow
module, Python compilation, prose, generated-skill, and diff checks passed.
The full-workspace LibreOffice rider remains the cleanly stopped environmental
attempt recorded in progress and is not a focused feature-gate failure.

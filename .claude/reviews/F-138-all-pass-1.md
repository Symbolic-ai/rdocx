# F-138, all, pass 1

**Reviewed**: complete working diff from claim base `a37af45`, 5 feature files
with 246 added and 10 removed lines, the approved plan, HLD12 and HLD15,
progress evidence, current CI and wheel workflows, and focused gates
**Verdict**: 3 defects, 1 smell, 0 nitpicks

## Defects

### D1, the contract does not require the workflow pull-request trigger
`scripts/test_sprint_workflow.py:111`
`scripts/test_sprint_workflow.py:118`

The helper requires a job-level pull-request condition but never parses the
workflow's top-level events. Removing `pull_request:` at
`.github/workflows/ci.yml:6` passed `assert_python_pr_job_contract`. The exact
job would then never be scheduled for a pull request, so its correct condition
cannot provide the PR-time gate required by `.claude/plans/F-138-design.md:23`.
The current workflow trigger is correct.

### D2, conditions and environment can suppress the exact pytest command
`scripts/test_sprint_workflow.py:197`
`scripts/test_sprint_workflow.py:207`

The helper compares the pytest script body and rejects two textual success
shortcuts, but it does not constrain the step condition or inherited
environment. Adding `if: false` to the full-suite step passed the contract and
makes both matrix rows skip the test gate successfully. Adding job-level
`PYTEST_ADDOPTS: --collect-only` also passed and makes the exact retained pytest
command collect without executing tests. Either mutation contradicts the
ordinary failure propagation specified at
`docs/hld/12-testing-strategy.md:428`. The current step at
`.github/workflows/ci.yml:67` is unconditional and has no such environment.

### D3, root permissions can grant publication authority to the job
`scripts/test_sprint_workflow.py:120`
`scripts/test_sprint_workflow.py:121`

The helper checks only the `python-bindings` block for permissions. Adding
root-level `contents: write` and `id-token: write` passed the contract, and the
job inherits those permissions because it has no job-level override. This
violates the plan requirement to keep publication permissions absent at
`.claude/plans/F-138-design.md:28`. The current workflow has no root permission
grant and is correct.

## Smells

### S1, the new job repeats mutable action references
`.github/workflows/ci.yml:38`
`.github/workflows/ci.yml:42`

The checkout, Rust toolchain, cache, and Python setup steps use movable major
or branch references rather than immutable revisions. The regression binds
only `actions/setup-python@v5` at `scripts/test_sprint_workflow.py:159`.
Changing checkout from `v5` to `v4` or rust-cache from `v2` to `v1` passed the
contract. More importantly, the unchanged references can move without any
repository diff, so code executed in the PR gate can change outside review.
The tag-release wheel workflow already demonstrates the repository's
immutable-action pattern at `.github/workflows/wheels.yml:29`.

## Nitpicks

None.

## Not found

The current YAML parses successfully and the job itself is otherwise correct.
Its matrix contains exactly `rdocx` and `rpptx`, with Python 3.12.9,
`maturin==1.13.3`, `pytest==9.1.1`, `python-docx==1.2.0`, and
`python-pptx==1.0.2` at `.github/workflows/ci.yml:27`. Each row creates a fresh
environment, runs `maturin develop --locked` before pytest, and targets the
complete package test directory. Poppler 26.01.0 is asserted by the rdocx test
suite, so an unreviewed Homebrew formula version fails closed.

All 11 implemented mutations are unique, non-vacuous, and rejected at
`scripts/test_sprint_workflow.py:230`. The complete 28-test workflow module and
focused two-test gate passed. The local sensitivity evidence records a real
rpptx oracle assertion failure and restoration. The hash harness reports all
28 entries unchanged. Prose, generated-skill, Python compilation, YAML parsing,
and diff checks passed.

The PR job does not duplicate the wheel workflow's purpose. It builds the
reviewed source with `maturin develop`, while the tag and manual workflow builds
and validates distributable artifacts. No source, manifest, lockfile, package
surface, publication action, secret, HLD-scope, WASM dependency, or artifact
finding was found. The recorded rdocx-wasm and inverse PyO3 dependency evidence
satisfies the approved risk rider, with rpptx-wasm remaining deferred to F-142.

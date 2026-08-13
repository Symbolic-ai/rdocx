# F-138, all, pass 2

**Reviewed**: complete working diff from claim base `a37af45`, 5 feature files
with 461 added and 13 removed lines, the approved revised plan, HLD12 and HLD15,
pass-1 findings, progress evidence, current workflows, and focused gates
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 D1 is resolved. The contract extracts the operative top-level event
block and requires exactly `push`, `pull_request`, and `schedule`, with an empty
pull-request mapping at `scripts/test_sprint_workflow.py:111`. Removing,
commenting, or restricting the pull-request trigger is rejected. The current
trigger is operative at `.github/workflows/ci.yml:3`.

Pass-1 D2 is resolved. The binding job's exact direct fields exclude a job
condition, environment, permission override, or other suppressor at
`scripts/test_sprint_workflow.py:131`. The pytest step permits exactly its bash
shell and run body at `scripts/test_sprint_workflow.py:248`. Job-level,
root-level, and step-level `PYTEST_ADDOPTS`, true or false job and test
conditions, either value of `continue-on-error`, success fallbacks, and extra
steps are rejected. The exact eight-step order makes environment creation and
`maturin develop --locked` precede the full package pytest directory.

Pass-1 D3 is resolved. Root permissions are exactly `contents: read`, and the
operative workflow contains no `id-token` or `write-all` grant at
`scripts/test_sprint_workflow.py:121`. Alternate content permission, additional
package or OIDC permission, and job-level permission mutations are rejected.
The current least-privilege block is at `.github/workflows/ci.yml:12`.

Pass-1 S1 is resolved. The job uses reviewed full commits for checkout,
rust-toolchain, rust-cache, and setup-python at
`.github/workflows/ci.yml:40`. Independent remote-ref inspection confirmed
checkout v6.0.2 at `de0fac2`, setup-python v6.2.0 at `a309ff8`, the peeled
rust-cache v2.9.1 commit at `c193711`, and the selected rust-toolchain stable
revision at `4360b52`. The regression binds the exact action multiset,
positions, comment-insensitive revisions, and empty or exact input maps at
`scripts/test_sprint_workflow.py:164`. Wrong revisions, comment-smuggled
revisions, checkout ref redirection, toolchain and cache inputs, and extra
setup-python inputs are rejected.

The current matrix still contains exactly the two reviewed package and oracle
rows. Python 3.12.9, maturin 1.13.3, pytest 9.1.1, python-docx 1.2.0,
python-pptx 1.0.2, isolated environments, Poppler installation, manifest paths,
and full test directories match the plan and HLD. The 34 negative mutations are
unique and non-vacuous, and all are rejected at
`scripts/test_sprint_workflow.py:285`. Equivalent trigger restriction, package
write, `if: always()`, checkout input, and extra-step probes also failed the
same helper.

The focused two-test gate and complete 28-test workflow module passed. The hash
harness independently reports all 28 entries unchanged. Prose, generated-skill,
Python compilation, YAML parsing, and diff checks passed. The recorded fresh
native evidence rebuilt both packages and passed all 33 rdocx tests and all 10
rpptx tests. No fresh correctness, shell, action, dependency, source, artifact,
wheel-workflow overlap, HLD-scope, WASM-risk, security, or test-vacuity finding
was found.

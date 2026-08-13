# F-137, all, pass 5

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 1,124 added plus 12 removed lines, the approved contract and HLD, all
four earlier reviews, progress evidence, and all pass-4 remediation
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, build action pins still accept the reviewed SHA only in a comment
`scripts/test_sprint_workflow.py:101`
`scripts/test_sprint_workflow.py:440`

The publication action and download action are now parsed and compared exactly,
but the remaining action pins are satisfied by whole-file substring searches.
Each operative `uses` value is checked only for some 40-digit hexadecimal
revision, not for its matching reviewed revision. Replacing the wheel job's
checkout pin with forty `1` digits and appending the required checkout pin in
the inline comment passed `assert_wheels_workflow_contract`. The equivalent
mutation of the wheel maturin action also passed. GitHub executes the first,
unreviewed SHA while the local gate sees the approved SHA only as comment text.
This leaves the immutable-action requirement at
`docs/hld/10-bindings-spec.md:244` mutation-insensitive. The current pins at
`.github/workflows/wheels.yml:29` are the correct reviewed commits.

### D2, dependency and upload failure policy still accept required text only in comments
`scripts/test_sprint_workflow.py:253`
`scripts/test_sprint_workflow.py:327`

Changing the publish dependency to `needs: build-wheels` while retaining
`needs: [build-wheels, build-sdists]` in an inline comment passed the contract.
The publish job can then start before the source distributions exist and fail
the count check even if their independent job later succeeds. This is not the
complete artifact dependency graph promised at
`docs/hld/10-bindings-spec.md:240`. Independently, changing a wheel upload to
`if-no-files-found: warn` while preserving `if-no-files-found: error` in its
comment also passed because the upload fields at lines 265 through 273 use raw
substring counts instead of operative direct lines. The current dependency at
`.github/workflows/wheels.yml:149` and both current upload failure policies are
correct.

## Smells

None.

## Nitpicks

None.

## Not found

Both pass-4 findings are resolved. The top-level trigger has exactly `push` and
`workflow_dispatch`, the push block has only the `py-v*` tag filter, and
commented or ignored forms are rejected at `scripts/test_sprint_workflow.py:110`.
The publish job has exactly three named steps in download, validation, publish
order. Its download inputs, validation globs and counts, action pins, and
publication directory are exact and comment-insensitive at
`scripts/test_sprint_workflow.py:370`.

All earlier matrix, package, platform, native, musllinux, typing, sdist,
permission, environment, condition, failure-suppression, artifact-name,
artifact-path, publication, and job-set findings remain resolved. The mutation
corpus contains exactly 55 non-vacuous cases and rejects every one at
`scripts/test_sprint_workflow.py:462`. The unmodified workflow passes.

No fresh current-YAML, expression, shell, runner, maturin, cp39-abi3, wheel or
sdist build, clean installation, pytest, oracle, typing, stubtest, artifact
count, secret, release metadata, HLD, scope, formatting, prose, hash, or
artifact finding was found. The complete 26-test workflow module, Python
compilation, prose, generated-skill, and diff checks passed. The full-workspace
LibreOffice rider remains the cleanly stopped environmental attempt recorded
in progress and is not a focused feature-gate failure.

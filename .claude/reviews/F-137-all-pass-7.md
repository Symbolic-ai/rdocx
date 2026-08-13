# F-137, all, pass 7

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 1,677 added plus 12 removed lines, the approved contract and HLD, all
six earlier reviews, progress evidence, and all pass-6 remediation
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, ordered command checks do not prove that shell control flow executes them
`scripts/test_sprint_workflow.py:94`
`scripts/test_sprint_workflow.py:110`

The revised helper rejects the three named success shortcuts and requires the
critical proof lines in order, but arbitrary shell control flow may still make
those lines unreachable. Wrapping the wheel metadata heredoc in `if false;
then ... fi` passed `assert_wheels_workflow_contract`. The same wrapper around
the native install and pytest body passed, as did a wrapper around the
publication validator. Separately, prepending `set +e` to metadata validation
and adding a final `:` also passed, which makes a failed assertion non-fatal
and lets the step return success. These mutations retain every required line in
order while disabling its gate semantics. They remain material to the fresh
installation and count guarantees at `docs/hld/10-bindings-spec.md:235` and
`docs/hld/10-bindings-spec.md:240`. The current bodies beginning at
`.github/workflows/wheels.yml:45`, `.github/workflows/wheels.yml:61`, and
`.github/workflows/wheels.yml:164` execute directly and are correct.

### D2, setup-python versions are not bound to their operative steps
`scripts/test_sprint_workflow.py:422`
`scripts/test_sprint_workflow.py:626`

The exact action set binds both setup-python actions to their identities and
reviewed SHA, but their `with` mappings are not structurally checked. Changing
the `cp39` step to Python 3.12 while preserving `python-version: "3.9"` in an
inline comment passed the contract. Changing `cp312` to Python 3.9 with its old
value in a comment also passed, as did changing the source-distribution setup
to Python 3.8. The first mutation no longer proves installation under the
minimum supported Python version, and the second no longer proves the exact
Python 3.12 typing environment recorded in the contract. The current setup
inputs at `.github/workflows/wheels.yml:30`, `.github/workflows/wheels.yml:80`,
and `.github/workflows/wheels.yml:129` are correct.

## Smells

None.

## Nitpicks

None.

## Not found

Both pass-6 findings are resolved for their reported forms. Exact
comment-insensitive maturin input maps now bind command, version, target,
compatibility, package directory, and output arguments at
`scripts/test_sprint_workflow.py:274` and `scripts/test_sprint_workflow.py:447`.
The five critical run bodies are parsed, required proof commands are ordered,
and operative `exit 0`, `return 0`, and `true` shortcuts are rejected while the
affirmative comment-only case remains accepted at
`scripts/test_sprint_workflow.py:1447`.

All earlier trigger, matrix, package, platform, action, permission, environment,
condition, upload, artifact, dependency, publication-order, and publication-input
findings remain resolved. The negative corpus contains 83 explicit mutations
plus 15 generated shortcut mutations, exactly 98 in total. Every mutation is
non-vacuous and rejected at `scripts/test_sprint_workflow.py:704`. The
unmodified workflow and the comment-only affirmative case pass.

No fresh current-YAML, GitHub expression, runner, maturin, action SHA, OIDC,
secret, artifact graph, release metadata, HLD, scope, formatting, prose, hash,
or generated-artifact finding was found. The complete 26-test workflow module,
Python compilation, prose, generated-skill, and diff checks passed. The
full-workspace LibreOffice rider remains the cleanly stopped environmental
attempt recorded in progress and is not a focused feature-gate failure.

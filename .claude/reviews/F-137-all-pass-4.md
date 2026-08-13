# F-137, all, pass 4

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 928 added plus 12 removed lines, the approved contract and HLD, all
three earlier reviews, progress evidence, and all pass-3 remediation
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the workflow trigger is still checked as a substring rather than a block
`scripts/test_sprint_workflow.py:101`

The contract requires only the text `tags: ["py-v*"]` somewhere in the file
and does not isolate the `on` mapping. Replacing the live filter with
`tags-ignore: ["py-v*"] # tags: ["py-v*"]` passed
`assert_wheels_workflow_contract`, as did commenting out the `push` key while
leaving the required tag text below it. Either mutation prevents the intended
release tag from starting the workflow, so the correctly gated publish job can
never run. This leaves the `py-v*` namespace requirement at
`docs/hld/10-bindings-spec.md:230` mutation-insensitive. The current trigger at
`.github/workflows/wheels.yml:3` is correct.

### D2, the publication pipeline contents and order are not structurally asserted
`scripts/test_sprint_workflow.py:307`
`scripts/test_sprint_workflow.py:343`

The helper isolates the validation and publish steps and now proves both are
unconditional, but it does not require their relative order or their operative
inputs. Moving the trusted-publishing action before validation passed the
contract, which makes the exact-count check ineffective as a publication
guard. Changing `packages-dir: dist` to `packages-dir: empty` and changing the
download destination from `dist` to `other` also passed. Further independent
mutations narrowed the validator's wheel glob to `rdocx-*.whl`, and changed its
required wheel count to one while preserving the old assertion in a comment.
Both passed because lines 307 through 313 search the whole workflow for
substrings. These mutations can publish before validation, publish nothing, or
validate a different set from the one supplied to PyPI while the local gate
remains green. They violate the exact collected publication set promised at
`docs/hld/10-bindings-spec.md:240`. The current download, validation, and
publication sequence at `.github/workflows/wheels.yml:156` is correct.

## Smells

None.

## Nitpicks

None.

## Not found

Both pass-3 findings are resolved for their reported forms. The publish header
now requires exactly the `pypi` environment and only `contents: read` plus
`id-token: write` at `scripts/test_sprint_workflow.py:328`. The musllinux step
requires exactly the musl predicate, and the validation and publishing steps
must be unconditional. All earlier matrix, native runtime, typing, upload,
sdist, job-set, permission, failure-suppression, and publication-condition
findings remain resolved.

The mutation corpus contains exactly 40 non-vacuous cases and rejects every
one at `scripts/test_sprint_workflow.py:381`. The unmodified workflow passes.
No fresh current-YAML, expression, shell, runner, maturin, cp39-abi3, wheel or
sdist build, clean installation, package test, oracle, typing, stubtest,
artifact naming, action pin, secret, release metadata, HLD, scope, formatting,
prose, hash, or artifact finding was found. The complete 26-test workflow
module, Python compilation, prose, generated-skill, and diff checks passed.
The full-workspace LibreOffice rider remains the cleanly stopped environmental
attempt recorded in progress and is not a focused feature-gate failure.

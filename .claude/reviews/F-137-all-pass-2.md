# F-137, all, pass 2

**Reviewed**: complete current working diff from claim base `6ade43a`, 7 feature
files and 521 added plus 12 removed lines, together with pass 1 and its D1
remediation
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the structural contract still accepts equivalent gate and OIDC weakenings
`scripts/test_sprint_workflow.py:94`
`scripts/test_sprint_workflow.py:140`
`scripts/test_sprint_workflow.py:195`

The revised helper correctly rejects all three pass-1 mutations. It requires
the exact two wheel-matrix axes, the exact direct conditions on both named
native steps, and the exact direct publish-job condition. It does not bound the
other job and step controls that determine whether those gates are effective.
Independent in-memory mutations adding `permissions: write-all` to
`build-wheels`, `continue-on-error: true` to either native gate or the
publication-set validator, an `rdocx`-only condition to `Upload wheel`, or a
push-only condition to `build-wheels` all passed
`assert_wheels_workflow_contract`. Adding a second unconditional publication
job after the checked job also passed. The first case grants build code OIDC,
the middle cases permit missing validation or artifacts, and the last two
break manual build coverage or tag-only publication. These directly violate
the permission and manual-dispatch boundary at
`docs/hld/10-bindings-spec.md:240` and the claimed sensitivity at
`docs/hld/12-testing-strategy.md:418`. The current YAML has none of these
weakenings, but the local contract remains insensitive to equivalent GitHub
Actions semantics.

### D2, the source-distribution matrix is not part of the exact-product assertion
`scripts/test_sprint_workflow.py:110`
`scripts/test_sprint_workflow.py:183`

The structural package assertion is scoped only to `build-wheels`. The sdist
section is checked for one generic `command: sdist`, output arguments, and a
generic artifact name, but its package matrix is never isolated or compared
with the expected two entries. Replacing the `rpptx` entry under
`build-sdists` with a second `rdocx` entry passed the contract unchanged. A
manual dispatch would therefore report success while producing no `rpptx`
source distribution, contrary to the exact two-source-distribution product at
`.github/workflows/wheels.yml:118` and `docs/hld/10-bindings-spec.md:233`.

## Smells

None.

## Nitpicks

None.

## Not found

No defect remains in the three exact pass-1 cases: the extra matrix axis,
package-restricted native runtime and typing conditions, and appended
manual-dispatch publish alternative are all rejected by the named mutation
test at `scripts/test_sprint_workflow.py:237`. The unmodified workflow passes
the same contract.

No fresh workflow implementation, YAML, expression, shell, runner target,
maturin, cp39-abi3, native or musllinux installation, pytest, oracle, typing,
stubtest, source build, artifact count, action pin, package metadata, HLD,
scope, formatting, prose, hash, or artifact finding was found. The current
workflow still has the exact intended two-by-six wheel matrix and two-package
sdist matrix, and its only publication job remains tag-gated, environment-bound,
secret-free, and isolated from source execution. The complete 26-test workflow
module, Python compilation, prose, generated-skill, and diff checks passed.
The full-workspace LibreOffice rider remains the recorded cleanly stopped
environmental attempt and is not a focused F-137 gate failure.

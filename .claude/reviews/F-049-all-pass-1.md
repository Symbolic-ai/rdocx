# F-049, all, pass 1

**Reviewed**: uncommitted working diff, 19 files, 157 insertions and 65 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, the incubating tag cannot pass through the sole release authority

`.github/workflows/publish.yml:5`
`.github/workflows/publish.yml:43`
`.claude/commands/release.md:19`
`.claude/commands/release.md:33`

The workflow now turns an `rpptx-v*` push into twelve real `cargo publish`
commands, but `/release` accepts only an exact `vX.Y.Z` argument, validates
only the seven stable packages, and creates only a `vX.Y.Z` tag. The only way
to activate the incubating path in this state is therefore to push its tag
outside `/release`, bypassing the reviewed-SHA checks, namespace-specific
version and package checks, remote-tag check, and separate final approval. This
contradicts the design rider that `/release` creates either release tag and the
repository rule that no other path may start crates.io publication. The
incubating trigger must remain unreachable until `/release` has an equivalent
namespace-aware approval path, or that path must be added with this change.

### D2, the allowlist test passes when tag routing or membership is wrong

`scripts/test_sprint_workflow.py:278`
`scripts/test_sprint_workflow.py:287`
`scripts/test_sprint_workflow.py:290`

The test counts both condition strings and each expected publish command, then
checks global text positions. It does not associate either condition with its
own command block and it does not reject unexpected publish commands. Swapping
the two `if` predicates would still pass while routing stable tags to the
incubating packages and incubating tags to the stable packages. Adding a new
command such as `cargo publish -p rdocx-wasm` would also pass. The design calls
for exact, disjoint allowlists selected by namespace, so the test must parse or
otherwise isolate each workflow step and compare its package sequence exactly.

### D3, the failure-propagation regression accepts other swallowed failures

`scripts/test_sprint_workflow.py:316`
`scripts/test_sprint_workflow.py:330`

The regression proves only that `--no-verify` and the literal text `|| echo`
are absent. A publish step with `continue-on-error: true`, `set +e`, `|| true`,
or another successful fallback would keep this test green while converting a
registry, authentication, compilation, or duplicate-version failure into a
successful job. That does not prove the design requirement that every real
publish failure propagates. The assertion needs to validate the publish steps'
error behavior rather than blacklist one spelling of a swallowed error.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the twelve implemented incubating candidates are the intended
  seven `oxml-*` and five `rpptx*` packages, and the stable and incubating
  command sequences respect the current Cargo normal-dependency graph.
- Contract: no package is uploaded by the implementation or its tests, and
  the hash harness and full workspace dry-run precede both real allowlists.
- Panics: no new Rust panic, unchecked index, slice, or arithmetic path was
  introduced.
- OOXML: no parser, serializer, namespace, schema-order, whitespace, or raw XML
  behavior changed.
- Tests: the changed manifest assertions enforce publication candidacy rather
  than weakening functional behavior. `rdocx-wasm` remains explicitly
  non-publishable, and no crates.io binding candidate was added.
- Structure: no trait, generic parameter, wrapper, feature flag, crate, module,
  or source file was added. The changes stay in existing manifests, workflow,
  assertions, and review records.
- Release mechanics: real publish commands retain Cargo verification, command
  failures currently propagate under the GitHub Actions bash shell, registry
  waits remain between dependency layers, the full dry-run archive-size check
  passed in the recorded worker evidence, and the hash harness is unchanged.

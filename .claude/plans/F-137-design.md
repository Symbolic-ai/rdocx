# F-137, wheels.yml

**Status**: completed
**Sprint**: S34
**Size**: M
**Depends on**: F-134, F-136

## Problem

No workflow builds either Python distribution outside a developer machine.
HLD10 requires six cp39-abi3 platform wheels per distribution plus source
distributions, clean-environment installation, and a separate `py-v*` trusted
publication path. The backlog currently depends only on F-134 even though the
workflow cannot build `rpptx` before F-136 creates it.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "Packaging" and "CI".
- `docs/hld/12-testing-strategy.md`, "What CI runs".
- `docs/hld/14-development-backlog.md`, "F-137, wheels.yml".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "CI job matrix".

## Approach

Add `.github/workflows/wheels.yml` for `py-v*` tags and manual dispatch. Build
both `rdocx` and `rpptx` as cp39-abi3 wheels for manylinux_2_28 x86_64 and
aarch64, musllinux_1_2 x86_64, macOS x86_64 and arm64, and Windows x86_64.
Build one source distribution per package. Use native public GitHub runners
where available and maturin-action for the declared compatibility policy.

Every matrix cell installs its wheel into a fresh environment and imports the
distribution. The package-focused cells run their installed pytest, typing,
and oracle gates when the target supports the pinned test tools. Upload each
artifact, collect all wheels and source distributions in a separate publish
job, and publish only for a `py-v*` tag. Grant `id-token: write` only to that
job, bind it to the `pypi` environment, and use PyPI trusted publishing with no
password secret. Do not create a tag or publish during this story.

Extend the existing workflow regression suite to parse the exact package and
target product, require clean-install steps and cp39-abi3 tags, require the
complete artifact dependency graph, and require tag-only OIDC publication.
Negative mutations remove a target, package, install step, artifact dependency,
or tag predicate and must fail the same contract test.

Read `.github/workflows/wheels.yml` as raw bytes and attest those exact reviewed
bytes with SHA-256 at the start of the positive contract. Decode strict UTF-8
only after the raw digest passes, then apply the structural semantic assertions.
This is an intentional supply-chain fail-closed boundary for the publication
workflow. It complements rather than replaces the semantic assertions, which
continue to explain the reviewed matrix, installation, artifact, and
publication contract.

## Rejected alternatives

- Build only rdocx. HLD10 defines both distribution names and one shared wheel
  release namespace.
- Publish from build jobs. Build code must not hold OIDC publication authority.
- Use a long-lived PyPI token. Trusted publishing is the specified boundary.
- Create a `py-v*` tag during the story. External publication belongs to the
  separately approved release path.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `wheels_workflow_covers_every_package_target_and_clean_install` | The exact 12-wheel matrix and two source distributions are complete |
| regression | workflow security mutations | OIDC stays tag-only and isolated from build jobs |
| regression, supply chain | reviewed workflow SHA-256 | Any byte change to the reviewed publication workflow fails before semantic validation |
| integration | native maturin wheel build and clean install | The current host wheel has cp39-abi3 metadata and imports from a fresh environment |

The backlog gate is represented locally by exact workflow contract tests and a
native clean-wheel install. Cross-platform execution occurs on the first
reviewed workflow dispatch or `py-v*` release run.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- WASM or PyO3 bindings. Retain workspace binding exclusions and run both WASM
  checks plus native clean-wheel installation.
- New file. Obtain explicit approval for `.github/workflows/wheels.yml`.
- Release scripting and version strings. Inspect both manifests, wheel and
  source-distribution versions, tag predicates, and OIDC permissions. Require a
  clean full gate. Do not tag or publish.

## Hash harness

Expected unchanged. Packaging automation does not alter document output.

## Implementation checklist

- [x] Add the exact two-package, six-target abi3 wheel matrix and two sdists.
- [x] Install and validate every produced artifact on its compatible runner.
- [x] Separate artifact collection and tag-only OIDC publication.
- [x] Add exact positive and negative workflow contract tests.
- [x] Build and install both native wheels locally.

## Open questions

None. The F-136 dependency, new workflow file, local static matrix proof, native
clean installs, and deferred first real cross-platform dispatch are explicitly
approved.

# F-X172, Unified release artifacts and provenance

**Status**: completed
**Sprint**: S86
**Size**: L
**Depends on**: F-X111, F-X094f

## Problem

Issue [266](https://github.com/tensorbee/rdocx/issues/266) asks for a current
release whose CLI archives and wheels carry verifiable build provenance, whose
GitHub release includes wheels and complete checksums, and whose CLI and Python
packages share one family tag. `.github/workflows/publish.yml` currently owns
Rust tags and only CLI assets. `.github/workflows/wheels.yml` owns separate
Python tags and PyPI publication. `.claude/commands/release.md` and
`docs/hld/15-build-and-toolchain.md` codify that split.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, release process, four tag namespaces
  and Python trusted publishing.
- `docs/hld/14-development-backlog.md`, F-X172 acceptance and Issue 266.
- `.claude/WORKFLOW.md`, release and sprint approval boundaries.

## Approach

Make `wheels.yml` the sole tag-triggered orchestrator for new `vX.Y.Z` and
`rpptx-vX.Y.Z` tags. Keep its PyPI publication job in that file, under the
`pypi` environment and OIDC permission, so the existing trusted publisher
identity stays valid. Move the selected CLI builds, crates.io publication and
GitHub release creation from `publish.yml` into that job graph. Manual
dispatch remains build-only and can build both Python projects before a tag.
The tag selects exactly one CLI and one Python distribution at matching
versions. It builds six CLI archives, six `cp39-abi3` wheels and one source
distribution. Attest each archive and Python artifact in its build job using a
reviewed, SHA-pinned GitHub action. Aggregate and validate the exact thirteen
payloads, generate one `SHA256SUMS`, and verify downloaded subjects with
`gh attestation verify FILE -R tensorbee/rdocx` before either registry job.
Create one GitHub release with the thirteen payloads and checksums only after
both registry jobs succeed. The release command verifies the published
registries and downloaded release payloads afterward. A partial
publication retains the immutable tag and is reported as a failure.

Update `.claude/commands/release.md`, `.claude/commands/release-notes.md`,
`.claude/WORKFLOW.md`, `scripts/sprint_workflow.py`, existing policy tests,
the build HLD and generated agent adapters. Release preparation finishes before `/close-sprint`. The release command
requires clean `main` at the exact closed sprint merge SHA, a matching
reviewed sprint source tree, a fresh full gate and a separate final approval
for each tag. Historical `py-*` tags
stay immutable and readable, but are not new publication triggers.

## Rejected alternatives

- Two tag-triggered workflows would race and require cross-run artifact
  transfer before one GitHub release could be created.
- Moving the PyPI job to `publish.yml` would require changing the external
  trusted-publisher workflow identity.
- A reusable PyPI publishing workflow is not currently a valid trusted
  publisher according to PyPI's troubleshooting guidance.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| workflow regression | `unified_release_contract_requires_complete_attested_family` | **Test gate.** Mutation checks reject omitted or extra CLI archives, wheels, source distribution, checksums, attestations, version mismatches and early registry or GitHub publication. |
| workflow regression | `manual_wheel_dispatch_is_build_only` | Manual dispatch cannot publish crates, PyPI files or a GitHub release. |
| policy | `python_release_artifacts` for `v*` and `rpptx-v*` | The selected six wheels and source distribution match name, version, metadata and target matrix. |
| local | full release workflow policy suite | Legacy published tag history remains valid and the new family contract is enforced. |

## HLD impact

- `docs/hld/15-build-and-toolchain.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Release scripting, version strings**. Read `.claude/commands/release.md`
  and `docs/hld/15-build-and-toolchain.md`. Inspect every release manifest,
  lockfile and README change. Require the clean full gate and a separate final
  approval for each tag.
- **A new feature flag or a change to default** does not match. No new flag is
  planned.

## Hash harness

Expected unchanged. The workflow and policy diff does not alter document
rendering. Confirm all 49 entries at the scoped and integrated gates.

## Implementation checklist

- [x] Consolidate the tag job graph in `wheels.yml` without a PyPI identity change.
- [x] Attest and verify exact artifacts, then write complete `SHA256SUMS`.
- [x] Gate registry and GitHub publication on the exact validated family.
- [x] Update release commands, workflow policy tests, HLD and generated adapters.
- [x] Run the scoped gate and a zero-finding microscope review.

## Open questions

None. S86 prepares `rpptx-v0.13.0` and `v0.15.0` as minor releases.
The published S85 main commit is the starting point, and the reviewed S86
source is the new release candidate. The workflow validator derives versions
from each requested tag.

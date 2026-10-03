# F-X173, Prepare unified rpptx 0.12.2 family

**Status**: approved
**Sprint**: S86
**Size**: M
**Depends on**: F-X172, F-X133, F-271, F-272, F-273

## Problem

The newest incubating tag is `rpptx-v0.12.1` and it predates the S85 main
merge. Issue [266](https://github.com/tensorbee/rdocx/issues/266) asks for a
current release with Python wheels on the GitHub release, verifiable build
provenance and matching CLI and Python versions.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, unified release process and package
  family inventory after F-X172.
- `docs/hld/14-development-backlog.md`, F-X173 release acceptance.
- `.claude/commands/release.md`, exact-SHA review and final approval gate.

## Approach

Prepare the 15 incubating package manifests and workspace pins at 0.12.2,
with the selected `rpptx-py` binding metadata at 0.12.2. Update `Cargo.lock`,
release assertions and exact archive measurements as needed. Add
`CHANGELOG.md` notes under `rpptx-v0.12.2` with highlights, additions, fixes,
compatibility and contributor credit for included work since 0.12.1. The
selected GitHub release will contain six rpptx CLI archives, six rpptx wheels,
one source distribution and checksums. Each archive and wheel must have a
verifiable attestation. Complete the F-ID after local preparation, the full
gate and clean sprint review on the integrated S86 branch. `/close-sprint`
then merges the prepared result to `main`. Run `/release rpptx-v0.12.2` at
the verified main merge SHA, obtain its separate final approval before any
release tag, and verify registry entries, assets, notes, owners and
contributor notifications after publication.

## Rejected alternatives

- Reusing `rpptx-v0.12.1` would move an immutable published tag.
- Publishing a new `py-rpptx-v*` tag would recreate the split Issue 266 asks
  to remove.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| release preparation | `rpptx_v0_12_2_unified_family_contract` | **Test gate.** Exact 15 crates, CLI and Python metadata share 0.12.2, with no stable family package in the selected publish set. |
| package | locally patched workspace publish dry run | All 22 candidate archives build and the selected 15 stay under 10 MiB. |
| Python | manual build-only `wheels.yml` run | Six wheels and source distribution pass metadata, clean-install and priority runtime checks without publication. |
| release preparation | main-SHA release preflight | The prepared manifests, notes and workflow contract support `/release rpptx-v0.12.2` after the S86 main merge. Hosted publication is a separate post-close gate. |

## HLD impact

- `docs/hld/15-build-and-toolchain.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Release scripting, version strings**. Read `.claude/commands/release.md`
  and `docs/hld/15-build-and-toolchain.md`. Inspect every manifest, lockfile
  and README version diff, run the publish dry run and require a separate
  exact-SHA approval before tagging.

## Hash harness

Expected unchanged. A version change does not change rendered output. Confirm
all 49 entries on the reviewed source.

## Implementation checklist

- [ ] Prepare and review all incubating versions, pins and Python metadata.
- [ ] Write exact changelog notes and contribution inventory.
- [ ] Complete scoped preparation, microscope, full gate and clean review.
- [ ] Complete the preparation story after the full sprint gate and review.
- [ ] After `/close-sprint`, obtain final approval and execute `/release rpptx-v0.12.2` from the reviewed main merge SHA.
- [ ] Verify publication and notifications as the post-close release gate.

## Open questions

None. The user selected both families and patch versions. This release uses
the reviewed S86 source descended from the S85 main merge.

# F-X174, Prepare unified rdocx 0.14.1 family

**Status**: approved
**Sprint**: S86
**Size**: M
**Depends on**: F-X173

## Problem

The newest stable tag is `v0.14.0` and it predates the S85 main merge. Issue
[266](https://github.com/tensorbee/rdocx/issues/266) asks for a current
release whose CLI and Python packages share one tag, with wheels on the
GitHub release and verifiable provenance.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, unified release process and stable
  package family inventory after F-X172.
- `docs/hld/14-development-backlog.md`, F-X174 release acceptance.
- `.claude/commands/release.md`, exact-SHA review and final approval gate.

## Approach

Prepare the seven stable package manifests and workspace pins at 0.14.1,
with `rdocx-py` binding metadata at 0.14.1. Pin any selected shared
dependencies to the published rpptx 0.12.2 family where the reviewed graph
requires them. Update `Cargo.lock`, release assertions and exact archive
measurements as needed. Add `CHANGELOG.md` notes under `v0.14.1` covering
included S85 and S86 work, Issue 266, compatibility and authenticated
contributor credit. The GitHub release will contain six rdocx CLI archives,
six rdocx wheels, one source distribution and checksums. Each archive and
wheel must have a verifiable attestation. Complete the F-ID after local
preparation, the full gate and clean sprint review on the integrated S86
branch. `/close-sprint` then merges the prepared result to `main`. Run
`/release v0.14.1` at that verified main merge SHA, obtain its own final
approval before any release tag, and verify registries, assets, notes, owners
and contributor notifications after publication.

## Rejected alternatives

- Reusing `v0.14.0` would move an immutable published tag.
- Publishing a new `py-rdocx-v*` tag would recreate the split Issue 266 asks
  to remove.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| release preparation | `rdocx_v0_14_1_unified_family_contract` | **Test gate.** Exact seven crates, CLI and Python metadata share 0.14.1, with no incubating package in the selected publish set. |
| package | locally patched workspace publish dry run | All 22 candidate archives build and the selected seven stay under 10 MiB. |
| Python | manual build-only `wheels.yml` run | Six wheels and source distribution pass metadata, clean-install and priority runtime checks without publication. |
| release preparation | main-SHA release preflight | The prepared manifests, notes and workflow contract support `/release v0.14.1` after the S86 main merge. Hosted publication is a separate post-close gate. |

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

- [ ] Prepare and review all stable versions, pins and Python metadata.
- [ ] Write exact changelog notes and contribution inventory.
- [ ] Complete scoped preparation, microscope, full gate and clean review.
- [ ] Complete the preparation story after the full sprint gate and review.
- [ ] After `/close-sprint`, obtain final approval and execute `/release v0.14.1` from the reviewed main merge SHA.
- [ ] Verify publication and notifications as the post-close release gate.

## Open questions

None. The user selected both families and patch versions. This release uses
the reviewed S86 source descended from the S85 main merge.

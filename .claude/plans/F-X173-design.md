# F-X173, Prepare unified rpptx 0.13.0 family

**Status**: completed
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

Prepare the 15 incubating package manifests and workspace pins at 0.13.0,
with the selected `rpptx-py` binding metadata at 0.13.0. Update `Cargo.lock`,
release assertions and exact archive measurements as needed. Add
`CHANGELOG.md` notes under `rpptx-v0.13.0` with highlights, additions, fixes,
compatibility and contributor credit for included work since 0.12.1. The
selected GitHub release will contain six rpptx CLI archives, six rpptx wheels,
one source distribution and checksums. Each archive and wheel must have a
verifiable attestation. Complete the F-ID after its scoped preparation gate
and dependency checkpoint, before F-X174 begins. The full gate and clean
sprint review run once on the final integrated S86 branch. `/close-sprint`
then merges the prepared result to `main`. Run `/release rpptx-v0.13.0` at
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
| release preparation | `rpptx_v0_13_0_unified_family_contract` | **Test gate.** Exact 15 crates, CLI and Python metadata share 0.13.0, with no stable family package in the selected publish set. |
| package | locally patched workspace publish dry run | All 22 candidate archives build and the selected 15 stay under 10 MiB. |
| Python | local selected wheel and source distribution check | Build on the available host, check metadata, clean-install and priority runtime behavior without publication. The six-platform manual build-only `wheels.yml` run follows the final reviewed S86 push, before `/close-sprint`. |
| release preparation | main-SHA release preflight | The prepared manifests, notes and workflow contract support `/release rpptx-v0.13.0` after the S86 main merge. Hosted publication is a separate post-close gate. |
| wheel smoke | Python 3.9 and 3.12 clean installs | The documented-example suite passes on the supported floor with equal-length geometry comparisons. The SHA-bound Issue 158 LibreOffice oracle is excluded from bare wheel runners. The Issue 217 CLI chain is excluded only from the Cargo-free musllinux container and remains in native and pinned CI suites. Windows CLI validation uses an eight MiB thread stack. |

## HLD impact

- `docs/hld/15-build-and-toolchain.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Release scripting, version strings**. Read `.claude/commands/release.md`
  and `docs/hld/15-build-and-toolchain.md`. Inspect every manifest, lockfile
  and README version diff, run the publish dry run and require a separate
  exact-SHA approval before tagging.
- **Wheel smoke environment**. A local Python 3.9 run found a test-only use of
  `zip(strict=True)`, which Python 3.9 cannot call, and an Issue 158 fixture
  requiring pinned LibreOffice. Keep equal-length assertions in that test.
  Exclude the viewer test from both native and musllinux release wheel smoke
  commands. The hosted rehearsal also showed that the Issue 217 chain needs
  Cargo in the musllinux container and that Windows CLI validation overflows
  the default main-thread stack. Exclude the chain only from that wheel-only
  container and run the CLI on an eight MiB thread stack on Windows. Retain
  the full chain in native and pinned CI suites.

## Hash harness

Expected unchanged. A version change does not change rendered output. Confirm
all 49 entries on the reviewed source.

## Implementation checklist

- [x] Prepare and review all incubating versions, pins and Python metadata.
- [x] Write exact changelog notes and contribution inventory.
- [x] Pass scoped preparation and a zero-finding microscope review.
- [x] Complete the preparation story at its dependency checkpoint before F-X174.

## Post-close release gate

The hosted manual build-only run uses the final reviewed sprint SHA after
`/run-sprint` pushes it. A worker does not push a preparation branch to create
this evidence. Its complete six-platform result must pass before sprint close.

After both family preparations pass the integrated full gate and clean sprint
review, `/close-sprint` merges them to `main`. `/release rpptx-v0.13.0` then
requires its own final approval at that reviewed merge SHA and verifies
publication and notifications.

## Open questions

None. The user selected both families and minor versions. This release uses
the reviewed S86 source descended from the S85 main merge.

# F-X174, Prepare unified rdocx 0.15.0 family

**Status**: completed
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

Prepare the seven stable package manifests and workspace pins at 0.15.0,
with `rdocx-py` binding metadata at 0.15.0. Pin any selected shared
dependencies to the published rpptx 0.13.0 family where the reviewed graph
requires them. Update `Cargo.lock`, release assertions and exact archive
measurements as needed. Add `CHANGELOG.md` notes under `v0.15.0` covering
included S85 and S86 work, Issue 266, compatibility and authenticated
contributor credit. The GitHub release will contain six rdocx CLI archives,
six rdocx wheels, one source distribution and checksums. Each archive and
wheel must have a verifiable attestation. Complete the F-ID after its scoped
preparation gate. The full gate and clean sprint review run once on the final
integrated S86 branch. `/close-sprint` then merges the prepared result to `main`. Run
`/release v0.15.0` at that verified main merge SHA, obtain its own final
approval before any release tag, and verify registries, assets, notes, owners
and contributor notifications after publication.
After both family releases pass, comment on Issue 266 with the two published
tag numbers and direct links to their verified GitHub releases.

## Rejected alternatives

- Reusing `v0.14.0` would move an immutable published tag.
- Publishing a new `py-rdocx-v*` tag would recreate the split Issue 266 asks
  to remove.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| release preparation | `rdocx_v0_15_0_unified_family_contract` | **Test gate.** Exact seven crates, CLI and Python metadata share 0.15.0, with no incubating package in the selected publish set. |
| package | locally patched workspace publish dry run | All 22 candidate archives build and the selected seven stay under 10 MiB. |
| Python | local selected wheel and source distribution check | Build on the available host, check metadata, clean-install and priority runtime behavior without publication. The six-platform manual build-only `wheels.yml` run follows the final reviewed S86 push, before `/close-sprint`. |
| wheel smoke | Python 3.9 and 3.12 clean installs | The release suite passes on the supported floor. The Issue 253 PDF text oracle is excluded from bare wheel runners and retained in CI with pinned Poppler 26.01.0. The Issue 158 CLI chain is excluded only from the Cargo-free musllinux container and remains in native and pinned CI suites. Windows CLI comparison uses an eight MiB thread stack. |
| release preparation | main-SHA release preflight | The prepared manifests, notes and workflow contract support `/release v0.15.0` after the S86 main merge. Hosted publication is a separate post-close gate. |

## HLD impact

- `docs/hld/15-build-and-toolchain.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Release scripting, version strings**. Read `.claude/commands/release.md`
  and `docs/hld/15-build-and-toolchain.md`. Inspect every manifest, lockfile
  and README version diff, run the publish dry run and require a separate
  exact-SHA approval before tagging.
- **Wheel smoke environment**. The Issue 253 PDF text test requires exact
  Poppler 26.01.0, which bare release wheel runners do not install. Exclude
  this test from native and musllinux wheel smoke. The hosted rehearsal also
  showed that the Issue 158 complete Word workflow needs Cargo, which the
  musllinux wheel-only container lacks. Windows CLI comparison overflowed the
  default main-thread stack. Exclude the chain only from musllinux and run
  Windows CLI work on an eight MiB thread stack. Retain the chain in native and
  pinned CI suites. Run the full local suite with pinned Poppler.

## Hash harness

Expected unchanged. A version change does not change rendered output. Confirm
all 49 entries on the reviewed source.

## Implementation checklist

- [x] Prepare and review all stable versions, pins and Python metadata.
- [x] Write exact changelog notes and contribution inventory.
- [x] Pass scoped preparation and a zero-finding microscope review.
- [x] Complete the preparation story before the integrated full gate and review.

## Post-close release gate

The hosted manual build-only run uses the final reviewed sprint SHA after
`/run-sprint` pushes it. A worker does not push a preparation branch to create
this evidence. Its complete six-platform result must pass before sprint close.

After both family preparations pass the integrated full gate and clean sprint
review, `/close-sprint` merges them to `main`. `/release v0.15.0` then
requires its own final approval at that reviewed merge SHA and verifies
publication and notifications. After both families pass, comment on Issue 266
with both published tag numbers and release links.

## Open questions

None. The user selected both families and minor versions. This release uses
the reviewed S86 source descended from the S85 main merge.

# F-X005, Tag rpptx-v0.1.2

**Status**: completed
**Sprint**: S32.2
**Size**: S
**Depends on**: F-047, F-048, F-049, F-050

## Problem

The immutable `rpptx-v0.1.0` workflow published `oxml-core` 0.1.0, then
crates.io rejected `oxml-opc` because its package description was empty. The
0.1.1 recovery added the missing descriptions, but its immutable workflow
stopped before any upload because the metadata step ran an unrelated stable
test that invokes `cargo-release`. That development tool is not installed on
the GitHub runner. The family therefore still lacks a complete registry-backed
version required by every released rdocx consumer cutover.

## Spec reference

- `docs/hld/03-architecture.md`, "Version trains".
- `docs/hld/14-development-backlog.md`, "F-X005, Tag rpptx-v0.1.2".
- `docs/hld/15-build-and-toolchain.md`, "Packaging" and "Release process".
- `.claude/commands/release.md`, "Incubating family", "Preconditions", and
  "Release".

## Approach

Retain the non-empty descriptions in every selected manifest. Narrow the
`publish.yml` metadata step to the self-contained incubating regression so the
runner verifies versions, descriptions, workspace pins, and lockfile entries
without requiring the unrelated `cargo-release` development tool. The full
release-workflow suite remains part of local and full sprint verification.

Prepare exactly `oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`,
`oxml-drawing`, `oxml-pdf`, `oxml-sml`, `rpptx-oxml`, `rpptx-chart`,
`rpptx-layout`, `rpptx-render`, and `rpptx` at 0.1.2, including their root
workspace dependency pins and `Cargo.lock`. Preserve the public
`rpptx-v0.1.0` and `rpptx-v0.1.1` tags at their reviewed SHAs. After a fresh
full verification and clean sprint review, invoke `/release rpptx-v0.1.2` and
request its separate final approval.

## Rejected alternatives

- Rerun either failed workflow. Both tags are immutable, and the 0.1.1 run
  would repeat the missing-tool failure before upload.
- Move or delete either earlier tag. Release tags are immutable under the
  release contract.
- Publish the remaining packages manually. Only the tagged GitHub workflow
  owns registry uploads.
- Mix earlier versions with 0.1.2 in one release family. The release contract
  requires one exact lockstep version.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_incubating_release_family_is_prepared_at_0_1_2` | Every selected manifest and workspace pin is 0.1.2, every package description is non-empty, and the lockfile agrees |
| workflow | targeted publish metadata step | The self-contained incubating regression runs before the first real upload without invoking `cargo-release` |
| integration | `cargo metadata --no-deps` | The exact 12-package family is publishable at 0.1.2 with consistent internal pins |
| release preflight | `/verify --full` and `/sprint-review S32.2` | A clean current HEAD satisfies every release precondition |
| publication | watched `publish.yml` run | All dependency-ordered uploads and the GitHub release succeed |
| registry, gate | `cargo info` and ownership checks | All 12 packages resolve at 0.1.2 with the expected owner and release SHA |

The backlog test gate is that all 12 incubating packages resolve from crates.io
at 0.1.2 with the expected owner, and the GitHub release targets the newly
reviewed sprint SHA.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

Describe 0.1.2 as the complete incubating family, retain the separate stable
version train, and require the self-contained package metadata regression in
the publication preflight.

## Risk routing

- Release scripting and version strings. Inspect every selected manifest,
  workspace pin, lockfile entry, workflow predicate, publish allowlist, tag,
  README version reference, and exact SHA. Require a clean full gate and the
  separate final approval before tagging.
- Crate dependency graph. Confirm the exact 12-package family and dependency
  order, including the single documented Theme adapter exception.
- Public API of published crates. State that this metadata-only patch has no
  Rust API impact. Run the exact locally patched workspace dry run and archive
  size assertion.
- Bundled fonts and assets. Verify both layout font and legal inventories, the
  `rpptx` default template, and all archive sizes below 10 MiB.

## Hash harness

Expected unchanged across all 28 entries. Package metadata, versions, and the
workflow preflight must not change document or render behaviour.

## Implementation checklist

- [x] Retain descriptions in all selected manifests.
- [x] Diagnose the 0.1.1 runner failure before any upload.
- [x] Target the self-contained regression in `publish.yml` before uploads.
- [x] Prepare exactly the 12-package family and matching pins at 0.1.2.
- [x] Run metadata, dependency, package, asset, archive, workspace, and hash gates.
- [x] Reach a clean sprint review at the exact release HEAD.
- [x] Request the separate final release approval with exact mutation details.
- [x] Run `/release rpptx-v0.1.2` and watch publication to completion.
- [x] Verify all registry owners, versions, and the GitHub release target SHA.
- [x] Update exactly the three listed HLD files and complete F-X005 only after verification.

## Open questions

None. The 0.1.1 workflow failed before any upload, and the immutable-tag
contract requires the next complete family version.

# F-X005, Tag rpptx-v0.1.1

**Status**: approved
**Sprint**: S32.2
**Size**: S
**Depends on**: F-047, F-048, F-049, F-050

## Problem

The immutable `rpptx-v0.1.0` workflow published `oxml-core` 0.1.0, then
crates.io rejected `oxml-opc` because its package description was empty. Nine
incubating manifests lack descriptions, and the local dry run did not enforce
that registry requirement. The remaining family therefore cannot be published
at 0.1.0, while all released rdocx consumer cutovers still require a complete
registry-backed shared family.

## Spec reference

- `docs/hld/03-architecture.md`, "Version trains".
- `docs/hld/14-development-backlog.md`, "F-X005, Tag rpptx-v0.1.1".
- `docs/hld/15-build-and-toolchain.md`, "Packaging" and "Release process".
- `.claude/commands/release.md`, "Incubating family", "Preconditions", and
  "Release".

## Approach

Add a concise, non-empty package description to each of the nine affected
manifests. Extend the existing release-workflow unit test to require a
description for every incubating package, and run that test in `publish.yml`
before any real upload.

Prepare exactly `oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`,
`oxml-drawing`, `oxml-pdf`, `oxml-sml`, `rpptx-oxml`, `rpptx-chart`,
`rpptx-layout`, `rpptx-render`, and `rpptx` at 0.1.1, including their root
workspace dependency pins and `Cargo.lock`. Preserve the public
`rpptx-v0.1.0` tag at its reviewed SHA. After a fresh full verification and
clean sprint review, invoke `/release rpptx-v0.1.1` and request its separate
final approval.

## Rejected alternatives

- Rerun the failed workflow. Its immutable tag still contains the missing
  metadata and would first fail because `oxml-core` 0.1.0 already exists.
- Move or delete `rpptx-v0.1.0`. Published release tags are immutable under the
  release contract.
- Publish the remaining packages manually. Only the tagged GitHub workflow
  owns registry uploads.
- Mix 0.1.0 and 0.1.1 in one release family. The incubating release contract
  requires one exact lockstep version.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_incubating_release_family_is_prepared_at_0_1_1` | Every selected manifest and workspace pin is 0.1.1, every package description is non-empty, and the lockfile agrees |
| workflow | publish metadata step | The metadata regression runs before the first real upload |
| integration | `cargo metadata --no-deps` | The exact 12-package family is publishable at 0.1.1 with consistent internal pins |
| release preflight | `/verify --full` and `/sprint-review S32.2` | A clean current HEAD satisfies every release precondition |
| publication | watched `publish.yml` run | All dependency-ordered uploads and the GitHub release succeed |
| registry, gate | `cargo info` and ownership checks | All 12 packages resolve at 0.1.1 with the expected owner and release SHA |

The backlog test gate is that all 12 incubating packages resolve from crates.io
at 0.1.1 with the expected owner, and the GitHub release targets the newly
reviewed sprint SHA.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

Describe 0.1.1 as the complete incubating family, retain the separate stable
version train, and require package descriptions in the publication preflight.

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

- [x] Add descriptions to all nine affected manifests.
- [x] Add and observe the failing release-metadata regression.
- [x] Run that regression in `publish.yml` before uploads.
- [x] Prepare exactly the 12-package family and matching pins at 0.1.1.
- [ ] Run metadata, dependency, package, asset, archive, workspace, and hash gates.
- [ ] Reach a clean sprint review at the exact release HEAD.
- [ ] Request the separate final release approval with exact mutation details.
- [ ] Run `/release rpptx-v0.1.1` and watch publication to completion.
- [ ] Verify all registry owners, versions, and the GitHub release target SHA.
- [ ] Update exactly the three listed HLD files and complete F-X005 only after verification.

## Open questions

None. The user selected the 0.1.1 recovery after the immutable partial 0.1.0
publication.

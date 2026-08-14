# F-X006, Tag the expanded rpptx family

**Status**: completed
**Sprint**: S37
**Size**: S
**Depends on**: F-143, F-144, F-145

## Problem

The immutable `rpptx-v0.1.2` release contains the earlier 12-package family.
The later `oxml-cli-support` and `rpptx-cli` packages are publishable but do
not exist at 0.1.2 on crates.io. The workspace still prepares all 14
publishable incubating manifests, plus unpublished `rpptx-wasm`, at 0.1.2 in
their manifests, root dependency pins, lockfile entries, static release
contracts, and local npm package check.

The expanded family cannot reuse the immutable 0.1.2 tag or version. It needs
one fresh common version, a complete metadata-only preparation diff, a clean
full verification and sprint review at the exact release SHA, and the separate
final approval required by `/release` before any push, tag, GitHub workflow, or
crates.io upload.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning".
- `docs/hld/14-development-backlog.md`, "F-X006, Tag the expanded rpptx
  family".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release.md`, "Incubating family", "Preconditions", "Final
  approval", and "Release".

## Approach

Prepare version 0.1.3 as the next patch version for exactly the 14 publishable
incubating packages: `oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`,
`oxml-drawing`, `oxml-pdf`, `oxml-sml`, `oxml-cli-support`, `rpptx-oxml`,
`rpptx-chart`, `rpptx-layout`, `rpptx-render`, `rpptx`, and `rpptx-cli`.
Update their explicit package versions, all 14 root workspace dependency pins,
and the corresponding `Cargo.lock` entries.

Keep unpublished `rpptx-wasm` in the existing `incubating` preparation group
and move its local manifest and lock entry to 0.1.3. Update the existing local
npm pack assertion to expect 0.1.3, but do not add npm credentials,
publication, a tag, or a registry workflow. The crates.io allowlist remains
exactly 14 packages.

Update the existing source-level manifest regressions, the structured release
metadata regression, the publish workflow's named preflight, and the local
WASM package expectation from 0.1.2 to 0.1.3. Preserve the exact publish
allowlist, dependency order, workflow authority, and package contents. Do not
change public Rust APIs, rendering behavior, the stable 0.4.1 family, or any
historical statement that specifically describes the immutable 0.1.2 release.

After implementation and independent microscope review, integrate F-X006 while
keeping its delivery status in progress. Run `/verify --full` and a clean
`/sprint-review S37` at the exact integrated SHA. Then invoke
`/release rpptx-v0.1.3`. The command must report the reviewed SHA, tag, remote,
14-package set, and workflow and ask for a separate final approval immediately
before its first external mutation. Approval of this design is not that final
release approval.

Only after the watched workflow succeeds, all 14 registry versions have the
expected owner, and the GitHub release targets the reviewed SHA may F-X006 be
completed in the sprint ledgers.

## Rejected alternatives

- Reuse 0.1.2. The tag and published versions are immutable, and the two new
  packages were not part of that release.
- Use 0.2.0. This metadata-only expansion does not justify a pre-1.0 minor
  version increase when the next patch version is free.
- Leave `rpptx-wasm` at 0.1.2. It is an existing member of the named
  `incubating` preparation group, so splitting it would contradict the current
  release metadata contract and local package version check.
- Publish the two missing packages manually at 0.1.2. Only the reviewed tagged
  workflow may publish, and crates.io versions cannot be added under an
  already immutable release tag.
- Publish npm packages during F-X006. The story and release command authorize
  only the 14-package crates.io family.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, red then green | `test_incubating_release_family_is_prepared_at_0_1_3` | All 14 publishable manifests, root pins, descriptions, and lock entries use exactly 0.1.3 |
| preparation group | `assert_release_preparation_metadata_contract` | The 15-member local preparation group includes unpublished `rpptx-wasm` at 0.1.3 while the crates.io allowlist remains 14 |
| workflow | `test_publish_workflow_preflights_and_propagates_failures` | The tagged workflow invokes the renamed 0.1.3 metadata gate before any publication and keeps failure propagation |
| local package | WASM package workflow regression | The local `@tensorbee/rpptx-wasm` pack and install check expects 0.1.3 without publication authority |
| metadata | `cargo metadata --no-deps --format-version 1` | The exact family, versions, publish eligibility, internal pins, and unchanged dependency edges are coherent |
| mutation | one manifest, workspace pin, lock entry, or workflow preflight is restored to 0.1.2 | The exact named metadata or workflow gate fails, then passes after byte-identical restoration |
| full release preflight | `/verify --full` and `/sprint-review S37` | The exact integrated release SHA is clean, all 21 package dry-runs pass, archives remain under 10 MiB, and hashes are unchanged |
| publication, backlog gate | watched `publish.yml`, `cargo info`, owner checks, and GitHub release inspection | All 14 packages resolve from crates.io at 0.1.3 with the expected owner, and `rpptx-v0.1.3` targets the reviewed sprint SHA |

The backlog test gate is that all 14 incubating packages resolve from crates.io
at the fresh version with the expected owner, and the GitHub release targets
the reviewed sprint SHA.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

Describe 0.1.3 as the current prepared 14-package incubating family before the
release, with publication explicitly pending. Describe it as published only
after the registry and GitHub release checks succeed. Preserve the immutable
0.1.2 history and the separate stable version train.

## Risk routing

- Release scripting and version strings. Inspect all 15 preparation manifests,
  all 14 workspace pins, `Cargo.lock`, source and workflow version assertions,
  README and changelog references, local and remote tags, exact action SHA, and
  registry availability. Require `/verify --full`, a clean sprint review, and
  the separate final approval before tagging.
- Crate dependency graph. Confirm no dependency edge changes, the exact
  14-package publishable family, the 15-member preparation group, and the
  dependency-ordered workflow allowlist.
- Public API of published crates. State that the patch changes package metadata
  only. Run the exact 21-package locally patched workspace dry run and reject
  any archive over 10 MiB.
- Bundled fonts and assets. Verify the complete `oxml-layout` font and legal
  inventory, confirm `rdocx-layout` does not duplicate those assets, and verify
  `rpptx/assets/default.pptx` in the generated archives.
- WASM bindings. Run the locked wasm32 checks and Node tests for both WASM
  packages, plus the local `rpptx-wasm` pack and fresh install gate at 0.1.3.

## Hash harness

Expected unchanged across all 28 entries. Version metadata and release
workflow assertions must not change document or render behavior.

## Implementation checklist

- [x] Record the 28-entry hash baseline and the exact starting manifest and
  lockfile inventory.
- [x] Add the failing 0.1.3 release metadata and workflow regressions.
- [x] Bump the 14 publishable manifests, 14 workspace pins, unpublished
  `rpptx-wasm`, and matching lock entries to 0.1.3.
- [x] Update only existing version-sensitive source, CI, and publish workflow
  assertions required by the new preparation version.
- [x] Prove manifest, pin, lockfile, and workflow sensitivity with
  byte-identical restoration.
- [x] Run metadata, dependency, WASM, package, asset, archive, workspace,
  supply-chain, prose, generated-skill, and hash gates.
- [x] Complete independent microscope review without completing the release
  story.
- [x] Prepare a validated feature handoff with the external gate deferred to
  `/release rpptx-v0.1.3`.
- [x] Integrate and run `/verify --full` plus a clean sprint review at the exact
  release SHA.
- [x] Invoke `/release rpptx-v0.1.3` and request its separate final approval.
- [x] Watch publication, verify all 14 registry entries and owners, and verify
  the GitHub release target SHA.
- [x] Update exactly the three listed HLD files to describe the prepared 0.1.3
  state with publication pending.
- [x] After successful external verification, update the three HLD files to
  the published state and complete F-X006.

## Open questions

Resolved. Version 0.1.3 and eventual tag `rpptx-v0.1.3` are approved. The
unpublished `rpptx-wasm` package remains in the existing lockstep preparation
group at 0.1.3, with no npm publication. This approval does not grant the
separate final approval required immediately before the later branch push, tag
creation, tag push, GitHub workflow, and crates.io publication.

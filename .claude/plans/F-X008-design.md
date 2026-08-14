# F-X008, Tag the updated stable rdocx family

**Status**: approved
**Sprint**: S38
**Size**: S
**Depends on**: F-X007

## Problem

The stable family is published at 0.4.1. PR 25 adds public authoring APIs and
F-X007 improves the package documentation, so the changed family needs one
fresh patch version. Direct publication would bypass the exact seven-package
contract, reviewed SHA, archive checks, and approval boundary owned by
`/release`.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-X008, Tag the updated stable rdocx
  family".
- `docs/hld/15-build-and-toolchain.md`, "The two release families" and
  "Release tags".

## Approach

Prepare the complete stable family at 0.4.2. Update workspace versions, exact
internal pins, the lockfile, and existing version-contract tests without
changing source behavior. Keep `rdocx-wasm` unpublished and do not modify the
incubating 0.1.3 family. After integration, full verification, and a clean
sprint review, invoke `/release v0.4.2`. Ask for the required separate final
approval at the exact reviewed SHA immediately before the first push or tag.

## Rejected alternatives

- Reuse 0.4.1. Published versions and tags are immutable.
- Bump only `rdocx` and `rdocx-oxml`. The release workflow owns one coherent
  seven-package stable family.
- Publish manually with Cargo. Only `/release` may start crates.io
  publication.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | stable release metadata contract | Seven publishable packages and internal pins are exactly 0.4.2 |
| integration | `/verify --full` package dry run | Exact 21-package union stages with stable 0.4.2 archives below 10 MiB |
| integration | post-release registry verification | Seven crates and the GitHub release resolve at the reviewed SHA |

The **test gate** is successful registry and GitHub verification for all seven
stable packages at 0.4.2, with the PR 25 contributor credit intact.

## HLD impact

- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect every stable manifest,
  workspace pin, lock entry, README version, and archive. Require a clean full
  gate, clean sprint review, and separate final approval before tagging.
- Crate dependency graph. Confirm only version constraints changed and the
  stable dependency direction is unchanged.

## Hash harness

Expected to be unchanged. This story changes release metadata only.

## Implementation checklist

- [ ] Confirm F-X007 is integrated, reviewed, and documented.
- [ ] Prove 0.4.2 and `v0.4.2` are absent locally and remotely.
- [ ] Prepare all stable versions, pins, lock entries, and contract tests.
- [ ] Run `/verify --full`, exact archive inventory, and hash harness.
- [ ] Obtain a clean microscope and sprint review at the release SHA.
- [ ] Ask for separate final release approval at that exact SHA.
- [ ] Run `/release v0.4.2` and watch the publication workflow.
- [ ] Verify all seven registry entries, owner, tag, release, and PR credit.

## Open questions

None. The smallest semver-compatible fresh version is 0.4.2, and the complete
stable family is the existing seven-package release set.

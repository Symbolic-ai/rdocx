# F-X008, Tag v0.5.0

**Status**: approved
**Sprint**: S38
**Size**: S
**Depends on**: F-X007

## Problem

The stable family is published at 0.4.1. F-X007 integrates public authoring
APIs and adds explicit preservation fields to the public numbering model. This
is an approved breaking pre-1.0 boundary, so the next coherent stable family is
0.5.0 rather than a 0.4 patch. Direct publication would bypass the exact
seven-package contract, reviewed SHA, archive checks, and approval boundary
owned by `/release`.

## Spec reference

- `docs/hld/11-migration-plan.md`, "What happens to the published crates" and
  "Release tooling".
- `docs/hld/14-development-backlog.md`, "F-X008, Tag v0.5.0".
- `docs/hld/15-build-and-toolchain.md`, "The two release families" and
  "Release tags".

## Approach

Prepare the complete stable family at 0.5.0 without publishing it. Set the root
workspace package version and the nine inherited-family workspace dependency
pins to 0.5.0. Regenerate the lockfile so the eleven packages in the workspace
shared-version group resolve at 0.5.0. Keep every explicit incubating package
and pin at 0.1.3, including unpublished `rpptx-wasm`.

Keep the two Python packages unpublished while setting their project metadata
to 0.5.0. Update the rdocx WASM package and CI contract literals to 0.5.0 while
retaining `publish = false`. Change the six stable README dependency examples
that still require 0.4 to require 0.5, and update the existing README contract
runner accordingly. No README prose is rewritten by a release tool.

Add one self-contained stable metadata regression to
`scripts/test_sprint_workflow.py`. It verifies the exact seven publishable
stable packages, the workspace version, nine pins, eleven lock entries, two
Python project versions, unpublished rdocx WASM state, 0.5 README requirements,
and unchanged incubating 0.1.3 versions and publication flags. Invoke that test
alongside the existing incubating regression in `.github/workflows/publish.yml`
before the patched workspace dry run.

The exact implementation files are `Cargo.toml`, `Cargo.lock`,
`crates/rdocx-py/pyproject.toml`, `crates/rpptx-py/pyproject.toml`,
`crates/rdocx-wasm/src/lib.rs`, `.github/workflows/ci.yml`,
`.github/workflows/publish.yml`, `scripts/test_sprint_workflow.py`,
`scripts/readme_doctests.py`, `README.md`, and the READMEs for `rdocx-cli`,
`rdocx-html`, `rdocx-layout`, `rdocx-opc`, and `rdocx-pdf`.

After implementation, full verification, a clean microscope, integrated
verification, and a clean sprint review, invoke `/release v0.5.0`. That command
must ask for separate final approval at the exact reviewed SHA immediately
before the first branch push, tag creation, tag push, or publication action.

## Rejected alternatives

- Reuse 0.4.1. Published versions and tags are immutable.
- Publish as 0.4.2. F-X007 intentionally adds public model fields and the user
  approved the breaking pre-1.0 0.5.0 boundary.
- Bump only `rdocx` and `rdocx-oxml`. The release workflow owns one coherent
  seven-package stable family.
- Publish manually with Cargo. Only `/release` may start crates.io
  publication.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_5_0` | Workspace version, nine pins, eleven lock entries, Python metadata, WASM literals, README requirements, exact stable publication set, and unchanged incubating 0.1.3 state are correct |
| regression | publish workflow preflight contract | The stable and incubating metadata tests run before the patched workspace dry run and no publication command gains a bypass |
| integration | `python3 scripts/readme_doctests.py` | All stable dependency examples require 0.5 and all twelve Rust examples still compile |
| integration | `/verify --full` package dry run | Exact 21-package union stages with stable 0.5.0 archives below 10 MiB |
| boundary | manifest, pin, lock, and workflow mutations | Each independent release-metadata mutation makes the stable preflight gate fail |
| integration | post-release registry verification | Seven crates and the GitHub release resolve at the reviewed SHA |

The implementation gate is the named stable metadata regression, the workflow
preflight contract, the README runner, unchanged 28-entry hash harness, exact
patched 21-package dry run, and seven stable archive inventories below 10 MiB.
The deferred release gate is successful registry and GitHub verification for
all seven stable packages at 0.5.0, with the PR 25 contributor credit intact.

## HLD impact

- `docs/hld/11-migration-plan.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Read `.claude/commands/release.md` and
  `docs/hld/15-build-and-toolchain.md`. Inspect every stable manifest,
  workspace pin, lock entry, README version, CI literal, workflow preflight,
  and archive. Run `/verify --full`, the exact patched 21-package dry run, and
  the seven stable archive inventories. Require a clean sprint review and
  separate final approval at the reviewed SHA before any external mutation.
- Crate dependency graph. Confirm only version constraints changed and the
  stable dependency direction is unchanged. Use `cargo tree`, exact metadata
  counts, and the lockfile inventory to prove the incubating 0.1.3 graph did
  not move.
- WASM or PyO3 bindings. Read `docs/hld/10-bindings-spec.md`. Run both WASM
  target checks, the locked local rdocx WASM package gate, and Python metadata
  checks without building, uploading, or publishing Python artifacts.

## Hash harness

Expected to be unchanged. This story changes release metadata only.

## Implementation checklist

- [x] Confirm F-X007 is integrated, reviewed, and documented.
- [x] Prove 0.5.0 and `v0.5.0` are absent locally and remotely.
- [x] Prepare all stable versions, pins, lock entries, and contract tests.
- [x] Run `/verify --full`, exact archive inventory, and hash harness.
- [ ] Obtain a clean microscope and sprint review at the release SHA.
- [ ] Ask for separate final release approval at that exact SHA.
- [ ] Run `/release v0.5.0` and watch the publication workflow.
- [ ] Verify all seven registry entries, owner, tag, release, and PR credit.

## Open questions

None. The approved breaking pre-1.0 version is 0.5.0, and the complete stable
family is the existing seven-package release set.

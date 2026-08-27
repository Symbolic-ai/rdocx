# F-X060, Tag v0.11.0

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-198, F-199, F-200, F-202, F-X059, F-X062, F-X063, F-X064, F-X065, F-X066, F-X067

## Problem

The stable family remains at 0.10.1. S58 introduces intentional pre-1.0 public
layout changes for language-aware hyphenation, complex-script clusters and
offsets, direction metadata, and bounded incremental layout. A stable 0.11.0
minor boundary is appropriate.

The stable packages must resolve against the shared 0.7.0 family published by
F-X059. They cannot be prepared or published before that registry graph, tag,
owners, release body, and any contribution notifications verify.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and package-family boundaries.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", "Packaging", and "WASM".
- `docs/hld/12-testing-strategy.md`, stable release metadata regressions and package checks.
- `docs/hld/14-development-backlog.md`, "F-X060, Tag v0.11.0".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release.md`, stable selection, approval, publication, verification, and notifications.

## Approach

After every dependency completes and shared 0.7.0 is independently verified,
move the workspace stable version, all stable workspace pins, inherited lock
packages, Python metadata, `rdocx-wasm` version carriers, stable CI identity,
stable README requirements, README tests, release regressions, and workflow
preflight to 0.11.0. Pin or confirm every shared dependency at 0.7.0.

Publish exactly `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`,
`rdocx-pdf`, `rdocx`, and `rdocx-cli` in dependency order. Python, WASM, npm,
PyPI, and all shared or PowerPoint crates remain outside this story's
publication authority.

Prepare `CHANGELOG.md` section `v0.11.0` from `v0.10.1` through the reviewed
SHA. Record multilingual Word layout, bounded incremental layout, the exact
pre-1.0 source impact from the final API diff, the shared 0.7.0 requirement,
and unchanged binding publication authority. Rebuild the selected-family
contribution inventory after every dependency lands. Do not duplicate records
already credited under `v0.10.1`.

The reviewed inventory must include Issues 53 and 54, authenticate
`@emptinessform`, classify the final F-X062 and F-X063 fixes, credit the
specific reporter evidence, and prepare one stable-release notification for
each issue. This story does not close either issue without separate authority.

The inventory must also include
`https://github.com/tensorbee/rdocx/pull/55` at
`056d48fdf23f35e3538ef3d6ff78cf9e3863e3a5`,
`https://github.com/tensorbee/rdocx/pull/56` at
`8b79c4cd0452defafe0a58e86b332c98e7fe52d7`, and
`https://github.com/tensorbee/rdocx/pull/57` at
`44498f042a2290ef40c7a6c26025f38e38e9ce2a`, and
`https://github.com/tensorbee/rdocx/pull/58` at
`c8fed1d1268fd765d602bac2da6524900c1c1cfd`. Authenticate contributor
`@pedroassumpcao` and classify each final outcome as a hardened equivalent or
direct adoption. Credit the specific table measurement, tracked-grid, and VML
reader contributions, plus the locked Word fidelity dependency preparation,
and prepare one release-bound thank-you for each open PR.
Publication must leave each PR open unless separate authority changes its
state. Record the intentional pre-1.0 `CT_TblGrid` literal impact if F-X065
adds the approved field. Do not claim that F-X066 renders horizontal rules.

Update the stable metadata regression for 0.11.0 and update the registry proof
so packaged `rdocx-layout@0.11.0` resolves published `oxml-layout@0.7.0`
without an `oxml-layout` patch.

After clean microscope, full verification, and sprint review at one SHA, leave
F-X060 reviewed and in progress under the release exception. `/release
v0.11.0` then asks for separate final approval and publishes exactly the seven
stable packages. F-X031 runs only after this release completes.

## Rejected alternatives

- Use 0.10.2. A patch understates intentional pre-1.0 public source changes.
- Publish before F-X059. The required shared registry graph would be unavailable.
- Publish shared and stable packages under one tag. The release contract selects exactly one family.
- Publish only `rdocx`. The stable family is a lockstep seven-package contract.
- Extend authority to Python, WASM, npm, or PyPI. S58 does not authorize those publications.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_11_0` | Stable carriers, pins, lock entries, READMEs, binding metadata, and flags agree at 0.11.0 |
| regression | published shared-family proof | Packaged `rdocx-layout@0.11.0` resolves registry `oxml-layout@0.7.0` without a local shared patch |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Family selection, order, notes, inventory, approval, and mutation authority remain pinned |
| release notes | `release-notes v0.11.0 --check` and `--render` | One deterministic stable body reconciles all selected claims and contributors |
| metadata | `cargo metadata --no-deps` | Exactly seven stable packages publish at 0.11.0 with shared pins at 0.7.0 |
| packaging | exact patched 22-package workspace dry run | Every archive verifies below 10 MiB with complete asset inventories |
| integration | Python metadata assertions and both WASM checks | Bindings track 0.11.0 without gaining publication authority |
| release | `/release v0.11.0` | Seven registry entries, owners, tag SHA, release body, and notifications verify |

The **test gate is release**. Preparation and every local gate pass at one
reviewed SHA. Completion additionally requires the separately approved real
publication and its verification.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Inspect every stable carrier,
  workflow literal, regression expectation, and changelog section. Require a
  clean full gate and separate immediate approval.
- **Public API of published crates**. Record the final pre-1.0 source impact,
  run the patched dry run, inspect archives, and enforce their limits.
- **Crate dependency graph**. Prove registry resolution against shared 0.7.0
  and preserve family direction.
- **Bundled fonts and assets**. Verify F-X058's fonts and legal files in
  `oxml-layout`, no duplication in `rdocx-layout`, and the `rpptx` template.
- **WASM and PyO3 bindings**. Update metadata only, keep crates unpublished,
  exclude both Python crates from workspace tests, and run both WASM targets.

## Hash harness

Expected unchanged relative to the reviewed S58 feature result. F-X060 changes
release metadata only and must preserve F-198's accepted baseline delta.

## Implementation checklist

- [ ] Verify every dependency and the published shared 0.7.0 family.
- [ ] Move all stable carriers to 0.11.0.
- [ ] Confirm all shared pins at 0.7.0.
- [ ] Update stable carrier and registry regressions.
- [ ] Prepare and validate selected-family notes and contribution inventory.
- [ ] Verify metadata, bindings, packages, assets, and supply chain.
- [ ] Run full verification, microscope, and clean sprint review at one SHA.
- [ ] Stop at `/release v0.11.0` for separate final approval.
- [ ] Verify all publications, owners, tag, body, and notifications.
- [ ] Complete F-X060 before F-X031 starts.

## Open questions

None. The user approved stable 0.11.0 after the verified shared 0.7.0 release.
The actual release retains its separate final go or no-go.

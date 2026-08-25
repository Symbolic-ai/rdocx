# F-X056, Tag rpptx-v0.6.0

**Status**: approved
**Sprint**: S56
**Size**: S
**Depends on**: F-X051, F-X052, F-X053, F-X054

## Problem

The immutable v0.10.0 publication attempt published `rdocx-opc` and
`rdocx-oxml`, then failed while verifying `rdocx-layout`. The packaged stable
crate resolved `oxml-layout@0.5.0` from crates.io, which does not contain
`FontManager::set_caller_aliases`, while the reviewed source calls that method
at `crates/rdocx-layout/src/engine.rs:934`. The current workspace hides this
registry gap because its package gate patches every internal dependency to the
local source graph.

The incubating family has real public and implementation changes since
`rpptx-v0.5.0`, including caller font aliases, bounded relayout work, raster
output changes, and Presentation CLI output selection. Its 0.5.0 versions and
workspace pins at `Cargo.toml:55-70` therefore no longer describe the source
that stable packages consume.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and the published family
  dependency direction.
- `docs/hld/10-bindings-spec.md`, "Packaging" and "WASM".
- `docs/hld/12-testing-strategy.md`, release regressions and full publication
  verification.
- `docs/hld/14-development-backlog.md`, "F-X056, Tag rpptx-v0.6.0".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release.md`, incubating family selection, approval,
  publication, registry verification, and contribution notification.

## Approach

Move all 15 publishable incubating manifests and the unpublished
`rpptx-wasm` preparation member from 0.5.0 to 0.6.0. Move the 16 incubating
workspace pins, lockfile records, README requirements, CI literals, WASM
package metadata, release regressions, publish preflight name, and reviewed
changelog section in the same prepared commit. Keep the stable workspace and
all stable pins at 0.10.0 for this release SHA.

The reviewed `rpptx-v0.6.0` notes cover only selected-family changes since
`rpptx-v0.5.0`. They include shared font alias and bounded relayout outcomes,
the current raster backend additions, and Presentation CLI export changes.
Issue 44, PR 45, and Issue 46 form the selected external record inventory for
the shared layout outcomes. Credit `@emptinessform`, prepare one exact
incubating-release notification per record, and do not close or otherwise
change any record state.

Run the complete local gate and the exact patched 22-package dry run. At the
reviewed clean SHA, `/release rpptx-v0.6.0` asks for a separate final approval,
publishes exactly the 15 incubating packages in dependency order, creates the
matching GitHub release, verifies every registry owner and release-body byte,
and posts the three reviewed notifications. Only after this release completes
does F-X057 change stable carriers or dependency pins.

## Rejected alternatives

- Publishing only `oxml-layout@0.5.1` breaks the lockstep family contract and
  leaves other changed incubating packages on a false immutable version.
- Moving or deleting `v0.10.0` would rewrite an externally visible release
  attempt and is forbidden.
- Re-running the failed workflow would stop on duplicate stable versions before
  reaching the missing dependency.
- Bumping stable packages at the same SHA would make the retained
  `oxml-drawing` dependency require an unpublished stable version during the
  incubating release.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_incubating_release_family_is_prepared_at_0_6_0` | All 16 incubating manifests and pins, lock entries, README requirements, WASM metadata, publication flags, and CI literals agree at 0.6.0 while stable remains 0.10.0. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Exact family allowlists, dependency order, release-note selection, contribution inventory, notifications, and tag authority remain mutation-tested. |
| release notes | `release-notes rpptx-v0.6.0 --check` and `--render` | One deterministic selected-family section contains complete highlights, additions, fixes, compatibility guidance, and authenticated credit. |
| metadata | `cargo metadata --no-deps` | Exactly 15 incubating packages are publishable at 0.6.0, the preparation-only member remains unpublished, and stable stays at 0.10.0. |
| packaging | patched 22-package workspace dry run | All local packages verify, all archives remain below 10 MiB, bundled fonts and legal files remain in `oxml-layout`, and `rpptx` retains `assets/default.pptx`. |
| integration | both WASM checks and binding metadata assertions | Unpublished bindings remain unpublished and the incubating WASM package reports 0.6.0 without entering the crates.io set. |
| release, gate | `/release rpptx-v0.6.0` post-approval verification | All 15 registry entries and owners resolve, the tag and release body match the reviewed SHA and notes, three notification URLs are recorded, and no stable package publishes. |

The **test gate is release**. All local metadata, package, supply-chain,
binding, WASM, notes, and hash checks pass at one reviewed SHA. After separate
final approval, the exact 15-package registry family, owners, tag, GitHub
release body, and selected notifications verify.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Inspect every incubating manifest,
  workspace pin, lock record, README, CI literal, workflow preflight, test
  expectation, and changelog change. Require a clean full gate and separate
  final approval before tagging.
- **Public API of published crates**. Record the pre-1.0 minor boundary, run the
  exact package dry run, verify archive contents, and enforce the 10 MiB limit.
- **Crate dependency graph**. Verify the complete 15-package allowlist,
  dependency order, stable 0.10.0 boundary, and absence of new reverse family
  edges with `cargo metadata` and `cargo tree`.
- **Bundled fonts and assets**. Verify all 20 TTFs and four legal files in the
  `oxml-layout` archive, no duplicated fonts in `rdocx-layout`, and
  `assets/default.pptx` in `rpptx`.
- **WASM or PyO3 bindings**. Run both WASM targets, both Python metadata checks,
  and workspace tests with both Python crates excluded.

## Hash harness

Expected unchanged across all 49 entries. This story changes release metadata
only. Any output delta blocks preparation.

## Implementation checklist

- [ ] Preserve the failed v0.10.0 tag and record its exact partial registry state.
- [ ] Move every incubating manifest, pin, lock record, README, WASM and CI literal, release regression, and workflow preflight to 0.6.0.
- [ ] Prepare and validate the selected-family `rpptx-v0.6.0` notes and three-record contribution inventory.
- [ ] Verify metadata, dependency order, binding isolation, package contents, archive limits, and supply chain.
- [ ] Run the full workspace gate, deterministic hash harness, and all risk riders.
- [ ] Reach a clean microscope and sprint review at the exact prepared SHA.
- [ ] Stop at `/release rpptx-v0.6.0` for separate final approval.
- [ ] After approval, verify 15 registry entries, owners, tag, release body, and three notification URLs without changing record states.

## Open questions

None. The user selected the full lockstep incubating release before the stable
patch recovery. The release command fixes the package set and approval boundary.

# F-X059, Tag rpptx-v0.7.0

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-X058

## Problem

F-X058 introduces the shared multilingual text substrate required by F-198,
F-199, and F-200. Its conditional breaks, script and font segmentation,
cluster and offset flow, direction, and visual ordering change published
pre-1.0 shared APIs and behavior. The immutable 0.6.0 versions no longer
describe that source.

The complete incubating family, its workspace pins, lock records, README
requirements, CI literals, release regressions, and the unpublished
`rpptx-wasm` preparation member must move together. The stable family remains
at 0.10.1 and stays outside this tag.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and published family direction.
- `docs/hld/10-bindings-spec.md`, "Packaging" and "WASM".
- `docs/hld/12-testing-strategy.md`, release regressions and publication verification.
- `docs/hld/14-development-backlog.md`, "F-X059, Tag rpptx-v0.7.0".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release.md`, incubating selection, approval, publication, and verification.

## Approach

Move all 15 publishable incubating manifests and workspace dependency pins,
the unpublished `rpptx-wasm` preparation manifest, and the matching lockfile
records from 0.6.0 to 0.7.0. Update every incubating README requirement, Rust
version assertion, CI literal, README gate, metadata regression, and publish
preflight. Keep the stable workspace, stable pins, stable lock packages,
Python metadata, and `rdocx-wasm` package version at 0.10.1.

Replace the incubating carrier regression with
`test_incubating_release_family_is_prepared_at_0_7_0`. Preserve the immutable
published stable graph as a historical registry regression for
`rdocx-layout@0.10.1` against `oxml-layout@0.6.0`. It must not follow the
current workspace pin to an unpublished 0.7.0 and deadlock this release.
Current 0.7.0 source coherence is proved by the exact patched 22-package dry
run.

Prepare `CHANGELOG.md` section `rpptx-v0.7.0` with Highlights, Added, Fixed,
Compatibility, and Contributors in that order. It covers only F-X058's shared
substrate. It does not claim the later stable Word integration or final oracle
acceptance owned by F-198, F-199, and F-200.

Rebuild the selected-family contribution inventory at the reviewed SHA. The
current evidence names no new external record after `rpptx-v0.6.0`. If F-X058
adds one, authenticate it, link it directly, classify the outcome, credit the
contributor, and prepare its exact post-release notification.

After clean microscope, full verification, and sprint review at one SHA, leave
F-X059 reviewed and in progress under the release exception. `/release
rpptx-v0.7.0` then asks for separate final approval and publishes exactly the
15 incubating packages.

## Rejected alternatives

- Publish only changed shared packages. The incubating family is a lockstep 15-package contract.
- Reuse `rpptx-v0.6.0`. Published versions and tags are immutable.
- Move stable versions at the same SHA. The two families require separate review and approval.
- Claim stable Word outcomes in these notes. Their integration and oracle gates have not completed.
- Require registry 0.7.0 before publication. That creates a release deadlock.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_incubating_release_family_is_prepared_at_0_7_0` | All incubating carriers and preparation-only WASM metadata agree at 0.7.0 while stable stays at 0.10.1 |
| regression | stable 0.10.1 carrier regression | No stable package enters the incubating set and current shared pins are 0.7.0 |
| regression | published stable graph regression | Immutable `rdocx-layout@0.10.1` still resolves published `oxml-layout@0.6.0` without a local patch |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Family allowlists, order, notes, inventory, approval, and mutation authority remain pinned |
| release notes | `release-notes rpptx-v0.7.0 --check` and `--render` | One deterministic selected-family body matches the reviewed inventory |
| metadata | `cargo metadata --no-deps` and selected `cargo tree` checks | Exactly 15 incubating packages publish at 0.7.0 with no reverse family edge |
| packaging | exact patched 22-package workspace dry run | Every archive verifies below 10 MiB with complete fonts, legal files, and template assets |
| integration | both WASM checks and binding metadata assertions | `rpptx-wasm` reports 0.7.0 without gaining publication authority |
| release | `/release rpptx-v0.7.0` | Registry entries, owners, tag SHA, release body, notifications, and stable exclusion verify |

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

- **Release scripting and version strings**. Inspect every carrier, workflow
  literal, test expectation, and changelog section. Require full verification
  and separate immediate approval.
- **Public API of published crates**. Record the pre-1.0 minor boundary, run
  the patched dry run, inspect archives, and enforce the 10 MiB ceiling.
- **Crate dependency graph**. Verify the exact family, order, stable isolation,
  and absence of reverse edges.
- **WASM bindings**. Run both WASM targets and metadata gates while keeping the
  package unpublished.

## Hash harness

Expected unchanged across all 49 entries relative to completed F-X058. This
story changes release metadata only. Any output delta blocks preparation.

## Implementation checklist

- [ ] Confirm F-X058 is completed and its shared boundary is final.
- [ ] Add the failing 0.7.0 carrier regression.
- [ ] Move every incubating carrier and preparation-only WASM value to 0.7.0.
- [ ] Preserve stable 0.10.1 carriers and publication exclusion.
- [ ] Preserve the historical stable registry proof without requiring unpublished 0.7.0.
- [ ] Prepare selected-family notes and contribution inventory.
- [ ] Verify metadata, order, bindings, packages, assets, and supply chain.
- [ ] Run full verification, microscope, and clean sprint review at one SHA.
- [ ] Stop at `/release rpptx-v0.7.0` for separate final approval.
- [ ] Verify all publications, owners, tag, body, and notifications.

## Open questions

None. The user approved the complete 0.7.0 incubating release boundary. Stable
integration remains in F-198, F-199, and F-200.

# F-X005, Tag rpptx-v0.1.0

**Status**: approved
**Sprint**: S32.2
**Size**: S
**Depends on**: F-047, F-048, F-049, F-050

## Problem

The 12 implemented shared and PowerPoint packages remain at the reserved
placeholder version 0.0.0 in their manifests and workspace pins. S32.2's
released rdocx consumers cannot pass registry-backed archive verification until
the real shared implementations exist on crates.io. The sole release workflow
also requires an in-progress F-ID named exactly for the requested tag.

## Spec reference

- `docs/hld/03-architecture.md`, version families and dependency direction.
- `docs/hld/11-migration-plan.md`, release tooling and package allowlists.
- `docs/hld/14-development-backlog.md`, "F-X005, Tag rpptx-v0.1.0".
- `docs/hld/15-build-and-toolchain.md`, publication candidates and release
  process.
- `.claude/commands/release.md`, exact incubating release contract.

## Approach

Use the configured `incubating` cargo-release group to prepare version 0.1.0
for exactly `oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`,
`oxml-drawing`, `oxml-pdf`, `oxml-sml`, `rpptx-oxml`, `rpptx-chart`,
`rpptx-layout`, `rpptx-render`, and `rpptx`. Update their corresponding root
workspace dependency pins and `Cargo.lock`. Inspect the diff to prove no stable
version, README prose, tag, push, or publication changes.

Integrate the version preparation as the F-X005 commit, keep the feature
reviewed and in progress, then run the full verification and clean sprint
review at that exact clean HEAD. Invoke `/release rpptx-v0.1.0` only after its
preconditions pass. Immediately before the first external mutation, report the
exact SHA, tag, package set, version, remote, and workflow, then request the
separate mandatory approval.

After approval, let `/release` push the active sprint branch, create and push
only the annotated requested tag, watch `publish.yml`, and verify all 12
registry versions, expected ownership, and the GitHub release target SHA. Only
then complete F-X005 and allow the released rdocx consumer cutovers to start.

## Rejected alternatives

- Cut consumers over to local path dependencies first. Published archives must
  resolve the real registry graph.
- Publish a partial subset. The incubating workflow and lockstep version group
  define exactly 12 packages.
- Use the reserved 0.0.0 placeholders. They expose no usable API.
- Create the tag before exact-SHA verification or reuse earlier approval. The
  release command requires a new approval at the mutation boundary.
- Publish from a local shell. Only the tagged GitHub workflow owns uploads.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| version preparation | cargo-release incubating group | Exactly 12 manifests, 12 workspace pins, and `Cargo.lock` move to 0.1.0 |
| regression | scoped preparation diff | No stable version, README, workflow allowlist, tag, push, or publication mutation occurs |
| integration | `cargo metadata --no-deps` | The exact family is publishable at one version and internal pins agree |
| release preflight | `/verify --full` and `/sprint-review S32.2` | The clean current HEAD satisfies every release precondition |
| publication | watched `publish.yml` run | All dependency-ordered uploads and the GitHub release succeed |
| registry, gate | `cargo info` and ownership checks | All 12 packages resolve at 0.1.0 with the expected owner and release SHA |

The backlog gate is successful registry resolution for all 12 packages and a
matching GitHub release at the reviewed SHA.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/15-build-and-toolchain.md`

Replace development-only 0.0.0 wording with the published 0.1.0 incubating
family and retain the separate stable version train.

## Risk routing

- Release scripting and version strings. Audit every manifest, pin, lockfile
  entry, tag predicate, allowlist, and exact SHA before external mutation.
- Crate dependency graph. Confirm the 12-package family and dependency order,
  including the single documented Theme adapter exception.
- Public API of newly published crates. Run the full locally patched workspace
  dry-run, archive verification, and metadata inspection.
- Bundled fonts and assets. Verify `oxml-layout` font and legal inventories,
  the `rpptx` default template, and all archive sizes below 10 MiB.
- External mutation. Stop for the separate `/release` approval immediately
  before the branch push and requested tag creation.

## Hash harness

Expected unchanged across all 28 entries. Version preparation and publication
must not change document or render behavior.

## Implementation checklist

- [ ] Prepare exactly the incubating family and matching pins at 0.1.0.
- [ ] Inspect the complete manifest and lockfile diff.
- [ ] Run metadata, dependency, package, asset, archive, workspace, and hash gates.
- [ ] Reach a clean sprint review at the exact release HEAD.
- [ ] Request the separate final release approval with exact mutation details.
- [ ] Run `/release rpptx-v0.1.0` and watch publication to completion.
- [ ] Verify all registry owners, versions, and the GitHub release target SHA.
- [ ] Update exactly the two listed HLD files and complete F-X005 only after verification.

## Open questions

None before the mandatory final release approval. That approval is deliberately
deferred until every precondition passes at the exact reviewed SHA.

# F-048, Automate split-family release preparation

**Status**: approved
**Sprint**: S32.1
**Size**: M
**Depends on**: none

## Problem

`Cargo.toml:26` defines one released workspace version while lines 39 through
56 carry two version trains in the internal dependency pins. There is no
`cargo-release` metadata that groups the stable rdocx packages separately from
the incubating `oxml-*` and `rpptx*` packages, so a preparation bump cannot yet
update one family and all of its dependency requirements mechanically.

## Spec reference

- `docs/hld/11-migration-plan.md`, "Release tooling".
- `docs/hld/14-development-backlog.md`, "F-048, Automate split-family release
  preparation".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".
- `docs/hld/15-build-and-toolchain.md`, "Release process".

## Approach

Configure `cargo-release` through existing Cargo metadata. Use named
`shared-version` groups for the stable rdocx train and the incubating shared
and PowerPoint train. The stable group uses `v{{version}}`, and the incubating
group uses `rpptx-v{{version}}`. Common workspace settings consolidate the
preparation commit, upgrade internal dependency requirements, retain archive
verification, and disable publishing, tagging, and pushing because `/release`
owns those external actions.

The stable preparation selects the released rdocx packages and updates
`[workspace.package]` plus every stable `[workspace.dependencies]` pin. The
incubating preparation selects the implemented `oxml-*` and `rpptx*` packages
and updates their explicit versions plus every corresponding workspace pin.
Neither group defines README replacements. A temporary clean worktree dry-run
proves the exact manifest and lockfile diff without retaining the trial bump.

## Rejected alternatives

- Restore a global search-and-replace release script. It can rewrite unrelated
  version prose and cannot express the two version trains safely.
- Let `cargo-release` tag, push, or publish. Those actions belong exclusively
  to the reviewed `/release` boundary.
- Add a separate release configuration file. Cargo metadata keeps the package
  group beside the version declaration it controls.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | stable `cargo release` preparation dry-run in a temporary clean worktree | `[workspace.package]`, stable package versions, internal stable pins, and `Cargo.lock` move together |
| integration | incubating `cargo release` preparation dry-run in a temporary clean worktree | Every implemented `oxml-*` and `rpptx*` version and matching workspace pin moves together under the `rpptx-v*` namespace |
| regression | inspect both dry-run diffs with `git diff --name-only` and `rg` | No README prose changes, no tag, no push, and no publication occur |
| unit | `python3 -m unittest scripts/test_sprint_workflow.py` | Stable lockstep invariants and split-family release metadata remain machine checked |

The backlog test gate is a dry-run bump of the workspace version updating
`[workspace.package]` and every `[workspace.dependencies]` pin while touching
no README prose.

## HLD impact

- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect every manifest, lockfile, and
  README diff from both dry-runs. Require a clean full gate and retain the
  separate `/release` approval before any tag or external mutation.
- Crate dependency graph. Run `cargo metadata --no-deps` after each prepared
  family bump and confirm no `oxml-*` dependency points into `rdocx-*` or
  `rpptx-*` beyond the documented Theme adapter exception.

## Hash harness

Expected to remain unchanged. Release metadata and version preparation do not
alter document or rendering behavior.

## Implementation checklist

- [ ] Add common preparation-only `cargo-release` workspace metadata.
- [ ] Assign every package to the stable or incubating named version group.
- [ ] Pin the stable and incubating tag-name templates without granting tag
      authority to `cargo-release`.
- [ ] Extend the existing workflow tests with split-family configuration
      invariants.
- [ ] Run and inspect both temporary preparation dry-runs.

## Open questions

None. The specification fixes the two tag namespaces and keeps all external
release actions behind `/release`.

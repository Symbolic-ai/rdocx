# F-049, Extend publish.yml to the extracted workspace

**Status**: completed
**Sprint**: S32.1
**Size**: M
**Depends on**: F-048

## Problem

`.github/workflows/publish.yml:5` accepts only stable `v*` tags, and lines 23
through 60 publish only the seven released rdocx crates. The implemented
shared and PowerPoint packages remain guarded by `publish = false`, so the
workflow cannot dry-run or publish the completed incubating graph after a
separately approved release preparation. The sole release authority in
`.claude/commands/release.md` also accepts and validates only the stable
namespace, so adding the incubating trigger without extending that authority
would leave no compliant path to activate it.

## Spec reference

- `docs/hld/11-migration-plan.md`, "Release tooling".
- `docs/hld/14-development-backlog.md`, "F-049, Extend publish.yml to the
  extracted workspace".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".
- `docs/hld/15-build-and-toolchain.md`, "Release process".

## Approach

Remove the development publication guards from the implemented `oxml-*` and
`rpptx*` packages, with `oxml-layout` eligibility supplied by F-047. Extend the
existing publication workflow to accept both `v*` and `rpptx-v*` tags. Keep one
explicit dependency-ordered stable allowlist and one explicit
dependency-ordered incubating allowlist, selected by the tag namespace.

Extend `/release` to accept either `vX.Y.Z` or `rpptx-vX.Y.Z`. The stable path
continues to validate the workspace version and exact seven-package rdocx
allowlist. The incubating path validates the explicit version shared by the
exact 12 completed `oxml-*` and `rpptx*` candidates and their workspace pins.
Both paths remain bound to the clean reviewed SHA, full verification, a clean
sprint review, absent local and remote tags, the separate final approval, and
post-publication registry plus GitHub release verification. Regenerate the
Codex skill adapters after changing the command.

Before either real allowlist runs, reproduce the hash harness and run
`cargo publish --workspace --dry-run`. Cargo's workspace dry-run stages the
selected local packages together, so dependent archive verification uses the
workspace candidates without uploading them. Keep every real `cargo publish`
step verified, propagate all failures, and retain the registry wait between
dependency layers. Extend the existing workflow tests to pin both allowlists,
their order, tag routing, and the absence of `--no-verify` or swallowed errors.
The tests isolate the two publish step blocks and compare each condition and
bare command sequence exactly. Negative mutations swap predicates, add an
extra package, set `continue-on-error`, and add successful fallback commands.

## Rejected alternatives

- Replace the allowlists with an unqualified real `cargo publish --workspace`.
  A stable tag must not publish the incubating family, and an incubating tag
  must not republish the stable family.
- Use `--no-verify` for unpublished dependency layers. Cargo's workspace
  dry-run verifies the full candidate graph without weakening archive builds.
- Publish during S32.1. This sprint prepares and verifies the workflow only.
- Leave the incubating workflow trigger unreachable through `/release`. That
  would force a future publisher to bypass the repository's sole reviewed tag
  authority.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `cargo publish --workspace --dry-run` | Every publishable package stages and verifies together without an upload |
| unit | `python3 -m unittest scripts/test_sprint_workflow.py` | Parsed workflow steps bind each tag predicate to exactly its expected dependency-ordered package sequence |
| regression | negative workflow mutations | Swapped predicates, extra packages, `continue-on-error`, and successful fallback commands are rejected |
| regression | release authority assertions | `/release` validates and creates only the requested stable or incubating tag after the shared reviewed-SHA and final-approval gates |
| integration | generated archive size check | Every full-workspace dry-run archive remains below 10 MiB |

The backlog test gate is a dry-run publish of the full workspace succeeding in
dependency order.

## HLD impact

- `docs/hld/11-migration-plan.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect every package version and
  workflow tag condition plus both `/release` branches. Run the clean full
  gate, validate each exact family version and package set, and retain the
  separate final approval before `/release` creates either release tag.
- Crate dependency graph. Confirm the explicit publish order follows Cargo
  metadata and preserves the family dependency rule from
  `docs/hld/03-architecture.md`.
- Public API of published crates. No API changes are planned. Run the full
  workspace publication dry-run and the archive-size assertion.
- Bundled fonts. Inspect the `oxml-layout` archive inventory and licences in
  the consolidated publication dry-run.

## Hash harness

Expected to remain unchanged. Publication metadata and workflow routing do not
change generated OOXML or rendering output.

## Implementation checklist

- [x] Make every implemented shared and PowerPoint crate an explicit
      publication candidate.
- [x] Route stable and incubating tags to separate ordered allowlists.
- [x] Extend `/release` with namespace-aware version, package, tag, approval,
      and external verification rules without weakening the stable path.
- [x] Run the hash harness and full workspace dry-run before real publish
      steps.
- [x] Preserve verified archives, propagated failures, and registry waits.
- [x] Parse and compare the exact workflow publish steps for both namespaces,
      including negative routing, membership, and failure-propagation cases.
- [x] Regenerate and verify the Codex skill adapters after the release command
      changes.
- [x] Run the full workspace publication dry-run without uploading anything.

## Open questions

None. The sprint plan and build specification define the candidate boundary,
tag namespaces, and no-publication constraint.

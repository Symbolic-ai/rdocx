# F-049, Extend publish.yml to the extracted workspace

**Status**: approved
**Sprint**: S32.1
**Size**: M
**Depends on**: F-048

## Problem

`.github/workflows/publish.yml:5` accepts only stable `v*` tags, and lines 23
through 60 publish only the seven released rdocx crates. The implemented
shared and PowerPoint packages remain guarded by `publish = false`, so the
workflow cannot dry-run or publish the completed incubating graph after a
separately approved release preparation.

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

Before either real allowlist runs, reproduce the hash harness and run
`cargo publish --workspace --dry-run`. Cargo's workspace dry-run stages the
selected local packages together, so dependent archive verification uses the
workspace candidates without uploading them. Keep every real `cargo publish`
step verified, propagate all failures, and retain the registry wait between
dependency layers. Extend the existing workflow tests to pin both allowlists,
their order, tag routing, and the absence of `--no-verify` or swallowed errors.

## Rejected alternatives

- Replace the allowlists with an unqualified real `cargo publish --workspace`.
  A stable tag must not publish the incubating family, and an incubating tag
  must not republish the stable family.
- Use `--no-verify` for unpublished dependency layers. Cargo's workspace
  dry-run verifies the full candidate graph without weakening archive builds.
- Publish during S32.1. This sprint prepares and verifies the workflow only.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `cargo publish --workspace --dry-run` | Every publishable package stages and verifies together without an upload |
| unit | `python3 -m unittest scripts/test_sprint_workflow.py` | Both tag namespaces select exactly one explicit allowlist in dependency order |
| regression | workflow text assertions | Hash verification precedes publishing, `--no-verify` is absent, and failures are not relabelled as success |
| integration | generated archive size check | Every full-workspace dry-run archive remains below 10 MiB |

The backlog test gate is a dry-run publish of the full workspace succeeding in
dependency order.

## HLD impact

- `docs/hld/11-migration-plan.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect every package version and
  workflow tag condition, run the clean full gate, and retain the separate
  final approval before `/release` creates either release tag.
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

- [ ] Make every implemented shared and PowerPoint crate an explicit
      publication candidate.
- [ ] Route stable and incubating tags to separate ordered allowlists.
- [ ] Run the hash harness and full workspace dry-run before real publish
      steps.
- [ ] Preserve verified archives, propagated failures, and registry waits.
- [ ] Extend the existing workflow tests for both namespaces and orders.
- [ ] Run the full workspace publication dry-run without uploading anything.

## Open questions

None. The sprint plan and build specification define the candidate boundary,
tag namespaces, and no-publication constraint.

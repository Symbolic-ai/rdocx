# F-X028, Repair the agent-facing documentation drift

**Status**: completed
**Sprint**: S44
**Size**: M
**Depends on**: none

## Problem

`CLAUDE.md` is the first repository description most agents read, but it
contains claims contradicted by the current workspace. It names stable crates
at 0.2.0 instead of 0.7.0 at `CLAUDE.md:15`, places bundled fonts under
`rdocx-layout` and behind a nonexistent `bundled-fonts` feature at
`CLAUDE.md:41`, and lists three M1 defects as still shipping at
`CLAUDE.md:159`. The implementation instead has Caladea legal files under
`crates/oxml-layout/fonts`, suffix-aware `MediaNamer::scan`, and separate normal
and deterministic layout caches.

The same drift appears in the wheel guidance at
`docs/hld/10-bindings-spec.md:249`, the release-train description at
`docs/hld/15-build-and-toolchain.md:229`, and the no-default-features command at
`.claude/commands/verify.md:54`. A manual correction alone would allow the next
stale path, version, feature, or package name to survive.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "Packaging".
- `docs/hld/15-build-and-toolchain.md`, "Feature flags", "Packaging", "Release
  process", and "CI job matrix".
- `docs/hld/14-development-backlog.md`, "F-X028, Repair the agent-facing
  documentation drift".

## Approach

Correct `CLAUDE.md` to the current stable version, font ownership, unconditional
bundled-font inventory, and optional `system-fonts` feature. Remove the obsolete
"Known defects being carried" section because every listed defect is already
fixed. Preserve the separate "Things that are deliberately wrong" section.

Correct `docs/hld/10-bindings-spec.md` so wheel packaging relies on bundled
fonts that are always compiled in and treats `system-fonts` as optional host
discovery. Correct the current stable and incubating train versions in
`docs/hld/15-build-and-toolchain.md`. Correct `.claude/commands/verify.md` to
run `cargo test -p oxml-layout --no-default-features` and describe it as the
system-font-discovery-off path. Regenerate the generated verify skill adapter.

Extend the existing `scripts/test_sprint_workflow.py` rather than creating a
new file. Add one structured contract helper that reads `CLAUDE.md`, the verify
command, and workspace manifests. Assert that every governed repository path
claim in both documents resolves, stated family versions match manifest truth,
named features exist on their claimed packages, the font path and legal
inventory resolve, and verify's package names are workspace members. Path
validation handles intentional globs, placeholders, line suffixes, and
generated-output roots explicitly. Mutation cases will introduce stale crate
and non-crate paths, a stale verify path, version, feature, and package name and
prove that each is rejected.

## Rejected alternatives

- Correct prose without a regression. The story exists because repeated manual
  sweeps failed to prevent the same class of drift.
- Add another script or test module. The existing standard-library regression
  module already owns agent and workflow contracts, and a new file needs an
  explicit ask.
- Keep a historical list of fixed defects in `CLAUDE.md`. Delivery history
  belongs in `AS_BUILT.md` and Git, not in current repository guidance.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_agent_facing_repository_claims_resolve_against_the_workspace` | Every governed path, version, feature, font inventory, and verify package claim resolves against current manifests and files |
| regression | `test_agent_facing_claim_contract_rejects_stale_mutations` | Mutated crate, non-crate, verify, stable version, feature, and package claims each fail the helper |

The backlog test gate is **regression**: a test asserts that every path,
version, and feature name `CLAUDE.md` and `.claude/commands/verify.md` cite
resolves against the workspace, so the next stale claim fails the gate rather
than surviving 40 sprints.

## HLD impact

- `docs/hld/10-bindings-spec.md`, "Packaging".
- `docs/hld/15-build-and-toolchain.md`, "Release process".

## Risk routing

- **Bundled fonts**. Read `docs/hld/15-build-and-toolchain.md`. Confirm every
  bundled family has its real licence file and that assets remain inside
  `crates/oxml-layout` for publication. The expected asset diff is empty.
- **Release scripting, version strings**. Read `.claude/commands/release.md` and
  `docs/hld/15-build-and-toolchain.md`. Inspect every manifest, lockfile, and
  README version diff, and require a clean full gate. The expected carrier diff
  is empty because only stale descriptive prose changes. Separate final
  approval remains mandatory before any tag, but this story performs no
  release.

## Hash harness

Expected unchanged at 49 of 49. Documentation, a standard-library regression,
and a generated skill adapter do not alter rendered output.

## Implementation checklist

- [x] Add failing agent-facing contract tests and stale-mutation cases.
- [x] Correct `CLAUDE.md` versions, paths, feature language, and fixed-defect
  guidance.
- [x] Correct the bindings packaging claim and release-train prose.
- [x] Correct verify's no-default-features package and explanation.
- [x] Regenerate the verify skill adapter.
- [x] Run focused tests, prose checking, and skill synchronization checks.
- [x] Inspect all manifest, lockfile, README, font, and version-carrier diffs
  and require no unintended changes.
- [x] Run microscope and contribute the risk riders to the integrated full
  gate.
- [x] Cover every governed repository path in `CLAUDE.md` and the verify
  command, with explicit handling for intentional dynamic forms.
- [x] Prove stale non-crate and verify path mutations fail, then re-review and
  re-run the affected full gate.

## Open questions

None.

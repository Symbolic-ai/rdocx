# F-X050, Tag v0.9.0

**Status**: approved
**Sprint**: S53
**Size**: S
**Depends on**: F-172, F-173, F-174, F-175, F-X048, F-X049

## Problem

The stable family remains prepared and published at 0.8.0 through the workspace
version at `Cargo.toml:34`. S52 and S53 add intentional pre-1.0 low-level
package, layout, provenance, table, and PDF changes plus additive native
encryption, signing, conformance, redaction, and editing APIs. Those changes
require a coherent stable 0.9.0 boundary after the incubating 0.5.0 dependency
graph is published.

The stable release must also close the community evidence loop. Issues 15, 23,
39, and 42 plus PRs 40, 41, and 43 are named by the release story. The current
0.8.0 changelog and metadata regressions do not describe their 0.9.0 outcomes,
direct versus hardened-equivalent status, authenticated contributors, or the
comments that must be posted after publication.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and current public boundaries.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability",
  "Packaging", and "WASM".
- `docs/hld/12-testing-strategy.md`, release regressions, README checks, and
  the full gate.
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `docs/hld/14-development-backlog.md`, "F-X050, Tag v0.9.0".
- `.claude/commands/release-notes.md`, contribution inventory, reviewed
  section, and notification preparation.
- `.claude/commands/release.md`, stable family, preconditions, final approval,
  publication, and notification evidence.

## Approach

After F-X049 is published and verified, move the shared workspace version and
every stable carrier to 0.9.0. Update the nine stable workspace pins, inherited
lockfile packages, both Python project versions, rdocx WASM contract literals,
stable CI literal, seven stable README requirements, workflow preflight names,
README checks, and self-test expectations. Keep every incubating manifest and
pin at the published 0.5.0 boundary.

Rename and strengthen the stable 0.9.0 metadata regression. It proves the exact
versions, pins, publication flags, lockfile set, Python and WASM metadata,
README requirements, and seven-package publication allowlist. Python, WASM,
npm, PyPI, and incubating publication remain unauthorized.

Use `/release-notes v0.9.0` with an evidence range beginning at `v0.8.0`.
Build the selected-family contribution inventory from repository and
authenticated GitHub evidence. At minimum include Issues 15, 23, 39, and 42
and PRs 40, 41, and 43. Credit `@mantissaman` and `@emptinessform` with the
specific included outcomes. Mark reference implementations that landed through
hardened equivalents as such. Close PR 43 only after F-X048 lands, with the
implementation SHA recorded, and classify it as addressed rather than merged.

The exact changelog section links every included record. Prepare one
record-specific post-release comment per inventory entry. Following a clean
full verification and sprint review at the final prepared SHA, `/release
v0.9.0` reports the exact inventory, comments, package set, rendered notes,
remote, workflow, and SHA, then asks for a new approval immediately before the
first external mutation.

## Rejected alternatives

- A stable patch release would understate intentional pre-1.0 source changes.
- Publishing before incubating 0.5.0 resolves would make the stable graph depend
  on unavailable registry versions.
- Publishing Python, WASM, npm, PyPI, or incubating crates is outside the exact
  stable family authority.
- Closing or merging PR 43 before F-X048 passes would misstate how the behavior
  landed.
- Generic contributor thanks would fail the reviewed record-to-outcome
  inventory contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_9_0` | Workspace version, nine pins, inherited lock entries, Python and WASM metadata, publication flags, README requirements, and CI literals agree at 0.9.0. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Stable allowlist, tag authority, release notes, inventory, notifications, and publication order remain mutation-tested. |
| release notes | `release-notes v0.9.0 --check` and `--render` | One exact stable section renders deterministically with every included record and authenticated contributor. |
| metadata | `cargo metadata --no-deps` | Exactly seven stable crates are publishable at 0.9.0 and all incubating pins resolve at 0.5.0. |
| packaging | patched 22-package workspace dry run | Every local package verifies, archives remain under 10 MiB, and all required font, ICC, and template assets are present. |
| integration | both WASM checks and Python metadata assertions | Unpublished binding packages compile or retain metadata without gaining publication authority. |
| release | `/release v0.9.0` post-approval verification | All seven registry entries and owners resolve, the release body is byte-identical, and every reviewed notification comment URL is recorded. |

The test gate is **release**. The stable metadata regression, full
verification, 22-package dry run, archive inventory, supply-chain gate,
binding and WASM isolation, and declared hashes pass. After separate final
approval, all seven crates resolve from crates.io at 0.9.0 and the GitHub
release body matches the reviewed notes with complete contribution evidence.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings: re-read `.claude/commands/release.md`
  and `docs/hld/15-build-and-toolchain.md`. Inspect every manifest, pin,
  lockfile, README, source assertion, CI, workflow, Python metadata, and
  changelog diff. Require a clean full gate and a separate immediate approval.
- Public API of published crates: state the stable pre-1.0 minor migration
  boundary, run the patched package dry run, and enforce archive limits.
- Crate dependency graph: re-read `docs/hld/03-architecture.md`. Verify the
  exact stable allowlist and published incubating 0.5.0 pins with
  `cargo metadata --no-deps`.
- WASM and PyO3 bindings: re-read `docs/hld/10-bindings-spec.md`. Check both
  Python versions, keep every binding unpublished, exclude both Python crates
  from workspace tests, and run both wasm32 checks.
- External record evidence: use authenticated GitHub records and direct links.
  Record direct versus hardened-equivalent status and never infer identity from
  commit trailers.

## Hash harness

Expected to be unchanged across all 49 entries. This story changes version and
release metadata only. Any output delta blocks preparation.

## Implementation checklist

- [ ] Confirm all dependencies, including published incubating 0.5.0, are
      complete before changing stable carriers.
- [ ] Move the workspace version, stable pins, lock entries, Python metadata,
      WASM and CI literals, README requirements, and self-tests to 0.9.0.
- [ ] Rename and strengthen the stable 0.9.0 metadata regression.
- [ ] Build and reconcile the selected-family contribution inventory.
- [ ] Close PR 43 as addressed only after F-X048 passes, with implementation
      evidence and contributor credit.
- [ ] Prepare the reviewed `v0.9.0` changelog section and one unposted comment
      per included record.
- [ ] Update exactly the listed HLD files for the prepared release state.
- [ ] Run full verification, package, asset, binding, WASM, supply-chain,
      notes, and hash gates.
- [ ] Stop at `/release v0.9.0` final approval.
- [ ] After approval, verify registry entries, owners, release SHA and body,
      post every reviewed comment, and record every comment URL.

## Release boundary

The **release** gate requires real publication. This F-ID remains reviewed in
run state and in-progress in delivery trackers until `/release v0.9.0`
succeeds and every notification verifies. The earlier incubating approval does
not count for this tag.

## Open questions

None. The backlog, published dependency order, release commands, and
contribution workflow fix the stable family, version, named external records,
and separate approval boundary.

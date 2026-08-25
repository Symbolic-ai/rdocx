# F-X055, Tag v0.10.0

**Status**: approved
**Sprint**: S56
**Size**: S
**Depends on**: F-180, F-181, F-182, F-X051, F-X052, F-X053, F-X054

## Problem

The stable family remains at 0.9.0 in the shared workspace version and release
contract at `Cargo.toml:34` and `docs/hld/15-build-and-toolchain.md:258`. S56
adds public ODT, EPUB, SVG, and ordered reader APIs. F-X054 also carries the
intentional PR 51 source incompatibility, which removes `Copy` and extends a
previously exhaustive public numbering enum.

Those additions and incompatibilities require the next pre-1.0 minor boundary,
0.10.0, without changing the incubating 0.5.0 family. The release must bind the
seven stable crates, reviewed notes, compatibility guidance, authenticated
contribution inventory, tag, registry packages, and post-release record
comments to one fully verified SHA.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and published family boundaries.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", "Packaging",
  and "WASM".
- `docs/hld/12-testing-strategy.md`, release regressions, README checks, and the
  full gate.
- `docs/hld/14-development-backlog.md`, "F-X055, Tag v0.10.0".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release-notes.md`, contribution inventory, reviewed notes,
  and notification preparation.
- `.claude/commands/release.md`, stable family, preconditions, separate final
  approval, publication, and notification evidence.

## Approach

After F-180, F-181, F-182, and F-X054 are integrated and reviewed, move the
shared workspace version and every stable carrier from 0.9.0 to 0.10.0. Update
the nine stable workspace pins, inherited lockfile packages, both Python project
versions, rdocx WASM contract literals, stable CI literal, seven stable README
requirements, workflow preflight names, README checks, and self-test
expectations. Keep every incubating manifest and pin at its published 0.5.0
boundary.

Rename and strengthen the stable metadata regression for 0.10.0. It proves the
exact versions, pins, publication flags, lockfile set, Python and WASM metadata,
README requirements, and seven-package publication allowlist. Python, WASM,
npm, PyPI, and incubating publication remain unauthorized.

Use `/release-notes v0.10.0` with an evidence range beginning at `v0.9.0`.
Build the selected-family inventory from repository evidence and authenticated
GitHub records. Include Issue 44, PR 45, Issue 46, and PRs 47 through 52. Credit
`@emptinessform` and `@pedroassumpcao` with their specific outcomes. The PR 51
compatibility section explicitly names the removed `Copy` guarantee and
retained producer-defined variants. Direct and hardened-equivalent labels must
match F-X054 evidence.

Prepare one record-specific post-release comment per inventory entry. After
clean full verification and sprint review at the final prepared SHA, `/release
v0.10.0` reports the exact inventory, comments, package set, rendered notes,
remote, workflow, and SHA. It then asks for a new approval immediately before
the first external mutation. After publication verifies, post the reviewed
comments and close PRs 47 through 52 with their true integration status.

## Rejected alternatives

- A patch release would understate public additions and the deliberate pre-1.0
  source incompatibility.
- Publishing the incubating family would exceed the stable release authority.
- Hiding the PR 51 break behind beta wording would fail the compatibility
  record.
- Merging the contributed pull requests only to claim attribution would
  misstate how hardened equivalents landed.
- Generic contributor thanks would fail the reviewed record-to-outcome
  inventory contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_10_0` | Workspace version, nine pins, inherited lock entries, Python and WASM metadata, publication flags, README requirements, and CI literals agree at 0.10.0. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Stable allowlist, tag authority, release notes, contribution inventory, notifications, and publication order remain mutation-tested. |
| release notes | `release-notes v0.10.0 --check` and `--render` | One exact stable section renders deterministically with every named external record, compatibility action, and contributor. |
| metadata | `cargo metadata --no-deps` | Exactly seven stable crates are publishable at 0.10.0 and all incubating pins resolve at 0.5.0. |
| packaging | patched 22-package workspace dry run | Every local package verifies, archives remain under 10 MiB, and all required font, ICC, and template assets are present. |
| integration | both WASM checks and Python metadata assertions | Unpublished binding packages compile or retain metadata without gaining publication authority. |
| release, gate | `/release v0.10.0` post-approval verification | All seven registry entries and owners resolve, the release body is byte-identical, and every reviewed notification comment URL is recorded. |

The **test gate** is release. The stable metadata and public API regressions,
full verification, exact 22-package dry run, archive inventory, supply-chain
gate, binding and WASM isolation, release-note validation, selected-family
contribution inventory, and declared hash result pass at one reviewed SHA.
After separate final approval, all seven crates resolve from crates.io at
0.10.0 and the GitHub release body and comments match the reviewed evidence.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Re-read
  `.claude/commands/release.md` and `docs/hld/15-build-and-toolchain.md`. Inspect
  every manifest, pin, lockfile, README, source assertion, CI, workflow, Python
  metadata, and changelog diff. Require a clean full gate and a separate
  immediate approval before tagging.
- **Public API of a published crate**. State the stable pre-1.0 minor migration
  boundary, run the patched package dry run, and enforce archive limits.
- **Crate dependency graph**. Re-read `docs/hld/03-architecture.md`. Verify the
  exact stable allowlist and published incubating 0.5.0 pins with
  `cargo metadata --no-deps`.
- **WASM or PyO3 bindings**. Re-read `docs/hld/10-bindings-spec.md`. Check both
  Python versions, keep every binding unpublished, exclude both Python crates
  from workspace tests, and run both wasm32 checks.

## Hash harness

Expected unchanged across all 49 entries. This story changes release metadata
only. Any output delta blocks preparation.

## Implementation checklist

- [x] Confirm every dependency and the reviewed S56 SHA before changing stable carriers.
- [x] Move the workspace version, stable pins, lock entries, Python metadata, WASM and CI literals, README requirements, and self-tests to 0.10.0.
- [x] Rename and strengthen the stable 0.10.0 metadata regression.
- [x] Build and reconcile the selected-family contribution inventory.
- [x] Prepare the reviewed `v0.10.0` changelog section and one unposted comment per included record.
- [x] Update exactly the listed HLD files for the prepared release state.
- [x] Run full verification, package, asset, binding, WASM, supply-chain, notes, and hash gates.
- [ ] Stop at `/release v0.10.0` final approval.
- [ ] After approval, verify registry entries, owners, tag and release body, post every reviewed comment, close PRs 47 through 52 accurately, and record every comment URL.

## Open questions

None. The backlog and release commands fix the version, stable family,
incubating exclusion, named external records, contributor identities, and
separate approval boundary.

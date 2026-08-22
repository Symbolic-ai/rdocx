# F-X036, Tag v0.8.0

**Status**: completed
**Sprint**: S51
**Size**: S
**Depends on**: F-166, F-167, F-168, F-X032, F-X033, F-X035, F-X038

## Problem

The stable family remains published and prepared at 0.7.0 even though the
completed M15 and M16 work contains intentional pre-1.0 low-level revision,
field, preservation, VML, paginator, and layout provenance source changes.
Those changes require a stable minor release rather than a 0.7 patch. The
additive native document automation, complete-layout, and ordered-body APIs
also need one coherent reviewed release boundary.

The complete incubating 0.4.0 dependency family is now published and verified,
so the stable graph can move without relying on an unpublished registry
dependency. The release must remain limited to the exact seven stable crates.
Python, WASM, npm, PyPI, and incubating publication remain unauthorized.

## Spec reference

- `docs/hld/03-architecture.md`, "Revision and field ownership" and the stable
  low-level 0.8 source boundary.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and the
  planned stable 0.8.0 compatibility boundary.
- `docs/hld/12-testing-strategy.md`, "README doctests" and the stable release
  endpoint inventory.
- `docs/hld/15-build-and-toolchain.md`, "Release process" and the stable
  package allowlist.
- `docs/hld/14-development-backlog.md`, "F-X036, Tag v0.8.0".
- `.claude/commands/release.md`, "Stable family", "Preconditions", and "Final
  approval".
- `.claude/commands/release-notes.md`, "Evidence" and "Write the reviewed
  section".

## Approach

Move the shared workspace version and every stable carrier to 0.8.0. Update
the nine stable workspace dependency pins, the eleven inherited lockfile
packages, both unpublished Python project versions, the rdocx WASM contract
literals, the stable CI literal, all seven stable README requirements, and
their exact workflow and README test expectations. Leave the incubating family
at its published 0.4.0 boundary.

Rename the stable metadata gate to
`test_stable_release_family_is_prepared_at_0_8_0`. It must prove the exact
versions, pins, publication flags, lockfile set, README requirements, Python
metadata, WASM metadata, and seven-package publication allowlist. Keep
`rdocx-wasm`, both Python bindings, npm, and PyPI outside the publication
surface.

Use `/release-notes v0.8.0` to add the exact changelog section with the ordered
Highlights, Added, Fixed, Compatibility, and Contributors headings. The
evidence range starts at `v0.7.0`. The notes cover the completed native Word
collaboration, chart, field, template, mail merge, comparison, watermark,
complete-layout, provenance, relayout-cache, and ordered-body work. They state
the low-level migration boundary and unchanged binding publication scope, and
credit the verified external contributors and issue reporters.

This feature prepares and reviews the release at one clean exact SHA. It does
not tag, push, publish, or create a GitHub release during implementation. After
a fresh clean full verification and sprint review at that SHA, `/release
v0.8.0` reports the exact package set and rendered notes and asks for a new
explicit approval immediately before the first external mutation.

## Rejected alternatives

- Publish only `rdocx`. The stable family is a lockstep seven-crate graph, and
  the release contract requires every selected package.
- Ship a 0.7 patch. The low-level Rust model and exhaustive layout literal
  changes are intentional pre-1.0 source breaks and require 0.8.0.
- Publish Python, WASM, npm, or PyPI artifacts with the Rust release. Those
  surfaces have separate authority and are explicitly outside this story.
- Reuse the incubating release approval. Every release tag requires a fresh
  approval at its own fully verified and reviewed SHA.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `test_stable_release_family_is_prepared_at_0_8_0` | The workspace version, nine stable pins, eleven lock entries, Python and WASM metadata, publication flags, README requirements, and CI literals match 0.8.0 |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | The stable family, notes, publication order, and mutation-sensitive workflow contracts remain complete |
| release notes | `release-notes v0.8.0 --check` and `--render` | One exact meaningful reviewed section renders deterministically for GitHub |
| metadata | `cargo metadata --no-deps` | Exactly the intended stable carriers use 0.8.0 while the incubating family remains at 0.4.0 |
| packaging | patched 22-package workspace dry run | Every package verifies against the local reviewed graph, every archive remains below 10 MiB, and required assets are present |
| integration | both WASM target checks and Python metadata assertions | Unpublished binding packages retain the exact family pins without gaining publication authority |
| release | `/release v0.8.0` post-approval verification | All seven registry versions and owners resolve, and the GitHub release tag resolves to the approved SHA with byte-identical reviewed notes |

The **test gate** is release. The stable metadata regression, full
verification, 22-package dry run, archive inventory, supply-chain gate, and
unchanged hash harness pass. After separate final approval, all seven stable
crates resolve from crates.io at 0.8.0, the GitHub release uses the reviewed
notes at the exact sprint SHA, and PR 36 credit remains visible.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Read `.claude/commands/release.md`
  and HLD 15. Inspect every manifest, lockfile, README, source assertion, CI,
  workflow, Python metadata, and changelog diff. Require a clean full gate and
  a new explicit approval before tagging.
- **Public API of published crates**. Read HLD 10 and the structural rules.
  State the pre-1.0 minor migration boundary, run the patched package dry run,
  and enforce the 10 MiB archive ceiling.
- **Crate dependency graph**. Read HLD 03. Verify `cargo metadata --no-deps`
  reports the exact selected versions, internal pins, package set, and
  publication eligibility without adding a forbidden dependency edge.
- **WASM and PyO3 bindings**. Read HLD 10. Check both Python project versions,
  keep binding packages unpublished, run workspace tests with both Python
  crates excluded, and run both wasm32 package checks.

Release-specific riders also require registry and owner checks for every
selected package, absence of the exact local and remote tag before approval,
the exact seven-package publication order, and byte equality between fresh
notes rendering and the GitHub release body.

## Hash harness

Expected unchanged across all 49 entries. This feature changes version and
release metadata only. Any output delta blocks release preparation.

## Implementation checklist

- [x] Move the workspace package version, nine stable workspace pins, eleven
  inherited lockfile packages, and both Python project versions to 0.8.0.
- [x] Update the rdocx WASM assertions, stable CI literal, seven stable README
  requirements, README checks, publish preflight, and workflow self-tests.
- [x] Rename and strengthen the stable 0.8.0 metadata regression.
- [x] Prepare and validate the reviewed `v0.8.0` changelog section with stable
  family scope, migration guidance, and verified contributor credit.
- [x] Update exactly HLD 03, HLD 10, HLD 12, and HLD 15 for the prepared state.
- [x] Run full verification, the 49-entry hash gate, patched 22-package dry
  run, archive inventory and size checks, both WASM checks, Python metadata
  assertions, no-default layout, docs, README tests, and supply-chain checks.
- [x] Run microscope and a clean sprint review at the exact prepared SHA.
- [x] Stop at `/release` final approval with the exact SHA, package set,
  rendered notes, tag, remote, and workflow reported to the user.
- [x] After approval, verify all seven registry entries and owners plus the
  exact GitHub release target and note bytes before completing the story.

## Open questions

None. The backlog, published 0.4.0 dependency graph, existing release command,
and the user's release request establish the family, version, notes scope, and
separate final approval requirement.

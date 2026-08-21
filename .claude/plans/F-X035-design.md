# F-X035, Tag rpptx-v0.4.0

**Status**: approved
**Sprint**: S51
**Size**: S
**Depends on**: F-X034, F-X037, F-X038

## Problem

The implemented incubating package family is still prepared at 0.3.0 even
though the current dependency graph contains `oxml-chart`, which has never
been published. The stable `rdocx` graph now requires that crate, so the next
stable release cannot resolve from crates.io until the complete incubating
family is published together.

F-X037 also added required `source: Option<SourceSpan>` fields to exhaustive
layout types, and F-X038 defines new process-lifetime normal-font cache
semantics. Those pre-1.0 compatibility boundaries require an incubating minor
release with reviewed migration notes, not a patch release.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and the
  planned incubating 0.4.0 source boundary.
- `docs/hld/15-build-and-toolchain.md`, "Release process" and the incubating
  package allowlist.
- `docs/hld/14-development-backlog.md`, "F-X035, Tag rpptx-v0.4.0".
- `.claude/commands/release.md`, "Incubating family", "Preconditions", and
  "Final approval".
- `.claude/commands/release-notes.md`, "Evidence" and "Write the reviewed
  section".

## Approach

Prepare the exact 15-package incubating crates.io family at 0.4.0 and prepare
the unpublished `rpptx-wasm` crate at the same version. Update all matching
workspace dependency pins, explicit package manifests, lockfile entries,
README examples, source assertions, CI literals, workflow preflight names,
and self-test expectations as one reviewed version boundary.

Rename the incubating metadata gate to
`test_incubating_release_family_is_prepared_at_0_4_0`. It must prove the exact
versions, pins, publication flags, lockfile set, README requirements, WASM
metadata, and 15-package publication allowlist. Keep `rpptx-wasm` unpublished.

Use `/release-notes rpptx-v0.4.0` to add the exact changelog section with the
ordered Highlights, Added, Fixed, Compatibility, and Contributors headings.
The evidence range starts at `rpptx-v0.3.0`. The notes cover only the shared
OOXML and PowerPoint family. They include the first `oxml-chart` publication,
layout provenance, cache behavior, relevant fixes, required source-field
migration, and verified contributor credit.

This feature prepares and reviews the release at a clean exact SHA. It does not
tag, push, publish, or create a GitHub release during implementation. After a
clean full verification and sprint review at that SHA, `/release
rpptx-v0.4.0` must report the exact package set and rendered notes and ask the
user for a fresh explicit approval immediately before the first external
mutation.

## Rejected alternatives

- Publish only `oxml-chart`. The incubating family is a lockstep versioned
  graph, and the release contract requires all 15 selected packages.
- Keep 0.3.0. Fourteen packages already occupy that immutable registry
  version, and F-X037 introduces a documented pre-1.0 source break.
- Move the stable family in the same feature. Stable 0.8.0 depends on verified
  registry availability of every incubating 0.4.0 package and has its own
  approval boundary.
- Generate GitHub notes from commits. F-X034 requires the reviewed changelog
  body to be published byte for byte.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `test_incubating_release_family_is_prepared_at_0_4_0` | All 16 prepared manifests, 15 workspace pins, 16 lock entries, publication flags, README requirements, source assertions, and CI literals match 0.4.0 |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | The release family, notes, publication order, and mutation-sensitive workflow contracts remain complete |
| release notes | `release-notes rpptx-v0.4.0 --check` and `--render` | One exact meaningful reviewed section renders deterministically for GitHub |
| metadata | `cargo metadata --no-deps` | Exactly the intended package family has 0.4.0 carriers and the stable family remains unchanged |
| packaging | patched 22-package workspace dry run | Every package verifies against the local reviewed graph, all archives remain below 10 MiB, and required assets are present |
| integration | both WASM target checks | Incubating and stable unpublished WASM packages still compile against their exact family pins |
| release | `/release rpptx-v0.4.0` post-approval verification | All 15 registry versions and owners resolve, and the GitHub release targets the approved SHA with byte-identical reviewed notes |

The **test gate** is release. The incubating metadata regression, full
verification, 22-package dry run, archive inventory, supply-chain gate, and
unchanged hash harness pass. After separate final approval, all 15 crates
resolve from crates.io at 0.4.0 and the GitHub release uses the reviewed notes
at the exact sprint SHA.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting, version strings**. Read `.claude/commands/release.md`
  and HLD 15. Inspect every manifest, lockfile, README, source assertion, CI,
  workflow, and changelog diff. Require a clean full gate and a separate final
  approval before tagging or publication.
- **Public API of published crates**. Read HLD 10 and the structural rules.
  State the pre-1.0 minor migration boundary, run the full package dry run,
  and enforce the 10 MiB archive ceiling.
- **Crate dependency graph**. Read HLD 03 and verify `cargo metadata --no-deps`
  reports the exact selected versions, pins, package set, and publication
  eligibility without adding a forbidden dependency edge.
- **WASM bindings**. Read HLD 10. Keep `rpptx-wasm` unpublished, update its
  metadata and CI literal, and run both wasm32 package checks.

Release-specific riders also require registry and owner checks for every
selected package, absence of the exact local and remote tag before approval,
the exact 15-package publication order, and byte equality between fresh notes
rendering and the GitHub release body.

## Hash harness

Expected unchanged across all 49 entries. This feature changes version and
release metadata only. Any output delta blocks release preparation.

## Implementation checklist

- [x] Move the 15 workspace dependency pins and 16 prepared manifests to
  0.4.0 while leaving stable-family versions unchanged.
- [x] Update the exact 16 incubating lockfile package entries.
- [x] Update the 12 incubating README files and their 13 version strings.
- [x] Update source assertions, both WASM-related literals, CI, publish
  preflights, README checks, and workflow self-tests.
- [x] Rename and strengthen the incubating 0.4.0 metadata regression.
- [x] Prepare and validate the reviewed `rpptx-v0.4.0` changelog section with
  verified family scope, compatibility guidance, and contributor credit.
- [x] Update exactly HLD 10 and HLD 15 for the prepared state.
- [x] Run full verification, the 49-entry hash gate, the patched 22-package
  dry run, archive inventory and size checks, both WASM checks, no-default
  layout, docs, README tests, and supply-chain checks.
- [ ] Run microscope and a clean sprint review at the exact prepared SHA.
- [ ] Stop at `/release` final approval with the exact SHA, package set,
  rendered notes, tag, remote, and workflow reported to the user.
- [ ] After approval, verify all 15 registry entries and owners plus the exact
  GitHub release target and note bytes before completing the story.

## Open questions

None. The backlog, current package graph, existing release command, and the
user's request establish the family, 0.4.0 boundary, meaningful reviewed notes,
and separate final approval requirement.

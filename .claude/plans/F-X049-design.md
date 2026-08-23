# F-X049, Tag rpptx-v0.5.0

**Status**: completed
**Sprint**: S53
**Size**: S
**Depends on**: F-172, F-173, F-174, F-175

## Problem

The incubating family remains prepared and published at 0.4.0 in the sixteen
explicit package manifests beginning at `crates/oxml-core/Cargo.toml:5` and in
the workspace pins beginning at `Cargo.toml:55`. S52 and S53 add package
encryption and signing, shared layout changes, semantic PDF, PDF/A output, and
redaction-related shared behavior. The public pre-1.0 boundary therefore needs
one lockstep 0.5.0 release before the stable 0.9.0 graph can resolve.

The current changelog ends the incubating family at `rpptx-v0.4.0`, and the
release preflights and CI literals still assert 0.4.0. Publication without a
complete carrier update and reviewed contribution inventory would either fail
on the tag or publish notes that omit the community work included in the
release.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning".
- `docs/hld/10-bindings-spec.md`, "Packaging" and "WASM".
- `docs/hld/12-testing-strategy.md`, release regressions, README checks, and
  the full gate.
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `docs/hld/14-development-backlog.md`, "F-X049, Tag rpptx-v0.5.0".
- `.claude/commands/release-notes.md`, contribution inventory, reviewed
  section, and notification preparation.
- `.claude/commands/release.md`, incubating family, preconditions, final
  approval, publication, and notification evidence.

## Approach

Prepare the exact fifteen-package incubating crates.io family at 0.5.0 and the
unpublished `rpptx-wasm` crate at the same version. Move all sixteen explicit
package manifests, fifteen workspace dependency pins, lockfile entries, README
examples, Rust assertions, CI literals, workflow preflights, and self-test
expectations together. Keep the stable workspace family at 0.8.0 during this
story.

Rename and strengthen the incubating metadata regression for 0.5.0. It proves
the exact package versions, internal pins, publication flags, lockfile set,
README requirements, WASM metadata, and fifteen-package publication allowlist.
The 22-package patched dry run continues to verify the whole local graph.

Use `/release-notes rpptx-v0.5.0` with an evidence range beginning at
`rpptx-v0.4.0`. Build the selected-family contribution inventory from reviewed
commits, delivery records, issues, and pull requests. Include PRs 40 and 41 and
credit authenticated contributor `@emptinessform` for the shared layout and
cache prototypes that landed through hardened equivalents. Include every other
external record whose shared or PowerPoint outcome is in the range, and exclude
stable-only records with a recorded reason.

The exact changelog section contains direct record links and specific
contributor credit. Prepare one record-specific post-release comment for every
included issue and pull request. Implementation stops after version and notes
preparation. Following clean full verification and sprint review, `/release
rpptx-v0.5.0` reports the exact SHA, inventory, rendered notes, package set, and
planned comments, then asks for a fresh explicit approval before any push, tag,
publication, or notification.

## Rejected alternatives

- Publishing only changed crates would break the lockstep incubating family
  and the exact allowlist contract.
- A 0.4 patch would understate intentional pre-1.0 public layout and PDF
  surface changes.
- Publishing `rpptx-wasm` to crates.io remains unauthorized.
- Reusing stable-family notes or approval would mix package families and erase
  the separate release boundary.
- Posting contributor comments during preparation would claim a release that
  does not yet exist.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_incubating_release_family_is_prepared_at_0_5_0` | Sixteen prepared manifests, fifteen pins, lock entries, publication flags, README requirements, source assertions, and CI literals agree at 0.5.0. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Package allowlists, tag authority, release notes, inventory, comments, and publication order remain mutation-tested. |
| release notes | `release-notes rpptx-v0.5.0 --check` and `--render` | One exact selected-family section renders deterministically with every included record link and authenticated contributor. |
| metadata | `cargo metadata --no-deps` | Exactly the intended family is prepared at 0.5.0 while stable packages remain at 0.8.0. |
| packaging | patched 22-package workspace dry run | Every local package verifies, archives remain under 10 MiB, and all required font, ICC, and template assets are present. |
| integration | both WASM target checks | `rpptx-wasm` compiles at 0.5.0 and remains unpublished. |
| release | `/release rpptx-v0.5.0` post-approval verification | All fifteen registry entries and owners resolve, the GitHub release body is byte-identical, and every reviewed notification comment URL is recorded. |

The test gate is **release**. The incubating metadata regression, full
verification, 22-package dry run, archive inventory, supply-chain gate, WASM
isolation, and declared hashes pass. After separate final approval, all fifteen
crates resolve from crates.io at 0.5.0 and the GitHub release body matches the
reviewed notes with verified contributor credit.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings: re-read `.claude/commands/release.md`
  and `docs/hld/15-build-and-toolchain.md`. Inspect every manifest, pin,
  lockfile, README, source assertion, CI, workflow, and changelog diff. Require
  a clean full gate and a separate immediate approval before tagging.
- Public API of published crates: state the pre-1.0 minor migration boundary,
  run the patched package dry run, and enforce the 10 MiB archive ceiling.
- Crate dependency graph: re-read `docs/hld/03-architecture.md`. Verify exact
  selected versions, pins, publication eligibility, and no forbidden family
  edge with `cargo metadata --no-deps`.
- WASM bindings: re-read `docs/hld/10-bindings-spec.md`. Keep `rpptx-wasm`
  unpublished and run both wasm32 checks.
- External record evidence: use authenticated GitHub issue and pull-request
  records, direct links, and exact author handles. Do not infer identity from
  commit trailers.

## Hash harness

Expected to be unchanged across all 49 entries. This story changes version and
release metadata only. Any output delta blocks preparation.

## Implementation checklist

- [x] Move sixteen prepared manifests and fifteen workspace pins to 0.5.0.
- [x] Regenerate lockfile entries and update all README, source, CI, workflow,
      WASM, and test carriers.
- [x] Rename and strengthen the incubating 0.5.0 metadata regression.
- [x] Build and reconcile the selected-family contribution inventory.
- [x] Prepare the reviewed `rpptx-v0.5.0` changelog section with direct links
      and specific authenticated credit.
- [x] Prepare one post-release comment per included record without posting it.
- [x] Update exactly the listed HLD files for the prepared release state.
- [x] Run full verification, package, asset, WASM, supply-chain, notes, and
      hash gates.
- [x] Stop at `/release rpptx-v0.5.0` final approval.
- [x] After approval, verify registry entries, owners, release SHA and body,
      post every reviewed comment, and record every comment URL.

## Release boundary

The implementation prepares this story, but the **release** gate requires real
publication. The F-ID remains reviewed in run state and in-progress in delivery
trackers until `/release rpptx-v0.5.0` succeeds and all notifications verify.
Its approval does not authorize F-X050.

## Open questions

None. The backlog, package graph, release commands, and contribution workflow
fix the family, version, evidence, and separate approval boundary.

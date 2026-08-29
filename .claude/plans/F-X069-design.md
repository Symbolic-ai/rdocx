# F-X069, Tag v0.11.1

**Status**: completed
**Sprint**: S58
**Size**: S
**Depends on**: F-198, F-199, F-200, F-202, F-X062, F-X063, F-X064, F-X065, F-X066, F-X067, F-X068

## Problem

The v0.11.0 tag and two published packages are immutable, but they do not form
a complete stable family. The remaining five packages and GitHub release do
not exist, and none of the six reviewed contribution notifications was posted.
After F-X068 publishes shared 0.8.0, the stable source can compile against a
complete registry family but needs a new stable version that cannot collide
with the partial attempt.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and stable package-family boundaries.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", "Packaging", and "WASM".
- `docs/hld/12-testing-strategy.md`, stable release regressions, registry proofs, and package checks.
- `docs/hld/14-development-backlog.md`, "F-X069, Tag v0.11.1".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release-notes.md`, contribution inventory and notification preparation.
- `.claude/commands/release.md`, stable selection, approval, publication, verification, and notifications.

## Approach

Move the stable workspace version, nine stable workspace pins, eleven inherited
lock packages, two Python project versions, `rdocx-wasm` carriers, stable CI
identity, seven stable README requirements, workflow preflights, and release
regressions from 0.11.0 to 0.11.1. Require every shared dependency at the
published 0.8.0 boundary from F-X068.

Prepare `CHANGELOG.md` section `v0.11.1` as the complete S58 stable outcome.
Describe the immutable partial v0.11.0 attempt, the shared 0.8.0 recovery, all
intentional pre-1.0 source impacts, and unchanged binding publication
authority. Carry forward the reviewed selected-family evidence from the full
`v0.10.1` to recovery range instead of treating the empty v0.11.0 notification
set as completed.

The contribution inventory includes Issues 53 and 54 by authenticated
`@emptinessform` and PRs 55 through 58 by authenticated
`@pedroassumpcao`. Classify every final outcome accurately. Prepare exactly one
release-bound thank-you per record. After publication verifies, post all six
comments and leave every issue and pull request open.

Publish exactly `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`,
`rdocx-pdf`, `rdocx`, and `rdocx-cli` in dependency order. Python, WASM, npm,
PyPI, shared, and PowerPoint packages remain outside this story's publication
authority. After a clean exact-HEAD gate and review, stop for separate final
approval before `/release v0.11.1` performs any external mutation.

## Rejected alternatives

- Complete v0.11.0. crates.io versions are immutable and the release family is
  already incomplete.
- Use 0.12.0. The source boundary was already reviewed as 0.11.0. The patch
  recovery version records publication repair without inventing a new feature
  boundary.
- Post notifications against v0.11.0. That release never completed and has no
  GitHub release body.
- Close Issues 53 and 54 or PRs 55 through 58. The approved contribution action
  is comment-only and leave-open.
- Publish bindings, WASM, npm, or PyPI packages. S58 does not authorize those
  distributions.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_11_1` | Stable carriers, pins, lock entries, READMEs, binding metadata, flags, CI literals, and allowlist agree at 0.11.1. |
| regression | published shared-family proof | Packaged `rdocx-layout@0.11.1` resolves registry `oxml-layout@0.8.0` without a local shared patch. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Family selection, order, recovery notes, six-record inventory, leave-open notifications, approval, and mutation authority remain pinned. |
| release notes | `release-notes v0.11.1 --check` and `--render` | One deterministic stable body records the partial attempt, complete recovery, source impacts, and all six contributions. |
| metadata | `cargo metadata --no-deps` | Exactly seven stable crates publish at 0.11.1 with shared pins at 0.8.0. |
| packaging | exact patched 22-package workspace dry run | Every archive verifies below 10 MiB with complete asset inventories. |
| integration | Python metadata assertions and both WASM checks | Bindings track 0.11.1 without gaining publication authority. |
| release | `/release v0.11.1` | Seven registry entries, owners, tag SHA, release body, and six leave-open notification URLs verify. |

The **test gate is release**. Preparation and every local gate pass at one
reviewed SHA. Completion additionally requires the separately approved real
publication, independent registry verification, byte-identical release body,
and all six reviewed leave-open comments.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Inspect every stable carrier,
  workflow literal, regression expectation, changelog section, and recovery
  notification. Require `/verify --full` and separate immediate approval.
- **Public API of published crates**. Record the final pre-1.0 source impacts,
  run the patched dry run, inspect archives, and enforce their limits.
- **Crate dependency graph**. Prove the exact stable allowlist and registry-only
  resolution against shared 0.8.0.
- **Bundled fonts and assets**. Verify F-X058 fonts, legal and provenance files
  in `oxml-layout`, no duplication in `rdocx-layout`, and the PowerPoint
  template inventory.
- **WASM and PyO3 bindings**. Update metadata only, keep all bindings
  unpublished, exclude both Python crates from workspace tests, and run both
  wasm32 checks.

## Hash harness

Expected unchanged across all 49 entries. This story changes release metadata
only and preserves the accepted S58 output baseline.

## Implementation checklist

- [x] Verify F-X068 publication and the immutable v0.11.0 partial evidence.
- [x] Move all stable carriers to 0.11.1 and all shared pins to 0.8.0.
- [x] Update stable carrier, recovery, registry, and notification regressions.
- [x] Prepare and validate the complete recovery notes and six-record inventory.
- [x] Update exactly the five listed HLD files.
- [x] Run `/verify --full`, packaging, assets, bindings, WASM, registry, supply-chain, notes, and hash gates.
- [x] Stop at `/release v0.11.1` for separate final approval.
- [x] Verify all publications, owners, tag, body, and six leave-open comments.

## Open questions

None. The user approved stable 0.11.1 as the complete recovery and approved six
leave-open comments. The real release retains its separate final go or no-go.

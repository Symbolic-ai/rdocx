# F-X076, Tag v0.12.0

**Status**: approved
**Sprint**: S64
**Size**: S
**Depends on**: F-X074, F-X075

## Problem

The stable Word family remains at 0.11.1 while reviewed source now contains
the hardened reader outcomes from PRs 61 through 64, the note-reference cache
fix from Issue 65, the aggregate restart-pagination work from Issue 66, and the
page-spanning paragraph correction from Issue 67. F-X074 has published the
shared OOXML and PowerPoint family at 0.9.0, so the stable family can now move
to one coherent 0.12.0 registry boundary without mixing release authority.

The release must publish exactly seven stable Rust packages. It must keep the
shared and PowerPoint crates at 0.9.0 and keep Python, WASM, npm, and PyPI
packages unpublished. The release notes and post-publication notifications
must credit every authenticated reporter and contributor accurately while
leaving record state unchanged.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and stable package-family boundaries.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability", "Packaging", and "WASM".
- `docs/hld/12-testing-strategy.md`, stable release regressions, registry proofs, and package checks.
- `docs/hld/14-development-backlog.md`, "F-X076, Tag v0.12.0".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release-notes.md`, contribution inventory and notification preparation.
- `.claude/commands/release.md`, stable selection, approval, publication, verification, and notifications.

## Approach

Move the stable workspace version, stable workspace pins, lock records, Python
project metadata, `rdocx-wasm` carrier, stable CI identity, stable README
requirements, workflow preflights, and release regressions from 0.11.1 to
0.12.0. Require every shared dependency at the published 0.9.0 boundary from
F-X074. Keep all binding publication flags unchanged.

Prepare `CHANGELOG.md` section `v0.12.0` as the complete stable outcome since
v0.11.1. Cover the reader facts from PRs 61 through 64 and the bounded cache
and pagination corrections from Issues 65 through 67. Record any pre-1.0
source impact explicitly and state that the selected native changes require no
migration unless a caller relied on the corrected behavior.

The contribution inventory contains PR 61, PR 62, PR 63, and PR 64 by
authenticated `@pedroassumpcao`, plus Issue 65, Issue 66, and Issue 67 by
authenticated `@emptinessform`. Classify all seven maintained outcomes as
hardened equivalents. Prepare one release-bound thank-you comment per record.
After publication verifies, post all seven comments and leave every record's
open or closed state unchanged.

Publish exactly `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`,
`rdocx-pdf`, `rdocx`, and `rdocx-cli` in dependency order. Shared,
PowerPoint, Python, WASM, npm, and PyPI packages remain outside this story's
publication authority. After a clean exact-HEAD gate and review, stop for
separate final approval before `/release v0.12.0` performs any external
mutation.

## Rejected alternatives

- Publish a stable 0.11.2 patch. The reader and cache work forms a reviewed
  post-0.11.1 feature boundary, and the user approved 0.12.0.
- Publish the shared family again. F-X074 already published the required 0.9.0
  dependency boundary and release tags are immutable.
- Publish bindings, WASM, npm, Python, or PyPI packages. This story grants no
  such authority.
- Close Issue 67 after release. The approved release action is comment-only
  unless the user separately authorizes an issue-state change.
- Attribute the contributions as direct merges. The maintained branch carries
  reviewed hardened equivalents rather than the closed unmerged pull-request
  commits themselves.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_12_0` | Stable carriers, pins, lock entries, READMEs, binding metadata, flags, CI literals, and allowlist agree at 0.12.0. |
| regression | published shared-family proof | Packaged `rdocx-layout@0.12.0` resolves registry `oxml-layout@0.9.0` without a local shared patch. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Family selection, order, notes, seven-record inventory, leave-state notifications, approval, and mutation authority remain pinned. |
| release notes | `release-notes v0.12.0 --check` and `--render` | One deterministic stable body describes the selected family and directly links and credits all seven contributions. |
| metadata | `cargo metadata --no-deps` | Exactly seven stable crates publish at 0.12.0 with shared pins at 0.9.0. |
| packaging | exact patched 22-package workspace dry run | Every archive verifies below 10 MiB with complete asset inventories. |
| integration | Python metadata assertions and all stable WASM checks | Bindings track 0.12.0 without gaining publication authority. |
| release | `/release v0.12.0` | Seven registry entries, owners, tag SHA, release body, exclusions, and seven notification URLs verify. |

The **test gate is release**. Preparation and every local gate pass at one
reviewed SHA. Completion additionally requires the separately approved real
publication, independent registry verification, byte-identical release body,
and all seven reviewed leave-state comments.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Inspect every stable carrier,
  workflow literal, regression expectation, changelog section, and
  notification. Require `/verify --full` and separate immediate approval.
- **Public API of published crates**. Record the pre-1.0 reader and layout
  boundary, run the patched dry run, inspect archives, and enforce their
  limits.
- **Crate dependency graph**. Prove the exact stable allowlist and registry-only
  resolution against the published shared 0.9.0 family.
- **Bundled fonts and assets**. Verify bundled fonts, legal and provenance
  files in `oxml-layout`, no duplication in `rdocx-layout`, and the PowerPoint
  template inventory.
- **WASM and PyO3 bindings**. Update metadata only, keep all bindings
  unpublished, exclude both Python crates from workspace tests, and run the
  stable default and no-default wasm32 checks.
- **Reviewed external contributions**. Pin all seven GitHub records and
  authenticated handles, classify hardened equivalents, prepare exact
  release-bound comments, and leave record state unchanged.

## Hash harness

Expected unchanged across all 49 entries. This story changes release metadata
only. Any output delta blocks preparation.

## Implementation checklist

- [ ] Confirm F-X074 and F-X075 are completed and the shared 0.9.0 family is published.
- [ ] Move every stable version carrier and internal stable dependency pin to 0.12.0.
- [ ] Update stable carrier, workflow, registry, and notification regressions.
- [ ] Prepare reviewed stable release notes with all seven linked contributions and exact credit.
- [ ] Update exactly the five listed HLD files.
- [ ] Run `/verify --full`, packaging, assets, bindings, WASM, dependency, supply-chain, notes, and hash gates.
- [ ] Stop at `/release v0.12.0` for separate final approval.
- [ ] Verify every publication, owner, tag, body, exclusion, and notification before completion.

## Open questions

None. The user approved addressing Issue 67 in this sprint and releasing the
stable Word family at 0.12.0 after `rpptx-v0.9.0`. Every external record keeps
its current open or closed state unless separately authorized.

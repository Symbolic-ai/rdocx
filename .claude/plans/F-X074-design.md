# F-X074, Tag rpptx-v0.9.0

**Status**: approved
**Sprint**: S64
**Size**: S
**Depends on**: F-224, F-225, F-X068

## Problem

The complete PresentationML depth boundary is implemented and reviewed, but
the incubating workspace and registry family remain at 0.8.0. Current source
now includes collaboration, encryption and signatures, timing and transitions,
media, SmartArt, embedded content, ODP interchange, package variants, notes and
handouts, animated export, and bounded HTML and PDF import. These capabilities
need one coherent registry boundary before S64 closes.

The release must move the exact 15-package incubating family in lockstep and
must not publish the stable Word, binding, WASM, Python, or npm packages. The
selected-family contribution inventory since `rpptx-v0.8.0` is empty. PRs 61
through 64 and Issues 65 and 66 changed only the stable Word family and must
not be claimed or notified by this release.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and package-family boundaries.
- `docs/hld/10-bindings-spec.md`, "Packaging" and "WASM".
- `docs/hld/12-testing-strategy.md`, release regressions and registry-family proofs.
- `docs/hld/14-development-backlog.md`, "F-X074, Tag rpptx-v0.9.0".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release.md`, incubating selection, approval, publication, and verification.

## Approach

Move the exact 15 publishable shared and PowerPoint manifests, their workspace
pins, lock records, README requirements, CI literals, source assertions,
release regression, and the unpublished `rpptx-wasm` preparation carrier from
0.8.0 to 0.9.0. Update stable source dependency pins to the published shared
0.9.0 boundary without changing the stable package version or granting stable
publication authority.

Prepare a reviewed `rpptx-v0.9.0` changelog section covering the complete M21
PresentationML depth boundary. The notes name the additive native pre-1.0
facade and model changes, the optional HTML and PDF dependency boundaries, and
the exact selected package family. The selected contribution inventory is
empty, so the release prepares no issue or pull-request notification.

Publish exactly `oxml-core`, `oxml-opc`, `oxml-media`, `oxml-layout`,
`oxml-drawing`, `oxml-pdf`, `oxml-sml`, `oxml-cli-support`, `oxml-chart`,
`rpptx-oxml`, `rpptx-chart`, `rpptx-layout`, `rpptx-render`, `rpptx`, and
`rpptx-cli` in dependency order.

After clean full verification and sprint review at one SHA, stop for a separate
final approval. `/release rpptx-v0.9.0` is the only authority to push, create
the tag, publish, or create the GitHub release. Verify all 15 registry entries
and owners, the absent `rpptx-wasm@0.9.0`, and the byte-identical release body
before completing the story.

## Rejected alternatives

- Publish only `rpptx`. The incubating family is a lockstep 15-package contract.
- Leave shared packages at 0.8.0. Current PresentationML source and public
  contracts span the selected family and need one coherent registry graph.
- Publish the stable family at the same time. Stable packages are outside this
  tag and no reviewed stable release is requested.
- Publish bindings, WASM, npm, Python, or PyPI packages. This story grants no
  such authority.
- Attribute stable-only issues or pull requests. The selected-family
  contribution inventory is empty.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_incubating_release_family_is_prepared_at_0_9_0` | Exact manifests, pins, lock entries, READMEs, source and CI literals, publication flags, and allowlist agree at 0.9.0. |
| regression | stable carrier isolation regression | Stable packages remain at 0.11.1 while every shared dependency pin is 0.9.0 and bindings remain unpublished. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Family selection, order, notes, approval boundary, and mutation authority stay pinned. |
| release notes | `release-notes rpptx-v0.9.0 --check` and `--render` | One deterministic incubating-family body describes M21 with an empty contribution inventory. |
| packaging | exact patched 22-package workspace dry run | Every local archive verifies below 10 MiB with complete font, legal, ICC, and template inventories. |
| release | `/release rpptx-v0.9.0` | Fifteen registry entries, owners, tag SHA, release body, stable exclusion, and absent `rpptx-wasm` verify. |

The **test gate is release**. Preparation and every local gate pass at one
reviewed SHA. Completion additionally requires separately approved real
publication and independent registry, owner, tag, and release-body
verification.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Inspect every incubating carrier,
  workflow literal, regression expectation, changelog section, and lock entry.
  Require `/verify --full` and separate immediate approval before tagging.
- **Public API of a published crate**. Record the additive M21 source boundary,
  run the patched dry run, inspect archives, and enforce the 10 MiB cap.
- **Crate dependency graph**. Prove the exact 15-package publication order and
  exclude the stable, binding, WASM, Python, and npm packages.
- **Bundled fonts and assets**. Verify bundled fonts, legal and provenance
  files in `oxml-layout`, no duplicate fonts in `rdocx-layout`, and
  `rpptx/assets/default.pptx`.
- **WASM and PyO3 bindings**. Keep every binding unpublished, exclude both
  Python crates from workspace tests, and run both wasm32 checks including the
  render-selected `rpptx-wasm` graph.
- **External oracles**. Re-run the pinned Chrome and Poppler differential gates
  at the exact reviewed release SHA.

## Hash harness

Expected unchanged across all 49 entries. This story changes release metadata
only. Any output delta blocks preparation.

## Implementation checklist

- [ ] Confirm F-224, F-225, F-X068, and all M21 implementation stories are completed.
- [ ] Move every incubating version carrier and shared dependency pin to 0.9.0.
- [ ] Update exact carrier, isolation, workflow, and release-note regressions.
- [ ] Prepare reviewed M21 release notes with an empty selected-family contribution inventory.
- [ ] Update exactly the five listed HLD files.
- [ ] Run `/verify --full`, packaging, assets, bindings, WASM, dependency, supply-chain, oracle, notes, and hash gates.
- [ ] Stop at `/release rpptx-v0.9.0` for separate final approval.
- [ ] Verify every publication, owner, tag, body, and exclusion before completion.

## Open questions

None. The user approved `rpptx-v0.9.0` as the S64 release. The real release
retains its separate final go or no-go immediately before external mutation.

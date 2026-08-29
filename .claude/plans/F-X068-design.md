# F-X068, Tag rpptx-v0.8.0

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-200, F-X064, F-X065, F-X066, F-X067

## Problem

The immutable v0.11.0 attempt published `rdocx-opc` and `rdocx-oxml`, then
failed while verifying packaged `rdocx-layout`. Current stable source constructs
`TextSegment.direction`, but registry `oxml-layout@0.7.0` predates that public
field. The prepared stable graph therefore cannot compile entirely from the
registry.

The current shared and PowerPoint carriers remain at 0.7.0 in their manifests,
workspace pins, lock records, README examples, CI literals, and release
regressions. The complete current shared source must be published as its own
0.8.0 family before stable publication is retried.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and package-family boundaries.
- `docs/hld/10-bindings-spec.md`, "Packaging" and "WASM".
- `docs/hld/12-testing-strategy.md`, release regressions and registry-family proofs.
- `docs/hld/14-development-backlog.md`, "F-X068, Tag rpptx-v0.8.0".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release.md`, incubating selection, approval, publication, and verification.

## Approach

Move the exact 15 publishable shared and PowerPoint manifests, their workspace
pins, 16 lock records, README requirements, CI literals, source assertions,
release regression, and the unpublished `rpptx-wasm` preparation carrier from
0.7.0 to 0.8.0. Keep the stable workspace and its binding metadata at 0.11.0
during this separate release. Update every stable shared dependency pin to
0.8.0 so the post-release registry proof exercises the graph that failed under
v0.11.0.

Prepare a reviewed `rpptx-v0.8.0` changelog section that describes the additive
direction carrier and the stable recovery purpose. The selected-family
contribution inventory is empty. Do not attribute Issues 53 and 54 or PRs 55
through 58 to this shared release. Publish exactly `oxml-core`, `oxml-opc`,
`oxml-media`, `oxml-layout`, `oxml-drawing`, `oxml-pdf`, `oxml-sml`,
`oxml-cli-support`, `oxml-chart`, `rpptx-oxml`, `rpptx-chart`, `rpptx-layout`,
`rpptx-render`, `rpptx`, and `rpptx-cli` in dependency order.

After clean full verification and sprint review at one SHA, stop for a separate
final approval. `/release rpptx-v0.8.0` is the only authority to create the tag,
push, publish, or create the GitHub release. Verify all 15 registry entries and
owners, the absent `rpptx-wasm@0.8.0`, the byte-identical release body, and a
registry-only build of the prepared stable layout graph before completing the
story.

## Rejected alternatives

- Retry v0.11.0. Published versions and tags are immutable, and the missing
  shared API remains unavailable at 0.7.0.
- Move or delete the v0.11.0 tag. That would rewrite verified release history.
- Publish only `oxml-layout`. The incubating family is a lockstep 15-package
  contract.
- Add a compatibility shim to 0.7.0. crates.io packages cannot be overwritten.
- Publish bindings, WASM, npm, or Python packages. This story grants no such
  authority.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_incubating_release_family_is_prepared_at_0_8_0` | Exact manifests, pins, lock entries, READMEs, source and CI literals, publication flags, and allowlist agree at 0.8.0. |
| regression | stable carrier isolation regression | Stable carriers remain 0.11.0 while every shared dependency pin is 0.8.0 and bindings remain unpublished. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Family selection, order, notes, approval boundary, and mutation authority stay pinned. |
| release notes | `release-notes rpptx-v0.8.0 --check` and `--render` | One deterministic shared-family body describes the compatibility boundary with an empty contribution inventory. |
| packaging | exact patched 22-package workspace dry run | Every local archive verifies below 10 MiB with complete font, legal, ICC, and template inventories. |
| registry | packaged stable layout graph against registry 0.8.0 | Current `rdocx-layout` source resolves and compiles against published `oxml-layout@0.8.0` without a shared patch. |
| release | `/release rpptx-v0.8.0` | Fifteen registry entries, owners, tag SHA, release body, stable exclusion, and absent `rpptx-wasm` verify. |

The **test gate is release**. Preparation and every local gate pass at one
reviewed SHA. Completion additionally requires the separately approved real
publication, independent registry and owner verification, and the stable graph
proof that closes the v0.11.0 failure.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Inspect every incubating carrier,
  workflow literal, regression expectation, and changelog section. Require
  `/verify --full` and separate immediate approval before tagging.
- **Public API of a published crate**. Record the additive
  `TextSegment.direction` source boundary, run the patched dry run, inspect
  archives, and enforce the 10 MiB cap.
- **Crate dependency graph**. Prove exact 15-package publication order and the
  registry-only stable consumer graph against shared 0.8.0.
- **Bundled fonts and assets**. Verify all four Noto fonts, legal and provenance
  files in `oxml-layout`, no duplicate fonts in `rdocx-layout`, and
  `rpptx/default.pptx`.
- **WASM and PyO3 bindings**. Keep every binding unpublished, exclude both
  Python crates from workspace tests, and run both wasm32 checks.

## Hash harness

Expected unchanged across all 49 entries. This story changes release metadata
only. Any output delta blocks preparation.

## Implementation checklist

- [ ] Confirm the immutable v0.11.0 failure evidence and all completed dependencies.
- [ ] Move all incubating carriers and shared pins to 0.8.0.
- [ ] Update exact carrier, isolation, workflow, and registry regressions.
- [ ] Prepare the reviewed shared 0.8.0 notes with an empty contribution inventory.
- [ ] Update exactly the five listed HLD files.
- [ ] Run `/verify --full`, packaging, assets, bindings, WASM, registry, supply-chain, notes, and hash gates.
- [ ] Stop at `/release rpptx-v0.8.0` for separate final approval.
- [ ] Verify all publications, owners, tag, body, exclusions, and the stable registry graph.

## Open questions

None. The user approved shared 0.8.0 as the first recovery release. The real
release retains its separate final go or no-go.

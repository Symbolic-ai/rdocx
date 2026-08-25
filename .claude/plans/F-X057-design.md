# F-X057, Tag v0.10.1

**Status**: approved
**Sprint**: S56
**Size**: S
**Depends on**: F-180, F-181, F-182, F-X051, F-X052, F-X053, F-X054, F-X056

## Problem

The v0.10.0 tag is immutable at the reviewed S56 SHA, but its workflow
published only `rdocx-opc` and `rdocx-oxml`. `rdocx-layout` failed package
verification against the older shared registry API, so the remaining five
stable packages and the GitHub release do not exist at 0.10.0.

After F-X056 publishes the current shared family, the stable family needs one
coherent patch version whose registry dependency graph resolves without local
patches. The recovery must not pretend that v0.10.0 completed or overwrite the
two immutable registry packages that did publish.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning" and published family boundaries.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability",
  "Packaging", and "WASM".
- `docs/hld/12-testing-strategy.md`, release regressions, README checks, and the
  full gate.
- `docs/hld/14-development-backlog.md`, "F-X057, Tag v0.10.1".
- `docs/hld/15-build-and-toolchain.md`, "Publishing" and "Release process".
- `.claude/commands/release.md`, stable family selection, approval,
  publication, registry verification, and contribution notification.

## Approach

After `rpptx-v0.6.0` and all 15 incubating registry entries verify, move the
workspace version and every stable carrier from 0.10.0 to 0.10.1. Move the nine
stable workspace pins, inherited lock entries, Python project metadata,
`rdocx-wasm` package literals, stable CI literal, seven stable README
requirements, workflow preflight name, README checks, and self-test
expectations. Pin all incubating workspace dependencies to the verified 0.6.0
family without changing their package versions at this SHA.

Prepare reviewed `v0.10.1` notes from the `v0.9.0` evidence boundary. Preserve
the complete stable additions, fixes, compatibility guidance, and contribution
inventory previously reviewed for v0.10.0. Add an accurate release-recovery
note stating that v0.10.0 published only two low-level packages before its
dependency verification failed and that 0.10.1 is the first complete stable
family carrying the S56 outcome.

Rebuild the nine-record inventory for Issue 44, PR 45, Issue 46, and PRs 47
through 52. Credit `@emptinessform` and `@pedroassumpcao`, prepare exact
v0.10.1 comments, and retain every hardened-equivalent classification. After
the complete release verifies, post all nine comments and close PRs 47 through
52 as F-X054 authorized.

Add a registry-graph regression that packages and verifies `rdocx-layout`
without patching `oxml-layout`, proving the precise v0.10.0 failure is closed.
Then run the full gate and exact patched 22-package dry run. `/release v0.10.1`
asks for a new approval at the final reviewed SHA before any tag, registry,
comment, or closure mutation.

## Rejected alternatives

- Reusing v0.10.0 is impossible because two package versions and the tag are
  immutable.
- Publishing only the missing five packages would mix source SHAs within one
  declared stable family and bypass the seven-package release contract.
- Yanking the two v0.10.0 crates does not make their versions reusable and is
  not required for dependency recovery.
- Keeping incubating pins at 0.5.0 would reproduce the failed package graph.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_10_1` | Workspace version, stable pins, inherited lock entries, Python and WASM metadata, publication flags, README requirements, and CI literals agree at 0.10.1 while incubating pins remain 0.6.0. |
| regression | `rdocx_layout_verifies_against_published_shared_family` | A clean packaged `rdocx-layout` resolves registry `oxml-layout@0.6.0` without a local patch and compiles the caller-alias path that failed at v0.10.0. |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | Stable allowlist, dependency order, release-note recovery claims, contribution inventory, notifications, closure authority, and tag authority remain mutation-tested. |
| release notes | `release-notes v0.10.1 --check` and `--render` | One deterministic stable section contains every addition, fix, compatibility action, recovery fact, direct record link, and authenticated credit. |
| metadata | `cargo metadata --no-deps` | Exactly seven stable packages are publishable at 0.10.1 and every shared registry requirement is 0.6.0. |
| packaging | patched 22-package workspace dry run | Every local package verifies, archives remain below 10 MiB, and all required font, legal, ICC, and template assets remain present. |
| integration | both WASM checks and Python metadata assertions | Binding packages compile or retain metadata without gaining crates.io publication authority. |
| release, gate | `/release v0.10.1` post-approval verification | All seven registry entries and owners resolve, the tag and release body match, nine notification URLs are recorded, and PRs 47 through 52 close accurately. |

The **test gate is release**. The stable metadata and API regressions, clean
registry dependency proof, full verification, exact package dry run, archive
inventory, supply-chain gate, binding isolation, release-note validation,
contribution inventory, and unchanged hash result pass at one reviewed SHA.
After separate final approval, all seven 0.10.1 crates, owners, tag, release
body, comments, and authorized PR closures verify.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Inspect every stable manifest,
  workspace pin, shared dependency requirement, lock record, README, CI
  literal, binding version, workflow preflight, test expectation, and changelog
  change. Require a clean full gate and separate approval before tagging.
- **Public API of published crates**. Record that 0.10.1 is a recovery patch
  over a partial pre-1.0 release, run the package dry run, and enforce archive
  contents and limits.
- **Crate dependency graph**. Verify all stable dependencies resolve against
  incubating 0.6.0 without local patches and that no reverse family edge is
  introduced.
- **Bundled fonts and assets**. Verify all expected font and legal files remain
  in `oxml-layout`, no fonts enter `rdocx-layout`, and `rpptx` retains its
  default template.
- **WASM or PyO3 bindings**. Run both WASM targets, pinned Python binding tests,
  and workspace tests with both Python crates excluded.

## Hash harness

Expected unchanged across all 49 entries. This story changes release metadata
and dependency versions only. Any output delta blocks preparation.

## Implementation checklist

- [x] Verify F-X056 registry entries, owners, tag, and release body before changing stable carriers.
- [x] Move every stable version carrier, binding literal, README, lock entry, test expectation, and workflow preflight to 0.10.1.
- [x] Pin every shared dependency requirement to the published 0.6.0 family.
- [x] Add the unpatched registry dependency proof that reproduces and closes the v0.10.0 failure.
- [x] Prepare and validate the complete `v0.10.1` notes and nine-record contribution inventory.
- [x] Verify metadata, binding isolation, package contents, archive limits, and supply chain.
- [x] Run the full workspace gate, deterministic hash harness, and all risk riders.
- [ ] Reach a clean microscope and sprint review at the exact prepared SHA.
- [ ] Stop at `/release v0.10.1` for separate final approval.
- [ ] After approval, verify seven registry entries, owners, tag, release body, nine notification URLs, and six authorized PR closures.

## Open questions

None. The user selected a full incubating release followed by a stable patch
release. The release command fixes each family set and approval boundary.

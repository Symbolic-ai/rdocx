# F-X010, Tag v0.6.0

**Status**: approved
**Sprint**: S39
**Size**: S
**Depends on**: F-X009

## Problem

F-X009 gives every workspace package a checked README, but crates.io releases
are immutable and the published stable 0.5.0 pages cannot acquire the new
documents. The user requested the next minor release of every crate. The
stable family therefore needs a coherent 0.6.0 preparation and its own
reviewed `v0.6.0` release. Direct publication would bypass the exact family,
archive, reviewed-SHA, and immediate-approval controls in `/release`.

## Spec reference

- `docs/hld/11-migration-plan.md`, "What happens to the published crates" and
  "Release tooling".
- `docs/hld/12-testing-strategy.md`, "README examples and package inventory".
- `docs/hld/14-development-backlog.md`, "F-X010, Tag v0.6.0".
- `docs/hld/15-build-and-toolchain.md`, "The two release families" and
  "Release process".
- `.claude/commands/release.md`, "Stable family", "Preconditions", "Final
  approval", and "Release".

## Approach

Move the eleven-package workspace shared-version train from 0.5.0 to 0.6.0.
Set the root workspace version and nine stable internal dependency pins to
0.6.0, regenerate the eleven matching lock entries, and update the two Python
project versions and rdocx WASM contract literals. The exact seven crates.io
packages remain `rdocx-opc`, `rdocx-oxml`, `rdocx-layout`, `rdocx-html`,
`rdocx-pdf`, `rdocx`, and `rdocx-cli`. The four other train members remain
unpublished.

Update stable README dependency examples to 0.6 and update the metadata-derived
README runner and release regressions to require the new version. Rename the
stable metadata preflight to 0.6.0 and keep it wired before the publish
workflow dry run. Preserve all fifteen incubating manifests and pins at 0.1.3.

After implementation, a clean microscope, `/verify --full`, and a clean sprint
review at one exact SHA, invoke `/release v0.6.0`. That command must request a
new immediate approval before its first branch push or tag mutation. After the
workflow succeeds, verify every registry version and owner, every crates.io
README, and the GitHub release target before completing F-X010.

## Rejected alternatives

- Publish 0.5.0 again. Published versions and tags are immutable.
- Use 0.5.1. The user requested the next minor version.
- Publish only `rdocx`. The stable release contract is one coherent
  seven-package family.
- Publish the Python or WASM packages. Their manifests explicitly prohibit
  crates.io publication, and no PyPI or npm release was requested.
- Publish manually with Cargo. Only `/release` may create the tag or start
  publication.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_stable_release_family_is_prepared_at_0_6_0` | Workspace version, nine pins, eleven lock entries, Python metadata, WASM literals, stable README requirements, exact seven-package crates.io set, and unchanged incubating 0.1.3 state are correct |
| regression | publish workflow preflight contract | Stable 0.6.0 and incubating 0.1.3 metadata tests run before the exact patched dry run and failures propagate |
| integration | `python3 scripts/readme_doctests.py` | All 26 README sources remain exact, examples compile or validate, and all 21 publishable archives contain the declared README |
| integration | `/verify --full` | Workspace gates, exact 21-package dry run, archive ceilings, assets, supply chain, and 28 hashes pass at the release SHA |
| boundary | manifest, pin, lock, workflow, and README version mutations | Each independent stale 0.5 requirement makes the named gate fail before byte-identical restoration |
| publication | crates.io and GitHub release inspection | Seven packages resolve at 0.6.0 under the expected owner, each registry README is present, and `v0.6.0` targets the reviewed SHA |

The story test gate is the named stable metadata regression before release and
successful seven-package registry, README, owner, and GitHub verification after
the separately approved release.

## HLD impact

- `docs/hld/11-migration-plan.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect every stable manifest,
  workspace pin, lock entry, README requirement, Python project version, WASM
  literal, CI literal, workflow preflight, and archive. Run `/verify --full`
  and require a clean sprint review plus immediate final approval before the
  first external mutation.
- Crate dependency graph. Confirm only version constraints change and every
  dependency direction remains identical with `cargo metadata` and `cargo
  tree`.
- WASM or PyO3 bindings. Run both WASM target checks and validate Python
  metadata without publishing Python, WASM, npm, or PyPI artifacts.
- Bundled fonts and assets. Inspect the `oxml-layout`, `rdocx-layout`, and
  `rpptx` archive inventories in the exact 21-package dry run.

## Hash harness

Expected unchanged across all 28 entries. Version and documentation metadata
must not change document or render output.

## Implementation checklist

- [x] Record the exact 0.5.0 stable and 0.1.3 incubating starting inventory.
- [x] Add the failing 0.6.0 metadata, workflow, and README version regressions.
- [x] Move the eleven stable-train packages, nine pins, Python project
  versions, WASM literals, and lock entries to 0.6.0.
- [x] Update stable README requirements and the existing metadata-derived
  README gate without changing publication eligibility.
- [x] Prove stale manifest, pin, lock, workflow, and README requirements fail
  their exact gates and restore every mutation byte-identically.
- [x] Run the full metadata, dependency, README, WASM, archive, asset,
  supply-chain, prose, generated-skill, and hash gates.
- [ ] Obtain a clean microscope and clean sprint review at the exact release
  SHA.
- [ ] Invoke `/release v0.6.0` and receive its separate immediate approval.
- [ ] Watch publication and verify all seven versions, owners, READMEs, tag,
  and GitHub release target.
- [ ] Complete the exact HLD and delivery records only after external
  verification succeeds.

## Open questions

None. The requested next stable minor is 0.6.0. Publication remains limited to
the existing seven-package crates.io family. Earlier authorization does not
replace `/release`'s immediate approval at the reviewed SHA.

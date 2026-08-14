# F-X011, Tag rpptx-v0.2.0

**Status**: approved
**Sprint**: S39
**Size**: S
**Depends on**: F-X010

## Problem

F-X009 gives every incubating package a checked README, but crates.io releases
are immutable and the published 0.1.3 pages cannot acquire the new documents.
The user requested the next minor release of every crate. The incubating family
therefore needs a coherent 0.2.0 preparation and its own reviewed
`rpptx-v0.2.0` release after the stable release completes.

## Spec reference

- `docs/hld/03-architecture.md`, "Versioning".
- `docs/hld/14-development-backlog.md`, "F-X011, Tag rpptx-v0.2.0".
- `docs/hld/15-build-and-toolchain.md`, "The two release families" and
  "Release process".
- `.claude/commands/release.md`, "Incubating family", "Preconditions", "Final
  approval", and "Release".

## Approach

Move the complete fifteen-package incubating preparation train from 0.1.3 to
0.2.0. Update the fourteen publishable package manifests, unpublished
`rpptx-wasm`, fourteen root workspace dependency pins, and fifteen matching
lock entries. Update version-sensitive source, workflow, CI, local WASM
package, metadata, and README requirements. The exact crates.io allowlist
remains fourteen packages, and `rpptx-wasm` remains `publish = false`.

Keep the completed stable train at 0.6.0. Do not publish npm, PyPI, Python,
WASM, or stable packages. After implementation, a clean microscope,
`/verify --full`, and a clean sprint review at one exact SHA, invoke `/release
rpptx-v0.2.0`. That command must request a new immediate approval before its
first branch push or tag mutation. After the workflow succeeds, verify every
registry version and owner, every crates.io README, and the GitHub release
target before completing F-X011.

## Rejected alternatives

- Publish 0.1.3 again. Published versions and tags are immutable.
- Use 0.1.4. The user requested the next minor version.
- Publish only packages whose README was newly added. The release contract
  requires one coherent fourteen-package family.
- Publish `rpptx-wasm` or an npm tarball. The WASM crate is only a local member
  of the preparation group and npm publication was not requested.
- Publish manually with Cargo. Only `/release` may create the tag or start
  publication.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_incubating_release_family_is_prepared_at_0_2_0` | Fourteen publishable manifests and pins, fifteen lock entries, descriptions, README requirements, unpublished WASM state, and unchanged stable 0.6.0 state are correct |
| regression | preparation and workflow contracts | The fifteen-member preparation group and fourteen-package crates.io set are exact, and the 0.2.0 preflight runs before publication |
| integration | `python3 scripts/readme_doctests.py` | All 26 README sources remain exact, examples compile or validate, and all 21 publishable archives contain the declared README |
| integration | `/verify --full` | Workspace gates, exact 21-package dry run, archive ceilings, assets, supply chain, WASM gates, and 28 hashes pass at the release SHA |
| boundary | manifest, pin, lock, workflow, README, and WASM version mutations | Each independent stale 0.1.3 requirement makes the named gate fail before byte-identical restoration |
| publication | crates.io and GitHub release inspection | Fourteen packages resolve at 0.2.0 under the expected owner, each registry README is present, and `rpptx-v0.2.0` targets the reviewed SHA |

The story test gate is the named incubating metadata regression before release
and successful fourteen-package registry, README, owner, and GitHub
verification after the separately approved release.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Release scripting and version strings. Inspect all fifteen manifests,
  fourteen workspace pins, lock entries, source assertions, README
  requirements, workflow preflight, local WASM version, and archives. Run
  `/verify --full` and require a clean sprint review plus immediate final
  approval before the first external mutation.
- Crate dependency graph. Confirm only version constraints change and every
  dependency direction remains identical with `cargo metadata` and `cargo
  tree`.
- WASM bindings. Run both locked WASM target and Node gates plus the local
  `rpptx-wasm` pack and fresh install check without npm publication.
- Bundled fonts and assets. Inspect the `oxml-layout`, `rdocx-layout`, and
  `rpptx` archive inventories in the exact 21-package dry run.

## Hash harness

Expected unchanged across all 28 entries. Version and documentation metadata
must not change document or render output.

## Implementation checklist

- [ ] Confirm F-X010 is published and completed at 0.6.0.
- [ ] Record the exact 0.1.3 incubating starting inventory.
- [ ] Add the failing 0.2.0 metadata, workflow, README, and WASM regressions.
- [ ] Move fourteen publishable manifests, unpublished `rpptx-wasm`, fourteen
  pins, and fifteen lock entries to 0.2.0.
- [ ] Update only existing version-sensitive source, README, CI, workflow, and
  local package assertions required by the new preparation version.
- [ ] Prove stale manifest, pin, lock, workflow, README, and WASM requirements
  fail their exact gates and restore every mutation byte-identically.
- [ ] Run the full metadata, dependency, README, WASM, archive, asset,
  supply-chain, prose, generated-skill, and hash gates.
- [ ] Obtain a clean microscope and clean sprint review at the exact release
  SHA.
- [ ] Invoke `/release rpptx-v0.2.0` and receive its separate immediate
  approval.
- [ ] Watch publication and verify all fourteen versions, owners, READMEs,
  tag, and GitHub release target.
- [ ] Complete the exact HLD and delivery records only after external
  verification succeeds.

## Open questions

None. The requested next incubating minor is 0.2.0. Publication remains limited
to the existing fourteen-package crates.io family. Earlier authorization does
not replace `/release`'s immediate approval at the reviewed SHA.

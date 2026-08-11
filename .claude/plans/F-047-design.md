# F-047, Packaging include and size gate

**Status**: completed
**Sprint**: S32.1
**Size**: M
**Depends on**: F-037

## Problem

`crates/oxml-layout/Cargo.toml:11` still prevents publication even though its
include list at lines 12 through 17 names the source, bundled fonts, licences,
and Caladea notice. The repository has no CI job that proves the generated
archive contains those files, verifies from the archive, and stays below the
crates.io 10 MiB limit.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-047, Packaging include and size
  gate".
- `docs/hld/15-build-and-toolchain.md`, "Packaging".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".

## Approach

Make `oxml-layout` a publication candidate by removing its development-only
`publish = false` guard while retaining the explicit include list. Add a
packaging job to `.github/workflows/ci.yml` that inspects `cargo package -p
oxml-layout --list`, requires all 20 TTFs and the three licence files plus the
Caladea notice, then runs verified packaging and rejects an archive larger than
10 MiB. Keep the gate as direct Cargo and shell steps in the existing workflow
so the package contract is visible where it runs.

## Rejected alternatives

- Use `--no-verify` to avoid resolving packaged dependencies. F-047 exists to
  remove that blind spot.
- Rely only on the workspace publication dry-run. It does not assert the exact
  font and licence inventory.
- Add a second packaging script. The focused checks fit in the existing CI job
  without adding another place to discover the policy.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `cargo package -p oxml-layout --list` inventory checks | All 20 TTFs, `LICENSE-Caladea`, `NOTICE-Caladea`, `LICENSE-Carlito`, and `LICENSE-Liberation` are packaged |
| integration | `cargo package -p oxml-layout` | Cargo verifies the crate from its generated archive without `--no-verify` |
| regression | archive byte-size assertion | The generated `oxml-layout-*.crate` is no larger than 10 MiB |

The backlog test gate is `cargo package --list` containing every TTF and
licence file with the archive under 10 MiB.

## HLD impact

- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Bundled fonts. Inspect the package list for every font family, its real
  licence, and the Caladea notice. Run verified packaging and the archive-size
  assertion from `docs/hld/15-build-and-toolchain.md`.
- Public API of a published crate. No Rust surface changes. Run
  `cargo publish --workspace --dry-run` and assert every generated archive is
  below 10 MiB.

## Hash harness

Expected to remain unchanged. Packaging metadata and CI do not alter rendered
output.

## Implementation checklist

- [x] Remove the `oxml-layout` publication guard without weakening its include
      list.
- [x] Add the exact package inventory check to the existing CI workflow.
- [x] Run verified packaging and enforce the 10 MiB limit in CI.
- [x] Run the focused inventory, packaging, and archive-size checks locally.

## Open questions

None. The backlog and packaging specification define the exact archive
contents and size limit.

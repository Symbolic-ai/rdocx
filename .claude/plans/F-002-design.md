# F-002, rust-toolchain.toml

**Status**: completed
**Sprint**: S01
**Size**: S
**Depends on**: none

## Problem

The repository has no `rust-toolchain.toml`. CI selects `stable` at
`.github/workflows/ci.yml:21`, while the separate MSRV job pins 1.93 at lines
58 to 66. Local development therefore has no repository-owned toolchain even
though repeatable formatting, linting, and WASM checks depend on one.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, "Toolchain pinning".

## Approach

Add the specified root `rust-toolchain.toml` with channel `1.97.1`, components
`rustfmt` and `clippy`, and target `wasm32-unknown-unknown`. Leave
`rust-version = "1.93"` and the explicit MSRV CI job unchanged because they
describe compatibility rather than the development toolchain.

## Rejected alternatives

- Pinning only the CI actions was rejected because local development would
  remain unpinned.
- Changing the workspace MSRV to 1.97.1 was rejected because it would discard
  the independently tested compatibility promise.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `rustup show active-toolchain` | A clean checkout selects Rust 1.97.1 from the repository file. |
| regression | inspect the MSRV job and workspace manifest | The 1.93 compatibility pin remains explicit and unchanged. |

The **test gate** is `rustup show active-toolchain`: it reports the pinned
channel in a clean clone, while the MSRV job still pins 1.93 separately.

## HLD impact

- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- New file. The structural rules in `CLAUDE.md` require explicit approval for
  `rust-toolchain.toml`. Verify its parsed values with `rustup show` and confirm
  the existing MSRV declarations remain 1.93.

## Hash harness

Expected to be unchanged.

## Implementation checklist

- [x] Add `rust-toolchain.toml` with the specified channel, components, and target.
- [x] Confirm rustup selects the pinned channel.
- [x] Confirm the workspace and CI MSRV pins remain 1.93.

## Open questions

None. The new root `rust-toolchain.toml` is approved.

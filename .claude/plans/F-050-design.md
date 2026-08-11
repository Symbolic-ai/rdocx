# F-050, CI matrix additions

**Status**: approved
**Sprint**: S32.1
**Size**: S
**Depends on**: none

## Problem

`.github/workflows/ci.yml:23` and line 105 run all-feature workspace tests
without the required Python binding exclusions. The workflow has no job for
`oxml-layout --no-default-features`, no wasm target check, and no tracked
Markdown prose or generated-skill drift gate, even though each is already part
of the local verification contract.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "CI" and "WASM".
- `docs/hld/12-testing-strategy.md`, "What CI runs".
- `docs/hld/14-development-backlog.md`, "F-050, CI matrix additions".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".

## Approach

Add three focused jobs to the existing CI workflow. One runs
`cargo test -p oxml-layout --no-default-features`, one installs the declared
wasm target and runs `cargo check --target wasm32-unknown-unknown -p
rdocx-wasm`, and one runs both the prose checker and generated-skill drift
check. Correct the ordinary and MSRV all-feature test jobs to exclude
`rdocx-py` and `rpptx-py`, matching the repository gate.

Do not add `rpptx-wasm`, which does not exist in the current workspace. The
focused checks execute every new job command locally before integration.

## Rejected alternatives

- Fold all additions into the general test job. Separate job names make the
  failing portability contract visible in branch protection and CI output.
- Add `wasm-pack test --node` now. The current `rdocx-wasm` crate has no wasm
  test target, and F-138 owns the future rpptx wasm surface.
- Include Python binding crates in all-feature workspace test binaries. Their
  `pyo3/extension-module` linkage requires a host interpreter and fails on the
  normal Rust test runner.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `cargo test -p oxml-layout --no-default-features` | Bundled-fonts-off layout compiles and tests without system font discovery |
| integration | `cargo check --target wasm32-unknown-unknown -p rdocx-wasm` | The existing wasm crate remains portable to its declared target |
| regression | `python3 scripts/prose_check.py` and `python3 scripts/sync_agent_skills.py --check` | Tracked prose and generated adapters satisfy the repository contract |

The backlog test gate is every new CI job passing on a clean tree.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- WASM or PyO3 bindings. Run the exact wasm target check. Keep
  `--exclude rdocx-py --exclude rpptx-py` on workspace all-feature test jobs as
  required by `docs/hld/10-bindings-spec.md`.

## Hash harness

Expected to remain unchanged. CI coverage does not modify product behavior or
rendered output.

## Implementation checklist

- [ ] Correct all-feature test jobs to exclude both Python binding crates.
- [ ] Add the `oxml-layout` no-default-features job.
- [ ] Add the existing `rdocx-wasm` target check job.
- [ ] Add prose and generated-skill drift checks.
- [ ] Run every added command locally.

## Open questions

None. The current workspace and verification command define the exact package
set and commands.

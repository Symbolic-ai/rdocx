# F-140, wasm CI job

**Status**: completed
**Sprint**: S35
**Size**: S
**Depends on**: F-139, F-142

## Problem

The pull-request WASM job only checks `rdocx-wasm` for the target. It neither
executes exported behavior in Node nor checks the new `rpptx-wasm` package.
Compile-only coverage allowed the destructive rdocx mini-model to remain
shipped and leaves two S35 wrappers outside behavioral CI.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "WASM" and "CI".
- `docs/hld/12-testing-strategy.md`, "What CI runs" and "Gaps being closed".
- `docs/hld/14-development-backlog.md`, "F-140, wasm CI job".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".

## Approach

Extend the existing `wasm` job in `.github/workflows/ci.yml`. Keep the operative
pull-request trigger, root `contents: read`, and immutable checkout, Rust, and
cache actions. Add exact Node 24.11.1 through reviewed setup-node commit
`249970729cb0ef3589644e2896645e5dc5ba9c38`, install exact wasm-pack 0.15.0
with `cargo install --locked`, then run locked wasm32 checks and
`wasm-pack test --node` for both `rdocx-wasm` and `rpptx-wasm`.

Strengthen `scripts/test_sprint_workflow.py` with structured, comment-insensitive
assertions for exact tools, package set, command order, immutable actions,
least privilege, and ordinary failure propagation. Reject conditions,
`continue-on-error`, fallback success, listing-only tests, missing `--node`,
and package omissions. F-139 and F-142 own non-vacuous crate-root Node tests.

Reconcile the integrated F-142 cargo-release bookkeeping in the same existing
regression file and HLD15. The `incubating` preparation group now contains the
12 published packages plus unpublished `rpptx-wasm`, while the crates.io
allowlist remains exactly 12. Require the exact 13-member preparation group and
prove a family-metadata mutation is rejected.

## Rejected alternatives

- Add a second WASM workflow. That duplicates trigger and permission ownership.
- Check only rdocx-wasm. S35 requires both real wrappers to be watched.
- Use floating Node or wasm-pack versions. Runner drift would become product
  regression noise.
- Add shell-only JavaScript smoke tests. The Rust wasm-bindgen tests are the one
  executable contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_wasm_pr_job_checks_both_targets_and_runs_node_tests` | Both packages receive locked target checks and Node execution in order |
| regression | `test_wasm_pr_job_rejects_skipped_or_weakened_gates` | Mutations cannot skip, collect only, weaken, or suppress either package gate |
| integration, gate | exact CI command sequence | `cargo check --target wasm32-unknown-unknown` and `wasm-pack test --node` execute successfully on pull requests |

The test gate is the backlog requirement that the target check and Node tests
both run on pull requests. Sensitivity makes one Node test panic, proves the
exact job command fails, restores byte-identically, and reruns green.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- WASM binding. Read HLD10. Run both target checks and Node suites while
  retaining required PyO3 workspace exclusions.
- Version strings. Pin Node, wasm-pack, and every new action to reviewed exact
  versions or commits. Inspect the workflow diff and perform no release action.
- Crate dependency graph. Confirm neither Node suite introduces `getrandom`,
  PyO3, or a forbidden format-family edge.
- No new file or module. The existing CI workflow and workflow regression file
  remain the sole owners.

## Hash harness

Expected unchanged. This story changes CI assertions and execution only.

## Implementation checklist

- [x] Update F-140 dependency prose to require both completed WASM packages.
- [x] Extend the existing WASM job with exact immutable tools.
- [x] Run target checks and Node suites for both packages.
- [x] Add structured positive and mutation-sensitive workflow regressions.
- [x] Prove a real Node failure propagates through the exact command.
- [x] Reconcile the 13-member incubating preparation group after F-142.

## Open questions

None. The expanded F-142 dependency, two-package CI scope, and exact Node,
setup-node, wasm-pack, and wasm-bindgen-test tool family are approved.

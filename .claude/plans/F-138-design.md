# F-138, PR-time Python job

**Status**: completed
**Sprint**: S34
**Size**: S
**Depends on**: F-137

## Problem

The existing CI workflow excludes binding crates from Rust workspace binaries,
but it never builds either extension or runs Python tests. A binding can
therefore regress while the pull request remains green.

## Spec reference

- `docs/hld/10-bindings-spec.md`, "CI".
- `docs/hld/12-testing-strategy.md`, "What CI runs".
- `docs/hld/14-development-backlog.md`, "F-138, PR-time Python job".
- `docs/hld/15-build-and-toolchain.md`, "CI job matrix".

## Approach

Add one visible Python bindings job to the existing PR workflow. Use an exact
Python version supported by the installed tooling. For each package, create an
isolated environment, run `maturin develop`, then run its complete pytest
suite. Install exact dependencies from the package test contract: Python
3.12.9, `maturin==1.13.3`, `pytest==9.1.1`, and the
applicable pinned oracle. Preserve both binding exclusions on every Rust
all-feature job. The operative top-level pull-request trigger schedules the
job without a job condition. Root permissions are exactly `contents: read`,
with no OIDC authority. Pin the job's checkout, Rust toolchain, cache, and
Python setup actions to reviewed immutable revisions with exact input maps.

Extend the existing workflow regression suite to require both package cells,
the operative pull-request trigger, least-privilege permissions, immutable
critical actions, the build-before-test order, and ordinary failure
propagation. A deliberate mutation that makes one binding test fail must make
the exact job command and the workflow contract regression fail.

## Rejected alternatives

- Add the binding crates to Rust workspace test binaries. The extension-module
  feature cannot link there.
- Reuse release wheel artifacts on pull requests. PR feedback should build the
  reviewed source directly.
- Allow test failures or continue-on-error. That defeats the named story gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | local exact `maturin develop` then full pytest for both packages | A binding test failure produces a nonzero job command |
| regression | `python_pr_job_builds_both_extensions_before_pytest` | CI contains both package cells in the required order |
| regression | deliberate test-failure and workflow mutations | Neither shell nor workflow can swallow a failing binding test |

The test gate is the backlog requirement that the job fails when a binding test
fails.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- WASM or PyO3 bindings. Retain both binding exclusions, run the existing
  rdocx-wasm check, and prove by dependency tree that PyO3 remains outside the
  WASM graph. The rpptx-wasm package remains deferred to F-142.

## Hash harness

Expected unchanged. CI coverage does not change the libraries.

## Implementation checklist

- [x] Add the two-package Python job to `.github/workflows/ci.yml`.
- [x] Install the exact build, test, and oracle dependencies per package.
- [x] Build each extension before running its full pytest suite.
- [x] Add positive and mutation-sensitive workflow regressions.
- [x] Prove a real binding failure propagates from the exact local command.
- [x] Bind operative PR scheduling and exact least-privilege permissions.
- [x] Pin every critical job action and constrain its exact input map.

## Open questions

None. F-137 establishes both package build paths, and the existing CI file is
the single PR workflow owner.

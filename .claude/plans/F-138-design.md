# F-138, PR-time Python job

**Status**: approved
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
suite. Install exact oracle and typing dependencies from the package's tracked
development group. Preserve both binding exclusions on every Rust all-feature
job and keep publication permissions absent.

Extend the existing workflow regression suite to require both package cells,
the build-before-test order, and ordinary failure propagation. A deliberate
mutation that makes one binding test fail must make the exact job command and
the workflow contract regression fail.

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

- WASM or PyO3 bindings. Retain both binding exclusions and run both WASM
  checks alongside installed binding suites.

## Hash harness

Expected unchanged. CI coverage does not change the libraries.

## Implementation checklist

- [ ] Add the two-package Python job to `.github/workflows/ci.yml`.
- [ ] Install tracked exact development dependencies per package.
- [ ] Build each extension before running its full pytest suite.
- [ ] Add positive and mutation-sensitive workflow regressions.
- [ ] Prove a real binding failure propagates from the exact local command.

## Open questions

None. F-137 establishes both package build paths, and the existing CI file is
the single PR workflow owner.

# F-X012, Restore pinned CI toolchains

**Status**: approved
**Sprint**: S40
**Size**: M
**Depends on**: none

## Problem

Hosted CI has five failing jobs at the S39 merge SHA. The Test and MSRV jobs
execute the F-043 `pdftoppm` gate without installing Poppler. The two macOS
oracle jobs call unpinned `brew install poppler`, which now supplies 26.07.0
instead of the reviewed 26.01.0. The WASM job verifies a checksum-pinned
Binaryen 125 archive but rejects its official Linux identity string because it
expects the shorter Homebrew spelling.

The same failure set is present in the S37 and S38 merge runs. This is a
workflow portability defect rather than an S39 product regression.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and "The render
  fidelity gate".
- `docs/hld/15-build-and-toolchain.md`, "A WASM target and Node job", "A
  pull-request Python bindings job", and "A dedicated Presentation fidelity
  job".
- `docs/hld/14-development-backlog.md`, "F-X012, Restore pinned CI
  toolchains".

## Approach

Add one authorized script, `scripts/install_pinned_poppler.py`. It downloads
the official Poppler 26.01.0 source archive, verifies SHA-256
`1cb944a4b88847f5fb6551683bc799db59f04990f5d8be07aba2acbf38601089`,
builds only the required command-line tools into a caller-selected prefix, and
verifies exact `pdftoppm`, `pdfinfo`, and `pdftotext` version identities. It
uses bounded extraction, an isolated build directory, explicit CMake options,
and no published-crate dependency.

Wire Test, MSRV, both Python binding matrix rows, and Presentation fidelity to
the same installer before any Poppler-dependent test. Platform package managers
may install build dependencies, but never Poppler itself. Keep LibreOffice
installation separate. Correct the WASM identity assertion to the exact
official Linux output `wasm-opt version 125 (version_125)` after the existing
archive checksum verification.

Extend the existing workflow regression suite. Do not add another test binary
or another workflow file.

## Rejected alternatives

- Upgrade the rendering baseline to Poppler 26.07.0. That would turn a CI
  packaging defect into an unrelated output change.
- Keep using `brew install poppler`. The formula is moving and has already
  invalidated the exact oracle contract.
- Duplicate the source-build commands in four jobs. One installer gives the
  version, checksum, build, and runtime checks one owner.
- Skip the Poppler-dependent Rust test in Test and MSRV. The broad workspace
  commands must remain honest and self-contained.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `test_pinned_poppler_installer_contract` | Exact version, source checksum, tools, extraction bound, and build options |
| regression | `test_every_poppler_consumer_uses_the_pinned_installer` | Test, MSRV, Python bindings, and Presentation fidelity install the oracle before use |
| regression | `test_wasm_pr_job_checks_both_targets_and_runs_node_tests` | Official Binaryen 125 Linux identity is accepted only after checksum verification |
| integration | installer on macOS and disposable Ubuntu 24.04 | All three Poppler tools report exact 26.01.0 and execute |
| integration | hosted pull-request CI run | Every job passes at the reviewed SHA |

The **test gate** is the workflow contract plus full verification and a hosted
pull-request CI run at the reviewed SHA, all with the 28-entry hash harness
unchanged.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- New file. The user explicitly authorized
  `scripts/install_pinned_poppler.py`. It is the single owner for the shared
  cross-platform oracle installation.
- External oracle comparison. Retain Poppler 26.01.0, verify the official
  source checksum and all three runtime identities, and run the existing exact
  pixel and fidelity gates without changing their baselines.

## Hash harness

Expected to be unchanged. The story changes CI tool installation and
validation only.

## Implementation checklist

- [ ] Record the five-job hosted failure set and focused workflow-test red.
- [ ] Add the authorized Poppler installer with exact source and runtime checks.
- [ ] Wire every Poppler consumer job to the shared installer.
- [ ] Correct the official Binaryen 125 Linux identity assertion.
- [ ] Add positive and mutation-sensitive workflow regressions.
- [ ] Prove the installer on macOS and disposable Ubuntu 24.04.
- [ ] Run full verification with all 28 hashes unchanged.
- [ ] Obtain a fully green hosted pull-request CI run at the reviewed SHA.
- [ ] Obtain a clean independent microscope and sprint review.

## Open questions

None. The user authorized S40, F-X012, and the single new installer path. The
reviewed Poppler and rendering baselines remain fixed.

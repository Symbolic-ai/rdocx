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
workflow portability defect rather than an S39 product regression. The first
hosted validation of the Poppler and Binaryen corrections then exposed two
previously unreachable corpus-test requirements in Test and MSRV. Those jobs
did not install the `uv` executable used by the pinned python-pptx oracles, and
their default Rust test-thread stack is too small for the largest modelled
corpus round trip on hosted Linux. The second hosted validation reached the
three `rpptx-chart` viewer gates and exposed one final clean-runner requirement:
Test and MSRV also need the exact LibreOffice oracle that those gates execute.

## Spec reference

- `docs/hld/12-testing-strategy.md`, "The golden-PNG gate" and "The render
  fidelity gate".
- `docs/hld/15-build-and-toolchain.md`, "A WASM target and Node job", "A
  pull-request Python bindings job", and "A dedicated Presentation fidelity
  job".
- `docs/hld/14-development-backlog.md`, "F-X012, Restore pinned CI
  toolchains".

## Approach

Add authorized script `scripts/install_pinned_poppler.py`. It downloads
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

In Test and MSRV, install exact `uv` 0.10.2 through the official setup action
at reviewed commit `20cfd1bf945f4377ade1205e4dbc17946fc9a30d`. Run the full
workspace suite with an isolated runner-temporary uv cache and an 8 MiB
`RUST_MIN_STACK`. This keeps the two concurrent pinned python-pptx oracles
self-contained and gives the existing largest corpus round trip a declared
hosted-Linux stack budget.

Add the separately authorized `scripts/install_pinned_libreoffice.py` for the
two Ubuntu 24.04 workspace jobs. It downloads the official LibreOffice 26.2.5 Linux
x86-64 Debian archive, verifies SHA-256
`2f03bfb2ac9f33ea7c77331b4b7a23300fb0ed7443566046bf8b5bc51c1bed1e`,
streams extraction under member and expanded-byte bounds, rejects unsafe
members and a populated `/opt/libreoffice26.2` prefix, installs only from the
reviewed archive, provisions the explicit Ubuntu system libraries needed by
that build, and verifies exact runtime identity
`LibreOffice 26.2.5.2 cd7284b4cbbfeb507e630c1aac019f4157393acb`.
Test and MSRV invoke it before the full workspace suite. The existing macOS
Presentation fidelity job retains its separately reviewed installation path.

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
- Skip the three `rpptx-chart` viewer tests on Ubuntu. They are part of the
  unconditional workspace suite and require the same reviewed oracle identity.
- Trust a preinstalled `soffice` based on version output. The installer must
  prove archive provenance and refuses any populated target prefix.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `test_pinned_poppler_installer_contract` | Exact version, source checksum, tools, extraction bound, and build options |
| regression | `test_every_poppler_consumer_uses_the_pinned_installer` | Test, MSRV, Python bindings, and Presentation fidelity install the oracle before use |
| regression | `test_workspace_oracle_jobs_pin_uv_cache_and_stack` | Test and MSRV pin official uv 0.10.2, isolate its cache, and run with the exact 8 MiB stack budget |
| unit | `test_pinned_libreoffice_installer_enforces_runtime_guards` | Checksum, download, member, expanded-byte, prefix, and runtime-identity guards execute |
| regression | `test_workspace_viewer_jobs_install_pinned_libreoffice` | Test and MSRV install exact LibreOffice unconditionally before the workspace suite |
| regression | `test_wasm_pr_job_checks_both_targets_and_runs_node_tests` | Official Binaryen 125 Linux identity is accepted only after checksum verification |
| integration | installers on macOS and disposable Ubuntu 24.04 | All three Poppler tools and pinned LibreOffice report their exact identities and execute |
| integration | hosted pull-request CI run | Every job passes at the reviewed SHA |

The **test gate** is the workflow contract plus full verification and a hosted
pull-request CI run at the reviewed SHA, all with the 28-entry hash harness
unchanged.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- New files. The user explicitly authorized
  `scripts/install_pinned_poppler.py` and
  `scripts/install_pinned_libreoffice.py`. The first owns the shared
  cross-platform Poppler installation. The second owns exact Ubuntu
  LibreOffice installation for broad workspace jobs.
- External oracle comparison. Retain Poppler 26.01.0, verify the official
  source checksum and all three runtime identities, and run the existing exact
  pixel and fidelity gates without changing their baselines.
- Hosted test runtime. Pin the official uv setup action and executable version,
  isolate its cache per job, and bind the explicit stack budget to only the two
  broad corpus-test jobs. No product runtime or published package changes.
- External LibreOffice oracle. Retain version 26.2.5.2 and the reviewed build
  identity. Verify the official Linux archive checksum and resource bounds,
  provision its explicit Ubuntu runtime libraries, then run the existing
  viewer gates without changing their expected output.

## Hash harness

Expected to be unchanged. The story changes CI tool installation and
validation only.

## Implementation checklist

- [x] Record the five-job hosted failure set and focused workflow-test red.
- [x] Add the authorized Poppler installer with exact source and runtime checks.
- [x] Wire every Poppler consumer job to the shared installer.
- [x] Correct the official Binaryen 125 Linux identity assertion.
- [x] Pin uv and the corpus-test stack budget in Test and MSRV.
- [x] Add the authorized pinned LibreOffice installer to Test and MSRV.
- [x] Add positive and mutation-sensitive workflow regressions.
- [x] Prove the installers on macOS and disposable Ubuntu 24.04.
- [ ] Run final clean-tree verification with all 28 hashes unchanged.
- [x] Obtain a fully green hosted pull-request CI run at the reviewed SHA.
- [x] Obtain a clean independent microscope.

## Open questions

None. The user authorized S40, F-X012, and both new installer paths. The
reviewed Poppler and LibreOffice rendering baselines remain fixed.

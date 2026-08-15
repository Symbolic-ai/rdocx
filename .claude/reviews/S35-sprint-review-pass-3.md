# S35 sprint review, pass 3

**Reviewed**: `sprint/s35` at `412a29ce5769117fca74d139981e8ccc9903d018`
against merge base `dafc783b1954aacec370ce38b889294aa8db0ebc`, 53 files,
3,938 insertions and 1,795 deletions, crates: `oxml-pdf`, `rdocx`,
`rdocx-layout`, `rdocx-wasm`, `rpptx`, `rpptx-render`, and `rpptx-wasm`, plus
their CLI, Python, and CI consumers
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have
**Run-sprint disposition**: 0 fix-now, 0 tracked-follow-up, 0 human-action,
0 refuted findings

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Earlier findings

### B1, resolved

The musllinux step now creates a fresh Python 3.9 Alpine environment, installs
the current package wheel and pinned oracles, and branches to the applicable
installed-wheel parity suite at `.github/workflows/wheels.yml:105`. The rdocx
cell runs its four approved test modules at `.github/workflows/wheels.yml:122`,
and the rpptx cell runs its documented-example parity module at
`.github/workflows/wheels.yml:128`.

The structured contract requires the exact fresh-environment and package
branches at `scripts/test_sprint_workflow.py:1069`. Its explicit
`musllinux-import-only` mutation at `scripts/test_sprint_workflow.py:1914`
removes the parity branch and must fail. The complete 33-test workflow module
passed independently during this review.

Corrected GitHub Actions run 31722258395 is a completed successful
`workflow_dispatch` at the exact reviewed SHA. Its two `Install and test
musllinux wheel` steps succeeded. The rdocx log reports 24 tests passed,
including its python-docx parity module, and the rpptx log reports all 10
documented-example tests passed. The other ten wheel jobs and both
source-distribution jobs also succeeded, exactly fourteen artifacts exist, and
the temporary `s35-wheel-acceptance` branch is deleted.

The publish job was skipped, as required for manual acceptance. The operative
guard limits publication to a `py-v*` tag push at
`.github/workflows/wheels.yml:167`, and the current binding specification says
manual dispatch cannot publish at `docs/hld/10-bindings-spec.md:241`. Future
PyPI publication is intentionally outside this sprint and is not a finding.

## Sprint definition of done

All six S35 items hold.

- The package-preserving rdocx facade wrapper, two-package locked WASM target
  and Node CI gates, embedded-font browser PDF, bounded rpptx default profile,
  facade-owned optional rendering, feature isolation, and unchanged hashes
  retain the integrated evidence recorded in pass 1. No Rust source, manifest,
  dependency, or public-surface file changed during B1 remediation.
- The hosted wheel requirement at `docs/sprints/CURRENT_SPRINT.md:55` is now
  satisfied by run 31722258395. Both packages built on all six target families,
  native cells ran the approved parity suites, and both musllinux cells ran the
  same package parity suites in fresh Alpine environments.
- Documentation matches the implemented and hosted behavior. HLD 10 requires
  each musllinux package parity suite at `docs/hld/10-bindings-spec.md:237`, HLD
  12 requires clean Alpine parity execution and mutation sensitivity at
  `docs/hld/12-testing-strategy.md:432`, and HLD 15 records the same build
  boundary at `docs/hld/15-build-and-toolchain.md:271`.
- Fresh integrated evidence passed the complete 33-test workflow module, prose
  validation, diff hygiene, and the hash harness with all 28 entries matching.
  The run state still records all four S35 stories completed at
  `.claude/scratch/S35-run.json:58`.

## Milestone gate

The M13 end gate is: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`.

The gate now holds. Corrected hosted run 31722258395 is bound to the reviewed
SHA and supplies successful installed-wheel parity evidence for rdocx and
rpptx across manylinux x86_64 and aarch64, musllinux x86_64, macOS x86_64 and
arm64, and Windows x86_64. Its twelve wheel jobs and two source-distribution
jobs all succeeded, and the artifact inventory has the exact fourteen products.

M13 itself remains open for genuinely future work. F-143 `oxml-cli-support`,
F-144 `rpptx-cli`, F-145 `rpptx-cli thumbnail and outline`, and F-146 `npm
publication` remain pending at `docs/sprints/BACKLOG.md:277`. Their remaining
product work does not invalidate the wheel gate or block S35 closure. PyPI
publication also remains deferred by intent.

## Not found

- **Interaction**: the parity correction changes only the Python wheel workflow,
  its contract, and matching HLD prose. It does not alter either WASM wrapper,
  facade, feature graph, renderer, or package model.
- **Duplication**: no second production helper or package test path was added.
  Native and Alpine cells select the same repository parity modules.
- **Layering**: no crate dependency edge changed during remediation, and the
  integrated diff still introduces no forbidden `oxml-*` dependency on a
  facade family.
- **Harness**: the remediation changes no document behavior or baseline, and a
  fresh check matched all 28 entries.
- **Gate**: every S35 definition-of-done item and the exact M13 wheel gate now
  have executable evidence.
- **Docs**: HLD 10, 12, and 15 consistently describe the corrected Alpine
  behavior, manual publication boundary, and hosted matrix.
- **Dependencies**: no dependency was added by remediation. Test packages are
  the already approved pinned workflow inputs.
- **Surface**: no Rust, Python, JavaScript, or publication surface changed.
- **Artifacts and security**: the hosted artifact count is exact, the temporary
  branch is gone, build jobs retain read-only authority, and manual dispatch did
  not receive or exercise PyPI publication authority.

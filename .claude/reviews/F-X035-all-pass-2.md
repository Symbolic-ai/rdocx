# F-X035, all, pass 2

**Reviewed**: the complete 49-file working diff, 374 changed lines, including pass 1 remediation, against the approved F-X035 release, release-note, HLD, dependency, archive, WASM, and workflow contracts
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 1 D1 is fixed at `.claude/commands/release.md:66`. The release
  precondition now states the exact 22-patch archive command. The positive and
  mutated forms are covered at `scripts/test_sprint_workflow.py:4432` and
  `scripts/test_sprint_workflow.py:4453`.
- Pass 1 D2 is fixed at `.claude/commands/verify.md:59`. The canonical full
  gate now checks both `rdocx-wasm` and `rpptx-wasm`, and the focused helper and
  mutation test at `scripts/test_sprint_workflow.py:5246` fail if the
  incubating package is removed.
- The regenerated release and verify adapters contain the exact SHA-256 values
  of their canonical commands at `.agents/skills/release/SKILL.md:10` and
  `.agents/skills/verify/SKILL.md:10`. The generated-skill drift gate passes.
- All 15 publishable incubating packages, their workspace pins, and the 16
  prepared manifest and lockfile carriers remain at 0.4.0. The stable family
  remains 0.7.0, and `rpptx-wasm` remains unpublished. Metadata reports exact
  internal requirements and no forbidden format dependency from an `oxml-*`
  crate.
- The publish workflow still has disjoint family predicates, the exact
  dependency-ordered 15-package incubating allowlist, bare fail-closed publish
  commands, and waits between dependency layers at
  `.github/workflows/publish.yml:55` and `.github/workflows/publish.yml:72`.
- The `rpptx-v0.4.0` notes at `CHANGELOG.md:103` pass deterministic check and
  render. Their claims remain supported by the `rpptx-v0.3.0..HEAD` evidence
  range, scoped to shared OOXML and PowerPoint changes, and matched to verified
  issue, pull-request, and commit contributor identities.
- The two plan-listed HLD edits retain prepared-state truth. HLD 10 identifies
  the prepared incubating 0.4.0 source boundary at
  `docs/hld/10-bindings-spec.md:343`. HLD 15 records the exact prepared family,
  separate final approval, and unpublished WASM state at
  `docs/hld/15-build-and-toolchain.md:173` and
  `docs/hld/15-build-and-toolchain.md:269` without claiming a release occurred.
- All 15 selected archives package successfully and remain below 10 MiB.
  `oxml-layout` retains the complete bundled TTF and legal-file inventory,
  `rdocx-layout` does not package those assets, and `rpptx` retains
  `assets/default.pptx`. The exact combined WASM command now documented by the
  full gate passes for both package graphs.
- All 63 sprint-workflow tests pass, including both remediation regressions and
  the renamed 0.4.0 publication preflight. Release-note check and render,
  metadata, prose, adapter drift, archive, and diff checks pass. The exact
  `rpptx-v0.4.0` tag remains absent locally and from `origin`.
- No panic, OOXML-order, unmodelled-XML, structural-rule, or unrelated metadata
  regression is introduced by the current diff.

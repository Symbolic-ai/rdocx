# F-X060, working, pass 1

**Reviewed**: complete uncommitted working diff against
`6a9a0560a83b17bab9d6b23950302b5895c69311`, 24 tracked files with 372
insertions and 112 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness and release contract: the workspace carrier is 0.11.0 and the
  nine inherited internal pins divide correctly between stable 0.11.0 and
  shared 0.7.0 at `Cargo.toml:34` and `Cargo.toml:55`. The preparation metadata
  continues to disable publication, tagging, and pushing at `Cargo.toml:44`.
- Family isolation and publication order: the carrier regression enumerates
  the exact seven publishable stable packages and all eleven inherited stable
  carriers at `scripts/test_sprint_workflow.py:4520`. The real workflow retains
  the seven-package dependency order at `.github/workflows/publish.yml:55` and
  keeps the incubating allowlist separate at `.github/workflows/publish.yml:72`.
- Registry proof: the new gate inspects the normalized packaged
  `rdocx-layout@0.11.0` manifest, requires registry `oxml-layout@0.7.0` without
  a path, and resolves exact published 0.7.0 with a fresh Cargo home and no
  shared patch at `scripts/test_sprint_workflow.py:4694`. The separate immutable
  consumer still requires `rdocx-layout@0.10.1`, resolves
  `oxml-layout@0.6.0`, and rejects 0.7.0 at
  `scripts/test_sprint_workflow.py:4773`.
- Contribution inventory and release notes: the deterministic notes regression
  requires Issues 53 and 54 and PRs 55 through 58 exactly twice, authentic
  contributor handles, the four exact PR heads, hardened-equivalent wording,
  open-state language, and the VML non-rendering boundary at
  `scripts/test_sprint_workflow.py:4479`. The reviewed body contains the same
  evidence and the intentional pre-1.0 source-impact statement at
  `CHANGELOG.md:7`, `CHANGELOG.md:72`, and `CHANGELOG.md:80`.
- Bindings and WASM: Python metadata follows the stable carrier without
  gaining publication authority, while the rdocx WASM contract checks 0.11.0
  and the PowerPoint WASM contract remains 0.7.0 at
  `.github/workflows/ci.yml:351`. Both target checks are recorded green at
  `.claude/scratch/F-X060-progress.md:100`.
- Packages, legal assets, and supply chain: the exact 22-package dry run,
  archive ceiling, 24-font and legal inventory, absence of duplicated fonts in
  `rdocx-layout`, PowerPoint template, README package checks, and Cargo deny
  result are recorded at `.claude/scratch/F-X060-progress.md:103`.
- Tests and hash: the complete 88-test workflow suite and deterministic notes
  commands are recorded at `.claude/scratch/F-X060-progress.md:41`. Full
  workspace, no-default, docs, WASM, prose, adapter, and 49 of 49 unchanged
  hash evidence are recorded at `.claude/scratch/F-X060-progress.md:87`.
- HLD discipline: exactly the five files listed by the plan at
  `.claude/plans/F-X060-design.md:104` change. They state current preparation
  separately from the last published 0.10.1 family and preserve final release
  approval at `docs/hld/15-build-and-toolchain.md:371`.
- Panics, errors, OOXML, and structure: the diff changes release metadata,
  release documentation, carrier assertions, and test orchestration only. It
  adds no runtime parser, serializer, namespace, schema-order, raw-preservation,
  public type, dependency, module, trait, generic, wrapper, or product error
  path. No publication, tag, push, notification, external-record mutation, or
  sprint-ledger mutation has occurred, as recorded at
  `.claude/scratch/F-X060-progress.md:113`.

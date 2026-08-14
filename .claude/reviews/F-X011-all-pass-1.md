# F-X011, all aspects, pass 1

**Reviewed**: the complete 42-file working diff at
`d02eefee21e795233def1b224d68067a8bdd8e71`, 122 additions and 88 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Focused evidence

- Fresh Cargo metadata reports exactly 15 incubating preparation members at
  0.2.0, with exactly 14 publishable crates and unpublished `rpptx-wasm`.
  Exactly eleven stable-train members remain at 0.6.0, with seven
  publishables. The root contains all fourteen 0.2.0 pins
  (`Cargo.toml:54`, `Cargo.toml:68`), and the lockfile contains exactly the
  fifteen matching entries. The named regression enumerates the fourteen
  crates.io packages, adds the WASM member for the preparation group, and
  asserts versions, pins, descriptions, publication flags, lock entries, and
  stable workspace isolation (`scripts/test_sprint_workflow.py:3102`,
  `scripts/test_sprint_workflow.py:3119`,
  `scripts/test_sprint_workflow.py:3131`,
  `scripts/test_sprint_workflow.py:3144`).
- The 0.2.0 README requirements cover every incubating README that carries a
  version-sensitive install or dependency example
  (`scripts/test_sprint_workflow.py:3154`). A fresh runner invocation compiled
  all 26 Rust examples and validated all 26 distinct README sources and 21
  publishable archive inventories. Its exact local patch contract remains
  metadata-validated before packaging (`scripts/readme_doctests.py:273`,
  `scripts/readme_doctests.py:457`).
- Version-sensitive source and workflow assertions are aligned. The
  `oxml-drawing`, `rpptx-chart`, `rpptx-oxml`, `rpptx-render`, and `rpptx`
  publication tests require 0.2.0
  (`crates/oxml-drawing/src/lib.rs:183`,
  `crates/rpptx-chart/src/lib.rs:15061`,
  `crates/rpptx-oxml/tests/integration.rs:281`,
  `crates/rpptx-render/src/lib.rs:3543`,
  `crates/rpptx/tests/integration.rs:3780`). The publish workflow invokes the
  0.6.0 stable and 0.2.0 incubating metadata regressions before the patched
  dry run (`.github/workflows/publish.yml:23`,
  `.github/workflows/publish.yml:26`). All 38 workflow tests and six focused
  release tests pass, including the WASM family and tag mutations and workflow
  failure propagation.
- `rpptx-wasm` is 0.2.0 but remains `publish = false` and outside workspace
  dependency pins (`scripts/test_sprint_workflow.py:3144`,
  `scripts/test_sprint_workflow.py:3150`). Its local package assertion and CI
  package check require 0.2.0 (`crates/rpptx-wasm/src/lib.rs:467`,
  `.github/workflows/ci.yml:181`). The rdocx WASM assertion also follows the
  shared `oxml-layout` 0.2.0 pin while retaining stable rdocx 0.6.0
  (`crates/rdocx-wasm/src/lib.rs:316`). Both locked WASM target checks pass.
  The progress evidence records passing Node suites and a fresh local
  `@tensorbee/rpptx-wasm@0.2.0` install without publication
  (`.claude/scratch/F-X011-progress.md:12`).
- All 21 current package archives are below 10 MiB. `oxml-layout` contains 20
  TTF files and four legal files, `rdocx-layout` contains no duplicate font or
  legal payload, and `rpptx` contains `assets/default.pptx`. Cargo metadata
  finds no prohibited reverse dependency and only the documented
  `oxml-drawing -> rdocx-oxml` exception. All 28 hash-harness entries remain
  unchanged.
- F-X010 is genuinely complete before F-X011. The delivery record identifies
  the published seven-package 0.6.0 family, successful workflow, exact tag
  peel, and absence of binding, WASM, npm, PyPI, or incubating publication
  (`docs/sprints/AS_BUILT.md:5657`, `docs/sprints/AS_BUILT.md:5663`,
  `docs/sprints/AS_BUILT.md:5669`). Fresh registry checks find all seven stable
  0.6.0 versions. This satisfies the F-X011 dependency
  (`.claude/plans/F-X011-design.md:6`).
- Local and remote `rpptx-v0.2.0`, its GitHub release, and all fourteen
  crates.io 0.2.0 versions are absent. The npm 0.2.0 package and both PyPI
  0.2.0 projects are also absent. No push, tag, upload, or publication
  occurred during this review. `/release` remains the only external authority,
  after a clean committed SHA, full verification, clean sprint review, and
  separate immediate approval (`.claude/plans/F-X011-design.md:34`,
  `.claude/plans/F-X011-design.md:36`).
- The plan's exact HLD impact is 03, 14, and 15
  (`.claude/plans/F-X011-design.md:70`). Those files are intentionally not in
  the implementation diff yet because the canonical lifecycle updates the HLD
  after a clean microscope during `/complete-feature`
  (`.claude/commands/complete-feature.md:29`). The progress checkpoint leaves
  exactly those three updates as the next completion action
  (`.claude/scratch/F-X011-progress.md:20`). This is consistent staging, not an
  unlisted HLD contradiction.
- F-X011 is in progress under `codex`, while F-X009 and F-X010 are complete
  (`docs/sprints/CURRENT_SPRINT.md:24`,
  `docs/sprints/CURRENT_SPRINT.md:25`,
  `docs/sprints/CURRENT_SPRINT.md:26`,
  `.claude/scratch/S39-run.json:16`). Formatting, Python compilation, prose,
  generated-skill sync, and diff hygiene pass.

## Not found

No wrong manifest version, missing pin or lock entry, publication-eligibility
error, stable-family regression, stale README requirement, source assertion
mismatch, workflow ordering defect, local-patch fallback, WASM package
authority leak, dependency-direction change, archive ceiling or asset failure,
hash delta, public API change, panic path, unbounded work, structural
indirection, unauthorized external artifact, HLD scope conflict, prose issue,
or process-state mismatch was found.

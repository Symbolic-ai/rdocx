# F-X008, all aspects, pass 1

**Reviewed**: the complete 18-file working diff from
`fd2778420a3886508865fe1f5ee3d5946c684449`, 273 additions and 76 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Focused evidence

- The approved plan defines one 0.5.0 workspace version, nine workspace pins,
  eleven inherited members, seven stable publishables, two unpublished Python
  projects, and fifteen unchanged incubating manifests
  (`.claude/plans/F-X008-design.md:27`, `.claude/plans/F-X008-design.md:39`).
  The root metadata has the requested workspace version and exact pins
  (`Cargo.toml:33`, `Cargo.toml:55`, `Cargo.toml:69`). The regression enumerates
  all eleven inherited members, all nine pins, the exact seven-package
  publishable set, both Python versions, and all fifteen incubating members
  (`scripts/test_sprint_workflow.py:2871`,
  `scripts/test_sprint_workflow.py:2886`,
  `scripts/test_sprint_workflow.py:2897`,
  `scripts/test_sprint_workflow.py:2954`,
  `scripts/test_sprint_workflow.py:2998`). `cargo metadata --no-deps` confirmed
  11 inherited packages at 0.5.0, exactly seven publishable packages in that
  family, and 15 explicit packages at 0.1.3. The lock regression checks all
  eleven selected entries at 0.5.0 (`scripts/test_sprint_workflow.py:2932`).
- The Python project metadata is 0.5.0 while both Cargo packages remain
  unpublished (`crates/rdocx-py/pyproject.toml:7`,
  `crates/rdocx-py/Cargo.toml:5`, `crates/rpptx-py/pyproject.toml:7`,
  `crates/rpptx-py/Cargo.toml:5`). `rdocx-wasm` inherits 0.5.0 but remains
  `publish = false`, and `rpptx-wasm` remains explicitly 0.1.3 and unpublished
  (`crates/rdocx-wasm/Cargo.toml:4`, `crates/rdocx-wasm/Cargo.toml:13`,
  `crates/rpptx-wasm/Cargo.toml:4`, `crates/rpptx-wasm/Cargo.toml:13`). Both
  `wasm32-unknown-unknown` checks passed. The CI contract packages and locally
  installs both WASM outputs without an npm publication command
  (`.github/workflows/ci.yml:160`, `.github/workflows/ci.yml:180`,
  `scripts/test_sprint_workflow.py:672`).
- The publish workflow runs the stable and incubating metadata checks after the
  hash harness and before the patched workspace dry run
  (`.github/workflows/publish.yml:20`, `.github/workflows/publish.yml:23`,
  `.github/workflows/publish.yml:26`). Its stable predicate contains exactly the
  seven dependency-ordered Cargo publishes and no binding or WASM package
  (`.github/workflows/publish.yml:51`, `.github/workflows/publish.yml:68`). The
  workflow contract rejects an added package, a bypass, or a missing local
  patch (`scripts/test_sprint_workflow.py:3353`,
  `scripts/test_sprint_workflow.py:3370`,
  `scripts/test_sprint_workflow.py:3432`). Reverting the preparation or its
  workflow invocation therefore makes a named pre-publication gate fail.
- All seven stable publishables have manifest-wired READMEs. The runner carries
  the exact seven-package inventory and requires every 0.5 dependency example
  (`scripts/readme_doctests.py:61`, `scripts/readme_doctests.py:71`). It compiled
  all 12 Rust snippets successfully. Fresh 0.5.0 archives were present for all
  seven packages, each contained exactly one README, and their observed sizes
  were 4 KiB through 100 KiB, below the 10 MiB ceiling.
- The HLD impact list is exactly 11, 14, and 15
  (`.claude/plans/F-X008-design.md:86`). Those files own the migration and
  release allowlists, the F-X008 story and its single external gate, and the
  publishing mechanism respectively (`docs/hld/11-migration-plan.md:174`,
  `docs/hld/14-development-backlog.md:1218`,
  `docs/hld/15-build-and-toolchain.md:158`). No changed dependency direction
  requires an additional HLD. `cargo tree --workspace --edges normal` confirmed
  the existing dependency graph, including only the documented
  `oxml-drawing -> rdocx-oxml` exception.
- The preparation does not cross the release boundary. The release command
  requires a clean reviewed SHA and a separate final go or no-go immediately
  before its first external mutation (`.claude/commands/release.md:45`,
  `.claude/commands/release.md:75`, `.claude/commands/release.md:84`). Local and
  remote `v0.5.0` tags were absent, `gh release view v0.5.0` reported no
  release, and registry reads found no 0.5.0 crates.io, npm, or PyPI release.
  The uncommitted preparation is not present on `origin/sprint/s38`.

## Checks run

- `python3 -m unittest scripts/test_sprint_workflow.py`, 37 passed.
- Stable metadata, incubating metadata, and publish preflight focused tests,
  passed.
- `python3 scripts/readme_doctests.py`, 12 examples passed.
- `cargo check --target wasm32-unknown-unknown -p rdocx-wasm -p rpptx-wasm`,
  passed.
- `python3 scripts/hash_harness.py --check`, 28 entries unchanged.
- `cargo metadata --no-deps --format-version 1` and
  `cargo tree --workspace --edges normal --prefix none`, passed inspection.
- `cargo fmt --all --check`, `python3 scripts/prose_check.py`,
  `python3 scripts/sync_agent_skills.py --check`, and `git diff --check`, passed.

## Not found

No incorrect version, pin, lock entry, publication eligibility, preflight
ordering, README inventory, archive boundary, dependency direction, external
mutation, HLD impact omission, approval-boundary bypass, or test-gate weakness
was found.

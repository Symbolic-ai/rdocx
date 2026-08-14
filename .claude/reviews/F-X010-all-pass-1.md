# F-X010, all aspects, pass 1

**Reviewed**: the complete 23-file tracked working diff at
`eb191df17ee33484227e8b1683a112dcfbdc77d8`, 181 additions and 87 deletions,
plus the untracked approved F-X010 design contract
**Verdict**: 1 defect, 0 smells, 1 nitpick

## Defects

### D1, the README archive gate does not enforce its exact local patch set

`scripts/readme_doctests.py:25`
`scripts/readme_doctests.py:288`
`docs/hld/12-testing-strategy.md:443`

The runner now carries the same 21 entries as the release workflow, but it only
passes that independent tuple to Cargo. It never verifies the tuple against the
publishable metadata set or the workflow dry-run set. A focused negative probe
removed the `oxml-core` entry, leaving 20 patches, then ran
`validate_inventory()`. The runner still reported all 26 README sources and all
21 publishable archive inventories valid because the already published 0.1.3
dependency satisfied Cargo resolution. The gate therefore permits registry
fallback while claiming to exercise the exact reviewed local source graph.

Make the exact patch set an asserted or derived contract and add a negative
test that removes an otherwise registry-resolvable patch. The gate must fail
when any of the 21 local source patches is missing, not only when an unpublished
0.6.0 dependency happens to make Cargo fail.

## Smells

None.

## Nitpicks

- `docs/sprints/CURRENT_SPRINT.md:15`, the sprint now contains F-X009, F-X010,
  and F-X011, but this reference still describes HLD 14 as only the exact
  F-X009 acceptance gate. HLD 14 contains all three story contracts
  (`docs/hld/14-development-backlog.md:1238`,
  `docs/hld/14-development-backlog.md:1251`,
  `docs/hld/14-development-backlog.md:1270`).

## Focused evidence

- Cargo metadata reports exactly eleven workspace-version members at 0.6.0,
  with exactly the seven intended crates.io publishables, and fifteen
  incubating members at 0.1.3 with fourteen publishables. The root workspace
  version and all nine stable pins are 0.6.0 (`Cargo.toml:33`,
  `Cargo.toml:55`, `Cargo.toml:69`). The lockfile contains exactly the eleven
  matching 0.6.0 entries, and the named regression enumerates the full stable
  train, pins, publication set, and incubating set
  (`scripts/test_sprint_workflow.py:2871`,
  `scripts/test_sprint_workflow.py:2886`,
  `scripts/test_sprint_workflow.py:2897`,
  `scripts/test_sprint_workflow.py:2906`).
- Both Python project versions are 0.6.0 while their Cargo manifests remain
  unpublished (`crates/rdocx-py/pyproject.toml:7`,
  `crates/rdocx-py/Cargo.toml:5`,
  `crates/rpptx-py/pyproject.toml:7`,
  `crates/rpptx-py/Cargo.toml:5`). `rdocx-wasm` inherits 0.6.0 and remains
  `publish = false` (`crates/rdocx-wasm/Cargo.toml:4`,
  `crates/rdocx-wasm/Cargo.toml:13`). Its source and CI contract literals are
  aligned (`crates/rdocx-wasm/src/lib.rs:318`,
  `.github/workflows/ci.yml:180`). Both locked WASM target checks passed.
- The stable metadata preflight is renamed and invoked before the exact patched
  workflow dry run (`.github/workflows/publish.yml:23`,
  `.github/workflows/publish.yml:26`). The stable and incubating allowlists are
  still disjoint and dependency ordered
  (`.github/workflows/publish.yml:51`, `.github/workflows/publish.yml:68`). All
  37 workflow tests pass, including the exact publication routing and failure
  propagation regressions (`scripts/test_sprint_workflow.py:3325`,
  `scripts/test_sprint_workflow.py:3397`).
- Every stable dependency example requires 0.6, and the named release
  regression owns the same seven README requirements
  (`scripts/test_sprint_workflow.py:2984`). The unmodified positive README run
  passed 26 distinct sources, 26 Rust examples, and all 21 package inventories.
  Every archive contains exactly one README and is below 10 MiB. The current
  `oxml-layout` archive contains 20 TTFs and four legal files,
  `rdocx-layout` contains no font payload, and `rpptx` contains
  `assets/default.pptx`, matching the package boundary
  (`docs/hld/15-build-and-toolchain.md:186`).
- The complete release version, README, Python, WASM, workflow, and HLD changes
  are metadata-only. All 28 deterministic hashes remain unchanged. Formatting,
  prose, generated-skill sync, diff checks, and `cargo deny check` pass. Cargo
  dependency inspection finds only the documented
  `oxml-drawing -> rdocx-oxml` exception and no new cross-family reverse edge
  (`AGENTS.md:49`).
- The exact HLD impact is 11, 12, 14, and 15
  (`.claude/plans/F-X010-design.md:77`). Those four files describe the pending
  0.6.0 train, README archive mechanism, F-X010 acceptance gate, and release
  process (`docs/hld/11-migration-plan.md:132`,
  `docs/hld/12-testing-strategy.md:428`,
  `docs/hld/14-development-backlog.md:1251`,
  `docs/hld/15-build-and-toolchain.md:215`). No other HLD file is modified by
  F-X010, and no changed dependency direction requires another HLD impact.
- F-X010 is in progress under owner `codex`, F-X009 is completed, and F-X011
  remains approved and pending (`docs/sprints/CURRENT_SPRINT.md:23`,
  `docs/sprints/BACKLOG.md:297`, `.claude/scratch/S39-run.json:9`). Local and
  remote `v0.6.0`, its GitHub release, all seven stable crates.io 0.6.0
  versions, the four unpublished train members on crates.io, npm 0.6.0, and
  both PyPI 0.6.0 projects remain absent. No tag, push, registry upload, or
  publication occurred during this review. This is the required prepared state
  before immediate final approval (`docs/hld/15-build-and-toolchain.md:219`).

## Not found

No incorrect stable version, pin, lock entry, publication eligibility,
allowlist order, preflight placement, README version, Python version, WASM
literal, archive ceiling, bundled asset inventory, dependency direction,
unauthorized publication, tag mutation, hash delta, HLD impact omission,
public API change, panic path, new structural indirection, prose violation, or
process-state mismatch was found beyond D1 and the recorded nitpick.

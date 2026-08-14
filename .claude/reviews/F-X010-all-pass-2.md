# F-X010, all aspects, pass 2

**Reviewed**: the complete 23-file tracked working diff at
`eb191df17ee33484227e8b1683a112dcfbdc77d8`, 241 additions and 88 deletions,
plus the untracked approved F-X010 design contract and pass 1 disposition
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, HLD 11 gives conflicting publication instructions for both deprecated shims

`docs/hld/11-migration-plan.md:147`
`docs/hld/11-migration-plan.md:148`
`.claude/plans/F-X010-design.md:33`
`docs/hld/14-development-backlog.md:1253`

HLD 11 says `rdocx-opc` is published in the approved cutover release and then
stops publishing, and gives `rdocx-pdf` the same fate. The cutover release is
the completed v0.5.0 release described in the same HLD
(`docs/hld/11-migration-plan.md:134`). F-X010 nevertheless defines both shims
as members of the exact seven-package v0.6.0 publication set, and the workflow
will publish both (`.github/workflows/publish.yml:54`,
`.github/workflows/publish.yml:62`). The HLD therefore simultaneously requires
and prohibits their next release. Resolve the current-intent contract before
release, either by retaining both shims in future stable trains and removing
the stop instruction, or by excluding them consistently from the v0.6.0 plan,
tests, allowlist, and sprint totals.

## Smells

None.

## Nitpicks

None.

## Pass 1 dispositions

- Pass 1 D1 is resolved. `validate_local_patches()` derives the publishable
  name and repository-relative path identities from Cargo metadata, rejects
  duplicates and any missing or unexpected tuple, and is called by the central
  inventory gate before archive construction
  (`scripts/readme_doctests.py:273`, `scripts/readme_doctests.py:296`,
  `scripts/readme_doctests.py:457`). Every package archive command receives all
  21 validated local patches (`scripts/readme_doctests.py:315`,
  `scripts/readme_doctests.py:324`). A direct comparison found the runner tuple
  and publish workflow tuple equal, with 21 unique identities in each.
- The new mutation removes the otherwise registry-resolvable `oxml-core`
  identity and invokes `validate_inventory()` itself, not only the helper
  (`scripts/test_sprint_workflow.py:3012`,
  `scripts/test_sprint_workflow.py:3025`,
  `scripts/test_sprint_workflow.py:3028`). The focused test passed and proved
  that the central gate now fails rather than accepting registry fallback.
- The pass 1 nitpick is resolved. The sprint reference now names all three HLD
  14 acceptance gates (`docs/sprints/CURRENT_SPRINT.md:15`).

## Focused evidence

- Fresh Cargo metadata reports exactly 26 workspace packages and 21
  publishables. Exactly eleven packages are at 0.6.0, seven of those are
  publishable, and exactly fifteen remain at 0.1.3, fourteen of those are
  publishable. The named regression enumerates the eleven stable members, nine
  pins, exact seven-package allowlist, and all fifteen unchanged incubating
  members (`scripts/test_sprint_workflow.py:2872`,
  `scripts/test_sprint_workflow.py:2887`,
  `scripts/test_sprint_workflow.py:2898`,
  `scripts/test_sprint_workflow.py:2907`). All 38 workflow tests passed.
- The stable metadata preflight is invoked before the exact patched dry run
  (`.github/workflows/publish.yml:23`, `.github/workflows/publish.yml:26`). The
  stable and incubating real allowlists remain disjoint
  (`.github/workflows/publish.yml:51`, `.github/workflows/publish.yml:68`), and
  the five metadata-unpublishable packages remain exactly `oxml-py-support`,
  `rdocx-py`, `rdocx-wasm`, `rpptx-py`, and `rpptx-wasm`.
- A fresh positive README run validated 26 distinct README sources and all 21
  publishable archive inventories, and compiled all 26 Rust examples. The
  runner checks the seven stable README dependency examples at 0.6
  (`scripts/readme_doctests.py:170`), and the stable release regression owns
  the same requirements (`scripts/test_sprint_workflow.py:2985`).
- Both locked WASM target checks passed. The two Python project versions and
  rdocx WASM contract literals remain aligned at 0.6.0 without gaining
  publication authority (`crates/rdocx-py/pyproject.toml:7`,
  `crates/rpptx-py/pyproject.toml:7`,
  `crates/rdocx-wasm/src/lib.rs:318`, `.github/workflows/ci.yml:180`). Cargo
  metadata found no prohibited reverse dependency and only the documented
  `oxml-drawing -> rdocx-oxml` exception.
- All 28 hash-harness entries remain unchanged. Formatting, prose,
  generated-skill sync, and diff checks pass. F-X010 remains running under
  `codex`, while F-X009 is complete and F-X011 is approved but pending
  (`docs/sprints/CURRENT_SPRINT.md:24`,
  `docs/sprints/CURRENT_SPRINT.md:25`,
  `docs/sprints/CURRENT_SPRINT.md:26`).
- No local or remote `v0.6.0` tag or GitHub release exists. All seven intended
  crates.io 0.6.0 versions remain absent, with 0.5.0 still the latest version.
  The npm 0.6.0 package and both PyPI 0.6.0 projects are absent. No release,
  push, tag, upload, or publication occurred during this review. This matches
  the separate immediate-approval boundary
  (`.claude/plans/F-X010-design.md:43`,
  `docs/hld/15-build-and-toolchain.md:223`).

## Not found

No incorrect version, pin, lock entry, publication flag, README version,
Python version, WASM literal, local-patch identity, mutation-path weakness,
archive failure, asset-boundary regression, dependency-direction regression,
hash delta, public API change, panic path, unbounded work, structural
indirection, prose violation, or process-state mismatch was found beyond D1.

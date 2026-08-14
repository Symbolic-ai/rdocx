# F-X010, all aspects, pass 3

**Reviewed**: the complete 23-file tracked working diff at
`eb191df17ee33484227e8b1683a112dcfbdc77d8`, 258 additions and 89 deletions,
plus the untracked approved F-X010 design contract and pass 1 and pass 2
dispositions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Prior-pass dispositions

- Pass 1 D1 remains resolved. The README runner validates its 21 unique local
  patch name and path identities against the complete Cargo metadata
  publishable set before it creates any archive
  (`scripts/readme_doctests.py:273`, `scripts/readme_doctests.py:296`,
  `scripts/readme_doctests.py:457`). The mutation removes the
  registry-resolvable `oxml-core` patch and calls the central inventory gate,
  which rejects it (`scripts/test_sprint_workflow.py:3022`,
  `scripts/test_sprint_workflow.py:3035`,
  `scripts/test_sprint_workflow.py:3038`).
- The pass 1 nitpick remains resolved. The sprint reference names the exact
  F-X009, F-X010, and F-X011 acceptance gates
  (`docs/sprints/CURRENT_SPRINT.md:15`).
- Pass 2 D1 is resolved. HLD 11 now keeps `rdocx-opc` and `rdocx-pdf` in the
  stable release train while the seven-package allowlist is current
  (`docs/hld/11-migration-plan.md:147`,
  `docs/hld/11-migration-plan.md:148`). It also states that both deprecated
  shims continue to publish with each coherent stable train and require an
  explicit redesign before removal (`docs/hld/11-migration-plan.md:164`).
  This matches the exact plan list, HLD 14 gate, and workflow commands
  (`.claude/plans/F-X010-design.md:33`,
  `docs/hld/14-development-backlog.md:1253`,
  `.github/workflows/publish.yml:54`, `.github/workflows/publish.yml:62`).
- The stable metadata regression is sensitive to the prior HLD conflict. It
  rejects the stale `then stop publishing` instruction and requires the new
  positive stable-train contract (`scripts/test_sprint_workflow.py:2963`,
  `scripts/test_sprint_workflow.py:2966`,
  `scripts/test_sprint_workflow.py:2967`). A focused in-memory reversion of the
  HLD table and paragraph made the regression fail on the stale instruction.

## Focused evidence

- All 38 workflow tests pass, including the stable 0.6.0 metadata regression,
  the stale-HLD boundary, exact release routing, publication failure
  propagation, and the missing-local-patch mutation. Cargo metadata reports
  exactly 26 packages, 21 publishables, eleven 0.6.0 stable members with seven
  publishables, and fifteen 0.1.3 incubating members with fourteen
  publishables (`scripts/test_sprint_workflow.py:2872`,
  `scripts/test_sprint_workflow.py:2898`,
  `scripts/test_sprint_workflow.py:2907`).
- The plan and all four listed HLD files consistently describe the pending
  0.6.0 train, exact seven stable publishables, 21 locally patched archive
  gate, unchanged 0.1.3 incubating family, and separate immediate approval
  boundary (`.claude/plans/F-X010-design.md:30`,
  `docs/hld/11-migration-plan.md:195`,
  `docs/hld/12-testing-strategy.md:442`,
  `docs/hld/14-development-backlog.md:1251`,
  `docs/hld/15-build-and-toolchain.md:215`). No other HLD file is modified.
- The publish metadata preflight still precedes the exact 21-package patched
  dry run (`.github/workflows/publish.yml:23`,
  `.github/workflows/publish.yml:26`). The README runner and workflow patch
  sets contain the same 21 unique name and path identities, and the runner
  validated them against current metadata. A fresh positive run compiled all
  26 Rust examples and validated all 26 README sources and 21 package
  archives (`scripts/readme_doctests.py:315`,
  `scripts/readme_doctests.py:470`).
- Both WASM target checks pass. The 11 inherited versions, nine pins, lock
  entries, two Python project versions, rdocx WASM literals, stable README
  requirements, and publication flags remain aligned. Cargo metadata finds no
  prohibited reverse dependency and only the documented
  `oxml-drawing -> rdocx-oxml` exception.
- All 28 hash-harness entries remain unchanged. Formatting and diff checks
  pass before this review record. F-X010 remains running, F-X009 remains
  complete, and F-X011 remains approved and pending
  (`docs/sprints/CURRENT_SPRINT.md:24`,
  `docs/sprints/CURRENT_SPRINT.md:25`,
  `docs/sprints/CURRENT_SPRINT.md:26`).
- No local or remote `v0.6.0` tag or GitHub release exists, and all seven
  crates.io 0.6.0 versions remain absent. No push, tag, upload, publication, or
  sprint-state mutation occurred during this review. Release still requires
  clean full verification, a clean sprint review at one exact SHA, and a new
  immediate approval (`.claude/plans/F-X010-design.md:43`,
  `docs/hld/15-build-and-toolchain.md:223`).

## Not found

No correctness defect, contract conflict, stale shim-publication instruction,
version or lock mismatch, publication-eligibility error, README or archive
regression, local-patch fallback, dependency-direction change, public API
change, panic path, unbounded work, structural violation, hash delta,
unauthorized release artifact, HLD impact omission, prose issue, or
process-state mismatch was found.

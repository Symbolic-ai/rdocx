# S44 sprint review, pass 2

**Reviewed**: `sprint/s44` against `327b05d`, 26 files, 3,193 changed lines,
crates: none
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Resolved pass-1 findings

### B1, resolved, the release-regression job provisions cargo-release

`.github/workflows/ci.yml:311`

The job now has exactly three ordered steps: checkout, installation of exact
cargo-release 1.1.3 with `--locked`, then the complete
`scripts.test_sprint_workflow` module. The contract at
`scripts/test_sprint_workflow.py:4389` asserts that exact version, locked
installation, step count, step order, whole-module command, and ordinary
failure propagation. Its mutation cases at
`scripts/test_sprint_workflow.py:4438` reject removing the installation or
moving it after the regressions. This establishes the external command used by
the stable-family loop at `scripts/test_sprint_workflow.py:4021` before the
module reaches it. The local executable reported cargo-release 1.1.3 and the
complete 56-test module passed.

### S1, resolved, the F-X028 completion evidence names real tests and sections

`docs/sprints/AS_BUILT.md:6820`

The entry now cites the actual `Release process` HLD heading and the delivered
test symbols
`test_agent_facing_repository_claims_resolve_against_the_workspace` and
`test_agent_facing_claim_contract_rejects_stale_mutations`. Both symbols
resolve at `scripts/test_sprint_workflow.py:5029` and
`scripts/test_sprint_workflow.py:5034`, and both passed in the complete module.

### N1, resolved, the sprint contract counts its four story definitions

`docs/sprints/CURRENT_SPRINT.md:19`

The spec-reference text now says four story definitions, matching F-X026
through F-X029 in the wave.

## Milestone gate

Cross-cutting has no single end-of-milestone gate in the backlog. The four
story gates and the sprint definition of done are therefore the operative gate,
and they hold on this reviewed tree.

F-X026's named release-regression job now has its pinned locked prerequisite,
and the full 56-test module passed. F-X027's real golden harness matched all 7
page-one pixel buffers at 150 DPI under pdftoppm 26.01.0. F-X028's claim
regressions passed with the corrected completion evidence. F-X029's path-filter
and aggregate-gate contracts passed as part of the same module. The independent
hash check reported 49 matching entries, the prose check reported zero
violations, and all 25 generated skill adapters were in sync.

## Not found

- **interaction**. F-X026 through F-X029 compose without a conflicting
  condition, ordering rule, setup dependency, or path route. The remediated
  cargo-release step precedes the whole module, and the golden step remains
  after the pinned Poppler setup and workspace suite.
- **duplication**. No duplicate production or test helper was introduced.
- **layering**. No crate file or manifest changed, so no `oxml-*` dependency
  edge changed.
- **harness**. Neither hash manifest changed. The independent hash check stayed
  at 49 of 49, consistent with all four AS_BUILT entries.
- **gate**. No gate defect remains. The release dependency and its ordering are
  now asserted, and the other sprint gate contracts passed.
- **docs**. The pass-1 record corrections are accurate. The CI matrix,
  golden-PNG description, version trains, font ownership, path-filter behavior,
  and completion ledger match the integrated workflow.
- **deps**. No Cargo dependency changed. cargo-release is pinned to exact 1.1.3
  with its locked graph, and the path-filter action remains pinned to immutable
  commit `ceb8a2b8f2d89434be7ff52d3de7ec3738c5cc9d`.
- **surface**. No public Rust, Python, WASM, or command-line API changed.

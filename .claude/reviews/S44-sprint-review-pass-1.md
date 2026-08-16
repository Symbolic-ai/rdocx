# S44 sprint review, pass 1

**Reviewed**: `sprint/s44` against `327b05d`, 25 files, 3,081 changed lines,
crates: none
**Verdict**: 1 blocking, 1 should-fix, 1 nice-to-have

## Blocking

### B1, the release-regression job does not provision cargo-release

`.github/workflows/ci.yml:311`

The new unconditional job checks out the repository and immediately runs the
whole Python module, but the module invokes `cargo release config` when
`CARGO_RELEASE_BIN` is unset at `scripts/test_sprint_workflow.py:4021`. Nothing
in the job installs or selects `cargo-release`, so a fresh hosted runner has no
workflow-established executable for those eight stable-family iterations. The
job can fail on its missing third-party prerequisite before it evaluates a
version carrier, which leaves F-X026's named CI gate unproved and can make every
pull request red. The fix must provision a reviewed cargo-release version before
the module, or remove that runtime dependency from the regression. The workflow
contract must also reject removal or reordering of whichever setup the fix
chooses.

## Should-fix

### S1, the F-X028 completion record names tests and an HLD section that do not exist

`docs/sprints/AS_BUILT.md:6820`

The entry cites an "agent-facing repository claims" section in
`docs/hld/15-build-and-toolchain.md`, but that file has no such heading. It also
names `test_agent_facing_repository_claims_match_the_tree` and
`test_agent_facing_repository_claims_reject_drift` at
`docs/sprints/AS_BUILT.md:6824`, while the delivered tests are
`test_agent_facing_repository_claims_resolve_against_the_workspace` and
`test_agent_facing_claim_contract_rejects_stale_mutations` at
`scripts/test_sprint_workflow.py:5009` and
`scripts/test_sprint_workflow.py:5014`. Correct the append-only completion
record so future sessions can resolve its evidence and so its touched sections
match the actual HLD headings.

## Nice-to-have

### N1, the sprint contract counts three story definitions for a four-story wave

`docs/sprints/CURRENT_SPRINT.md:19`

The spec-reference text says the backlog citation covers three story
definitions, while the wave immediately below contains F-X026 through F-X029.
Change "three" to "four" when the sprint record is next amended.

## Milestone gate

Cross-cutting has no single end-of-milestone gate in the backlog. The four
story gates and the sprint definition of done are therefore the operative gate.

F-X026 requires that "the module runs in a named CI job" and that a stale
version literal fails it at `docs/hld/14-development-backlog.md:2045`. That gate
does not yet hold because B1 leaves the module's cargo-release prerequisite
unestablished on the hosted runner.

The other gates hold on the reviewed tree. The golden contract regression and
the real harness both passed, with 7 of 7 page-one buffers matching at 150 DPI
under pdftoppm 26.01.0. Both actual agent-facing claim regressions passed. All
three path-routing regressions passed, including must-trigger and
must-not-trigger coverage, the documentation-only route, scheduled supply-chain
selection, and aggregate failure handling. `python3 scripts/hash_harness.py
--check` independently reported 49 matching entries. `prose_check.py` reported
zero violations and the generated-skill check reported all 25 adapters in sync.

## Not found

- **interaction**. Other than B1, F-X026 through F-X029 compose without a
  conflicting condition, ordering rule, or path route. The golden step remains
  after the pinned Poppler setup and workspace suite, and the filter for its
  containing job covers `crates/**` and `scripts/**`.
- **duplication**. No duplicate production or test helper was introduced.
- **layering**. No crate file or manifest changed, so no `oxml-*` dependency
  edge changed.
- **harness**. Neither hash manifest changed. The independent hash check stayed
  at 49 of 49, consistent with all four AS_BUILT entries.
- **gate**. No further gate defect was found beyond B1.
- **docs**. No further contradiction was found beyond S1 and N1. The CI matrix,
  golden-PNG description, version trains, font ownership, and path-filter
  behavior match the integrated workflow.
- **deps**. No Cargo dependency changed. The new path-filter action has a named
  consumer and is pinned to immutable commit
  `ceb8a2b8f2d89434be7ff52d3de7ec3738c5cc9d`.
- **surface**. No public Rust, Python, WASM, or command-line API changed.

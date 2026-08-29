# F-X067, working, pass 1

**Reviewed**: complete uncommitted working diff against
`0541c5461f6550da87fd26e8ed21c55c5f7d19d6`, 6 files with 83 insertions and
26 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the Word fidelity job contains one direct `cargo fetch
  --locked` step at `.github/workflows/ci.yml:434`. It follows the exact pinned
  Rust cache at `.github/workflows/ci.yml:433` and precedes tool installation,
  corpus fetching, and the harness at `.github/workflows/ci.yml:436` and
  `.github/workflows/ci.yml:447`.
- Contract: the shared assertion fixes the cache at step two, the dependency
  prime at step three, and the exact operative command cardinality at
  `scripts/test_sprint_workflow.py:623`. It also orders the prime before the
  corpus and harness at `scripts/test_sprint_workflow.py:661`.
- Negative coverage: the existing workflow test rejects missing, unlocked,
  duplicated, post-harness, and wrong-job priming at
  `scripts/test_sprint_workflow.py:687`. Each mutation is required to fail the
  same complete Word fidelity contract at
  `scripts/test_sprint_workflow.py:724`.
- Offline preservation: the unchanged acceptor command retains both `--locked`
  and `--offline` at `scripts/docx_ssim_harness.py:41`. Its existing regression
  protects both flags at `scripts/docx_ssim_harness.py:569`.
- Oracle and evidence contract: the job still runs the pinned five-document
  harness and retains both required files with failure on absent output at
  `.github/workflows/ci.yml:447` and `.github/workflows/ci.yml:453`. The
  contribution SHA, hosted run and job, nonempty artifact, and deferred
  integrated-hosted rider are recorded at
  `docs/hld/12-testing-strategy.md:1185`.
- HLD discipline: the current testing contract is stated at
  `docs/hld/12-testing-strategy.md:1158`, the current F-X067 acceptance remains
  in `docs/hld/14-development-backlog.md:3405`, and the build boundary is
  described at `docs/hld/15-build-and-toolchain.md:610`. These are exactly the
  three files listed by the approved plan at
  `.claude/plans/F-X067-design.md:77`.
- Panics and errors: the production diff is declarative workflow YAML and adds
  no parser, indexing, arithmetic, unwrap, exception-swallowing condition, or
  product error path. The complete Word job continues to reject
  `continue-on-error` at `scripts/test_sprint_workflow.py:660`.
- OOXML and public surface: no product, parser, serializer, schema-order,
  namespace, package dependency, public type, binding, or generated-output code
  is changed. The approved plan records that routing boundary at
  `.claude/plans/F-X067-design.md:85`.
- Tests and verification: the focused two tests, complete 86-test workflow
  suite, pinned Word oracle, full verification, 49 of 49 unchanged hash result,
  package dry-run, archive ceiling, and supply-chain result are recorded at
  `.claude/scratch/F-X067-progress.md:20`.
- Structure: the implementation adds one named workflow step and extends the
  existing workflow-test helper and test. It adds no action, dependency,
  script, module, test binary, trait, generic, wrapper, or indirection at
  `.github/workflows/ci.yml:434` and `scripts/test_sprint_workflow.py:610`.

# F-X012, all aspects, pass 5

**Reviewed**: the six-file post-pass-3 amendment at
`b27d067be51d54e149d38369bab530f01c3434b4`, 167 additions and 9 deletions,
then the complete twelve-file F-X012 delta from
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 1,052 additions and 35 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the stack-scope regression still accepts an equivalent global YAML key

`scripts/test_sprint_workflow.py:198`
`scripts/test_sprint_workflow.py:537`
`scripts/test_sprint_workflow.py:543`
`.claude/plans/F-X012-design.md:102`
`.claude/plans/F-X012-design.md:104`
`docs/hld/15-build-and-toolchain.md:363`

The new workflow-wide guard counts only the literal text `RUST_MIN_STACK:`.
YAML permits a mapping key to be quoted, so adding
`"RUST_MIN_STACK": "8388608"` to the workflow-global `env` widens the stack
budget to every job but does not increase that literal count. A focused
in-memory mutation made exactly that change and
`assert_workspace_oracle_environment_contract()` accepted it. The committed
mutation covers only the unquoted spelling. Count parsed or normalized
operative environment keys, then require that the only two occurrences belong
to the two named workspace-suite steps. Add the quoted-key mutation to keep the
exclusive scope mutation-sensitive.

## Smells

None.

## Nitpicks

None.

## Pass 4 finding dispositions

- Pass 4 D1 is fixed. Both Test and MSRV apply the shared successful
  short-circuit rejection to the full workspace step, and each job independently
  mutates an `exit 0` before Cargo
  (`scripts/test_sprint_workflow.py:181`,
  `scripts/test_sprint_workflow.py:195`,
  `scripts/test_sprint_workflow.py:544`,
  `scripts/test_sprint_workflow.py:582`).
- Pass 4 D2 is only partially fixed. The current unquoted global-stack mutation
  is rejected and the positive workflow has exactly the intended two local
  settings, but D1 shows that the raw-text count does not enforce the promised
  YAML key scope (`scripts/test_sprint_workflow.py:198`,
  `scripts/test_sprint_workflow.py:538`,
  `.github/workflows/ci.yml:45`, `.github/workflows/ci.yml:396`).

## Focused evidence

- Test and MSRV each use the exact official setup action commit, request uv
  0.10.2, disable the action cache, and place the runner-temporary uv cache and
  8 MiB stack only on the named workspace-suite step
  (`.github/workflows/ci.yml:26`, `.github/workflows/ci.yml:30`,
  `.github/workflows/ci.yml:42`, `.github/workflows/ci.yml:48`,
  `.github/workflows/ci.yml:377`, `.github/workflows/ci.yml:381`,
  `.github/workflows/ci.yml:393`, `.github/workflows/ci.yml:399`). The present
  workflow therefore has the intended runtime scope despite D1's regression
  bypass.
- The mutation matrix independently changes the action, uv version, cache,
  local stack, and successful exit in both jobs, followed by the separate
  unquoted global-stack mutation
  (`scripts/test_sprint_workflow.py:537`,
  `scripts/test_sprint_workflow.py:544`,
  `scripts/test_sprint_workflow.py:549`,
  `scripts/test_sprint_workflow.py:559`,
  `scripts/test_sprint_workflow.py:566`,
  `scripts/test_sprint_workflow.py:575`,
  `scripts/test_sprint_workflow.py:582`,
  `scripts/test_sprint_workflow.py:593`). All intended mutations are routed
  through the same central validation helper.
- The four focused workflow tests pass, followed by all 44 workflow tests. Both
  Python files compile. The hash harness remains 28 of 28. Prose, generated-skill
  synchronization, and diff hygiene pass. The progress record also reports the
  complete `rpptx` suite under the exact environment with 19 unit and 86
  integration tests passing and 7 ignored
  (`.claude/scratch/F-X012-progress.md:92`,
  `.claude/scratch/F-X012-progress.md:97`,
  `.claude/scratch/F-X012-progress.md:103`).
- The amendment changes exactly the approved plan, workflow, existing workflow
  regression file, and HLD12, HLD14, and HLD15. The plan's HLD impact list is
  exact (`.claude/plans/F-X012-design.md:88`,
  `.claude/plans/F-X012-design.md:92`). HLD12 owns the two job commands, HLD14
  owns the story gate, and HLD15 owns the exact runtime mechanism
  (`docs/hld/12-testing-strategy.md:458`,
  `docs/hld/12-testing-strategy.md:468`,
  `docs/hld/14-development-backlog.md:1300`,
  `docs/hld/14-development-backlog.md:1308`,
  `docs/hld/15-build-and-toolchain.md:358`,
  `docs/hld/15-build-and-toolchain.md:364`). Their current-state descriptions
  agree.

## Complete feature evidence

- Pass 3's clean findings remain valid for the exact Poppler source and
  checksum, bounded download and streaming extraction, safe archive handling,
  empty-prefix provenance, three runtime identities, four unconditional
  consumers, and Binaryen checksum and identity
  (`.claude/reviews/F-X012-all-pass-3.md:22`,
  `.claude/reviews/F-X012-all-pass-3.md:48`,
  `.claude/reviews/F-X012-all-pass-3.md:64`,
  `.claude/reviews/F-X012-all-pass-3.md:86`). The amendment does not touch the
  installer or the pre-existing consumer and WASM corrections.
- The installer still enforces its checksum and three resource ceilings before
  bounded extraction and construction, rejects a populated destination, limits
  build parallelism to four, verifies every copied tool, and exposes the path
  only after verification (`scripts/install_pinned_poppler.py:18`,
  `scripts/install_pinned_poppler.py:39`,
  `scripts/install_pinned_poppler.py:52`,
  `scripts/install_pinned_poppler.py:60`,
  `scripts/install_pinned_poppler.py:118`,
  `scripts/install_pinned_poppler.py:161`,
  `scripts/install_pinned_poppler.py:175`,
  `scripts/install_pinned_poppler.py:179`).
- The complete delta contains no crate source, manifest, lockfile, public API,
  dependency, package version, publication workflow, rendering baseline, or
  hash-baseline change. F-X012 remains the sole in-progress S40 story and the
  ledgers agree on 165 done, one in progress, and zero pending
  (`docs/sprints/CURRENT_SPRINT.md:23`, `docs/sprints/BACKLOG.md:32`,
  `docs/sprints/BACKLOG.md:33`, `docs/sprints/BACKLOG.md:299`).

## Not found

No present workflow stack leak, successful Test or MSRV short circuit, wrong uv
action or version, enabled action cache, shared active uv cache, incorrect local
stack value, wrong job ordering, installer correctness or resource regression,
Poppler consumer regression, Binaryen regression, product or public API change,
crate or dependency change, package or publication change, rendering or hash
delta, unlisted HLD edit, HLD contradiction, sprint-state mismatch, structural
indirection, panic, prose violation, smell, or nitpick was found beyond D1.

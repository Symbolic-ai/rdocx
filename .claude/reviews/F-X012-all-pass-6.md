# F-X012, all aspects, pass 6

**Reviewed**: the six-file post-pass-3 amendment at
`b27d067be51d54e149d38369bab530f01c3434b4`, 183 additions and 9 deletions,
then the complete twelve-file F-X012 delta from
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 1,068 additions and 35 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, normalized stack-key counting misses whitespace before the YAML colon

`scripts/test_sprint_workflow.py:56`
`scripts/test_sprint_workflow.py:62`
`scripts/test_sprint_workflow.py:209`
`scripts/test_sprint_workflow.py:548`
`scripts/test_sprint_workflow.py:559`
`.claude/plans/F-X012-design.md:102`
`.claude/plans/F-X012-design.md:104`
`docs/hld/15-build-and-toolchain.md:363`

The new helper strips quote characters from the candidate key but does not
strip whitespace between that key and the colon. YAML treats both
`RUST_MIN_STACK : "8388608"` and
`"RUST_MIN_STACK" : "8388608"` as the same `RUST_MIN_STACK` mapping key. A
focused in-memory mutation added each form to the workflow-global `env`.
`assert_workspace_oracle_environment_contract()` accepted both and still
reported only two keys, even though either form widens the stack budget to
every job. The committed mutations cover unquoted and quoted keys only when
the colon immediately follows the key. Normalize surrounding key whitespace
before removing optional quotes, and add at least one spaced-colon mutation to
make the exclusive scope contract sensitive to ordinary equivalent YAML.

## Smells

None.

## Nitpicks

None.

## Pass 5 finding disposition

- Pass 5 D1 is partially fixed. The helper now recognizes the exact unquoted,
  double-quoted, and single-quoted key spellings, and the regression rejects
  separate global unquoted and double-quoted mutations
  (`scripts/test_sprint_workflow.py:56`,
  `scripts/test_sprint_workflow.py:62`,
  `scripts/test_sprint_workflow.py:549`,
  `scripts/test_sprint_workflow.py:554`). D1 shows that the same normalization
  does not yet cover valid whitespace before the mapping colon.

## Focused evidence

- Test and MSRV each retain exactly one pinned official setup action, exact uv
  0.10.2, disabled action caching, runner-temporary uv caching, and the 8 MiB
  stack on the named workspace-suite step
  (`.github/workflows/ci.yml:26`, `.github/workflows/ci.yml:30`,
  `.github/workflows/ci.yml:42`, `.github/workflows/ci.yml:48`,
  `.github/workflows/ci.yml:377`, `.github/workflows/ci.yml:381`,
  `.github/workflows/ci.yml:393`, `.github/workflows/ci.yml:399`). The present
  workflow has no stack-scope leak. D1 is a regression-gate bypass.
- The central helper requires the exact action input map, exact local
  environment map, Cargo command, action ordering, failure propagation, and no
  successful short circuit in both jobs
  (`scripts/test_sprint_workflow.py:168`,
  `scripts/test_sprint_workflow.py:182`,
  `scripts/test_sprint_workflow.py:192`,
  `scripts/test_sprint_workflow.py:205`,
  `scripts/test_sprint_workflow.py:208`). The mutation matrix independently
  changes action, version, cache, local stack, and exit behavior for Test and
  MSRV, and routes every mutation through that helper
  (`scripts/test_sprint_workflow.py:560`,
  `scripts/test_sprint_workflow.py:565`,
  `scripts/test_sprint_workflow.py:575`,
  `scripts/test_sprint_workflow.py:582`,
  `scripts/test_sprint_workflow.py:591`,
  `scripts/test_sprint_workflow.py:598`,
  `scripts/test_sprint_workflow.py:609`).
- The four focused workflow tests pass, followed by all 44 workflow tests. Both
  Python files compile. The hash harness remains 28 of 28. Prose, generated-skill
  synchronization, and diff hygiene pass. The progress record reports the full
  `rpptx` suite under the exact environment with 19 unit and 86 integration
  tests passing and 7 ignored
  (`.claude/scratch/F-X012-progress.md:92`,
  `.claude/scratch/F-X012-progress.md:97`,
  `.claude/scratch/F-X012-progress.md:106`).
- The amendment edits exactly the approved plan, CI workflow, existing workflow
  regression file, and HLD12, HLD14, and HLD15. The HLD impact list is exact
  (`.claude/plans/F-X012-design.md:88`,
  `.claude/plans/F-X012-design.md:92`). HLD12 describes both job commands,
  HLD14 owns the story gate, and HLD15 owns the exact action, cache, and stack
  mechanism (`docs/hld/12-testing-strategy.md:458`,
  `docs/hld/12-testing-strategy.md:468`,
  `docs/hld/14-development-backlog.md:1300`,
  `docs/hld/14-development-backlog.md:1308`,
  `docs/hld/15-build-and-toolchain.md:358`,
  `docs/hld/15-build-and-toolchain.md:364`). The current-state documents agree.

## Complete feature evidence

- Pass 3's clean dispositions remain valid for the exact Poppler source and
  checksum, bounded download and streaming extraction, safe archive handling,
  empty-prefix provenance, all three runtime identities, all four unconditional
  consumers, and the Binaryen checksum and identity
  (`.claude/reviews/F-X012-all-pass-3.md:22`,
  `.claude/reviews/F-X012-all-pass-3.md:48`,
  `.claude/reviews/F-X012-all-pass-3.md:64`,
  `.claude/reviews/F-X012-all-pass-3.md:86`). The pass-6 remediation touches
  only the existing workflow regression file.
- The installer still verifies the reviewed SHA-256, caps the compressed
  download, member count, and expanded bytes, streams extraction, rejects
  unsafe members and populated prefixes, bounds build parallelism to four,
  verifies every requested tool, and exposes the path only after success
  (`scripts/install_pinned_poppler.py:18`,
  `scripts/install_pinned_poppler.py:39`,
  `scripts/install_pinned_poppler.py:52`,
  `scripts/install_pinned_poppler.py:60`,
  `scripts/install_pinned_poppler.py:70`,
  `scripts/install_pinned_poppler.py:118`,
  `scripts/install_pinned_poppler.py:161`,
  `scripts/install_pinned_poppler.py:178`).
- The complete delta contains no crate source, manifest, lockfile, public API,
  dependency, package version, publication workflow, rendering baseline, or
  hash-baseline change. F-X012 remains the sole in-progress S40 story, and the
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

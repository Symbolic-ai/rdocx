# F-X012, all aspects, pass 7

**Reviewed**: the six-file post-pass-3 amendment at
`b27d067be51d54e149d38369bab530f01c3434b4`, 193 additions and 9 deletions,
then the complete twelve-file F-X012 delta from
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 1,078 additions and 35 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Prior finding dispositions

- Pass 4 D1 remains fixed. Both full workspace steps reject successful
  short circuits, and Test and MSRV each have an independent `exit 0` mutation
  (`scripts/test_sprint_workflow.py:192`,
  `scripts/test_sprint_workflow.py:206`,
  `scripts/test_sprint_workflow.py:570`,
  `scripts/test_sprint_workflow.py:608`).
- Pass 4 D2 and pass 5 D1 remain fixed. The central helper requires exactly two
  normalized `RUST_MIN_STACK` mapping keys in the workflow, while both named
  steps require the exact local key and value
  (`scripts/test_sprint_workflow.py:56`,
  `scripts/test_sprint_workflow.py:62`,
  `scripts/test_sprint_workflow.py:198`,
  `scripts/test_sprint_workflow.py:209`). Separate unquoted and quoted global
  mutations fail (`scripts/test_sprint_workflow.py:549`,
  `scripts/test_sprint_workflow.py:554`).
- Pass 6 D1 is fixed. Candidate-key whitespace is stripped before and after
  optional quote removal, and independent spaced and quoted-spaced global
  mutations now fail (`scripts/test_sprint_workflow.py:62`,
  `scripts/test_sprint_workflow.py:559`,
  `scripts/test_sprint_workflow.py:564`). An independent probe also confirmed
  rejection of all four committed forms and a single-quoted spaced form, with
  three normalized occurrences found in every mutated workflow.

## Focused evidence

- Test and MSRV each retain exactly one reviewed official setup action, exact
  uv 0.10.2, disabled action caching, a runner-temporary uv cache, and the 8 MiB
  stack bound only to the named workspace-suite step
  (`.github/workflows/ci.yml:26`, `.github/workflows/ci.yml:30`,
  `.github/workflows/ci.yml:42`, `.github/workflows/ci.yml:48`,
  `.github/workflows/ci.yml:377`, `.github/workflows/ci.yml:381`,
  `.github/workflows/ci.yml:393`, `.github/workflows/ci.yml:399`). The setup
  precedes the suite in both jobs, and no other current workflow location sets
  either runtime variable.
- The central validation path checks the exact action and input map, exact
  environment map, Cargo command, ordering, failure propagation, successful
  short circuits, and workflow-wide stack scope for both jobs
  (`scripts/test_sprint_workflow.py:168`,
  `scripts/test_sprint_workflow.py:182`,
  `scripts/test_sprint_workflow.py:192`,
  `scripts/test_sprint_workflow.py:205`,
  `scripts/test_sprint_workflow.py:208`,
  `scripts/test_sprint_workflow.py:209`). The regression independently mutates
  action, version, cache, local stack, and exit behavior in Test and MSRV, then
  sends every mutation through that same path
  (`scripts/test_sprint_workflow.py:570`,
  `scripts/test_sprint_workflow.py:575`,
  `scripts/test_sprint_workflow.py:585`,
  `scripts/test_sprint_workflow.py:592`,
  `scripts/test_sprint_workflow.py:601`,
  `scripts/test_sprint_workflow.py:608`,
  `scripts/test_sprint_workflow.py:619`).
- The four focused workflow tests pass, followed by all 44 workflow tests. Both
  Python files compile. The hash harness remains 28 of 28. Prose, generated-skill
  synchronization, and diff hygiene pass. The progress record also reports the
  full `rpptx` suite under the exact runtime with 19 unit and 86 integration
  tests passing and 7 ignored
  (`.claude/scratch/F-X012-progress.md:92`,
  `.claude/scratch/F-X012-progress.md:97`,
  `.claude/scratch/F-X012-progress.md:108`,
  `.claude/scratch/F-X012-progress.md:111`).

## Complete feature evidence

- The approved plan requires the exact shared Poppler installer, all four
  consumer jobs, official Binaryen identity, pinned uv runtime, unchanged
  rendering baselines, and unchanged hashes
  (`.claude/plans/F-X012-design.md:35`,
  `.claude/plans/F-X012-design.md:45`,
  `.claude/plans/F-X012-design.md:48`,
  `.claude/plans/F-X012-design.md:52`,
  `.claude/plans/F-X012-design.md:84`,
  `.claude/plans/F-X012-design.md:106`). The complete implementation remains
  within that boundary.
- Pass 3's clean dispositions remain valid for the exact Poppler source and
  checksum, bounded download and streaming extraction, safe archive handling,
  empty-prefix provenance, all three runtime identities, four unconditional
  consumers, and Binaryen checksum and identity
  (`.claude/reviews/F-X012-all-pass-3.md:22`,
  `.claude/reviews/F-X012-all-pass-3.md:48`,
  `.claude/reviews/F-X012-all-pass-3.md:64`,
  `.claude/reviews/F-X012-all-pass-3.md:86`). Passes 4 through 7 change only
  the plan, workflow, existing regression file, and approved HLD files.
- HLD12 describes the exact Test and MSRV commands and pinned oracle mechanism,
  HLD14 owns the story test gate, and HLD15 owns the exact Poppler, uv, cache,
  and stack mechanism (`docs/hld/12-testing-strategy.md:412`,
  `docs/hld/12-testing-strategy.md:458`,
  `docs/hld/12-testing-strategy.md:468`,
  `docs/hld/12-testing-strategy.md:478`,
  `docs/hld/14-development-backlog.md:1290`,
  `docs/hld/14-development-backlog.md:1308`,
  `docs/hld/15-build-and-toolchain.md:345`,
  `docs/hld/15-build-and-toolchain.md:358`,
  `docs/hld/15-build-and-toolchain.md:364`). The plan lists exactly HLD12,
  HLD14, and HLD15 (`.claude/plans/F-X012-design.md:88`,
  `.claude/plans/F-X012-design.md:92`). Their current-state contracts agree.
- The complete delta contains no crate source, manifest, lockfile, public API,
  dependency, package version, publication workflow, rendering baseline, or
  hash-baseline change. F-X012 remains the sole in-progress S40 story, and the
  ledgers agree on 165 done, one in progress, and zero pending
  (`docs/sprints/CURRENT_SPRINT.md:23`, `docs/sprints/BACKLOG.md:32`,
  `docs/sprints/BACKLOG.md:33`, `docs/sprints/BACKLOG.md:299`). The remaining
  hosted green run is an explicit completion action, not an implementation
  defect (`.claude/plans/F-X012-design.md:82`,
  `.claude/scratch/F-X012-progress.md:117`).

## Not found

No YAML scalar-key normalization bypass among unquoted, quoted, spaced,
quoted-spaced, or single-quoted-spaced forms, stack-scope leak, successful Test
or MSRV short circuit, wrong uv action or version, enabled action cache, shared
active uv cache, incorrect stack value, wrong job ordering, installer
correctness or resource regression, Poppler consumer regression, Binaryen
regression, product or public API change, crate or dependency change, package
or publication change, rendering or hash delta, unlisted HLD edit, HLD
contradiction, sprint-state mismatch, structural indirection, panic, prose
violation, smell, or nitpick was found.

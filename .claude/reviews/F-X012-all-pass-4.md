# F-X012, all aspects, pass 4

**Reviewed**: the six-file post-pass-3 amendment at
`b27d067be51d54e149d38369bab530f01c3434b4`, 123 additions and 9 deletions,
then the complete twelve-file F-X012 delta from
`a75e2b906eb632d8543ebde9db6922bfda653d44`, 1,008 additions and 35 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the runtime regression accepts a successfully skipped workspace suite

`scripts/test_sprint_workflow.py:181`
`scripts/test_sprint_workflow.py:194`
`scripts/test_sprint_workflow.py:196`
`.claude/plans/F-X012-design.md:53`
`.claude/plans/F-X012-design.md:79`

The helper proves that the named step contains the Cargo command text and has
the exact environment, but it never applies the existing successful
short-circuit check to the run body. A focused mutation inserted `exit 0` before
`cargo test` in Test. `assert_workspace_oracle_environment_contract()` still
accepted the workflow, even though the step would succeed without running the
suite under either the uv cache or stack environment. Apply
`assert_no_success_short_circuit()` to both workspace-suite run bodies and add
the mutation for Test and MSRV.

### D2, the regression does not enforce the exclusive stack scope

`scripts/test_sprint_workflow.py:157`
`scripts/test_sprint_workflow.py:191`
`scripts/test_sprint_workflow.py:535`
`scripts/test_sprint_workflow.py:548`
`.claude/plans/F-X012-design.md:102`
`docs/hld/15-build-and-toolchain.md:363`

The helper requires the two local `RUST_MIN_STACK` entries but never rejects
the same variable outside those steps. A focused mutation added
`RUST_MIN_STACK=8388608` to the workflow-global environment while leaving the
two local entries intact. The new regression still passed, even though that
widens the stack budget to every job and contradicts the plan and HLD promise
that it is bound only to the two corpus-heavy suites. Assert exactly two
operative occurrences, both owned by the named test steps, and add a
scope-widening mutation.

## Smells

None.

## Nitpicks

None.

## Amendment evidence

- The current workflow correctly pins official
  `astral-sh/setup-uv` commit
  `20cfd1bf945f4377ade1205e4dbc17946fc9a30d` in Test and MSRV, requests exact
  uv 0.10.2, and disables the action cache
  (`.github/workflows/ci.yml:26`, `.github/workflows/ci.yml:30`,
  `.github/workflows/ci.yml:377`, `.github/workflows/ci.yml:381`). Independent
  GitHub API inspection confirms that SHA is the official `v10.0.1` tag commit,
  its `action.yml` owns the `version` and `enable-cache` inputs, and uv 0.10.2
  is a published non-prerelease release.
- Each current workspace-suite step uses exactly
  `${{ runner.temp }}/uv-cache` and `RUST_MIN_STACK=8388608`, with neither
  variable present elsewhere in the workflow
  (`.github/workflows/ci.yml:42`, `.github/workflows/ci.yml:48`,
  `.github/workflows/ci.yml:393`, `.github/workflows/ci.yml:399`). The action
  setup precedes the suite in both jobs. The present implementation therefore
  has the intended scope despite D1 and D2's regression gaps.
- The focused amendment test, Poppler installer test, consumer-policy test, and
  WASM contract pass. All 44 workflow tests pass, both Python files compile,
  the hash harness remains 28 of 28, and prose, generated-skill synchronization,
  and diff hygiene pass. The progress record also reports the full `rpptx`
  suite passing locally under the exact cache and stack environment with 19
  unit and 86 integration tests passing and 7 ignored
  (`.claude/scratch/F-X012-progress.md:92`,
  `.claude/scratch/F-X012-progress.md:97`).
- The amendment modifies exactly the approved plan, CI workflow, existing
  workflow regression file, and HLD12, HLD14, and HLD15. The HLD impact list is
  unchanged and exact (`.claude/plans/F-X012-design.md:88`,
  `.claude/plans/F-X012-design.md:92`). HLD12 owns the two job commands, HLD14
  owns the story gate, and HLD15 owns the exact action, cache, and stack
  mechanism (`docs/hld/12-testing-strategy.md:458`,
  `docs/hld/12-testing-strategy.md:468`,
  `docs/hld/14-development-backlog.md:1300`,
  `docs/hld/14-development-backlog.md:1308`,
  `docs/hld/15-build-and-toolchain.md:358`,
  `docs/hld/15-build-and-toolchain.md:364`). The documents describe current
  mechanism rather than change history and do not contradict another HLD.

## Full feature evidence

- Pass 3 remains clean for the Poppler source checksum, all three resource
  bounds, streaming member handling, safe paths and types, each tool identity,
  populated-prefix refusal, the four Poppler consumers, and the Binaryen
  checksum and official identity
  (`.claude/reviews/F-X012-all-pass-3.md:14`,
  `.claude/reviews/F-X012-all-pass-3.md:48`,
  `.claude/reviews/F-X012-all-pass-3.md:70`). The amendment does not touch the
  installer or those workflow steps.
- The complete feature delta contains no `crates/` file, manifest, lockfile,
  public API, product runtime, package version, publication workflow, rendering
  baseline, or hash-baseline change. The hosted stack variable is currently
  step-local in Test and MSRV, so the implementation itself does not expand
  product runtime behavior (`docs/hld/15-build-and-toolchain.md:361`,
  `docs/hld/15-build-and-toolchain.md:364`).
- The temporary draft PR and branch remain evidence-only. The progress record
  identifies candidate SHA `b27d067be51d54e149d38369bab530f01c3434b4`, draft
  PR 26, and the two newly reached hosted failures that justify the amendment
  (`.claude/scratch/F-X012-progress.md:77`,
  `.claude/scratch/F-X012-progress.md:86`). The plan still requires a fully
  green hosted run at the amended reviewed SHA before completion
  (`.claude/plans/F-X012-design.md:81`,
  `.claude/plans/F-X012-design.md:85`).

## Not found

No incorrect official action SHA or tag, floating uv version, enabled action
cache, non-temporary active uv cache, incorrect 8 MiB value, current stack-scope
leak, wrong job ordering, missing Test or MSRV setup, product or public API
change, crate or dependency change, package or publication change, rendering or
hash-baseline delta, unlisted HLD edit, stale HLD mechanism, structural
indirection, panic, prose violation, smell, or nitpick was found beyond D1 and
D2.

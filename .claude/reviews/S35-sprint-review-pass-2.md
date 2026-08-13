# S35 sprint review, pass 2

**Reviewed**: `sprint/s35` at `e7bf86a249610b2357b9c3fd196668af2a2eca65`
against merge base `dafc783b1954aacec370ce38b889294aa8db0ebc`, 51 files,
3,734 insertions and 1,777 deletions, crates: `oxml-pdf`, `rdocx`,
`rdocx-layout`, `rdocx-wasm`, `rpptx`, `rpptx-render`, and `rpptx-wasm`, plus
their CLI, Python, and CI consumers
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the musllinux cells do not run the parity suites

`.github/workflows/wheels.yml:58`

The pass-1 observation that no hosted run existed is resolved. GitHub Actions
run 31714021807 was a successful `workflow_dispatch` at the exact reviewed SHA.
All twelve wheel jobs and both source-distribution jobs succeeded, exactly
fourteen artifacts exist, and the temporary `s35-wheel-acceptance` branch is
gone.

The run does not establish the whole S35 gate. Both musllinux jobs skipped
`Install and test native wheel` because that step is restricted to native cells
at `.github/workflows/wheels.yml:58`. Their musllinux-specific step installs the
wheel and imports its module, but runs no rdocx or rpptx parity test at
`.github/workflows/wheels.yml:105`. The structured workflow contract fixes that
import-only command as the exact expected musllinux behavior at
`scripts/test_sprint_workflow.py:1069`. GitHub's job evidence matches those
paths: both musllinux jobs report the parity-bearing native step skipped and
the import-only musllinux step successful.

S35 requires both wheels to pass their parity suites on every M13 target at
`docs/sprints/CURRENT_SPRINT.md:55`. Success for an install and import check is
not evidence that either parity suite passes on musllinux. The implementation
and its structured sensitivity must make each musllinux package cell execute
the applicable installed-wheel parity suite in a fresh environment. A new
reviewed hosted run must then supply successful evidence at the corrected SHA.

**Run-sprint disposition**: `fix-now`.

## Should-fix

None.

## Nice-to-have

None.

## Run-sprint disposition

- `fix-now`: B1. Add musllinux parity execution for both packages and make the
  structured workflow contract reject an import-only substitution.
- `tracked-follow-up`: none.
- `human-action`: after B1 is corrected and reviewed, run the hosted acceptance
  workflow again at the corrected SHA. A dispatch alone cannot close the
  current implementation gap.
- `refuted`: the pass-1 claim that no hosted run exists. Run 31714021807 now
  supplies the hosted matrix, source-distribution, artifact, and branch-cleanup
  evidence described above.

## Sprint definition of done

Five of the six S35 items hold. The hosted wheel item remains incomplete only
for parity execution in the two musllinux package cells.

- The hosted run is bound to the exact reviewed SHA, uses the two-package by
  six-platform matrix declared at `.github/workflows/wheels.yml:17`, and
  succeeded in all fourteen build jobs. Native manylinux, macOS, and Windows
  cells ran their package parity commands at `.github/workflows/wheels.yml:71`.
  Both musllinux cells instead ran only the import command cited in B1.
- The run produced exactly twelve wheel artifacts and two source-distribution
  artifacts. `Publish Python distributions` was skipped, which is the intended
  manual-dispatch boundary. The workflow permits publication only for a
  `py-v*` tag push at `.github/workflows/wheels.yml:146`, and the M13 story
  explicitly says manual dispatch never publishes at
  `docs/hld/14-development-backlog.md:1074`.
- The five non-wheel items retain the evidence recorded in pass 1. The only
  repository delta since that reviewed sprint head is the pass-1 review file.
  Integrated diff hygiene and prose checks remain green, and the run state
  continues to record four completed stories and unchanged full-verification
  hashes at `.claude/scratch/S35-run.json:72`.

## Milestone gate

The M13 end gate is: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`.

The hosted run proves both wheels build across all six target families and
proves parity on the five native target families. It proves only clean install
and import on musllinux. The end gate therefore does not yet hold.

M13 also retains F-143 `oxml-cli-support`, F-144 `rpptx-cli`, F-145 `rpptx-cli
thumbnail and outline`, and F-146 `npm publication`, all pending at
`docs/sprints/BACKLOG.md:277`. Those are genuinely future milestone stories and
do not block the completed WASM feature work by themselves. Future PyPI
publication is likewise not an S35 acceptance requirement, and the correctly
skipped manual-dispatch publish job is not a finding.

## Not found

No new interaction, duplicated helper, forbidden dependency direction,
unexplained hash delta, HLD contradiction beyond B1, unowned dependency,
speculative public surface, feature-unification leak, package-model fork,
render-path fork, release-authority leak, artifact-count mismatch, stale remote
acceptance branch, status mismatch, or tracked generated artifact was found.

The previously reviewed integrated implementation did not change between
passes. The fresh review checked the exact hosted run identity and conclusion,
all job and step conclusions, the fourteen-artifact inventory, deletion of the
temporary remote branch, manual publication isolation, diff hygiene, prose,
and sprint workflow status.

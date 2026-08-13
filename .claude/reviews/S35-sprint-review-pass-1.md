# S35 sprint review, pass 1

**Reviewed**: `sprint/s35` at `7e65d066909e9c7801cd91fb910ae6398e674a84`
against merge base `dafc783b1954aacec370ce38b889294aa8db0ebc`, 50 files,
3,616 insertions and 1,777 deletions, crates: `oxml-pdf`, `rdocx`,
`rdocx-layout`, `rdocx-wasm`, `rpptx`, `rpptx-render`, and `rpptx-wasm`, plus
their CLI, Python, and CI consumers
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the required hosted wheel matrix has not run

`docs/sprints/CURRENT_SPRINT.md:55`

S35 makes a reviewed hosted run of both Python wheels on every M13 target an
explicit sprint definition-of-done item. The durable F-137 record instead says
that the first real hosted cross-platform run remains future evidence and that
no dispatch occurred at `docs/sprints/AS_BUILT.md:5022` and
`docs/sprints/AS_BUILT.md:5031`. A fresh read of the
GitHub `Python wheels` workflow on 2026-08-13 returned zero runs, so there is no
external execution evidence that could satisfy the contract. The run state
nevertheless records full verification as passed at
`.claude/scratch/S35-run.json:66`.

This blocks the sprint review exit condition. A reviewed manual dispatch must
complete successfully, and its six-target evidence for both wheels must be
inspected and recorded before the sprint can be treated as done. The workflow's
manual path does not publish, but dispatching it is an external action outside
this review's authority.

**Run-sprint disposition**: `human-action`.

## Should-fix

None.

## Nice-to-have

None.

## Run-sprint disposition

- `fix-now`: none.
- `tracked-follow-up`: none.
- `human-action`: B1. Obtain and review the hosted wheel matrix evidence that
  S35 explicitly requires.
- `refuted`: none.

## Sprint definition of done

Five of the six S35 items hold. The hosted wheel item does not.

- `rdocx-wasm` owns one concrete facade document and delegates byte round trip,
  mutation, text, and PDF calls at `crates/rdocx-wasm/src/lib.rs:8`. Fresh
  native tests passed 3 of 3, and fresh Node tests passed both package
  preservation and embedded Carlito PDF checks.
- The pull-request WASM job uses reviewed immutable action commits, exact Node
  24.11.1 and wasm-pack 0.15.0, locked checks for both targets, and both
  unfiltered Node suites at `.github/workflows/ci.yml:85`. The complete 33-test
  structured workflow contract also passed during this review.
- `rpptx-wasm` owns one concrete facade presentation, keeps `toPdf` behind the
  render feature, and delegates it to the facade at
  `crates/rpptx-wasm/src/lib.rs:5`. Fresh default Node tests passed 1 of 1 and
  render-profile Node tests passed 2 of 2.
- The exact optimized normal-default size gate at
  `crates/rpptx-wasm/src/lib.rs:308` passed with wasm-pack 0.15.0, wasm-opt 125,
  deterministic gzip, and a fresh result of 519,060 decimal bytes. The default
  dependency tree omitted `rpptx-render` and `oxml-pdf`. The render tree added
  both without host font discovery.
- The document and presentation native defaults still enable system fonts,
  while both WASM graphs use defaults-off workspace edges at `Cargo.toml:55`.
  The facade owns presentation package-to-render assembly at
  `crates/rpptx/src/lib.rs:504`, and the corpus example delegates to it at
  `crates/rpptx/examples/render_deck.rs:156`.
- The hash harness independently matched all 28 entries. The authoritative
  integrated verification also records an unchanged harness at
  `.claude/scratch/S35-run.json:63`.

All four sprint rows are done and unowned at
`docs/sprints/CURRENT_SPRINT.md:32`. BACKLOG agrees on F-139 through F-142 at
`docs/sprints/BACKLOG.md:273`, the tracker has four matching S35 rows at
`docs/sprints/SPRINT_TRACKER.md:204`, and each feature has one AS_BUILT entry
beginning at `docs/sprints/AS_BUILT.md:5071`.

## Milestone gate

The M13 end gate is: "wheels install and pass the parity suites on every target
platform" at `docs/hld/14-development-backlog.md:994`.

That gate does not hold. The repository records local native wheel and source
distribution evidence, but it expressly leaves the first reviewed hosted
cross-platform execution outstanding at `docs/sprints/AS_BUILT.md:5022`. The
fresh GitHub workflow query confirms there is still no hosted run to inspect.

M13 is not otherwise ready to close. Four planned stories remain: F-143
`oxml-cli-support`, F-144 `rpptx-cli`, F-145 `rpptx-cli thumbnail and outline`,
and F-146 `npm publication`, all pending at `docs/sprints/BACKLOG.md:277`.
Those remaining stories explain why the end-of-milestone gate can remain open,
but they do not waive the same hosted evidence after S35 put it in this
sprint's own definition of done.

## Not found

No other integrated interaction, duplicated helper, forbidden dependency
direction, unexplained hash delta, sprint-gate failure, HLD contradiction,
unowned dependency, speculative public surface, feature-unification leak,
package-model fork, render-path fork, release-authority change, status mismatch,
or tracked generated artifact was found.

The four final independent feature reviews report zero defects, smells, or
nitpicks at `.claude/reviews/F-139-all-pass-4.md:4`,
`.claude/reviews/F-140-all-pass-3.md:4`,
`.claude/reviews/F-141-all-pass-1.md:4`, and
`.claude/reviews/F-142-all-pass-4.md:4`. Fresh diff hygiene, locked wasm32
checks, both default Node suites, the local presentation render Node suite,
native wrapper tests, the exact optimized size gate, the structured workflow
tests, and the hash harness all passed in the integrated worktree.

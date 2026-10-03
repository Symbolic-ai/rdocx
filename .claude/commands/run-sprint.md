---
description: Run every unfinished F-ID in the current sprint. Designs first, implements in safe parallel waves, completes scoped dependency-prefix checkpoints and verifies the final integrated result, then loops on review until clean.
---

# /run-sprint [--max-review-passes N] [--max-workers N]

Drive the whole active sprint. Design every story before implementing any of
them, run independent stories in parallel worktrees, complete scoped dependency-prefix
checkpoints and verify the final integrated result, then loop on `/sprint-review` until
it comes back clean.

Defaults are three review passes and as many workers as the wave allows.

Each worker keeps its implementation scoped to its approved F-ID, with focused
tests, the applicable oracle cases, `/verify --scoped` and a zero-finding
`/microscope`. Reserve `/verify --full`, the union of sprint risk riders and
`/sprint-review` for the integrated final result. A dependency-prefix
checkpoint uses its scoped evidence and focused reconciliation checks.

**`scripts/sprint_workflow.py` is the state authority.** Everything below is
resumable through `.claude/scratch/SNN-run.json`. Reuse it rather than starting
over.

**This command never merges to `main` or creates a release tag.** It ends by
telling you the exact `/close-sprint` invocation. Release preparation F-IDs
finish with local evidence. Publication follows sprint close through
`/release` from the reviewed `main` merge SHA.

## 1. Initialise

1. Read `CLAUDE.md`, `AGENTS.md`, `.claude/WORKFLOW.md` and
   `docs/sprints/CURRENT_SPRINT.md`.
2. Confirm the canonical worktree is on the matching `sprint/sNN` branch.
3. Refuse unrelated uncommitted changes. Changes belonging to a previous
   interrupted run of this command are resumed from state, not discarded.
4. Initialise or resume:

   ```bash
   python3 scripts/sprint_workflow.py init SNN --resume \
     --max-review-passes {N} [--max-workers {N}]
   ```

   Resume refreshes each known F-ID's title and size from
   `CURRENT_SPRINT.md`. It preserves phase, feature state, owner, wave, worker,
   verification, and review facts already recorded in the run state.

5. Audit for leftovers from an interrupted run. `git worktree list` and
   `git branch --list 'work/*'` against the run state. Report anything the state
   does not know about. Remove only clean completed worker worktrees whose
   handoff and integration commit are recorded. Keep their branches until
   `/close-sprint` has pushed the integrated sprint.
6. Report every F-ID that is not `completed`, with its state, its dependencies,
   and the skills its diff will trigger.

### Validation-only route

When the state contains zero features, first confirm that `CURRENT_SPRINT.md`
declares `**Validation-only**: yes` and that `SPRINT_PLAN.md` explicitly defines
the sprint as a validation boundary. Refuse an accidentally empty or malformed
wave.

For a valid zero-feature sprint:

1. Commit the regenerated `CURRENT_SPRINT.md` as `SNN, open validation sprint`
   so every later evidence record can bind to a clean, stable HEAD.
2. Skip design, questions, waves, workers, integration and feature ledgers.
3. Set the phase directly to `verification` and run step 6, including every
   boundary-specific gate in the sprint definition of done.
4. Set the phase to `review` and run step 8. Commit review records before
   recording the clean review and repeat the full verification if that commit
   changed HEAD. Review and full verification must both cover the final HEAD.
5. Continue at step 9. Report zero integrated F-IDs and no retained workers.

Do not manufacture an F-ID, design plan, handoff, tracker row or `AS_BUILT.md`
entry for a validation-only sprint.

## 2. Design everything first

1. Run `/design F-XXX --draft` for every unfinished story. **No implementation
   in this phase.**
2. Record ambiguities in each plan's `## Open questions` and keep going. Do not
   interrupt the batch to ask.
3. Apply `.claude/skills/risk-routing.md` to every plan. Record matched rows and
   their extra checks in `## Risk routing`.
4. Compare the drafts against each other and find:
   - Dependency order between F-IDs.
   - Files, crates and generated artefacts two stories both edit.
   - Stories that both expect to move the hash-harness baseline.
   - Crate-boundary work, where one story's extraction changes what another
     story is building on.
5. **Ask one consolidated round of questions** with AskUserQuestion. Group a
   shared decision once and name every F-ID it affects. If no material question
   exists, approve without pausing.
6. Apply each answer to every affected plan, clear its open question, and set
   `**Status**: approved`.
7. Commit all approved plans together:

   ```
   SNN, approve sprint designs

   One paragraph: the shared decisions taken and which stories they settled.

   Tests, not applicable
   Harness, unchanged
   ```

   This restores a clean canonical worktree before any claim, and gives every
   worker the same immutable base. Do not push.
8. Mark each plan `approved` in state, then
   `set-phase SNN implementation`.

Do not start implementing while any plan is `draft` or carries an unresolved
material question.

## 3. Build the waves

Build a dependency graph from the approved plans. Two F-IDs share a wave only if
they are dependency-independent **and** conflict-free.

Treat these as exclusive. One story per wave may hold each:

| Resource | Why |
|---|---|
| The same source or test file | Ordinary merge conflict |
| The same integration test binary | Adding a file there adds a link target. Stories add modules to the existing entrypoint, which is one file |
| The hash-harness baseline | There is one baseline. Two stories re-recording it in parallel produces a delta nobody can attribute |
| `CURRENT_SPRINT.md`, `BACKLOG.md`, `SPRINT_TRACKER.md`, `AS_BUILT.md` | The delivery record. Workers do not write the last two at all |
| The same `docs/hld/` section, when the edit is semantic | Two rewrites of one paragraph is a decision, not a merge |
| A crate's `Cargo.toml` dependency list | A dependency-direction violation is invisible in either half of the diff |

Record waves, dependencies, exclusive resources, branches and worktrees in the
state with `mark-feature --wave N`. **Report the wave plan before launching
anything.**

Mark a dependency-prefix checkpoint before any later wave when a formal
dependency is integrated and `reviewed` but not `completed`. Every formal edge
is a completion barrier, including ordinary A to B to C chains. A release F-ID
that is a dependency of an unfinished story adds the release extension below.
Neither checkpoint is a reason to put dependent work in the same wave.

## 4. Run each wave

Run ordinary waves only while their dependencies are completed. When the next
boundary is a dependency-prefix checkpoint, prepare its dependency prefix, then
follow the checkpoint route below instead of waiting for all later waves.

Per ordinary F-ID in the wave:

1. Claim it from the canonical worktree following
   `.claude/commands/claim-feature.md`. The orchestrator issues these claims
   without asking separately.
2. In the worker worktree, run `/implement-feature F-XXX`. The approved plan is
   reused. Do not design again.
3. Run the focused checks for the changed crates, plus every rider the plan's
   `## Risk routing` declared.
4. Run `/microscope F-XXX --working` against the worker diff. Iterate in
   numbered passes until zero defects and zero smells. **Not skippable.**
5. Run `/complete-feature F-XXX --prepare`, which writes and validates
   `.claude/handoffs/F-XXX-ready.md` and commits feature-local work to the
   worker branch.
6. Record progress with `mark-feature`, including the head sha and the handoff
   path.

A worker failure blocks that F-ID and everything depending on it, and nothing
else. Mark it `blocked`, say why, and carry on with the independent waves.

### Dependency-prefix checkpoint

Use this route when a later F-ID formally depends on an integrated story that
is still `reviewed`. The prerequisite must become `completed` before the
consumer starts, but full sprint verification and `/sprint-review` stay at
sprint closure. Keep the same sprint run state.

1. Integrate the prepared prerequisite through `/integrate-feature ...
   --batch`. Review its incremental integration diff against its approved
   plan. A semantic conflict receives another `/microscope` pass after
   reconciliation.
2. Confirm the worker's `/verify --scoped`, focused oracle cases, hash result,
   test gate and zero-finding microscope remain valid for the integrated
   prefix. Run focused checks for any integration-only reconciliation. Do not
   run `/verify --full` or `/sprint-review` at this checkpoint.
3. Apply the non-release documentation and delivery-record steps in section 7
   for that prerequisite. Mark it `completed`, clear its owner, and commit
   those records. Keep the worker branch for final sprint review. Remove its
   clean worktree after recording the integration commit to save disk space.
4. Return the phase to `implementation` and start the dependent wave. Record
   that full verification and sprint review are still due at final closure.

The final `/verify --full` and `/sprint-review` cover the complete integrated
sprint at its current HEAD. A dependency checkpoint never supplies closure
evidence for that final result.

#### Release preparation dependencies

A release preparation F-ID is an ordinary dependency until its reviewed local
workflow, artifact and version preflights pass. Complete its delivery records
through the dependency-prefix checkpoint before starting any dependent story.
Do not create or push a release tag, publish a registry package, or call
`/release` inside `/run-sprint`. Publication follows `/close-sprint` from the
reviewed `main` merge SHA under separate approval for each family tag.

Every verification and review record remains bound to its exact current HEAD.
Never use checkpoint evidence for final closure after later waves or delivery
records change HEAD. The ordinary final integration, verification, review,
`close-preflight SNN`, and sprint push still run after all waves finish.

## 5. Integrate

From the canonical worktree, integrate every remaining prepared branch that a
dependency-prefix checkpoint did not already consume. Process them in
dependency order, one at a time:

```text
/integrate-feature F-XXX work/<fid-lower>-<agent> --batch
```

Set the phase to `integration` before the first one. Never resolve a semantic
conflict automatically. A conflict between two sprint features is reconciled
against both approved plans and re-reviewed. Any other conflict stops for a
human.

## 6. Verify once, over the integrated result

For the ordinary final gate, only after every branch is integrated or consumed
by a dependency-prefix checkpoint:

1. Run `/verify --full`. Not per worker, not `--fast`, and not `--scoped`.
2. Add the union of every `## Risk routing` rider the sprint's plans declared.
   A rider one story earned runs once here for the whole sprint.
3. **The hash harness is the step that matters most.** Every delta must trace to
   a story that declared it in its `## Hash harness` section, and the totals
   must reconcile. A delta no plan predicted stops the sprint. It is not a
   prompt to re-record the baseline.
4. Record the exact commands, outcomes and anything skipped:

   ```bash
   python3 scripts/sprint_workflow.py record-verification SNN \
     --scope full --passed --harness "unchanged | <delta>"
   ```

5. Fix failures and re-run the affected commands. Do not push while the sprint
   is red. Record an incomplete gate as a failure, never as a pass.

## 7. Finalise the record

After verification passes:

For every integrated F-ID, including a release preparation story:

1. Apply the `/complete-feature` documentation steps:
   update exactly the HLD files its plan listed, append its `AS_BUILT.md` entry
   with the consolidated evidence, and append its `SPRINT_TRACKER.md` row.
2. Set `done` in `BACKLOG.md` and `CURRENT_SPRINT.md`, clear its `Owner` cell,
   and regenerate the AUTOGEN counts.
3. Set its design plan to `**Status**: completed`.
4. Delete its `.claude/scratch/F-XXX-progress.md`. Its durable facts are in
   AS_BUILT now.
5. `mark-feature SNN F-XXX completed --clear-owner`.
6. Commit the ledgers as one `SNN, sprint ledgers` commit. Do not push.
7. `set-phase SNN review`.

When the sprint is validation-only, skip this section and use the route in
section 1.

## 8. Review and remediate

Run `/sprint-review SNN --pass N`. Classify every finding:

| Class | Meaning |
|---|---|
| `fix-now` | An actionable defect, smell or in-scope documentation gap |
| `tracked-follow-up` | Real but not blocking. Needs a backlog home, created now |
| `human-action` | Something an agent cannot safely do, such as opening a corpus deck in PowerPoint |
| `refuted` | Contradicted by concrete evidence in the repository, cited |

A pass with no `fix-now` findings is clean. **Stop there.** Do not run a
confirmation pass after a clean pass.

Otherwise: fix every safe `fix-now` finding, re-run the impacted checks, commit
the remediation separately, and start a fresh independent pass. Reuse finding
IDs across passes so a reader can follow one defect through the sprint. Record
each pass with `record-review`. Continue this loop inside the current
`/run-sprint` invocation. A review pass remains read-only, but returning from it
to a distinct remediation phase does not require another user invocation.

At the bound, if actionable findings remain, `set-phase SNN blocked`, do not
push, and report what is outstanding. Closure stays forbidden.

## 9. Finish

When the latest pass is clean:

1. Run `close-preflight SNN`. It refuses on an unconsumed handoff, a feature
   that is neither completed nor carried, a blocking review finding, a missing
   full verify, or a tracker that disagrees with the run state.
2. Push `sprint/sNN` once, carrying every F-ID, ledger, review and remediation
   commit.
3. Report:
   - The sprint base and head.
   - Integrated F-IDs, and anything blocked or carried.
   - The verification evidence, especially the harness result.
   - Review passes and their verdicts.
   - **Retained worker branches and any remaining worktrees**, which
     `/close-sprint` will clean after the sprint merge and tag are pushed.
   - The prepared family tags to release from `main` after sprint close.
   - The exact next command:

     ```text
     /close-sprint SNN --next SMM
     ```

## Refused situations

- **Implementing while any plan is `draft`.**
- **Verifying per worker instead of over an integrated checkpoint or the final
  integrated result.** That is precisely the failure `/sprint-review` exists to
  catch.
- **Re-recording the hash baseline to make step 6 pass.**
- **Deleting a worker branch before `/close-sprint`, or removing a dirty,
  carried or unrelated worktree.** Only clean integrated worker worktrees may
  be removed early.
- **Running a confirmation pass after a clean review pass.**
- **Asking the user to rerun `/run-sprint` solely to cross from a completed
  review pass into its remediation phase.**
- **Merging to `main` or creating a tag directly.** `/close-sprint` owns the
  merge and sprint tag. `/release` owns release tags.

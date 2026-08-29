---
description: Run every unfinished F-ID in the current sprint. Designs first, implements in safe parallel waves, verifies dependency-prefix checkpoints and the final integrated result, then loops on review until clean.
---

# /run-sprint [--max-review-passes N] [--max-workers N]

Drive the whole active sprint. Design every story before implementing any of
them, run independent stories in parallel worktrees, verify dependency-prefix
checkpoints and the final integrated result, and loop on `/sprint-review` until
it comes back clean.

Defaults are three review passes and as many workers as the wave allows.

**`scripts/sprint_workflow.py` is the state authority.** Everything below is
resumable through `.claude/scratch/SNN-run.json`. Reuse it rather than starting
over.

**This command never merges to `main`.** It normally creates no tag and ends by
telling you the exact `/close-sprint` invocation. A release F-ID is the one
exception described below, and it delegates the release tag to `/release`.

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
   does not know about. **Do not delete a worktree or branch.** Worker cleanup
   belongs to `/close-sprint` after the integrated sprint is pushed.
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

Use this route whenever the step 3 trigger matches. Keep the same sprint state
file and repeat the route before every later wave whose formal dependency is not
completed. Do not claim the dependent wave while its prerequisite is only
prepared, integrated, or reviewed.

Pass numbering remains global in the sprint state. Treat each prefix-record,
prepared-release, post-publication, and final-closure review as a distinct
boundary with at most the configured review-pass bound. When the global pass
number exceeds that bound only because earlier boundaries finished clean, use
`record-review --extend` and record `scheduled dependency-prefix boundary` as
the reason in the review file. This does not extend the current boundary's own
remediation limit. If that limit is reached with actionable findings, mark the
sprint blocked.

1. Set the phase to `integration`. Integrate the prepared dependency prefix in
   dependency order through `/integrate-feature ... --batch`. Do not integrate
   or claim its unfinished consumer.
2. Run `/verify --full`, add the prefix's risk riders, and record the passing
   result at the current HEAD.
3. Finalise the reviewed non-release prefix through `## 7. Finalise the record`.
   Update its HLD and delivery records, mark each prefix F-ID completed, clear
   its owner, and commit the records.
4. Run `/verify --full` at the record HEAD, then complete the bounded
   `/sprint-review` loop. When a pass is clean, preserve this exact order:
   - Commit the clean review file without any other change.
   - Run `record-review` to record the clean review at the resulting HEAD.
   - Rerun `/verify --full` because the review commit changed HEAD, and record
     that passing result at the same HEAD.

   Do not run a confirmation review solely because its clean review file was
   committed. The committed review remains the review evidence for that HEAD,
   and the repeated full verification proves the commit introduced no failure.
5. Return the phase to `implementation`. Start only waves whose formal
   dependencies are now completed. For A to B to C, complete this route for A
   before B starts and again for B before C starts.

#### Release dependency extension

When a release F-ID is a dependency of any unfinished story in the same sprint,
first use the ordinary checkpoint above to complete its non-release dependency
prefix, then extend the route:

1. Prepare and integrate the release F-ID through the normal worker,
   `/microscope`, `/complete-feature --prepare`, and batch integration route.
   Keep it `reviewed` in run state and `in-progress` in both delivery trackers
   under the release exception.
2. Run `/verify --full` and `/sprint-review` at the prepared release current
   HEAD. Apply every release risk rider and use the exact clean-review commit,
   record, and verification sequence from the ordinary checkpoint.
3. Follow `/release <tag>` at that exact reviewed and fully verified HEAD.
   `/release` remains the only tag and publication authority and obtains its
   separate final approval immediately before the first external mutation. If
   approval is withheld or publication fails, preserve the recorded state and
   stop without starting a consumer.
4. Finalise the release F-ID's delivery records only after `/release` verifies
   every required registry entry, tag, release body, owner, notification, and
   other selected-family evidence. Commit those records, mark the release F-ID
   completed, clear its owner, then run the ordinary checkpoint's full
   verification and clean-review evidence sequence at the new current HEAD.
5. Return the phase to `implementation` again. Resume only the later waves whose
   dependencies are now completed. Never start an unfinished dependent story
   before verified publication and release-F-ID completion.

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

1. Run `/verify --full`. Not per worker, and not `--fast`.
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

First identify any remaining release F-ID whose gate requires real
publication. Leave it
`reviewed` in run state and `in-progress` in the delivery trackers, retain its
owner and approved plan, and defer its delivery ledgers until step 9. It still
participates in the integrated full verification and sprint review.

For every other integrated F-ID:

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
each pass with `record-review`.

At the bound, if actionable findings remain, `set-phase SNN blocked`, do not
push, and report what is outstanding. Closure stays forbidden.

## 9. Finish

When the latest pass is clean:

0. If a remaining unfinished release F-ID requires a real publication gate,
   pause at the reviewed and fully verified SHA. Report the exact `/release
   vX.Y.Z` command and follow it. `/release` performs its own separate final
   approval before any external mutation. After the release is verified, create
   that F-ID's delivery records, set its plan and state to completed, clear its
   owner, re-run the affected checks and the bounded sprint review, then
   continue here.

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
   - **Retained worker branches and worktrees**, which `/close-sprint` will
     remove after the sprint merge and tag are pushed.
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
- **Deleting a worker branch or worktree.** `/close-sprint` owns cleanup after
  the sprint is safely pushed.
- **Running a confirmation pass after a clean review pass.**
- **Merging to `main` or creating a tag directly.** `/close-sprint` owns the
  merge and sprint tag. `/release` owns release tags.

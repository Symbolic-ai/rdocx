# F-X061, Support staged dependency checkpoints in run-sprint

**Status**: completed
**Sprint**: S58
**Size**: S
**Depends on**: none

## Problem

`/run-sprint` currently integrates every feature before finalising any delivery
record. A later wave cannot start when its formal dependency is integrated but
not completed. Release dependencies add a second requirement because real
publication must also finish before their consumers start. The state machine
can already retain multiple HEAD-bound verification and review records, but the
command lacks the prefix-finalisation and resume route.

Wave numbers alone cannot solve this. `/start-feature` correctly refuses an
unfinished dependency, while `/release` correctly requires completed
dependencies plus full verification and clean review at the prepared current
HEAD.

## Spec reference

- `docs/hld/12-testing-strategy.md`, workflow regression and HEAD-bound evidence.
- `docs/hld/14-development-backlog.md`, "F-X061, Support staged dependency checkpoints in run-sprint".
- `docs/hld/15-build-and-toolchain.md`, sprint automation, publication, and release process.
- `.claude/WORKFLOW.md`, resumable sprint state and command authority.
- `.claude/commands/run-sprint.md`, wave, verification, review, release, and finish sequence.
- `.claude/commands/release.md`, release preconditions and separate final approval.

## Approach

Add concise ordinary and release dependency checkpoint routes to the canonical
`.claude/commands/run-sprint.md`. Trigger a checkpoint whenever a later wave's
formal dependency is integrated or reviewed but not completed.

At an ordinary checkpoint:

1. Integrate and fully verify the prepared dependency prefix.
2. Finalise that prefix's delivery records and commit them.
3. Run full verification and clean sprint review at the new current HEAD.
4. Commit the clean review file, record that review at the resulting HEAD, and
   rerun full verification because the evidence commit changed HEAD. Do not add
   a confirmation review solely for its own review-file commit.
5. Return the same sprint state to implementation and continue only the waves
   whose dependencies are now completed.
6. For a release dependency, prepare and review the release after its ordinary
   prefix is completed, invoke `/release`, and stop for its mandatory separate
   final approval.
7. After verified publication, finalise the release records and repeat the
   same HEAD-bound review-file commit, review record, and verification sequence.

Repeat the route for each dependency boundary. The ordinary final
verification, review, close-preflight, and sprint push still run after all
waves. Never treat an earlier checkpoint as closure evidence for a later HEAD.
Never start an unfinished dependent story before publication is verified and
the release F-ID is completed.

No new script command or state field is required. `set-phase` already permits
the resumable phase sequence, and review and verification records already bind
to their exact HEAD. Extend the existing state-machine regression to exercise
two returns from review to implementation. Add an A to B to C state regression
and mutation tests that require the ordinary checkpoint trigger, ordered
HEAD-bound evidence sequence, no self-confirming review, separate release
approval, and final normal closure.

Resuming an existing run must refresh each known F-ID's title and size from
`CURRENT_SPRINT.md` while preserving its state, owner, wave, worker, and
evidence facts. Add a focused regression for that refresh.

Edit only the canonical command, the existing workflow script and test file,
and the HLD files listed below. Regenerate
`.agents/skills/run-sprint/SKILL.md` through
`python3 scripts/sync_agent_skills.py`. Never edit the generated adapter by
hand and do not add a new skill folder, module, or test binary.

## Rejected alternatives

- Ignore unfinished dependencies until sprint finish. The dependent feature cannot legally start.
- Mark an unpublished release F-ID completed. That weakens the real release gate into local preparation.
- Split S58 after work has begun. The approved goal is one resumable S58 delivery record.
- Add a second sprint state file. The repository requires exactly one shared state authority.
- Add a new state-machine command. Existing phase and evidence records already represent the required sequence.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_run_sprint_phase_sequence_is_accepted` | Two review-to-implementation returns remain resumable before final closure |
| regression | `test_run_sprint_ordinary_dependency_chain_is_accepted` | A is completed before B starts, B is completed before C starts, and one sprint state returns to implementation after each prefix |
| regression | `test_init_resume_refreshes_feature_metadata_without_losing_progress` | Resume refreshes canonical title and size while retaining state, owner, wave, and worker evidence |
| workflow | `test_run_sprint_requires_ordinary_dependency_prefix_checkpoints` | The canonical command orders ledger finalisation, review-file commit, review recording, HEAD verification, and resumption without a confirmation review |
| workflow | `test_run_sprint_requires_dependency_release_checkpoints` | A release dependency extends the ordinary route with preparation, publication, and separate approval |
| workflow | release authority regression | Only `/release` creates release tags and every checkpoint retains separate immediate approval |
| generated skill | `python3 scripts/sync_agent_skills.py --check` | The Codex adapter matches the canonical command SHA and content |

The **test gate is regression**. The workflow contracts and state regressions
prove ordinary and release dependency checkpoints can return to implementation
before final close-preflight without weakening approval or HEAD-bound evidence.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Release scripting and version strings**. Preserve `/release` as the only
  tag and publication authority, keep immediate approval, and mutation-test
  the ordered checkpoint contract.
- **Agent skill workflow**. Edit the canonical command only, regenerate the
  adapter, run sync drift checks, and validate the generated skill.
- **Sprint state workflow**. Mutation-test checkpoint ordering and prove resume
  refreshes canonical metadata without discarding progress or evidence.
- **Tracked Markdown**. Run prose and link-sensitive workflow tests.

## Hash harness

Expected unchanged across all 49 entries. This story changes workflow
automation and HLD text only. It must not edit render code, samples, or
baselines.

## Implementation checklist

- [x] Add failing phase-sequence and command-contract regressions.
- [x] Add the ordinary dependency-prefix checkpoint route and release extension
  to the canonical command.
- [x] Preserve final verification, review, close-preflight, and sprint push.
- [x] Update exactly the three listed HLD files.
- [x] Regenerate and validate the `run-sprint` adapter.
- [x] Refresh title and size metadata on resume without losing run facts.
- [x] Run workflow tests, prose, sync drift, hash, microscope, and full verification.
- [x] Prepare and validate the handoff without pushing.

## Open questions

None. The user approved resumable dependency-prefix checkpoints inside the
existing S58 state and retained every separate `/release` approval boundary.

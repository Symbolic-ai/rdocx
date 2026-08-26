# F-X061, Support staged release checkpoints in run-sprint

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: none

## Problem

`/run-sprint` currently integrates every feature before full verification and
defers release F-IDs to its final step. That works only when publication has no
unfinished consumer in the same sprint. S58 requires `rpptx-v0.7.0` before
F-198 and F-199 can start, and `v0.11.0` before F-X031 can start. The current
workflow therefore deadlocks both dependency edges even though the state
machine can already retain multiple HEAD-bound verification and review records.

Wave numbers alone cannot solve this. `/start-feature` correctly refuses an
unfinished dependency, while `/release` correctly requires completed
dependencies plus full verification and clean review at the prepared current
HEAD.

## Spec reference

- `docs/hld/12-testing-strategy.md`, workflow regression and HEAD-bound evidence.
- `docs/hld/14-development-backlog.md`, "F-X061, Support staged release checkpoints in run-sprint".
- `docs/hld/15-build-and-toolchain.md`, sprint automation, publication, and release process.
- `.claude/WORKFLOW.md`, resumable sprint state and command authority.
- `.claude/commands/run-sprint.md`, wave, verification, review, release, and finish sequence.
- `.claude/commands/release.md`, release preconditions and separate final approval.

## Approach

Add one concise dependency-release checkpoint route to the canonical
`.claude/commands/run-sprint.md`. Trigger it when a release F-ID is a dependency
of any unfinished story in the same sprint.

At a checkpoint:

1. Integrate and fully verify the completed non-release dependency prefix.
2. Finalise that prefix's delivery records, commit them, and repeat full
   verification plus sprint review at the new current HEAD.
3. Return to implementation, then claim, implement, microscope, prepare, and
   integrate the release F-ID.
4. Run full verification and clean sprint review at the prepared release HEAD.
5. Leave the release F-ID reviewed and in progress, invoke `/release`, and stop
   for its mandatory separate final approval.
6. After verified publication, finalise the release records, commit them, and
   rerun full verification plus bounded sprint review at the evidence HEAD.
7. Return the same sprint state to implementation and continue only the waves
   whose dependencies are now completed.

Repeat the route for each dependency release boundary. The ordinary final
verification, review, close-preflight, and sprint push still run after all
waves. Never treat an earlier checkpoint as closure evidence for a later HEAD.
Never start an unfinished dependent story before publication is verified and
the release F-ID is completed.

No new script command or state field is required. `set-phase` already permits
the resumable phase sequence, and review and verification records already bind
to their exact HEAD. Extend the existing state-machine regression to exercise
two returns from review to implementation. Add a command-contract regression
that requires the checkpoint trigger, ordered steps, separate release approval,
HEAD-bound reruns, and final normal closure.

Edit only the canonical command, its existing workflow test file, and the HLD
files listed below. Regenerate `.agents/skills/run-sprint/SKILL.md` through
`python3 scripts/sync_agent_skills.py`. Never edit the generated adapter by
hand and do not add a new skill folder, module, or test binary.

## Rejected alternatives

- Ignore unfinished release dependencies until sprint finish. The dependent feature cannot legally start.
- Mark an unpublished release F-ID completed. That weakens the real release gate into local preparation.
- Split S58 after work has begun. The approved goal is one resumable S58 delivery record.
- Add a second sprint state file. The repository requires exactly one shared state authority.
- Add a new state-machine command. Existing phase and evidence records already represent the required sequence.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_run_sprint_phase_sequence_is_accepted` | Two review-to-implementation returns remain resumable before final closure |
| workflow | `test_run_sprint_requires_dependency_release_checkpoints` | The canonical command detects an unfinished consumer, orders prefix evidence and release approval, reruns evidence at changed HEADs, and resumes later waves |
| workflow | release authority regression | Only `/release` creates release tags and every checkpoint retains separate immediate approval |
| generated skill | `python3 scripts/sync_agent_skills.py --check` | The Codex adapter matches the canonical command SHA and content |

The **test gate is regression**. The workflow contract and phase-state
regression prove two verify-review-release checkpoints can return to
implementation before final close-preflight without weakening approval or
HEAD-bound evidence.

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
- **Tracked Markdown**. Run prose and link-sensitive workflow tests.

## Hash harness

Expected unchanged across all 49 entries. This story changes workflow and HLD
text only. It must not edit render code, samples, or baselines.

## Implementation checklist

- [ ] Add failing phase-sequence and command-contract regressions.
- [ ] Add the dependency-release checkpoint route to the canonical command.
- [ ] Preserve final verification, review, close-preflight, and sprint push.
- [ ] Update exactly the three listed HLD files.
- [ ] Regenerate and validate the `run-sprint` adapter.
- [ ] Run workflow tests, prose, sync drift, hash, microscope, and full verification.
- [ ] Prepare and validate the handoff without pushing.

## Open questions

None. The user approved resumable release checkpoints inside the existing S58
state and retained every separate `/release` approval boundary.

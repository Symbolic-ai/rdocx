# F-X067, Prime Word fidelity Cargo dependencies

**Status**: approved
**Sprint**: S58
**Size**: S
**Depends on**: F-X064

## Problem

PR 58 identifies a cold-run failure in the Word fidelity job. The job restores
the pinned Rust cache, then `scripts/docx_ssim_harness.py` intentionally builds
`rdocx` with `--locked --offline`. A cache miss can therefore stop the job with
status 101 before it produces render evidence, even though the same source
passes when the dependency graph is already present locally.

The current `word-fidelity` job in `.github/workflows/ci.yml` does not fetch the
complete locked Cargo graph before the harness starts. The workflow contract in
`scripts/test_sprint_workflow.py` checks the corpus, harness, and evidence
upload, but does not protect the dependency-priming boundary.

## Spec reference

- `docs/hld/12-testing-strategy.md`, the Word fidelity gate and retained
  evidence contract.
- `docs/hld/14-development-backlog.md`, "F-X067, Prime Word fidelity Cargo
  dependencies".
- `docs/hld/15-build-and-toolchain.md`, pinned CI actions, locked dependency
  resolution, and offline fidelity builds.

## Approach

Add one named `cargo fetch --locked` step to the `word-fidelity` job immediately
after the pinned Rust cache and before the corpus fetch and fidelity harness.
Keep the harness build locked and offline. The explicit fetch makes the network
boundary visible and ensures the later offline build has the complete lockfile
graph.

Strengthen the existing workflow regression so the exact command appears once
in the Word fidelity job, after the cache, and before any harness invocation.
Mutation cases reject a missing step, an unlocked fetch, a fetch after the
offline consumer, and a step added to the wrong job.

Use PR 58 at commit `c8fed1d1268fd765d602bac2da6524900c1c1cfd`
as contribution evidence. Implement the direct outcome with repository
hardening from the integrated sprint head. Do not merge, retarget, comment on,
or close the PR.

## Rejected alternatives

- Remove `--offline` from the harness build. That weakens the deterministic
  fidelity contract and hides undeclared network access inside the renderer
  gate.
- Rely on `Swatinem/rust-cache`. A cache is an optimization and may be cold or
  incomplete.
- Run `cargo fetch` without `--locked`. That permits dependency resolution to
  drift from the reviewed lockfile.
- Teach the harness to fetch implicitly. The workflow owns network setup, and
  the harness should retain its portable offline contract.
- Add a new action, script, module, or test binary. The existing workflow and
  workflow-test module are sufficient.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_word_fidelity_primes_locked_dependencies_before_offline_build` | The exact locked fetch appears once after the pinned cache and before the Word harness |
| negative | `test_word_fidelity_ci_gate_rejects_weakened_invocations` | Missing, unlocked, misplaced, or wrong-job dependency priming is rejected |
| workflow | `python3 -m unittest scripts.test_sprint_workflow` | The complete workflow contract remains coherent |
| integration | current Word fidelity harness | The pinned corpus produces nonempty retained evidence after the locked offline build |
| CI evidence | PR 58 Word fidelity job | The submitted cold-run path produces complete retained evidence on a hosted runner |

The **test gate is regression**. The focused workflow regressions, current Word
fidelity harness, PR 58 hosted evidence, and `/verify --full` must pass. The
integrated hosted Word job remains a sprint-completion rider after the sprint
branch is pushed.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **An external oracle comparison**. Keep the corpus, LibreOffice, Poppler,
  render metric, and thresholds pinned through the existing Word fidelity
  harness. Require nonempty retained evidence from the hosted contribution run
  and the final integrated sprint run.

No product API, parser, serializer, rendering, package-version, dependency,
binding, or public-surface row is triggered by the diff.

## Hash harness

Expected unchanged at 49 of 49. This story changes CI dependency preparation
only and must not change generated output or baselines.

## Implementation checklist

- [ ] Add failing dependency-priming and mutation regressions in the existing workflow test module.
- [ ] Add the exact locked fetch step to the Word fidelity job.
- [ ] Preserve the locked offline harness invocation and retained evidence contract.
- [ ] Run the workflow suite, current Word fidelity gate, and hash harness.
- [ ] Update exactly the three plan-listed HLD files.
- [ ] Run microscope and `/verify --full`.
- [ ] Record PR 58 and its exact source SHA in handoff and release inventory evidence.
- [ ] Record the integrated hosted Word fidelity job as a sprint-completion rider.

## Open questions

None. The user asked to fold every newly uncovered issue and PR into S58. The
submitted locked fetch is the bounded fix, while the offline fidelity build
remains authoritative.

# F-X025, /verify must run the release regressions

**Status**: approved
**Sprint**: S43
**Size**: S
**Depends on**: none

## Problem

`.github/workflows/publish.yml:24` invokes two tests by their full dotted path
as the publication gate:

```
python3 -m unittest \
  scripts.test_sprint_workflow.SprintWorkflowTests.test_stable_release_family_is_prepared_at_0_7_0 \
  scripts.test_sprint_workflow.SprintWorkflowTests.test_incubating_release_family_is_prepared_at_0_3_0
```

`.claude/commands/verify.md` runs eleven steps and none of them is
`python3 -m unittest scripts.test_sprint_workflow`. The release family
preflights therefore run for the first time at publication, on a tag, after the
sprint is closed.

S42 demonstrated the gap. F-X022 moved every version carrier under `crates/`,
passed the entire local gate, and left the incubating preflight and the
`ci.yml` WASM literal asserting the old version. It would have failed in CI at
publication time, with nothing local having said so.

Two further facts make the same point. The suite takes 4 seconds, so nothing
about the omission was a cost decision. And
`docs/hld/15-build-and-toolchain.md:166-173` currently describes the stable
regression as requiring workspace 0.6.0 and the incubating one as requiring
0.2.0, while `Cargo.toml` is at 0.7.0 and `crates/rpptx/Cargo.toml` at 0.3.0.
The prose describing the gate has drifted from the gate, which is the same
defect one layer up.

## Spec reference

- `docs/hld/15-build-and-toolchain.md`, the `publish.yml` paragraph beginning
  "`publish.yml` accepts stable `v*` and incubating `rpptx-v*` tags". That
  paragraph names the preflight regressions and their required versions.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", for the `regression`
  category the gate uses, and the README-inventory paragraph ending "The current
  stable 0.6.0 release also verifies every crates.io README endpoint", which
  carries the second instance of the same stale figure.
- `docs/hld/14-development-backlog.md`, "F-X025, /verify must run the release
  regressions".

## Approach

Three edits, none of them clever.

**1. `.claude/commands/verify.md`, step 6.** Step 6 already holds the two
standard-library checks that keep the process documents honest, the prose rules
and the Codex adapter drift check. The release regressions join it:

```bash
python3 -m unittest scripts.test_sprint_workflow
```

Step 6 runs in the default and `--full` modes and is skipped by `--fast`, along
with the rest of steps 5 to 11, which is the right side of the line for a
4-second check that gates publication. The step text states what a failure
means, which is that a version carrier moved without its assertions moving with
it, and that the fix is the carrier or the assertion, never deleting the test.

**2. Regenerate `.agents/skills/`.** `verify.md` is a command, so
`python3 scripts/sync_agent_skills.py` runs and the regenerated adapter is
committed with the change. `/verify` step 6 fails on drift, which this story is
in the middle of editing, so the regeneration is not optional bookkeeping.

**3. `scripts/test_sprint_workflow.py`, two tests.** The suite already
mutation-tests its own assertions, and this follows that shape:

- The wiring test reads `.claude/commands/verify.md` and asserts it invokes
  `python3 -m unittest scripts.test_sprint_workflow`. This is what makes the
  gate self-defending: removing the step from `verify.md` fails the suite that
  the step runs.
- The name-resolution test reads `publish.yml`, extracts every
  `scripts.test_sprint_workflow.<Class>.<method>` path it invokes, and asserts
  each resolves to a real test method on a real class in this module. A rename
  here breaks publication on a tag today, and this turns that into a local
  failure.

Both follow `test_release_command_is_the_only_release_tag_authority` at
`scripts/test_sprint_workflow.py:3975`, which already reads command documents
and asserts on their content, so no new mechanism is introduced.

## Rejected alternatives

- **Run only the two tests `publish.yml` names.** Reproduces the coupling that
  caused the problem: a third preflight added later would sit unrun until
  someone updated two places. The whole module costs 4 seconds.
- **Add a twelfth step instead of extending step 6.** Step 6 is already "the
  standard-library checks over the process documents". A step per script would
  make the gate a list rather than a shape.
- **Make `publish.yml` invoke the whole module and drop the named paths.** A
  real simplification and out of scope for a story about the local gate. If it
  is still worth doing after this lands, it is a backlog item, not a rider.
- **Run it in `--fast` too.** `--fast` is the inner loop over changed crates.
  A version carrier moves in a commit that is going to be verified in full
  before it completes.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `test_verify_runs_the_release_regressions` | `.claude/commands/verify.md` invokes `python3 -m unittest scripts.test_sprint_workflow`, and removing that line from a copy of the document fails the same assertion. The gate defends its own wiring |
| regression | `test_every_test_publish_yml_names_resolves_to_a_real_test` | Every `scripts.test_sprint_workflow.<Class>.<method>` path `publish.yml` invokes resolves to a real test in this module, so a rename fails locally rather than on a tag |

**Test gate**, from the backlog: the first regression, which is the half that
makes a stale literal reach `/verify --full` at all. The second row closes the
rename hole in the same wiring.

A third row was planned, exercising the preflights against mutated version
carriers, and dropped as redundant.
`test_release_preparation_metadata_rejects_wasm_tag_and_version_mutations` at
`scripts/test_sprint_workflow.py:3951` already mutates a version literal and
asserts the contract rejects it, through the injectable
`assert_release_preparation_metadata_contract` helper. Writing a second one
would pin the same behaviour twice.

The end-to-end halves of the backlog's sentence are demonstrated rather than
asserted, because they are statements about a tree that does not exist in the
repository:

- `crates/rpptx/Cargo.toml` moved from 0.3.0 to 0.3.1: the module fails, with
  `test_incubating_release_family_is_prepared_at_0_3_0` and
  `test_stable_release_family_is_prepared_at_0_7_0` both reporting.
- `ci.yml`'s `@tensorbee/rpptx-wasm` literal moved back to 0.2.0, which is
  exactly the S42 defect: the module fails, with three tests reporting,
  including both WASM job assertions.
- The clean tree: 48 tests, OK.

## HLD impact

- `docs/hld/15-build-and-toolchain.md`, the `publish.yml` paragraph. It gains
  the statement that the same preflight regressions run in the canonical local
  gate, and its stale figures are corrected to workspace 0.7.0 and incubating
  0.3.0, which is what the tests it describes actually assert.
- `docs/hld/12-testing-strategy.md`, the README-inventory paragraph. "The
  current stable 0.6.0 release" becomes 0.7.0, which is the second instance of
  the same drift and was settled in the S43 question round as in scope here
  rather than deferred to `/realign-docs`.

Both corrections are confined to the two sentences carrying the figure. Neither
touches "The hash harness" section, which F-X021 owns this sprint, so the two
stories are kept out of one another's waves rather than out of one another's
files.

## Risk routing

Matched row: **Release scripting, version strings**.

- Read `.claude/commands/release.md` and `docs/hld/15-build-and-toolchain.md`
  before editing.
- Inspect every manifest, lockfile and README version diff. This story changes
  no version carrier, so the expected inspection result is an empty diff, and
  the two figures it corrects are prose in one HLD paragraph.
- Require a clean full gate and a separate final approval before tagging. No
  tag is created by this story. It creates no release authority and touches no
  `publish.yml` command.

## Hash harness

**Expected unchanged.** No Rust source, no sample generator input and no
rendering path is touched. A delta would mean something outside this story's
diff moved.

## Implementation checklist

- [x] Record the pre-change harness state, 28 of 28 at the sprint base
- [x] `verify.md` step 6 gains the release regressions and what a failure means
- [x] `python3 scripts/sync_agent_skills.py`, regenerated adapter committed
- [x] The wiring test, with its own mutation half, and the name-resolution test
- [x] The mutated-carrier test, dropped as redundant against
      `test_release_preparation_metadata_rejects_wasm_tag_and_version_mutations`
- [x] Demonstrate both end-to-end halves, a stale manifest version and the
      reproduced S42 `ci.yml` literal
- [x] Update the `publish.yml` paragraph in `15-build-and-toolchain.md`,
      including the two stale figures
- [x] Correct the stale stable figure in the `12-testing-strategy.md`
      README-inventory paragraph, and nothing else in that file
- [x] `python3 -m unittest scripts.test_sprint_workflow`,
      `python3 scripts/sync_agent_skills.py --check`,
      `/microscope F-X025 --working`, `/verify`

## Open questions

None. The scope question, whether to run the module or the two named tests, is
settled in the plan by the 4-second measurement. The documentation-drift
question went to the S43 consolidated round and was settled as fixing both
instances in this story, which costs F-X025 its place in F-X021's wave.

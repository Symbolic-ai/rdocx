# F-X175, Refresh CLI archive evidence after release hardening

**Status**: completed
**Sprint**: S87
**Size**: S
**Depends on**: F-X174

## Problem

The S86 main CI Docs and Release regressions jobs fail after the reviewed
Windows CLI stack change. `crates/rdocx-cli/README.md:28` and
`crates/rpptx-cli/README.md:23` retain the pre-change archive sizes, mirrored
in `scripts/readme_doctests.py:394`. Fresh Linux package builds measure 70,175
compressed and 311,470 member bytes for `rdocx-cli`, and 40,876 compressed
and 179,264 member bytes for `rpptx-cli`.

## Spec reference

`docs/hld/14-development-backlog.md`, F-X175, defines the release regression
gate. `docs/hld/15-build-and-toolchain.md`, the unified family publication
section, defines the package and release boundary.

## Approach

Regenerate the two CLI source archives from the tracked tree and update their
README measurement rows and the corresponding `ARCHIVE_MEASUREMENTS` entries.
Keep release metadata, application code, workflow, and version carriers unchanged.
Check that the declared macOS measurement date and platform remain accurate
for the regenerated local observations, then compare a fresh Linux hosted run.
Update the roadmap policy assertion so its expected M24 sprint set reflects
the user-approved move of the S87 story wave to S88.

## Rejected alternatives

- Widening the archive size tolerance would hide a stale source inventory.
- Skipping README checks in CI would weaken the release gate.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| package | `build_package_archive` and `validate_archive_measurement` for both CLI crates | Both source archives have current size and member counts |
| regression | `python3 -m unittest scripts.test_sprint_workflow.SprintWorkflowTests.test_readme_depth_footprint_and_speed_claims_are_evidence_backed scripts.test_sprint_workflow.SprintWorkflowTests.test_measurement_rows_match_rederived_archive_footprint` | Enforced rows match rebuilt archives |
| roadmap | `python3 -m unittest scripts.test_sprint_workflow.SprintWorkflowTests.test_m23_m24_roadmap_has_no_duplicate_or_dangling_story` | Every M24 story has one planned sprint after the focused S87 repair |
| release regression | Hosted Docs and Release regressions jobs on the final S87 SHA | Linux package measurements pass |
| integrated | `/verify --full` and hash harness | Full gate passes with 49 unchanged hashes |

The backlog test gate is release regression.

## HLD impact

None. The package inventory rule is unchanged.

## Risk routing

The release scripting and version strings row applies to the measured release
package inventory in `scripts/readme_doctests.py`. Read
`.claude/commands/release.md` and `docs/hld/15-build-and-toolchain.md`, inspect
all manifest, lockfile and README version diffs, run the complete package dry
run and full gate, and require separate final approval before either tag.

## Hash harness

Expected unchanged, with all 49 entries matching.

## Implementation checklist

- [x] Rebuild both CLI archives from the tracked source.
- [x] Update only their two README rows and matching enforced measurements.
- [x] Update the roadmap policy assertion for the S87 to S88 move.
- [x] Run focused README and release-regression checks.

## Open questions

None. The user selected a focused S87 repair and moved the planned related-story
work to S88.

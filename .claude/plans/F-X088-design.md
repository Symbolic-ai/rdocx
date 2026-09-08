# F-X088, Verify and close Issue 69 after S70 fixes

**Status**: completed
**Sprint**: S71
**Size**: S
**Depends on**: F-X084, F-X085, F-X086

## Problem

Issue 69 remains open even though its three independently reported performance
mechanisms were implemented in S70. The current branch contains note-aware
paragraph invalidation, once-per-layout restart identity memoization, and
provenance-safe restart after body-length changes. The public v0.13.1 release
predates those fixes and remains affected.

Closing the issue without verifying the combined S71 state would disconnect the
external report from the exact tested source. Reimplementing the mechanisms
would duplicate completed work and risk changing already reviewed behavior.

## Spec reference

- `docs/hld/08-rendering-spec.md`, paragraph caching, safe restart checkpoints,
  and result-local source provenance.
- `docs/hld/12-testing-strategy.md`, Issue 69 evidence and contribution records.
- `docs/hld/14-development-backlog.md`, "F-X088, Verify and close Issue 69 after
  S70 fixes".
- GitHub Issue 69 and offered commits
  `4777a74167495a5116289e1f905dfd9ad4dbe807`,
  `eff0ea0c28b5eaf08180b09b58e0c0f486b7433b`,
  `9e48bc86876c294b8daa314e577e84b6fcd7ac97`, and
  `c8315b92857c951146fc866cd044b214194a09a8`.

## Approach

Treat this as verification and authenticated issue closure, with no source-code
change expected. Confirm that all three completed S70 commits are ancestors of
S71 and absent from v0.13.1. Run the six existing focused regressions in
deterministic font mode:

- `note_part_changes_invalidate_only_referencing_paragraphs`
- `note_only_invalidation_preserves_unreferenced_entries_for_next_layout`
- `restart_body_identities_are_computed_at_most_once_per_layout`
- `sourced_insert_and_delete_restart_instead_of_repaginating`
- `sourced_length_change_never_reuses_shifted_tail_pages`
- `sourced_enter_merge_and_selection_delete_restart_from_safe_prefix`

Run the complete `/verify --full` gate at the reviewed S71 SHA and require the
49-entry hash harness to remain unchanged. Authenticate the reporter fork and
use its timing harness if committed. When no committed harness exists,
reconstruct the reported 700 four-line paragraphs, one 3 by 3 table every 50
paragraphs, 63-page workload in temporary instrumentation. Compare exact
v0.13.1 and S71 source archives in the same release-mode deterministic-font
environment. Use a warmup and multiple alternating rounds at three body
positions. Report min and median timing, available work counters, exact
environment, and every material difference from the reporter environment. Do
not commit the harness or generated artifacts.

Post Issue 69 evidence that credits `@emptinessform`, identifies the exact
regressions and reviewed SHA, states that v0.13.1 remains affected, and says the
fixes will be included in the next stable release without promising a date.
Close the issue only when correctness and timing evidence both support closure,
then verify its final state. Preserve the issue and four offered commit
identities in the next stable release contribution inventory.

## Rejected alternatives

- Reapply the offered commits. S70 already integrated hardened equivalents with
  regression coverage.
- Close on commit ancestry alone. The issue reports interacting hot paths that
  require combined current-state verification.
- Claim the reporter's exact editor harness was reproduced. Its committed fork
  tree does not contain that harness. The temporary direct-engine reconstruction
  must distinguish matched workload properties from OS, hardware, and editor
  integration differences.
- Claim v0.13.1 contains the fix. Its release SHA predates F-X084 through
  F-X086.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | two note invalidation tests | Only note-referencing paragraphs rebuild and unaffected entries remain reusable. |
| regression | restart identity memo test | Each candidate body identity is computed at most once per layout within the bounded memo. |
| regression | three sourced body edit tests | Insert, delete, Enter, merge, and selection delete reuse a safe prefix without stale tail provenance. |
| release comparison | S71 ancestry and v0.13.1 exclusion | The completed fixes are present in S71 and absent from the released tag. |
| timing comparison | temporary release-mode Issue 69 reconstruction | Exact v0.13.1 and S71 source archives run in one deterministic-font environment with warmup, alternating rounds, three positions, min and median timing, and cache or page work counters. |
| full gate | `/verify --full` | Workspace, deterministic oracle, package, supply-chain, prose, and 49-entry hash gates pass at one reviewed SHA. |
| external state | Issue 69 closure verification | The authenticated comments contain exact correctness and qualified timing evidence plus contributor credit, and the issue is closed as completed. |

The **test gate is regression**. All six focused tests, the full gate, and the
same-environment timing comparison must pass before final external issue
closure.

## HLD impact

- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Layout, pagination, line breaking, and text shaping**. Run deterministic
  bundled-font regressions and require warm-to-fresh output and provenance
  equality.
- **External issue state and contribution credit**. Bind the closure comment to
  the reviewed SHA, verify the final issue state, and retain all four offered
  commit identities for release notification.
- **New files**. This approved design plan is the only new tracked file. No new
  source or test file is expected.

## Hash harness

Expected unchanged across all 49 entries. This story verifies existing warm
layout optimizations and must not change deterministic cold output.

## Implementation checklist

- [x] Confirm F-X084, F-X085, and F-X086 are ancestors of S71 and absent from v0.13.1.
- [x] Run all six focused deterministic-font regressions.
- [x] Run `/verify --full` and confirm the hash harness remains 49 of 49.
- [x] Authenticate the reporter fork and determine whether its timing harness is committed.
- [x] Compare exact v0.13.1 and S71 archives with a temporary release-mode deterministic-font reconstruction.
- [x] Record warmup, alternating rounds, three-position min and median timing, work counters, and environment caveats.
- [x] Record the exact reviewed SHA and evidence in the Issue 69 comment.
- [x] Credit `@emptinessform` and state the v0.13.1 and next stable release facts.
- [x] Close Issue 69 and verify its authenticated final state.
- [x] Preserve Issue 69 and all four offered commits in the next stable release inventory.
- [x] Update exactly the listed HLD files.

## Open questions

None. The user explicitly approved folding Issue 69 closure into S71.

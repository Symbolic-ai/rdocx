# S36 sprint review, pass 2

**Reviewed**: `sprint/s36` at
`3343e1729ef0ef31b1e88f7fb727e2e6c5ddedcf` against merge base
`3c2a019fffccbdd7c7e6465c3c004a74c75dc486`, 66 files, 5,307 insertions
and 966 deletions, crates: `oxml-cli-support`, `rdocx`, `rdocx-cli`,
`rdocx-wasm`, `rpptx`, `rpptx-cli`, and `rpptx-wasm`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the deferred publication record still contradicts itself

`.claude/plans/F-143-design.md:29`

The explicit publication deferral resolves the substantive release-policy gap
from pass 1. S36 is now the v1 implementation gate and registry publication is
deferred at `docs/sprints/SPRINT_PLAN.md:562`. F-X006 is pending in S37 at
`docs/sprints/BACKLOG.md:293`, and its contract requires a fresh version above
0.1.2 while preserving the immutable prior release at
`docs/hld/14-development-backlog.md:1176`. The current architecture likewise
states that the earlier twelve-package release remains at 0.1.2 and the two new
packages remain unpublished at `docs/hld/03-architecture.md:136`.

Two completed design contracts still make the opposite factual claim. F-143
says to create a published `oxml-cli-support` 0.1.2 at the citation above, and
F-144 says to create a published `rpptx-cli` 0.1.2 at
`.claude/plans/F-144-design.md:30`. Their own closing text says that no package
was published at `.claude/plans/F-143-design.md:101` and
`.claude/plans/F-144-design.md:112`. These approaches need the same truthful
publishable and initially unpublished wording now used by AS_BUILT at
`docs/sprints/AS_BUILT.md:5235` and `docs/sprints/AS_BUILT.md:5269`.

The variance record also still says S36 closed the remaining backlog with all
159 stories done at `docs/sprints/SPRINT_TRACKER.md:309`. That conflicts with
the authoritative total of 160 stories, 159 done, and one pending at
`docs/sprints/BACKLOG.md:32`. It also contradicts the S37 F-X006 row cited
above. The record must describe the one deferred release story rather than
claiming that the backlog is closed.

This is the remaining part of pass-1 B1. It is a bounded delivery-record repair.
No implementation or external publication action is required.

**Run-sprint disposition**: `fix-now`.

## Should-fix

None.

## Nice-to-have

None.

## Run-sprint disposition

- `fix-now`: B1. Reconcile the two completed plan approaches and the S36
  variance row with the approved deferral and pending F-X006.
- `tracked-follow-up`: none. F-X006 is already the tracked S37 release story.
- `human-action`: none. The publication-policy decision has been made, and no
  registry mutation is part of this repair.
- `refuted`: the pass-1 claim that the deferral lacked a backlog home. F-X006
  now supplies the fresh-version release contract.

## Earlier finding

### B1, partially resolved

The main sprint and HLD records now agree on the approved policy. S36 delivered
the CLI and local package implementation surfaces without publishing Rust or
npm packages at `docs/sprints/SPRINT_TRACKER.md:53`. The expanded Rust family
must use one fresh common version and a separately approved release at
`docs/sprints/SPRINT_PLAN.md:567`. The immutable `rpptx-v0.1.2` release and its
twelve packages remain unchanged at `docs/hld/14-development-backlog.md:1180`.
The remaining contradictions are exactly those cited in B1.

## Sprint definition of done

All nine technical S36 definition-of-done items continue to hold. The only
delta after the pass-1 reviewed implementation is the pass-1 review artifact
and publication-deferral documentation. No source, manifest, test, workflow,
or package file changed.

- The shared CLI gate, nine presentation commands, seven document command
  integrations, local two-package npm installation, six README examples, sole
  sample generator, concurrent path isolation, full verification, and all 28
  hashes retain the integrated evidence recorded in pass 1.
- All eight S36 rows remain done and unowned at
  `docs/sprints/CURRENT_SPRINT.md:32`. The workflow state also reports eight
  completed stories and review phase.
- The final independent feature reviews remain clean, including F-143 at
  `.claude/reviews/F-143-all-pass-4.md:6`, F-144 at
  `.claude/reviews/F-144-all-pass-5.md:6`, F-145 at
  `.claude/reviews/F-145-all-pass-4.md:5`, and the four clean first or second
  cross-cutting passes.
- Fresh pass-2 checks found zero prose violations, all 25 generated skills in
  sync, no integrated diff-hygiene error, and a clean worktree before this
  review artifact was written.

B1 blocks truthful closure records, not the implemented definition-of-done
gates.

## Milestone gate

The M13 installed-wheel parity gate remains satisfied by the hosted evidence
bound in the prior sprint review at
`.claude/reviews/S35-sprint-review-pass-3.md:80`. S36 changes no Python package
or wheel workflow surface. Its v1 implementation gate is now correctly distinct
from the deferred registry-release work at `docs/sprints/SPRINT_PLAN.md:562`.

## Not found

- **Interaction**: the deferral remediation changes only tracked delivery and
  HLD prose. It introduces no product interaction or cross-story conflict.
- **Duplication**: shared CLI ownership and the sole sample generator remain
  unchanged.
- **Layering**: no crate dependency edge changed after pass 1, and the integrated
  implementation retains no forbidden `oxml-*` dependency direction.
- **Harness**: no generator, renderer, or baseline changed after the fresh
  28-entry pass-1 check.
- **Gate**: no technical S36 gate failure was found.
- **Docs**: apart from B1, CURRENT, BACKLOG, SPRINT_PLAN, SPRINT_TRACKER,
  AS_BUILT, HLD 03, HLD 14, and HLD 15 agree on local implementation,
  publication deferral, and fresh-version release ownership.
- **Dependencies**: no new dependency or publication authority was introduced
  by the remediation.
- **Surface**: no Rust, CLI, WASM, JavaScript, or package surface changed after
  pass 1.
- **Release safety**: no existing tag or registry version is assigned new
  content, and no publication occurred during remediation.

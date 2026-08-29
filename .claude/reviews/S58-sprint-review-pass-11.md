# S58 sprint review, pass 11

**Reviewed**: `sprint/s58` at
`5a51be624679b09cc6d90b67e4f6eb8d04dd43ef` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 140 files, 11,107 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This eleventh pass is the explicitly authorized checkpoint after F-X067 and
F-X065 completion. It audits their integrated interaction, exact HLD scope,
delivery records, contribution evidence, and exact-SHA verification while the
remaining M20 work stays open. Recording the reason here satisfies the
later-pass exception required by `.claude/commands/sprint-review.md:45` and
`.claude/commands/sprint-review.md:86`.

## Blocking

None. 0 blocking findings.

## Should-fix

None. 0 should-fix findings.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this dependency-prefix checkpoint. F-198 is in
progress, F-199 and F-200 remain pending, and F-X060, F-X066, and F-X031 remain
pending at `docs/sprints/CURRENT_SPRINT.md:41` through
`docs/sprints/CURRENT_SPRINT.md:47`. The corresponding language, shaping,
direction, stable-publication, VML, hosted Word, and branch-protection
conditions remain open at `docs/sprints/CURRENT_SPRINT.md:67` through
`docs/sprints/CURRENT_SPRINT.md:98`.

The applicable dependency-prefix gate holds. F-X067 places one exact
`cargo fetch --locked` immediately after the pinned cache and before the corpus
and locked offline harness at `.github/workflows/ci.yml:433` through
`.github/workflows/ci.yml:451`. F-X065 recognizes and preserves the historical
grid while keeping active columns authoritative at
`crates/rdocx-oxml/src/table.rs:685` through
`crates/rdocx-oxml/src/table.rs:815` and
`crates/rdocx-layout/src/table.rs:895`.

The exact external LibreOffice chart and ODT oracles passed at integrated code
SHA `1f15e2774bef1b843d686c2c15a7cef41d1cc929`, whose full result and unchanged
hash are recorded at `.claude/scratch/S58-run.json:355`. The later
`5a51be624679b09cc6d90b67e4f6eb8d04dd43ef` commit changes only sprint delivery
documents. At that record SHA, an active desktop LibreOffice session caused
idle launch stalls, so the record-SHA rerun excluded those two already-proved
external cases. Every other full verification step and the pinned Word gate
passed, and the exact record-SHA result remains 49 of 49 unchanged at
`.claude/scratch/S58-run.json:361`.

## Not found

- **F-X067 correctness, 0 findings**: the workflow has one singular locked
  dependency fetch in the required step position at
  `.github/workflows/ci.yml:423` through `.github/workflows/ci.yml:451`. The
  complete assertion fixes action identities, order, cardinality, offline
  evidence, and failure behavior at `scripts/test_sprint_workflow.py:610`
  through `scripts/test_sprint_workflow.py:663`. Missing, unlocked, duplicated,
  misplaced, and wrong-job mutations are rejected at
  `scripts/test_sprint_workflow.py:671` through
  `scripts/test_sprint_workflow.py:727`. Both focused workflow regressions pass
  independently at this checkpoint.
- **F-X065 correctness and preservation, 0 findings**: grid, column, width, and
  historical-change selection uses the in-scope Word namespace at
  `crates/rdocx-oxml/src/table.rs:701` through
  `crates/rdocx-oxml/src/table.rs:793`. A second modeled change fails closed,
  retained raw children receive necessary ancestor bindings, and serialization
  emits active columns before the historical subtree at
  `crates/rdocx-oxml/src/table.rs:720` through
  `crates/rdocx-oxml/src/table.rs:815`.
- **F-X065 regression strength, 0 findings**: canonical, aliased, foreign,
  ancestor-bound, duplicate, and repeated-round-trip cases are covered at
  `crates/rdocx-oxml/src/table.rs:1969` through
  `crates/rdocx-oxml/src/table.rs:2085`. The additive facade save-reopen test is
  at `crates/rdocx/src/table.rs:805`, and the historical-width isolation test is
  at `crates/rdocx-layout/src/table.rs:894`. All six focused regressions pass
  independently at this checkpoint.
- **Interaction, 0 findings**: F-X067 changes only dependency preparation for
  the existing Word fidelity job. Its offline helper remains locked at
  `scripts/docx_ssim_harness.py:41`. F-X065 adds table-history preservation and
  a native presence query without feeding the historical bytes into layout at
  `crates/rdocx/src/table.rs:606` and
  `crates/rdocx-layout/src/table.rs:895`. Neither path changes the completed
  F-202 restart, F-X062 note eligibility, F-X063 font comparison, or F-X058
  multilingual contracts.
- **HLD scope, 0 findings**: F-X065 lists exactly HLD 04, 08, 10, 12, and 14 at
  `.claude/plans/F-X065-design.md:70`, and those files describe current
  namespace, layout, API, test, and backlog reality at
  `docs/hld/04-opc-and-packaging.md:391`,
  `docs/hld/08-rendering-spec.md:533`,
  `docs/hld/10-bindings-spec.md:354`,
  `docs/hld/12-testing-strategy.md:77`, and
  `docs/hld/14-development-backlog.md:3376`. F-X067 lists exactly HLD 12, 14,
  and 15 at `.claude/plans/F-X067-design.md:77`, with current behavior at
  `docs/hld/12-testing-strategy.md:1193`,
  `docs/hld/14-development-backlog.md:3407`, and
  `docs/hld/15-build-and-toolchain.md:610`. No unlisted HLD file changed for
  either integrated feature.
- **Delivery records, 0 findings**: F-X067 and F-X065 are done with no owner at
  `docs/sprints/CURRENT_SPRINT.md:39`, done in the backlog at
  `docs/sprints/BACKLOG.md:520` and `docs/sprints/BACKLOG.md:522`, and each has
  one tracker row at `docs/sprints/SPRINT_TRACKER.md:339`. Their AS_BUILT
  entries agree on scope, HLD files, review verdicts, gates, and unchanged
  49-entry hashes at `docs/sprints/AS_BUILT.md:9688` through
  `docs/sprints/AS_BUILT.md:9775`. The backlog summary remains arithmetically
  consistent at `docs/sprints/BACKLOG.md:38` through
  `docs/sprints/BACKLOG.md:42`.
- **Contribution evidence, 0 findings**: the plans bind F-X065 to PR 56 SHA
  `8b79c4cd0452defafe0a58e86b332c98e7fe52d7` at
  `.claude/plans/F-X065-design.md:44` and F-X067 to PR 58 SHA
  `c8fed1d1268fd765d602bac2da6524900c1c1cfd` at
  `.claude/plans/F-X067-design.md:43`. The durable records credit
  `@pedroassumpcao` and preserve the open, unchanged claims at
  `docs/sprints/AS_BUILT.md:9700` and
  `docs/sprints/AS_BUILT.md:9743`. Independent read-only GitHub inspection
  confirms both pull requests remain open at those exact head SHAs.
- **Duplication, 0 findings**: F-X067 adds one direct workflow step and extends
  the existing workflow assertion. F-X065 reuses the established namespace and
  raw-binding machinery at `crates/rdocx-oxml/src/table.rs:9` through
  `crates/rdocx-oxml/src/table.rs:16`. Neither feature adds a forwarding-only
  helper, parallel parser, module, or test binary.
- **Layering and deps, 0 findings**: the checkpoint changes no manifest or
  lockfile and introduces no dependency edge. Table parsing remains in
  `rdocx-oxml`, table layout remains in `rdocx-layout`, facade inspection
  remains in `rdocx`, and workflow preparation remains in CI.
- **Harness, 0 findings**: both plans declare 49 of 49 unchanged at
  `.claude/plans/F-X065-design.md:90` and
  `.claude/plans/F-X067-design.md:93`. Both AS_BUILT records agree at
  `docs/sprints/AS_BUILT.md:9724` and `docs/sprints/AS_BUILT.md:9772`, and the
  exact record-SHA verification independently records 49/49 unchanged at
  `.claude/scratch/S58-run.json:361`.
- **Gate, 0 findings**: F-X067 microscope pass 1 reports zero defects and zero
  smells at `.claude/reviews/F-X067-working-pass-1.md:6`. F-X065 microscope
  pass 2 reports zero defects and zero smells after the ancestor-binding
  remediation at `.claude/reviews/F-X065-working-pass-2.md:6`. The focused
  regressions, integrated code-SHA external oracles, record-SHA verification,
  and pinned Word evidence support this prefix without claiming the unfinished
  M20 end gate.
- **Docs, 0 findings**: the plan-listed HLD updates state current behavior and
  ownership without change-history prose. CURRENT_SPRINT keeps the remaining
  dependency order and acceptance conditions explicit at
  `docs/sprints/CURRENT_SPRINT.md:49` through
  `docs/sprints/CURRENT_SPRINT.md:98`.
- **Surface, 0 findings**: `TableRef::has_grid_change()` is the approved
  additive native query at `crates/rdocx/src/table.rs:606`. The public
  `CT_TblGrid` preservation fields and intentional pre-1.0 literal impact are
  documented at `docs/hld/10-bindings-spec.md:354`. F-X067 adds no product API,
  feature flag, action, dependency, or binding surface.

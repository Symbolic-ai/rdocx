# S58 sprint review, pass 14

**Reviewed**: `sprint/s58` at
`71dda36bfaded68f03983f48fa3f45cd0fc0bbd3` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 163 files, 13,730 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This fourteenth pass is the explicitly authorized checkpoint after remediation
of pass 13. The remediation changes the F-198 plan, HLD dependency, and
delivery wording, plus records pass 13. It changes no implementation, manifest,
lockfile, or render baseline.

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
explicitly unclaimed at this dependency-prefix checkpoint. F-199, F-200,
F-X060, and F-X031 remain pending at
`docs/sprints/CURRENT_SPRINT.md:43` through
`docs/sprints/CURRENT_SPRINT.md:47`. Their complex-script, directional,
stable-publication, and repository-protection acceptance conditions remain
open at `docs/sprints/CURRENT_SPRINT.md:71` through
`docs/sprints/CURRENT_SPRINT.md:98`.

The completed F-198 prefix gate remains supported by the deterministic
Poppler 26.01.0 manifest at `scripts/golden_pixel_manifest.json:3` through
`scripts/golden_pixel_manifest.json:16`. The full verification record at the
pre-remediation sprint checkpoint reports the expected five-key delta with all
49 entries matching at `.claude/scratch/S58-run.json:420` through
`.claude/scratch/S58-run.json:424`. Pass-13 remediation changes documentation
and review records only, so it neither extends that evidence nor claims M20.

## Not found

- **Pass-13 S1 closure, 0 findings**: the delivery record now says exactly
  three modeled language attributes and separately retained foreign attributes
  at `docs/sprints/AS_BUILT.md:9834` through
  `docs/sprints/AS_BUILT.md:9836`. This agrees with the approved `val`,
  `eastAsia`, and `bidi` contract at `.claude/plans/F-198-design.md:37` and
  the current low-level binding description at
  `docs/hld/10-bindings-spec.md:99` through
  `docs/hld/10-bindings-spec.md:102`.
- **Pass-13 S2 closure, 0 findings**: the plan and canonical backlog now both
  name F-X066 as an F-198 dependency at
  `.claude/plans/F-198-design.md:6` and
  `docs/hld/14-development-backlog.md:1840`. The plan lists HLD 14 as part of
  its impact at `.claude/plans/F-198-design.md:89` through
  `.claude/plans/F-198-design.md:96`, and the delivery record lists the same
  six HLD files at `docs/sprints/AS_BUILT.md:9857` through
  `docs/sprints/AS_BUILT.md:9860`. The HLD states current dependency reality
  without change-history prose.
- **F-198 interaction, 0 findings**: automatic hyphenation remains part of
  reusable engine-context equality at `crates/rdocx-layout/src/engine.rs:472`
  and `crates/rdocx-layout/src/engine.rs:606`. The compact authoritative
  restart identity remains at `crates/rdocx-layout/src/engine.rs:719` through
  `crates/rdocx-layout/src/engine.rs:799`, and the four related-story tests
  retain their 700-paragraph contracts at
  `crates/rdocx-layout/src/engine.rs:8963` through
  `crates/rdocx-layout/src/engine.rs:9102`. The remediation does not alter
  F-X062, F-X063, or F-X066 behavior.
- **Harness, 0 findings**: the approved output movement remains exactly the
  five `feature_showcase` keys at `scripts/hash_baseline.json:10` through
  `scripts/hash_baseline.json:14`, with its English hyphenation reason at
  `scripts/hash_baseline.json:53`. The AS_BUILT record names the same five
  keys and reason at `docs/sprints/AS_BUILT.md:9872` through
  `docs/sprints/AS_BUILT.md:9875`. No baseline or golden manifest changed in
  the remediation.
- **Docs and delivery records, 0 findings**: F-198 is done in the current
  sprint at `docs/sprints/CURRENT_SPRINT.md:42` and in the backlog at
  `docs/sprints/BACKLOG.md:400`. Its single tracker entry agrees on sprint,
  size, estimate, actual, date, and scope at
  `docs/sprints/SPRINT_TRACKER.md:342`. The revised HLD impact list and
  AS_BUILT list agree exactly.
- **Duplication, layering, deps, and surface, 0 findings**: the remediation
  adds no production helper, crate edge, dependency, feature flag, public API,
  or test binary. Shared line breaking remains owned by `oxml-layout`, as
  documented at `docs/hld/03-architecture.md:135` through
  `docs/hld/03-architecture.md:142`.
- **Gate and structure, 0 findings**: the latest feature review remains clean
  at `.claude/reviews/F-198-correctness-pass-5.md:3` through
  `.claude/reviews/F-198-correctness-pass-5.md:19`. Pass 13 is retained as the
  finding record, and this pass verifies its two documentation corrections
  without changing the reviewed implementation.

# S58 sprint review, pass 13

**Reviewed**: `sprint/s58` at
`001da6df8c73b05c799a008b2d3fc84bea1f2d08` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 162 files, 13,580 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 2 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This thirteenth pass is the explicitly authorized checkpoint after F-198
integration and delivery recording. It audits the new language-aware Word
boundary, the declared output movement, prior retained-layout interactions,
and the dependency-prefix ledgers while later M20 work remains open. Recording
the reason here satisfies the later-pass exception at
`.claude/commands/sprint-review.md:45` through
`.claude/commands/sprint-review.md:87`.

## Blocking

None. 0 blocking findings.

## Should-fix

### S1, the F-198 delivery record invents a fourth modeled language attribute

`docs/sprints/AS_BUILT.md:9834`
`.claude/plans/F-198-design.md:37`
`crates/rdocx-oxml/src/properties.rs:791`

The delivery record says F-198 projects, preserves, and authors all four
language attributes. The approved contract and implementation model exactly
three Word `w:lang` attributes: `val`, `eastAsia`, and `bidi`. Foreign
attributes are retained separately and are not a fourth modeled language
attribute. Change the delivery record to say all three modeled attributes plus
retained foreign attributes, so it agrees with the plan and public fields.

### S2, the canonical F-198 dependency list omits its approved F-X066 dependency

`.claude/plans/F-198-design.md:6`
`docs/hld/14-development-backlog.md:1840`
`docs/sprints/CURRENT_SPRINT.md:57`

The completed design contract depends on F-X066, and the sprint sequencing
places F-X066 before the reconstructed stable Word work. The canonical backlog
entry still lists only F-197 and F-X059. Reconcile the HLD story dependency
with the approved plan and sequencing, including the plan impact and delivery
record if HLD 14 is added to F-198's completed scope. Leaving the three
authorities different makes later dependency planning ambiguous.

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
`docs/sprints/CURRENT_SPRINT.md:47`. Their complex-script, bidirectional,
stable-publication, and repository-protection acceptance conditions remain
open at `docs/sprints/CURRENT_SPRINT.md:71` through
`docs/sprints/CURRENT_SPRINT.md:98`.

The applicable F-198 prefix gate holds. The deterministic manifest binds the
reviewed Writer-matched break to pinned Poppler 26.01.0 at
`scripts/golden_pixel_manifest.json:3` through
`scripts/golden_pixel_manifest.json:16`. The hash manifest changes only the
five declared `feature_showcase` keys at
`scripts/hash_baseline.json:10` through `scripts/hash_baseline.json:14`, with
the matching reason at `scripts/hash_baseline.json:53`. The exact-HEAD full
verification record reports the expected five-key delta with all 49 entries
matching at `.claude/scratch/S58-run.json:420` through
`.claude/scratch/S58-run.json:424`. This evidence proves the dependency prefix
only and does not claim the M20 end gate.

## Not found

- **F-198 correctness, 0 findings**: omission keeps automatic hyphenation off,
  authoring rewrites only the modeled setting, and layout receives the resolved
  boolean at `crates/rdocx-oxml/src/settings.rs:293` through
  `crates/rdocx-oxml/src/settings.rs:315` and
  `crates/rdocx/src/document.rs:5392` through
  `crates/rdocx/src/document.rs:5399`. The parser, schema-order, raw-child,
  collision-safe settings-part, and facade round-trip regressions remain at
  `crates/rdocx-oxml/src/settings.rs:754` through
  `crates/rdocx-oxml/src/settings.rs:800`,
  `crates/rdocx/src/document.rs:10572`, and
  `crates/rdocx/tests/regression_test.rs:8386`.
- **Interaction, 0 findings**: automatic hyphenation participates in reusable
  engine-context equality at `crates/rdocx-layout/src/engine.rs:472` and
  `crates/rdocx-layout/src/engine.rs:606`. The authoritative restart identity
  remains collision-safe at `crates/rdocx-layout/src/engine.rs:719` through
  `crates/rdocx-layout/src/engine.rs:799`. The four F-X062 related-story gates
  retain their 700-paragraph contracts at
  `crates/rdocx-layout/src/engine.rs:8963` through
  `crates/rdocx-layout/src/engine.rs:9102`, while F-X063 font equality and
  F-X066 raw-run exclusion remain intact.
- **Duplication, 0 findings**: Word projects document enablement and effective
  language into the existing F-X058 `HyphenatedText` path. It adds no second
  Liang implementation, source module, test binary, trait, generic, feature
  flag, or forwarding-only wrapper.
- **Layering and deps, 0 findings**: F-198 changes no manifest or lockfile. The
  `hypher` dependency remains owned by published `oxml-layout`, and no
  `oxml-*` crate gains a Word or PowerPoint dependency. The current ownership
  is stated at `docs/hld/03-architecture.md:135` through
  `docs/hld/03-architecture.md:142`.
- **Harness, 0 findings**: the AS_BUILT entry names the same five keys and
  isolated page-one English reason as both manifests at
  `docs/sprints/AS_BUILT.md:9871` through
  `docs/sprints/AS_BUILT.md:9874`. No other baseline key moves.
- **Gate, 0 findings**: the latest feature review reports zero defects, zero
  smells, and zero nitpicks at
  `.claude/reviews/F-198-correctness-pass-5.md:3` through
  `.claude/reviews/F-198-correctness-pass-5.md:19`. It explicitly rechecks the
  canonical restart identity, the four 700-paragraph workloads, cache context,
  registry isolation, and five-key movement at
  `.claude/reviews/F-198-correctness-pass-5.md:23` through
  `.claude/reviews/F-198-correctness-pass-5.md:76`.
- **Docs and delivery records, 0 additional findings**: apart from S1 and S2,
  the implementation changes exactly the five HLD files listed at
  `.claude/plans/F-198-design.md:89` through
  `.claude/plans/F-198-design.md:95`. The AS_BUILT entry lists those same five
  files at `docs/sprints/AS_BUILT.md:9856` through
  `docs/sprints/AS_BUILT.md:9859`. F-198 is done in the current sprint at
  `docs/sprints/CURRENT_SPRINT.md:42`, and its tracker row agrees on sprint,
  size, estimate, actual, date, and scope at
  `docs/sprints/SPRINT_TRACKER.md:342`.
- **Surface, 0 findings**: the approved authoring surface remains limited to
  document enablement and the direct Latin language value at
  `crates/rdocx/src/document.rs:3750` through
  `crates/rdocx/src/document.rs:3762` and `crates/rdocx/src/run.rs:410`
  through `crates/rdocx/src/run.rs:428`. HLD 10 records the intentional
  pre-1.0 low-level struct-literal impacts at
  `docs/hld/10-bindings-spec.md:99` through
  `docs/hld/10-bindings-spec.md:106`.

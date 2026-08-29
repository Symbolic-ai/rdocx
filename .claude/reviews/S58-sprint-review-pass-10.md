# S58 sprint review, pass 10

**Reviewed**: `sprint/s58` at
`836c116768c340087e4bc99cc3159f15ae138f88` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 129 files, 10,072 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This tenth pass is the explicitly authorized checkpoint after F-X064
completion. It audits the newly integrated parser, its three planned HLD
updates, delivery records, and interaction with the previously clean pass-9
prefix. Recording the reason here satisfies the later-pass exception required
by `.claude/commands/sprint-review.md:45` and
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
explicitly unclaimed at this dependency-prefix checkpoint. F-198 is still in
progress, F-199 and F-200 remain pending, and stable release F-X060 remains
pending at `docs/sprints/CURRENT_SPRINT.md:41` through
`docs/sprints/CURRENT_SPRINT.md:45`. The language, complex-script,
bidirectional, stable-publication, and final repository-protection acceptance
conditions remain open at `docs/sprints/CURRENT_SPRINT.md:66` through
`docs/sprints/CURRENT_SPRINT.md:90`.

The applicable F-X064 checkpoint gate holds. The exact parser uses checked
`i32` conversion without floating point at
`crates/rdocx-oxml/src/table.rs:178`, and every affected width path reaches it
at `crates/rdocx-oxml/src/table.rs:200` and
`crates/rdocx-oxml/src/table.rs:331`. The four lexical, namespace, negative,
and canonical round-trip regressions are at
`crates/rdocx-oxml/src/table.rs:1843`,
`crates/rdocx-oxml/src/table.rs:1881`,
`crates/rdocx-oxml/src/table.rs:1918`, and
`crates/rdocx-oxml/src/table.rs:1959`. They pass independently at this review
checkpoint. Full verification at exact HEAD records all 49 hashes unchanged at
`.claude/scratch/S58-run.json:318`.

## Not found

- **F-X064 correctness, 0 findings**: integer and whole-valued decimal lexical
  forms use one private checked parser, while fractional, exponent,
  empty-fraction, overflow, percentage, unit, malformed, and empty forms are
  rejected by the regression matrix at
  `crates/rdocx-oxml/src/table.rs:1918`. Missing width keeps the existing
  default at `crates/rdocx-oxml/src/table.rs:323`.
- **Namespace and OOXML preservation, 0 findings**: public width parsing
  resolves the bound Word namespace before selecting attributes at
  `crates/rdocx-oxml/src/table.rs:317`. Serialization retains fixed `w`
  attributes and canonical integers at `crates/rdocx-oxml/src/table.rs:340`,
  and the round-trip regression proves property order and retained unmodelled
  XML at `crates/rdocx-oxml/src/table.rs:1959`.
- **HLD scope, 0 findings**: the plan lists exactly
  `docs/hld/04-opc-and-packaging.md`, `docs/hld/12-testing-strategy.md`, and
  `docs/hld/14-development-backlog.md` at
  `.claude/plans/F-X064-design.md:65`. Those three files describe current
  parser, namespace, serialization, corpus, and hash reality at
  `docs/hld/04-opc-and-packaging.md:380`,
  `docs/hld/12-testing-strategy.md:68`, and
  `docs/hld/14-development-backlog.md:3355`. No unlisted HLD file changed for
  F-X064.
- **Delivery records, 0 findings**: F-X064 is completed with no owner at
  `docs/sprints/CURRENT_SPRINT.md:38`, done in the backlog at
  `docs/sprints/BACKLOG.md:519`, and recorded once in the tracker at
  `docs/sprints/SPRINT_TRACKER.md:338`. The AS_BUILT entry agrees on scope,
  contribution SHA, review result, exact HLD list, gates, and unchanged 49 of
  49 harness at `docs/sprints/AS_BUILT.md:9646` through
  `docs/sprints/AS_BUILT.md:9682`.
- **Contribution evidence, 0 findings**: the approved plan binds the hardened
  equivalent to PR 55 source SHA
  `056d48fdf23f35e3538ef3d6ff78cf9e3863e3a5` and forbids mutation at
  `.claude/plans/F-X064-design.md:40`. The durable delivery record preserves
  that SHA, credits `@pedroassumpcao`, and records the pull request as open and
  unchanged at `docs/sprints/AS_BUILT.md:9658`.
- **Interaction, 0 findings**: F-X064 depends on the completed F-X059 boundary
  at `.claude/plans/F-X064-design.md:6`, changes only table parsing in
  `rdocx-oxml`, and leaves completed F-202, F-X061, F-X062, F-X063, F-X058,
  and F-X059 recorded intact at `docs/sprints/CURRENT_SPRINT.md:33` through
  `docs/sprints/CURRENT_SPRINT.md:43`. F-X065 remains the explicit next table
  consumer and depends on F-X064 at
  `docs/hld/14-development-backlog.md:3381`.
- **Duplication, 0 findings**: one private parser serves both existing width
  consumers at `crates/rdocx-oxml/src/table.rs:178`. The change adds no second
  parser, forwarding wrapper, module, or test binary.
- **Layering, 0 findings**: the F-X064 prefix changes no manifest or lockfile
  and introduces no dependency edge. The parser remains within its existing
  `rdocx-oxml` owner at `crates/rdocx-oxml/src/table.rs:178`.
- **Harness, 0 findings**: F-X064 declares an unchanged 49 of 49 expectation at
  `.claude/plans/F-X064-design.md:82`, the durable completion record reports
  the same result at `docs/sprints/AS_BUILT.md:9682`, and exact-HEAD full
  verification independently records 49/49 unchanged at
  `.claude/scratch/S58-run.json:318`.
- **Gate, 0 findings**: all four focused regressions pass, F-X064 microscope
  pass 1 reports zero defects, zero smells, and zero nitpicks at
  `.claude/reviews/F-X064-working-pass-1.md:6`, and exact-HEAD full
  verification is recorded as passed at `.claude/scratch/S58-run.json:318`.
  This checkpoint does not infer M20 completion from one parser story.
- **Docs, 0 findings**: the three plan-listed HLD updates describe current
  behavior without change-history prose. The sprint contract keeps the
  remaining table-grid and VML outcomes pending at
  `docs/sprints/CURRENT_SPRINT.md:39` and
  `docs/sprints/CURRENT_SPRINT.md:40`.
- **Deps and surface, 0 findings**: F-X064 adds no dependency, feature flag,
  module, file, binding, or public type. `CT_TblWidth` retains its existing
  public fields and constructors at `crates/rdocx-oxml/src/table.rs:286`, while
  the new helper remains private at `crates/rdocx-oxml/src/table.rs:178`.

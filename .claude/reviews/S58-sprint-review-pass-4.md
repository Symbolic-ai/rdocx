# S58 sprint review, pass 4

**Reviewed**: `sprint/s58` at
`0abca4c9fea0aba892cd1516c024f2b4c27a93f8` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 82 files, 8,388 changed
lines. Crates: `oxml-drawing`, `oxml-layout`, `oxml-pdf`, `rdocx`,
`rdocx-layout`, `rpptx`, `rpptx-layout`, and `rpptx-render`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This fourth pass is the explicitly authorized review after F-X058 integration
and prefix finalization. It audits a new implementation, dependency, font,
backend, HLD, and delivery-ledger delta rather than repeating pass 3 over an
unchanged tree. Recording the reason here satisfies the fourth-pass exception
required by `.claude/commands/sprint-review.md:45` and
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

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It does not
yet hold at this scheduled dependency-prefix checkpoint. F-X059 remains
pending, F-198 is in progress, and F-199 and F-200 remain pending at
`docs/sprints/CURRENT_SPRINT.md:37`,
`docs/sprints/CURRENT_SPRINT.md:41`, and
`docs/sprints/CURRENT_SPRINT.md:42` through
`docs/sprints/CURRENT_SPRINT.md:44`. F-X058 deliberately leaves the stable Word
consumer and final Word oracle acceptance to those later stories, as recorded
at `docs/hld/14-development-backlog.md:3259`. The checkpoint therefore does
not claim the end-of-milestone gate or sprint closure.

The applicable completed-prefix gate holds at the reviewed HEAD. Sprint state
records a passing full verification at the exact SHA with all 49 hashes
unchanged at `.claude/scratch/S58-run.json:203`. The F-X058 delivery record
also names the conditional-hyphen, Arabic, Indic, Thai, CJK, bidi,
DrawingML-direction, rich PowerPoint, source-compatibility, deterministic-font,
workspace, no-default, WASM, documentation, package, archive, and dependency
gates at `docs/sprints/AS_BUILT.md:9587`. The approved exact Arabic shaping
regression is present at `crates/oxml-layout/src/font.rs:2464`, line-local L1
then L2 coverage is present at `crates/oxml-layout/src/line.rs:1698`, and the
multi-style forced-break regression is present at
`crates/rpptx-render/src/text.rs:3167`.

## Not found

- **Interaction, 0 findings**: F-X058 keeps legacy Latin output on `GlyphRun`
  at `docs/hld/08-rendering-spec.md:464` and introduces rich output through an
  already non-exhaustive positioned-element surface at
  `crates/oxml-layout/src/output.rs:297`. F-X063's warm path still reaches its
  font-elided retained-context check only after the authoritative exact font
  load at `crates/rdocx-layout/src/engine.rs:1015`. F-X062 and F-202 restart
  reuse still requires exact header, footer, footnote, and endnote context at
  `crates/rdocx-layout/src/engine.rs:608` and the existing restart eligibility
  and font trace at `crates/rdocx-layout/src/engine.rs:1345`. The shared rich
  path does not weaken those completed contracts.
- **Duplication, 0 findings**: multilingual shaping, breaking, direction, and
  cluster ownership stay in the existing format-neutral layout modules, as
  specified at `docs/hld/03-architecture.md:122`. PowerPoint carries direction
  through the approved sibling resolver result at
  `crates/rpptx-layout/src/context.rs:374` while the established resolver keeps
  returning the existing slide shape at `crates/rpptx-layout/src/context.rs:360`.
  No second font manager, line-layout authority, restart cache, or delivery
  ledger was added.
- **Layering, 0 findings**: the new `hypher`, `icu_segmenter`, and
  `unicode-bidi` edges are all direct dependencies of `oxml-layout` at
  `crates/oxml-layout/Cargo.toml:29`. The HLD names that single format-neutral
  consumer and both portability graphs at
  `docs/hld/15-build-and-toolchain.md:647`. No `oxml-*` manifest gains an
  `rdocx-*` or `rpptx-*` dependency.
- **Harness, 0 findings**: F-202, F-X062, F-X063, and F-X058 each declare 49 of
  49 unchanged at `docs/sprints/AS_BUILT.md:9434`,
  `docs/sprints/AS_BUILT.md:9511`, `docs/sprints/AS_BUILT.md:9547`, and
  `docs/sprints/AS_BUILT.md:9593`. The exact-HEAD full verification record
  independently agrees at `.claude/scratch/S58-run.json:203`, and the sprint
  delta changes no harness script or baseline.
- **Gate, 0 findings**: the completed prefix has direct regressions for
  multilingual logical and visual order at
  `crates/oxml-layout/src/line.rs:1630`, deterministic complex-script shaping
  at `crates/oxml-layout/src/font.rs:2464`, and rich backend consumption at
  `crates/rpptx-render/src/text.rs:3154`. The exact-HEAD full verification
  remains the integrated authority. The remaining M20 Word acceptance is
  correctly left open rather than inferred from shared-layout evidence.
- **Docs, 0 findings**: F-X058 updates exactly the seven HLD files listed by its
  approved plan at `.claude/plans/F-X058-design.md:148`, and its delivery entry
  lists the same seven files at `docs/sprints/AS_BUILT.md:9581`. The additions
  describe current shared behavior and named verification contracts at
  `docs/hld/08-rendering-spec.md:452` and
  `docs/hld/12-testing-strategy.md:923`. The previously completed F-202,
  F-X061, F-X062, and F-X063 impact lists remain consistent with their delivery
  entries beginning at `docs/sprints/AS_BUILT.md:9404`.
- **Deps, 0 findings**: exact `hypher` 0.1.7, `icu_segmenter` 2.3.0, and
  `unicode-bidi` 0.3.18 constraints are declared at `Cargo.toml:111`, with the
  named hyphenation, complex-boundary, and UAX 9 consumers documented at
  `docs/hld/15-build-and-toolchain.md:652`. The dependency diff contains no
  unrelated production package.
- **Surface, 0 findings**: the added public direction, cluster, validated
  segment, rich run, and sibling resolver and renderer entrypoints are the
  surface requested by `.claude/plans/F-X058-design.md:36`. Existing public
  enums were already non-exhaustive, the legacy stable struct-literal fixture
  remains at `crates/oxml-layout/src/line.rs:1821`, and the HLD records that no
  Python, WASM, or CLI authoring surface was added at
  `docs/hld/10-bindings-spec.md:621`.
- **OOXML and assets, 0 findings**: DrawingML direction remains in its typed
  schema position with unknown-attribute coverage at
  `crates/oxml-drawing/src/text/paragraph.rs:1958`. The package includes TTFs,
  licences, notices, and subset provenance at
  `crates/oxml-layout/Cargo.toml:13`, while the family-to-licence assertion is
  at `crates/oxml-layout/src/bundled_fonts.rs:135`. The final F-X058 microscope
  confirms authentic wording, subset reproduction evidence, package inventory,
  and no semantic change after the licence whitespace correction at
  `.claude/reviews/F-X058-working-pass-6.md:24`.
- **Ledgers, 0 findings**: F-202, F-X061, F-X062, F-X063, and F-X058 are done
  in the current sprint at `docs/sprints/CURRENT_SPRINT.md:33` through
  `docs/sprints/CURRENT_SPRINT.md:43`. Their tracker rows and actuals are
  present at `docs/sprints/SPRINT_TRACKER.md:332`, and the backlog agrees at
  `docs/sprints/BACKLOG.md:404` and
  `docs/sprints/BACKLOG.md:513`. Pending consumers and release stories remain
  pending, so the delivery record does not overstate the dependency prefix.

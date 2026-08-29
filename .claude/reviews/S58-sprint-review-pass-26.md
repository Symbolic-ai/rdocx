# S58 sprint review, pass 26

**Reviewed**: `sprint/s58` at
`ac1660409b993398d2c00a84d2d16501b787796f` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 217 files, 25,938 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-cli`, `rdocx-html`, `rdocx-layout`,
`rdocx-opc`, `rdocx-oxml`, `rdocx-pdf`, `rdocx-py`, `rdocx-wasm`,
`rpptx`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-py`, `rpptx-render`, and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This twenty-sixth pass is the explicitly requested final end boundary after the
F-X031 external protection change, live pull-request proofs, cleanup,
integration, and delivery records. Since pass 25, the boundary adds the
pass-25 review, the completed F-X031 plan and working review, two current-intent
HLD updates, and four integrator-owned delivery ledger updates. It changes no
product source, workflow, manifest, dependency, public API, crate, module, or
render baseline.

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

The gate is defined at `docs/hld/14-development-backlog.md:1817` through
`docs/hld/14-development-backlog.md:1818`. **The M20 gate holds and can be
claimed at this final boundary.**

The broader five-document corpus trend remains deliberately advisory. Its
reviewed calibration has one of 18 pages at 0.95 SSIM, and its hard gate is
successful renderer execution plus complete nonempty evidence at
`docs/hld/12-testing-strategy.md:741` through
`docs/hld/12-testing-strategy.md:752`. The declared hard script-fidelity gate is
separate. It requires raw luminance SSIM of at least 0.95 on at least 80 percent
of the Arabic, Devanagari, Thai, Simplified Chinese, and mixed bidirectional
fixture pages at `docs/hld/12-testing-strategy.md:754` through
`docs/hld/12-testing-strategy.md:764`. The reviewed evidence passes five of
five pages. F-199 records the four complex-script results at
`docs/sprints/AS_BUILT.md:9919` through
`docs/sprints/AS_BUILT.md:9927`, and F-200 records the complete five-page set,
including bidirectional text, at `docs/sprints/AS_BUILT.md:9971` through
`docs/sprints/AS_BUILT.md:9978`.

The milestone ledger is complete at 7 of 7 at
`docs/sprints/BACKLOG.md:38`. This evidence satisfies the gate without
misstating the advisory external-corpus trend as a hard threshold.

## Sprint final gate

**The S58 final gate holds and can be claimed at this final boundary.** Every
wave row is terminal at `docs/sprints/CURRENT_SPRINT.md:31` through
`docs/sprints/CURRENT_SPRINT.md:50`. F-X060 is accurately archived as the
immutable partial 0.11.0 attempt, and its recovery and cleanup are delivered by
F-X068, F-X069, and F-X070 as specified at
`docs/sprints/CURRENT_SPRINT.md:64` through
`docs/sprints/CURRENT_SPRINT.md:72`.

The exact reviewed HEAD is clean and has a recorded full verification with 49
of 49 hashes unchanged at `.claude/scratch/S58-run.json:713` through
`.claude/scratch/S58-run.json:717`. The completed F-X031 record confirms that
the same integrated verification included workspace, no-default, WASM,
documentation, README, the exact 22-package dry run, archive-size, and
supply-chain gates at `docs/sprints/AS_BUILT.md:10173` through
`docs/sprints/AS_BUILT.md:10188`. F-198's only deliberate rendering change is
the isolated and reviewed five-key `feature_showcase` delta at
`docs/sprints/AS_BUILT.md:9872` through
`docs/sprints/AS_BUILT.md:9875`.

## Not found

- **F-X031 external protection, proof, and cleanup, 0 findings**: current HLD
  intent records active ruleset `21823007`, default-branch targeting, exact
  required status `CI gate`, and the sole `RepositoryRole` administrator ID 5
  `always` bypass at `docs/hld/15-build-and-toolchain.md:504` through
  `docs/hld/15-build-and-toolchain.md:511`. Independent read-only GitHub
  inspection agrees. PR 59 is closed and unmerged after successful run
  `33275852961`. PR 60 is closed and unmerged after failed run `33276064981`
  produced a failed aggregate gate and blocked merge state. The proof refs are
  absent. The exact job and cleanup evidence is recorded at
  `docs/hld/12-testing-strategy.md:1280` through
  `docs/hld/12-testing-strategy.md:1295` and
  `docs/hld/15-build-and-toolchain.md:513` through
  `docs/hld/15-build-and-toolchain.md:521`.
- **F-X031 contract and delivery ledgers, 0 findings**: the completed plan binds
  the inspected gate to the reviewed SHA, preserves existing protection,
  requires real success and failure probes, and limits bypass authority at
  `.claude/plans/F-X031-design.md:30` through
  `.claude/plans/F-X031-design.md:56`. Its implementation checklist is complete
  at `.claude/plans/F-X031-design.md:97` through
  `.claude/plans/F-X031-design.md:107`, and its HLD diff is exactly the two
  planned files at `.claude/plans/F-X031-design.md:83` through
  `.claude/plans/F-X031-design.md:87`. The story appears once as done in the
  current sprint at `docs/sprints/CURRENT_SPRINT.md:50`, once in the backlog at
  `docs/sprints/BACKLOG.md:486`, once in the tracker at
  `docs/sprints/SPRINT_TRACKER.md:348`, and once in AS_BUILT at
  `docs/sprints/AS_BUILT.md:10149`.
- **Release recovery and post-yank truth, 0 findings**: F-X068 records the
  complete shared 0.8.0 family and verified release evidence at
  `docs/sprints/AS_BUILT.md:9994` through
  `docs/sprints/AS_BUILT.md:10030`. F-X069 records the complete seven-package
  stable 0.11.1 family, six contribution notifications, and registry evidence
  at `docs/sprints/AS_BUILT.md:10044` through
  `docs/sprints/AS_BUILT.md:10091`. Independent crates.io readback agrees that
  exactly `rdocx-opc@0.11.0` and `rdocx-oxml@0.11.0` are yanked, the other five
  0.11.0 stable package endpoints are absent, and all seven 0.11.1 packages are
  live and unyanked. The immutable v0.11.0 tag and absent GitHub release match
  `docs/sprints/AS_BUILT.md:10106` through
  `docs/sprints/AS_BUILT.md:10141`.
- **Multilingual, hyphenation, direction, and output interaction, 0 findings**:
  the shared substrate keeps legacy public shapes and Latin behavior while
  carrying cluster-safe shaping, conditional hyphenation, paragraph direction,
  logical searchable text, and validated positioned glyphs at
  `docs/sprints/AS_BUILT.md:9560` through
  `docs/sprints/AS_BUILT.md:9579`. F-198 preserves source spans and isolates its
  expected output delta. F-199 records focused hyphenation, field, drawing,
  cache, and paragraph-wide UAX 9 interaction coverage at
  `docs/sprints/AS_BUILT.md:9888` through
  `docs/sprints/AS_BUILT.md:9927`. F-200 records the parser, hybrid-line,
  source-less, field, table, note, cache, reflow, and vertical interaction
  coverage at `docs/sprints/AS_BUILT.md:9960` through
  `docs/sprints/AS_BUILT.md:9980`.
- **Incremental layout and retained-state interaction, 0 findings**: the
  thousand-page engine and facade paths rebuild at most two pages while keeping
  the 1,024-entry and byte ceilings, exact retained equality, and safe full
  fallback at `docs/sprints/AS_BUILT.md:9410` through
  `docs/sprints/AS_BUILT.md:9434`. Related-story invalidation remains explicit,
  and warm caller-font reuse removes the redundant 22 MiB comparison only after
  the authoritative exact check at `docs/sprints/AS_BUILT.md:9523` through
  `docs/sprints/AS_BUILT.md:9547`.
- **Reader fixes, XML preservation, and schema ownership, 0 findings**:
  whole-valued decimal measurements use exact non-floating-point parsing and
  reject unsupported unions at `docs/sprints/AS_BUILT.md:9652` through
  `docs/sprints/AS_BUILT.md:9682`. Historical table grids remain preserved but
  inactive for layout, including ancestor namespace bindings, at
  `docs/sprints/AS_BUILT.md:9736` through
  `docs/sprints/AS_BUILT.md:9772`. Legacy VML horizontal rules are classified
  once at the OXML boundary, preserve raw XML, and are not rendered at
  `docs/sprints/AS_BUILT.md:9784` through
  `docs/sprints/AS_BUILT.md:9822`.
- **Dependency, layering, public surface, duplication, HLD discipline, and
  structure, 0 findings**: the integrated delta adds no forbidden dependency
  direction, speculative trait, generic, wrapper, feature flag, crate, or
  module. The public changes are the planned additive or pre-1.0 surfaces, and
  the full package graph verifies. The F-X031 end delta changes only its plan,
  working review, two planned current-intent HLD files, and four delivery
  ledgers. No stale historical narrative was added to an HLD, and no second
  tracker, plan, or completion record was created.
- **Sprint contract, verification, and deterministic harness, 0 findings**:
  all acceptance conditions at `docs/sprints/CURRENT_SPRINT.md:75` through
  `docs/sprints/CURRENT_SPRINT.md:111` have matching integrated evidence. The
  exact-head full verification is current, includes all 22 publishable archives,
  and reports 49 of 49 deterministic entries unchanged after the one isolated
  reviewed F-198 delta.

# S58 sprint review, pass 12

**Reviewed**: `sprint/s58` at
`64b7867ebeede1a9d91f6396ba01b3e043bd6cc6` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 149 files, 12,040 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This twelfth pass is the explicitly authorized checkpoint after F-X066
completion. It audits the integrated parser and facade behavior, the exact HLD
scope, delivery records, contribution evidence, and exact-HEAD verification
while the remaining M20 work stays open. Recording the reason here satisfies
the later-pass exception at `.claude/commands/sprint-review.md:45` through
`.claude/commands/sprint-review.md:87`.

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
progress, F-199 and F-200 remain pending, and F-X060 and F-X031 remain pending
at `docs/sprints/CURRENT_SPRINT.md:42` through
`docs/sprints/CURRENT_SPRINT.md:47`. Their language, shaping, direction,
stable-publication, hosted Word, and branch-protection acceptance conditions
remain open at `docs/sprints/CURRENT_SPRINT.md:67` through
`docs/sprints/CURRENT_SPRINT.md:98`.

The applicable F-X066 prefix gate holds. The parser requires expanded Word,
VML, and Office names, one enabled rectangle, and whitespace-only surrounding
content at `crates/rdocx-oxml/src/text.rs:445` through
`crates/rdocx-oxml/src/text.rs:599`. The facade exposes the approved exact raw
view at `crates/rdocx/src/run.rs:117` through
`crates/rdocx/src/run.rs:185`. Positive, adversarial, ordering, package,
equality, and allocation regressions are present at
`crates/rdocx/src/run.rs:785` through `crates/rdocx/src/run.rs:900` and
`crates/rdocx/tests/regression_test.rs:1898` through
`crates/rdocx/tests/regression_test.rs:1974`.

Full verification is recorded at the integrated implementation SHA and again
at exact review HEAD. Both records report 49 of 49 unchanged at
`.claude/scratch/S58-run.json:387` through
`.claude/scratch/S58-run.json:396`. This proves the dependency prefix only and
does not claim the M20 end gate.

## Not found

- **F-X066 correctness, 0 findings**: classification occurs only for an
  unknown Word `pict` at the OXML parse boundary, keeps the raw bytes intact,
  and records one compact flag at `crates/rdocx-oxml/src/text.rs:829` through
  `crates/rdocx-oxml/src/text.rs:840`. XML, namespace, attribute, duplicate,
  malformed, and unexpected event cases fail to the unsupported result at
  `crates/rdocx-oxml/src/text.rs:455` through
  `crates/rdocx-oxml/src/text.rs:599`.
- **Raw preservation and ordering, 0 findings**: every position mutation
  decodes and retains the semantic flag at
  `crates/rdocx-oxml/src/text.rs:614` through
  `crates/rdocx-oxml/src/text.rs:713`. Serialization decodes the boundary at
  `crates/rdocx-oxml/src/text.rs:1027` through
  `crates/rdocx-oxml/src/text.rs:1038`, while the save and reopen regression
  proves ancestor-bound namespaces, exact subtree bytes, and item order at
  `crates/rdocx/tests/regression_test.rs:1898` through
  `crates/rdocx/tests/regression_test.rs:1933`.
- **Public surface, 0 findings**: the approved borrowed raw accessor and
  `LegacyHorizontalRule` variant are at `crates/rdocx/src/run.rs:117` through
  `crates/rdocx/src/run.rs:185`. The enum remains non-exhaustive, and the
  existing public `CT_R` literal shape is compiled by the regression at
  `crates/rdocx/src/run.rs:784` through `crates/rdocx/src/run.rs:795`.
- **Interaction, 0 findings**: run classification does not enter layout or
  rendering. The cache-safety path still rejects ordinary paragraphs with raw
  run children at `crates/rdocx-layout/src/engine.rs:2240` through
  `crates/rdocx-layout/src/engine.rs:2256`, and header and footer projection
  clears paired raw bytes and positions together at
  `crates/rdocx-layout/src/engine.rs:2280` through
  `crates/rdocx-layout/src/engine.rs:2293`. F-X066 therefore does not weaken
  the completed F-202, F-X062, F-X063, or F-X058 cache, note, font, shaping,
  source, and backend contracts.
- **HLD scope and docs, 0 findings**: the plan lists exactly HLD 04, 08, 10,
  12, and 14 at `.claude/plans/F-X066-design.md:68` through
  `.claude/plans/F-X066-design.md:74`. Those five files state the current parse
  boundary and preservation contract at `docs/hld/04-opc-and-packaging.md:178`,
  the non-rendering boundary at `docs/hld/08-rendering-spec.md:839`, the
  additive native API at `docs/hld/10-bindings-spec.md:468`, the regression
  matrix at `docs/hld/12-testing-strategy.md:62`, and the current story gate at
  `docs/hld/14-development-backlog.md:3392`. No unlisted HLD file changed for
  F-X066, and none of the five additions is change-history prose.
- **Delivery records, 0 findings**: F-X066 is completed with no owner in the
  current sprint at `docs/sprints/CURRENT_SPRINT.md:41` and is done in the
  backlog at `docs/sprints/BACKLOG.md:521`. Its single tracker row agrees on
  size, estimate, actual, date, and scope at
  `docs/sprints/SPRINT_TRACKER.md:341`. The AS_BUILT entry agrees on behavior,
  review history, exact HLD files, gates, and unchanged hashes at
  `docs/sprints/AS_BUILT.md:9778` through
  `docs/sprints/AS_BUILT.md:9826`. The generated backlog totals remain
  arithmetically consistent at `docs/sprints/BACKLOG.md:38` through
  `docs/sprints/BACKLOG.md:42`.
- **Contribution evidence, 0 findings**: the approved plan binds the hardened
  equivalent to PR 57 source SHA
  `44498f042a2290ef40c7a6c26025f38e38e9ce2a` and forbids mutation at
  `.claude/plans/F-X066-design.md:43` through
  `.claude/plans/F-X066-design.md:45`. The durable record credits
  `@pedroassumpcao` and records the pull request as open and unchanged at
  `docs/sprints/AS_BUILT.md:9791` through
  `docs/sprints/AS_BUILT.md:9795`. Independent read-only GitHub inspection
  confirms PR 57 remains open at that exact head SHA.
- **Duplication, 0 findings**: F-X066 reuses the existing scope binding carrier
  at `crates/rdocx-oxml/src/numbering.rs:579` through
  `crates/rdocx-oxml/src/numbering.rs:623` and the existing run parser and
  item iterator. It adds no module, test binary, trait, generic, feature flag,
  forwarding-only wrapper, or parallel parser path.
- **Layering and deps, 0 findings**: classification remains in `rdocx-oxml`,
  native inspection remains in `rdocx`, and layout only observes the existing
  raw-child presence boundary. F-X066 changes no manifest or lockfile and adds
  no dependency edge.
- **Harness and gate, 0 findings**: the plan requires 49 of 49 unchanged at
  `.claude/plans/F-X066-design.md:85` through
  `.claude/plans/F-X066-design.md:88`, the AS_BUILT record agrees at
  `docs/sprints/AS_BUILT.md:9815` through
  `docs/sprints/AS_BUILT.md:9822`, and the exact-HEAD full verification record
  agrees at `.claude/scratch/S58-run.json:393` through
  `.claude/scratch/S58-run.json:396`. Microscope pass 3 reports zero defects,
  zero smells, and zero nitpicks at
  `.claude/reviews/F-X066-working-pass-3.md:3` through
  `.claude/reviews/F-X066-working-pass-3.md:18`.

# S58 sprint review, pass 16

**Reviewed**: `sprint/s58` at
`13b620fcf11dffd1b36e89e7cf221aa60a5a3c91` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 187 files, 22,417 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This sixteenth pass is the explicitly authorized dependency-prefix checkpoint
after F-200 integration and delivery recording. Pass 15 was clean. The
incremental feature delta adds the reviewed bidirectional Word consumer and its
fifth oracle page, then records the completed delivery state. The full sprint
delta remains in scope for interaction review.

## Blocking

### B1, the named F-200 golden test does not exercise the renderer or oracle

`.claude/plans/F-200-design.md:89`
`scripts/docx_ssim_harness.py:1093`
`.claude/commands/complete-feature.md:25`

The design contract names
`rtl_corpus_document_matches_the_reviewed_oracle` as the golden gate and says
it proves the pinned visual result. The test only builds the fixture and checks
that its source XML contains direction, alignment, indentation, and text
tokens. It never calls the Rust renderer, LibreOffice, SSIM scoring,
`run_gate`, or the recorded evidence assertion. It therefore still passes if
the complete production bidi implementation is reverted, which the completion
workflow explicitly refuses.

The exact-head five-page oracle result is real and remains recorded separately,
so this finding does not dispute the current pixels. Repair the durable test
contract by making the approved story gate the real
`docx_ssim_harness.py --check` path, or by otherwise binding the named gate to
the rendered oracle evidence. Then demonstrate that the corrected gate fails
with the F-200 production behavior reverted and passes at the remediation SHA.
Do not change the raw 0.95 threshold or 80 percent coverage rule.

## Should-fix

None. 0 should-fix findings.

## Nice-to-have

None. 0 nice-to-have findings.

## Milestone gate

The M20 end gate is:

> The Word corpus renders at the declared SSIM threshold, and text shaping is
> correct for the scripts the corpus contains.

The gate is defined at `docs/hld/14-development-backlog.md:1817`. It remains
explicitly unclaimed at this scheduled dependency-prefix checkpoint. All seven
M20 backlog rows are done at `docs/sprints/BACKLOG.md:393` through
`docs/sprints/BACKLOG.md:405`, but F-X060 and F-X031 remain pending at
`docs/sprints/CURRENT_SPRINT.md:46` through
`docs/sprints/CURRENT_SPRINT.md:47`. Their stable-publication and protected
branch conditions remain open at `docs/sprints/CURRENT_SPRINT.md:75` through
`docs/sprints/CURRENT_SPRINT.md:76` and
`docs/sprints/CURRENT_SPRINT.md:92` through
`docs/sprints/CURRENT_SPRINT.md:96`. B1 also prevents a final gate claim at
this SHA.

The completed F-200 prefix does have exact-head visual evidence. The full
verification record reports 49 of 49 unchanged and five of five multi-script
and bidirectional pages at raw SSIM 0.95 or better at
`.claude/scratch/S58-run.json:473` through
`.claude/scratch/S58-run.json:482`. The HLD records the five exact scores and
unchanged threshold at `docs/hld/12-testing-strategy.md:754` through
`docs/hld/12-testing-strategy.md:762`. This supports the current dependency
prefix, but it does not cure B1 or claim the M20 end gate.

## Not found

- **F-198, F-199, and F-200 interaction, 0 additional findings**: the
  production regression keeps RTL-first Arabic and hyphenatable English in one
  visual bidi paragraph while retaining logical extraction at
  `crates/rdocx-layout/src/engine.rs:7602` through
  `crates/rdocx-layout/src/engine.rs:7662`. A second regression retains the
  conditional hyphen when rich Arabic activates F-199 shaping at
  `crates/rdocx-layout/src/engine.rs:8807` through
  `crates/rdocx-layout/src/engine.rs:8829`. The package-backed PDF and SVG gate
  covers the same hybrid interaction at
  `crates/rdocx/tests/integration_test.rs:5483` through
  `crates/rdocx/tests/integration_test.rs:5559`.
- **F-X062 restart and related-story interaction, 0 findings**: the production
  table and header regressions traverse the real retained containers, preserve
  resolved right-to-left bases, and rebind current source paths at
  `crates/rdocx-layout/src/engine.rs:10189` through
  `crates/rdocx-layout/src/engine.rs:10328`. Footnote and endnote PDF and SVG
  extraction retain logical field order at
  `crates/rdocx/tests/integration_test.rs:5605` through
  `crates/rdocx/tests/integration_test.rs:5675`.
- **Parser, OOXML, and raw preservation, 0 findings**: the Word round trip
  proves typed `w:bidi` and `w:rtl` around foreign content at
  `crates/rdocx-oxml/src/properties.rs:2687` through
  `crates/rdocx-oxml/src/properties.rs:2729`. Interleaved valid and malformed
  occurrences preserve their relative positions through repeated
  serialization at `crates/rdocx-oxml/src/properties.rs:2799` through
  `crates/rdocx-oxml/src/properties.rs:2856`.
- **HLD scope and current-state wording, 0 findings**: the plan lists exactly
  HLD 03, 05, 08, 10, and 12 at
  `.claude/plans/F-200-design.md:95` through
  `.claude/plans/F-200-design.md:101`. The delivery record lists the same five
  files at `docs/sprints/AS_BUILT.md:9967` through
  `docs/sprints/AS_BUILT.md:9969`. Those files describe the current logical and
  visual ordering boundary, typed DrawingML direction, intentional pre-1.0 Word
  fields, and the retained quarter-turn approximation at
  `docs/hld/03-architecture.md:127` through
  `docs/hld/03-architecture.md:145`,
  `docs/hld/05-drawingml-model.md:207` through
  `docs/hld/05-drawingml-model.md:214`, and
  `docs/hld/08-rendering-spec.md:459` through
  `docs/hld/08-rendering-spec.md:494`.
- **Delivery and harness records, 0 findings**: F-200 is done in the current
  sprint at `docs/sprints/CURRENT_SPRINT.md:45` and in the backlog at
  `docs/sprints/BACKLOG.md:402`. Its tracker entry agrees on sprint, size,
  estimate, actual, date, and scope at
  `docs/sprints/SPRINT_TRACKER.md:344`. The AS_BUILT entry records the same five
  scores, 22-package gate, and unchanged 49 of 49 result at
  `docs/sprints/AS_BUILT.md:9971` through
  `docs/sprints/AS_BUILT.md:9980`.
- **Duplication, layering, deps, surface, and structure, 0 findings**: F-200
  consumes the shared multilingual direction and line-ordering model described
  at `docs/hld/03-architecture.md:127` through
  `docs/hld/03-architecture.md:142`. Its incremental integration changes no
  Cargo manifest or lockfile, adds no module or test binary, and keeps direction
  transport private where public aggregate shapes are exhaustive. The binding
  specification records only the approved pre-1.0 Word property additions at
  `docs/hld/10-bindings-spec.md:655` through
  `docs/hld/10-bindings-spec.md:664`.
- **Panic and error paths, 0 findings**: the final feature review found no new
  public-input panic, unchecked indexing, arithmetic overflow, or suppressed
  parser error and reported zero defects, zero smells, and zero nitpicks at
  `.claude/reviews/F-200-working-pass-11.md:3` through
  `.claude/reviews/F-200-working-pass-11.md:19`.

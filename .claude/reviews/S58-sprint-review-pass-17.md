# S58 sprint review, pass 17

**Reviewed**: `sprint/s58` at
`a46b858e1b56345add7662abcdfff6a5bfcbad87` against merge base
`4a37c6791ca2606df36db94e1fe713722d7bd600`, 188 files, 22,617 changed
lines. Crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx`, `rdocx-layout`, `rdocx-oxml`, `rdocx-wasm`, `rpptx`,
`rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`, `rpptx-render`,
and `rpptx-wasm`.
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Bound extension

**Extension reason**: scheduled dependency-prefix boundary

This seventeenth pass is the explicitly authorized review of pass 16 B1
remediation at the same dependency-prefix boundary. The remediation changes
only the DOCX SSIM harness and retains pass 16 as the finding record. No
product crate, manifest, HLD, plan, delivery ledger, or render baseline changes.

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
explicitly unclaimed at this scheduled dependency-prefix checkpoint. F-X060
and F-X031 remain pending at `docs/sprints/CURRENT_SPRINT.md:46` through
`docs/sprints/CURRENT_SPRINT.md:47`. Their stable-publication and protected
branch conditions remain open at `docs/sprints/CURRENT_SPRINT.md:75` through
`docs/sprints/CURRENT_SPRINT.md:76` and
`docs/sprints/CURRENT_SPRINT.md:92` through
`docs/sprints/CURRENT_SPRINT.md:96`.

The dependency prefix has exact-head evidence. The full verification record at
`a46b858e1b56345add7662abcdfff6a5bfcbad87` reports 49 of 49 unchanged and the
F-200 named gate bound to five of five rendered SSIM pages plus the
below-threshold mutation control at `.claude/scratch/S58-run.json:491` through
`.claude/scratch/S58-run.json:495`. This supports the current checkpoint but
does not claim the final M20 or sprint end gate.

## Not found

- **Pass-16 B1 closure, 0 findings**: the XML fixture assertion now has the
  accurate structural name at `scripts/docx_ssim_harness.py:1111`. The
  plan-named `rtl_corpus_document_matches_the_reviewed_oracle` test requires a
  corpus evidence path and skips when that evidence is absent at
  `scripts/docx_ssim_harness.py:1125` through
  `scripts/docx_ssim_harness.py:1129`. The `--check` flow first renders and
  scores through `run_gate`, then sets that exact evidence path and reruns the
  named test at `scripts/docx_ssim_harness.py:1237` through
  `scripts/docx_ssim_harness.py:1244`. The named golden gate therefore no
  longer passes on fixture XML alone.
- **Bidirectional hard threshold, 0 findings**: the evidence assertion reads
  the emitted results table, selects every bidirectional page, refuses missing
  evidence, and rejects any score below the unchanged 0.95 target at
  `scripts/docx_ssim_harness.py:739` through
  `scripts/docx_ssim_harness.py:754`. The focused mutation uses 0.949999999 and
  requires that rejection at `scripts/docx_ssim_harness.py:1131` through
  `scripts/docx_ssim_harness.py:1153`.
- **Overall 80 percent rule, 0 findings**: `multi_script_gate_met` still uses
  the common coverage calculation without changing either constant at
  `scripts/docx_ssim_harness.py:675` through
  `scripts/docx_ssim_harness.py:683`. The self-test retains the exact four of
  five pass and three of five fail controls at
  `scripts/docx_ssim_harness.py:1088` through
  `scripts/docx_ssim_harness.py:1109`. The real gate still refuses the run when
  aggregate multi-script coverage is below 80 percent at
  `scripts/docx_ssim_harness.py:870` through
  `scripts/docx_ssim_harness.py:897`.
- **Exact evidence and oracle identity, 0 findings**: `run_gate` creates the
  five fixtures, runs both renderers, scores their union pages, and writes the
  results and evidence before the named assertion can run at
  `scripts/docx_ssim_harness.py:757` through
  `scripts/docx_ssim_harness.py:897`. The established pinned environment and
  five exact reviewed scores remain current at
  `docs/hld/12-testing-strategy.md:754` through
  `docs/hld/12-testing-strategy.md:762`.
- **F-200 delivery and HLD consistency, 0 findings**: the remediation changes
  neither the completed delivery claims nor the five-file HLD scope. The
  AS_BUILT record continues to name the real five-page raw oracle result and
  unchanged 49 of 49 harness at `docs/sprints/AS_BUILT.md:9971` through
  `docs/sprints/AS_BUILT.md:9980`. The plan and delivery record still agree on
  HLD 03, 05, 08, 10, and 12 at `.claude/plans/F-200-design.md:95` through
  `.claude/plans/F-200-design.md:101` and
  `docs/sprints/AS_BUILT.md:9967` through
  `docs/sprints/AS_BUILT.md:9969`.
- **Interaction, duplication, layering, deps, surface, and structure, 0
  findings**: the remediation adds one direct evidence assertion in the
  existing harness and two existing-module tests. It changes no production
  path, dependency, feature flag, module, test binary, or public API. The prior
  clean F-200 interaction audit remains represented by the production
  hyphenation and bidi regression at
  `crates/rdocx/tests/integration_test.rs:5483` through
  `crates/rdocx/tests/integration_test.rs:5559`, and the final feature review
  remains clean at `.claude/reviews/F-200-working-pass-11.md:3` through
  `.claude/reviews/F-200-working-pass-11.md:19`.

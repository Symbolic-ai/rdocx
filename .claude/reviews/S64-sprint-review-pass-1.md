# S64 sprint review, pass 1

**Reviewed**: clean `sprint/s64` at
`df67f1388ed815c67281f438b08f04cbe6ee340c` against
`0582da0a38886f5ceeb65ab9afcd0797f6fa14b0`, 79 files, 14,334 changed
lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`, `oxml-drawing`,
`oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`, `oxml-sml`,
`rdocx-wasm`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`, `rpptx-oxml`,
`rpptx-render`, `rpptx-wasm`, and `rpptx`
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, the M21 representative-deck gate has not been performed

`docs/hld/14-development-backlog.md:1896`

M21 requires one representative modern deck to round-trip comments, sections,
SmartArt, media, animation timeline, signatures, and package variant without
repair, with static frames, animated export, notes, and handouts matching the
pinned PowerPoint oracle. The prior sprint explicitly left that gate open at
`docs/sprints/SPRINT_TRACKER.md:80`. S64 completes the last two M21 backlog rows
at `docs/sprints/BACKLOG.md:423`, but the integrated delta supplies only
feature-specific HTML, PDF, and earlier M21 regressions. It contains no one-deck
round-trip and oracle observation covering the required union. The release
notes nevertheless say that the M21 boundary is complete at `CHANGELOG.md:11`,
and this sprint requires the reviewed release before closure at
`docs/sprints/CURRENT_SPRINT.md:64`.

The release and sprint closure must remain blocked until one representative
deck exercises the complete required state and all four output classes against
the pinned PowerPoint oracle without repair. The evidence must identify the
exact source deck, PowerPoint build, round-trip observation, and output
comparisons. If that cannot be established, M21 and the release claim must stay
open.

## Should-fix

### S1, the generated backlog summary still reports both S64 stories as pending

`docs/sprints/BACKLOG.md:39`

The summary reports M21 as 13 done and 2 pending, while the authoritative M21
rows mark all 15 stories done at `docs/sprints/BACKLOG.md:412`. The overall
summary at `docs/sprints/BACKLOG.md:42` is consequently also two stories behind.
Regenerate the backlog summary so the M21 and overall counts agree with the
rows before the delivery ledger is used for closure.

## Nice-to-have

None. Count: 0.

## Milestone gate

The gate requires "one representative modern deck" to round-trip the complete
modern-content union without repair and requires its static frames, animated
export, notes, and handouts to match the pinned PowerPoint oracle at
`docs/hld/14-development-backlog.md:1896`. It does not hold. The repository
records that it was still open after S63 at `docs/sprints/SPRINT_TRACKER.md:80`,
and no S64 test or recorded observation performs the combined gate. The focused
timeline oracle remains a separate animation test at
`crates/rpptx/tests/integration.rs:16090`. It does not round-trip or compare the
representative deck required by the milestone gate.

## Not found

- No interaction defect was found between F-224 and F-225. Their sequencing
  contract requires shared image, text, path, link, diagnostic, and
  transactional semantics at `docs/sprints/CURRENT_SPRINT.md:42`. The two
  imports use the existing `Presentation` facade and remain behind their named
  `default-template` and `render` features at `crates/rpptx/Cargo.toml:20`.
- No actionable duplication was found. The import implementations remain
  source-specific and reuse the existing PresentationML authoring, layout,
  package, and rendering layers required at
  `docs/sprints/CURRENT_SPRINT.md:60`.
- No layering violation was found. The new direct dependencies have named
  consumers: `scraper` belongs to `default-template` and `lopdf` belongs to
  `render` at `crates/rpptx/Cargo.toml:23`. No `oxml-*` crate gained an
  `rdocx-*` or `rpptx-*` dependency.
- No unexplained harness delta was found. F-X074 declares all 49 entries
  unchanged at `.claude/plans/F-X074-design.md:114`, its reviewed evidence
  records the unchanged gate at `.claude/reviews/F-X074-correctness-pass-1.md:49`,
  and the exact reviewed tree passes the 49-entry harness check.
- No unplanned public surface or HLD scope drift was found. The F-224 and F-225
  HLD impact lists name the same nine changed HLD files at
  `.claude/plans/F-224-design.md:160` and `.claude/plans/F-225-design.md:185`.
  F-X074's five release HLD files are the expected subset at
  `.claude/plans/F-X074-design.md:88`.
- No release-family, packaging-authority, or contribution-inventory defect was
  found apart from blocking gate B1. The exact 15-package family is named at
  `CHANGELOG.md:53`, the stable and binding exclusions are stated at
  `CHANGELOG.md:59`, and the empty selected-family contribution inventory is
  stated at `CHANGELOG.md:66`. The regression checks the exact selected family,
  stable 0.11.1 isolation, and unpublished WASM carrier at
  `scripts/test_sprint_workflow.py:5033`. F-X074 correctly remains in progress
  at `docs/sprints/CURRENT_SPRINT.md:36`, with tagging and publication reserved
  for separate approval at `.claude/plans/F-X074-design.md:127`.

# S32 sprint review, pass 3

**Reviewed**: `sprint/s32` at
`dc1120b5c342969b962877592604e6822c521231` against
`ad7661152b266462134ce0de4d0d88744191a32e`, 37 files, 3,740 changed lines,
crates: `oxml-layout`, `rpptx-oxml`, `rpptx-layout`, `rpptx-chart`,
`rpptx-render`, `rpptx`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M12 gate is: "a chart created by rpptx opens in PowerPoint, its data is
editable, and it renders."

The gate holds. Microsoft PowerPoint 16.104 build 16.104.25121423 opened the
SHA-256-bound authored chart without repair, and Edit Data showed the authored
Category, Revenue, and Cost values at `docs/sprints/AS_BUILT.md:4120`. The
bound digest is also pinned by the manual acceptance test at
`crates/rpptx/tests/integration.rs:55`.

The passing `authored_chart_relationship_enters_presentation_renderer` test
starts from the authored package, calls the production package renderer, and
requires finite native paths and shaped labels at
`crates/rpptx/tests/integration.rs:508`. The colour gate covers the four mapped
accents at `crates/rpptx-chart/src/lib.rs:10782`, and the passing package
fallback gate covers compatible PNG and JPEG previews plus visible labels for
unusable previews at `crates/rpptx/tests/integration.rs:552`. The focused chart,
layout, package, and preservation checks pass, including the three pinned
LibreOffice viewer tests. The fresh hash-harness check reports all 28 entries
unchanged. Every S32 PresentationML crate remains version 0.0.0 with
`publish = false`, including `crates/rpptx/Cargo.toml:3`,
`crates/rpptx/Cargo.toml:12`, `crates/rpptx-chart/Cargo.toml:3`, and
`crates/rpptx-chart/Cargo.toml:12`.

## Tracker closure

The S32 summary is arithmetically consistent. The two completed feature rows
record estimates of 2 and 1 days and actuals of 1 and 1 days at
`docs/sprints/SPRINT_TRACKER.md:176`. These sum to the summary's 2 planned, 2
done, 0 carried, 3 estimated days, and 2 actual days at
`docs/sprints/SPRINT_TRACKER.md:47`. The velocity is therefore
`2 / 2 * 5 = 5.00`, as recorded at `docs/sprints/SPRINT_TRACKER.md:222`.

Two actual days against three estimated days is a 33.3 percent variance, so the
escalation row is required. Its 33 pending stories and about 7 active weeks at
five stories per week agree with the backlog total and rounding at
`docs/sprints/SPRINT_TRACKER.md:258` and `docs/sprints/BACKLOG.md:33`. The M12
summary independently records 12 of 12 stories done at
`docs/sprints/BACKLOG.md:30`.

## Not found

- `interaction`: the pass-1 colour-map precedence defect remains fixed. The
  end-to-end regression distinguishes master, layout, and slide mappings at
  `crates/rpptx/tests/integration.rs:214` before the chart colour resolver uses
  the effective map.
- `duplication`: one colour resolver, native chart renderer, scoped package
  assembly path, and fallback selector serve the integrated route.
- `layering`: the new `rpptx-layout` to `rpptx-chart` dependency remains within
  the PresentationML family. No `oxml-*` crate gained a format-specific edge.
- `harness`: both S32 AS_BUILT entries declare no delta, and the fresh harness
  check reports 28 matches.
- `gate`: the manual PowerPoint and Edit Data observation is SHA-bound, while
  native rendering, exact series colours, cached images, labelled fallbacks,
  diagnostics, and preservation have passing executable evidence.
- `docs`: F-127 updated HLD 09. F-128 updated HLD 03, 06, 07, 08, and 09 as
  approved. No additional HLD contradiction was found.
- `deps`: `rpptx-layout` consumes `rpptx-chart` for native projection. The
  `rpptx` development dependency on `miniz_oxide` supports bounded PNG preview
  admission in the shared package renderer.
- `surface`: the chart resource types, read-only OOXML projections, frozen
  group content, and shared-font lowering entry point are consumed by the
  approved presentation rendering path. No unrelated public API was found.
- `fallback and preservation`: relationship scope, bounded cached-image
  admission, stable diagnostics, labelled fallback, and raw ChartML and
  alternate-content preservation remain intact.
- `tracker`: the S32 summary, feature rows, velocity, estimate-variance
  escalation, backlog total, and M12 completion count agree.

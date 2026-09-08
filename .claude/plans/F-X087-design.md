# F-X087, Portable authored Word charts from PR 71

**Status**: approved
**Sprint**: S71
**Size**: L
**Depends on**: F-158, F-245, F-249

## Problem

Kevin Brown's PR 71 adds authored Word chart features that are not present on
S71, but its branch predates the current sprint foundations. The current shared
`ChartData` surface lacks axis titles and an RGB palette
(`crates/oxml-chart/src/lib.rs:137`). Word chart insertion exists
(`crates/rdocx/src/document.rs:6890`), but PR 71's direct package edits do not
stage the complete typed theme and deterministic identifier state introduced by
F-245 and F-249. Its original F-X077 identity is already occupied by strict XML
validation, and its S65 delivery records are stale.

The contribution must be folded into S71 as a hardened equivalent, not copied
as an unreviewed historical branch. Authorship and credit must remain explicit.

## Spec reference

- `docs/hld/04-opc-and-packaging.md`, package relationship ownership, content
  types, deterministic part allocation, and atomic mutation.
- `docs/hld/05-drawingml-model.md`, shared RGB colours and Office themes.
- `docs/hld/09-charts-spec.md`, chart authoring, caches, schema order, and
  editable workbook requirements.
- `docs/hld/10-bindings-spec.md`, native facade and re-export stability.
- `docs/hld/12-testing-strategy.md`, differential and cross-viewer evidence.
- `docs/hld/14-development-backlog.md`, "F-X087, Portable authored Word charts
  from PR 71".
- GitHub PR 71 at head `36641cff5a66d80a2e20a925a136b3a19011cbe9`.

## Approach

Port only the production and test behavior from PR 71. Do not import its stale
sprint ledgers, design identity, reviews, or generated artifacts. Extend the
existing concrete `ChartData` with optional category and value axis titles and
a `Vec<RgbColor>` palette. Derive `Default` so existing callers can migrate
without a wrapper or builder.

Serialize line and bar axes explicitly, including titles and value-axis number
formats, in XSD sequence. Apply palette colours deterministically to series and
category points. Preserve pie and doughnut legends, percentage labels, and an
explicit doughnut hole while leaving numeric caches on the existing `General`
omission path.

Re-export the existing `oxml_drawing::color::RgbColor` type through
`oxml-chart`, then through the `rdocx` and `rpptx` facades. Do not introduce a
second colour type.

Stage the complete `Document`, package, typed theme state, and deterministic
identifier owner before publication. A reusable theme requires an internal
relationship to an existing part, the exact theme content type, and successful
DrawingML theme parsing. A missing, mistyped, or malformed theme receives a
collision-safe Office default on the staged candidate. Retained malformed or
mistyped source bytes remain unchanged unless the staged mutation succeeds.

## Rejected alternatives

- Merge PR 71 directly to `main`. Only `/close-sprint` may merge `main`, and
  the contribution requires reconciliation with the active S71 foundations.
- Cherry-pick the PR's sprint records. F-X077 and S65 already describe other
  completed work.
- Accept a correctly typed but malformed theme. Content type alone does not
  establish a valid typed package graph.
- Add an rdocx-specific RGB wrapper. The shared DrawingML type already has
  active Word and PowerPoint consumers.
- Publish package edits before typed theme and identifier state. That would
  permit partial mutation after validation failure.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `word_authored_line_and_bar_charts_reopen_with_axes_and_palette` | Axis titles, visible axes, number formats, deterministic series colours, caches, and workbook data survive save and reopen. |
| round-trip | `word_authored_pie_and_doughnut_charts_reopen_with_point_colours` | Category-point colours, legends, percentages, and doughnut hole remain typed and editable. |
| regression | `word_chart_theme_validation_is_atomic` | Missing, mistyped, and malformed correctly typed themes fail or are replaced on a staged candidate without changing retained source bytes. |
| regression | `word_chart_theme_reuse_keeps_typed_document_state_consistent` | `Document::theme`, `set_theme`, related bytes, and reopened theme state agree after reuse and allocation. |
| regression | `word_chart_allocation_is_deterministic` | Equivalent construction orders allocate identical chart, workbook, theme, relationship, and drawing identifiers. |
| facade | `chart_rgb_colour_is_reexported_by_all_three_facades` | The same public `RgbColor` compiles through `oxml-chart`, `rdocx`, and `rpptx`. |
| differential | reviewed Word and Pages fixture | The exact candidate opens without repair, stays editable, renders the declared visual facts, and exports with exact chart and workbook data. |

The **test gate is differential**. Source-built line, bar, pie, and doughnut
documents must survive save and reopen, then pass the pinned Word and Pages
observations at the reviewed implementation SHA without an unexplained hash
delta.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/09-charts-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Theme colour and inheritance resolution**. Verify the Office theme, direct
  RGB overrides, and deterministic series and point palette precedence.
- **Any parser or serializer**. Test XSD child order, fixed write prefixes,
  prefix-tolerant reads, and byte-exact retention of unmodeled content.
- **Layout, pagination, line breaking, and text shaping**. Use deterministic
  bundled fonts for every render comparison.
- **Dependency direction or crate wiring**. Run the crate DAG check and keep
  the existing documented `oxml-drawing -> rdocx-oxml` exception as the only
  reverse edge.
- **Public API of a published crate**. Record the intentional pre-1.0
  `ChartData` struct-literal break, document `Default` migration, run package
  dry-runs, and assert archive sizes.
- **External oracle**. Bind Word 16.112.2 build 16.112.26082125 and Pages 14.5
  build 7045.0.17 evidence to the exact candidate SHA and file digest.
- **New files**. This approved design plan is the only new tracked file before
  implementation. Production and tests stay in existing modules.

## Hash harness

Expected unchanged across all 49 entries. Existing sample generators do not
exercise the new chart authoring options. Any observed delta is blocking.

## Semver impact

Adding fields to the public pre-1.0 `ChartData` struct intentionally breaks
external struct literals. Deriving `Default` is the migration path. Re-exporting
the existing `RgbColor` type is additive. No Python, WASM, or other binding
entrypoint changes in this story.

## Implementation checklist

- [ ] Port only reviewed PR 71 production behavior and preserve Kevin Brown's credit.
- [ ] Extend `ChartData` and serialize axes, titles, number formats, legends, palettes, and doughnut holes in schema order.
- [ ] Re-export the one shared `RgbColor` through all three facades.
- [ ] Stage complete document, theme, package, and identifier state atomically.
- [ ] Reject or safely replace mistyped and malformed related themes.
- [ ] Add deterministic allocation, facade, round-trip, and failure regressions.
- [ ] Record Word and Pages evidence at the exact candidate SHA and digest.
- [ ] Run the full gate, hash harness, crate DAG, package dry-run, and archive-size checks.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The user approved folding PR 71 into S71, preserving contributor credit,
and allowing it to reach `main` only through the sprint close workflow.

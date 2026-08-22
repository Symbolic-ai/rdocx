# F-X048, Dense form table fidelity

**Status**: approved
**Sprint**: S53
**Size**: L
**Depends on**: F-X040, F-X045, F-X047

## Problem

Nested tables are deliberately flattened into paragraph blocks at
`crates/rdocx-layout/src/table.rs:364`, so a dense form loses its inner grid,
fills, borders, and local coordinate system. Row height is always the maximum
of content and declared height at `crates/rdocx-layout/src/table.rs:285`, which
ignores exact-height clipping and charges all vertical-merge content to the
restart row.

Table styles also lose information. `CT_Style` models paragraph and run
properties only at `crates/rdocx-oxml/src/styles.rs:42`, and its parser skips
other style children at `crates/rdocx-oxml/src/styles.rs:104`. A round trip can
therefore drop table properties, while layout sees neither style-supplied
borders nor paragraph spacing. Cell-anchored drawings and empty paragraph-mark
metrics compound the error in the one-page hospital receipt reported in
Issue 42 and illustrated by PR 43.

## Spec reference

- ECMA-376 Part 1, `CT_Style`, `CT_TblPrBase`, `CT_TblStylePr`, `CT_TrPr`,
  `CT_TcPr`, vertical merges, and anchored drawings.
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/08-rendering-spec.md`, "Tables", pagination, and Word drawing
  placement.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability" and "WASM".
- `docs/hld/12-testing-strategy.md`, "The hash harness", "The golden-PNG
  gate", and the table rendering gate.
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".
- GitHub Issue 42 and PR 43, authenticated report and reference implementation
  by `@emptinessform`.

## Approach

Replace `TableCell::paragraphs` with one local `CellBlock` enum whose two
present implementers are paragraph and recursively laid-out table blocks.
Render nested blocks in source order inside the cell coordinate space. Include
the recursive payload in the F-X040 table-cache retained-byte accounting,
fingerprints, warm-cold equality, and transactional cache insertion boundary.

Resolve vertical-merge groups over exact grid spans before final row heights.
Non-merged cells establish ordinary row minima. A merge restart contributes
content to its full row span, growing the final eligible row only when needed.
Exact rows remain pinned and clip overflowing content, while minimum rows may
grow. Painting spans merge shading and side borders and suppresses physical
inside edges that cross the merge.

Extend `CT_Style` with preserved, schema-positioned table property and
conditional-style bytes plus the typed projections layout actually uses.
Match both element and attribute expanded names. Resolve `basedOn` chains,
table-region priority, table borders, and table-style paragraph properties
between document defaults and paragraph styles. Keep unmodelled style children
byte-identical and serialize every modeled child in schema order.

Carry cell-anchored foreground drawings into the cell text coordinate space.
Route `behindDoc` anchors to the paginator's existing page-behind layer. At an
outer table edge only, let explicit cell `nil` or `none` yield to a visible
table-level border, matching the pinned Word behavior from Issue 42. Interior
`nil` continues to suppress the edge.

Resolve an empty paragraph's line metrics from the paragraph mark's direct run
properties while preserving the F-X047 zero-width, no-glyph carrier. Add
`Paragraph::add_run_inheriting_mark(&mut self, text: &str) -> Run<'_>` as the
single native facade addition. It clones the mark run properties onto a newly
appended run.

## Rejected alternatives

- Merging PR 43 would import superseded cache and engine surfaces, unresolved
  schema-order and namespace issues, and no focused current-base gate.
- Flattening nested tables after painting still loses their independent grid
  and border conflict rules.
- Treating exact row height as a minimum repeats the reported four-page
  expansion.
- Making every cell `nil` yield to table borders would erase intentional
  interior suppression.
- Adding a new integration-test binary would increase link cost. Tests join the
  existing crate entrypoints.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `table_style_properties_are_namespace_aware_schema_ordered_and_preserved` | Alias prefixes parse by expanded name, typed changes serialize once in sequence, and unmodelled style bytes round trip unchanged. |
| regression | `nested_tables_remain_recursive_cell_blocks` | Inner grid, fills, borders, text, provenance, and anchors render inside the owning cell rather than as flattened paragraphs. |
| regression | `vertical_merges_and_row_height_rules_share_the_exact_grid_span` | Restart content uses the full merge span, crossing borders disappear, exact rows clip, and minimum rows grow only as required. |
| regression | `table_style_cascade_resolves_borders_and_paragraph_spacing` | Direct, based-on, and conditional layers resolve with deterministic priority without changing unrelated paragraph styles. |
| regression | `cell_anchors_use_cell_coordinates_and_page_behind_order` | Foreground stamps render over their cell and `behindDoc` drawings remain under every page element. |
| regression | `outer_nil_border_matches_word_without_changing_interior_nil` | Only exact outer edges fall back to table borders. Interior suppression remains intact. |
| regression | `empty_form_paragraphs_use_mark_metrics_and_new_runs_inherit_them` | Empty 7pt cells use 7pt metrics, emit no glyph, and appended text inherits the mark run properties. |
| golden | `dense_form_matches_reviewed_one_page_geometry` | A readable in-code fixture containing all seven concerns renders as one deterministic PDF and raster page with reviewed bounds and pixels. |
| regression | `dense_form_caches_are_transactional_bounded_and_exact` | Warm and cold output, source maps, diagnostics, failure state, and retained-byte limits agree with recursive payloads included. |

The test gate is **golden**. The readable in-code dense form covers the full
backlog matrix, both backends, round-trip XML, provenance, warm-cold equality,
transactional failure, memory bounds, both WASM targets, and the declared hash
result.

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Layout, pagination, and text shaping: re-read
  `docs/hld/08-rendering-spec.md`. Use deterministic bundled fonts for every
  PDF and raster baseline, and review any pixel or page-count delta.
- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add namespace-alias, schema-order,
  and byte-preservation tests for style XML.
- Public API of published crates: document the additive paragraph method and
  changed low-level table payload, run package dry runs for `rdocx-oxml`,
  `rdocx-layout`, and `rdocx`, and assert archive sizes.
- WASM bindings: run both WASM checks and the caller-font deterministic path.
- External oracle comparison: use Microsoft Word 16.104 build
  16.104.25121423 only to record one-page reference geometry and the deliberate
  outer-border divergence from strict ECMA behavior. The in-code fixture and
  reviewed golden stay authoritative, and no binary receipt enters the repo.

## Hash harness

Expected to be unchanged for the seven generated samples. The new dense-form
golden is isolated in the existing layout and rendering test entrypoints. If an
existing sample moves, stop and revise this plan with the exact named entries
before updating any baseline.

## Implementation checklist

- [ ] Preserve nested tables as recursive cell blocks and account for their
      retained bytes.
- [ ] Resolve vertical-merge spans and exact versus minimum row heights.
- [ ] Preserve and project table-style properties in schema order.
- [ ] Resolve table borders and paragraph properties through based-on and
      conditional layers.
- [ ] Render cell anchors in cell coordinates and route behind-page drawings.
- [ ] Implement the reviewed outer-edge `nil` compatibility rule.
- [ ] Resolve paragraph-mark metrics and add the inheriting-run facade method.
- [ ] Add the readable one-page golden and focused round-trip, cache, failure,
      provenance, WASM, packaging, oracle, and hash checks.

## Open questions

None. The backlog and the maintainer response on Issue 42 already select the
seven behaviors, the outer-edge compatibility rule, the no-binary-fixture
boundary, and reimplementation on the current cache contracts.

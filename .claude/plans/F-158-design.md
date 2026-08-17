# F-158, Document::add_chart

**Status**: completed
**Sprint**: S45
**Size**: M
**Depends on**: F-157

## Problem

The Presentation facade exposes data-first native chart authoring at
`crates/rpptx/src/lib.rs:1367`, but its validation and ChartML/workbook assembly
remain private to that facade at `crates/rpptx/src/lib.rs:1445` and
`crates/rpptx/src/lib.rs:1503`. The Word facade has no equivalent public API.
Callers can add a raster picture at `crates/rdocx/src/document.rs:661`, but that
cannot preserve categories, series, number formats, or Edit Data behavior.

Duplicating the current authoring logic in `rdocx` would create two sources for
the cache and workbook contract. The shared chart crate needs to own the common
validated data-to-parts operation now that two concrete facades consume it.

## Spec reference

- `docs/hld/03-architecture.md`, shared ownership and dependency direction.
- `docs/hld/04-opc-and-packaging.md`, atomic mutation and part allocation.
- `docs/hld/09-charts-spec.md`, "Cached values are not optional" and
  "Authoring API".
- `docs/hld/12-testing-strategy.md`, regression naming and native acceptance
  evidence.
- `docs/hld/14-development-backlog.md`, "F-158, Document::add_chart".

## Approach

Move the existing format-neutral `ChartKind`, `ChartData`, validation, and
typed ChartML plus workbook construction from the `rpptx` facade into
`oxml-chart`. Expose one concrete helper that accepts the workbook relationship
ID and returns the serialized chart and workbook bytes derived from the same
validated source. Re-export `ChartKind` and `ChartData` from `rpptx` so its
current source paths and behavior remain compatible, then make both facades
call the shared helper.

Add the Word surface beside `add_picture`:

```rust
impl Document {
    pub fn add_chart(
        &mut self,
        kind: ChartKind,
        width: Length,
        height: Length,
        data: &ChartData,
    ) -> Result<Paragraph<'_>>;
}
```

The method authors an inline chart because a Word body is flow content and has
no slide index or absolute `left` and `top` coordinates. Its data types,
validation, and chart-family behavior match `Presentation::add_chart`, while
its placement arguments match `Document::add_picture`. It stages package and
document changes, uses F-157's private package assembly, appends one paragraph
containing `CT_Inline::new_chart`, invalidates layout only after success, and
returns the paragraph for normal Word formatting.

Support the story's bar, line, and pie gate through the shared authoring path.
Keep the other chart kinds already supported by `ChartKind` available so the
two facades genuinely share one data contract rather than introducing a
Word-only subset enum.

## Rejected alternatives

- Copy the Presentation helper into `rdocx`. Cache and workbook serialization
  would immediately have two implementations that can diverge.
- Put package mutation in `oxml-chart`. The shared chart crate does not own an
  `OpcPackage`, and both facades already own their relationship scopes.
- Use slide-style `left` and `top` arguments for Word. Inline Word content is
  positioned by pagination, so those coordinates have no coherent meaning.
- Add a builder or placement trait. The requested data value has three fields,
  and there are exactly two concrete facade methods today.
- Add a new source module. The shared chart root and Word document file already
  own the relevant behavior.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression, gate | `added_bar_line_and_pie_charts_keep_source_data` | Saved and reopened charts retain every series name, category, numeric value, and requested number format |
| regression | existing `rpptx` add-chart tests | Moving common authoring into `oxml-chart` leaves Presentation output and validation unchanged |
| unit | `word_add_chart_writes_cache_and_workbook_from_one_source` | Chart formulae and caches match the embedded worksheet cells for every authored series |
| negative | `word_add_chart_rejects_invalid_data_without_mutation` | Empty, ragged, nonfinite, invalid-format, nonpositive-size, and multi-series pie inputs return errors without changing state |
| integration | `word_add_chart_uses_inline_flow_placement` | The public call appends one paragraph with the requested extents and the document-to-chart relationship |

The test gate is regression. A bar, line, and pie chart added to a document
carry the series, categories, and number formats they were given.

## HLD impact

- `docs/hld/09-charts-spec.md`

Describe the shared authoring data contract, the two owning facade methods,
Word's inline placement arguments, common validation, and atomic package
mutation.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Run the existing ChartML
  child-order, fixed-prefix, cache, workbook, and round-trip tests plus the new
  Word authoring round-trip.
- Crate dependency graph and new cross-family uses. Read HLD 03. Confirm both
  facades depend inward on `oxml-chart` and `oxml-sml`, with no reverse shared
  edge.
- Public API of published crates. Read HLD 10 and the structural rules. State
  the additive Word API and the source-compatible Presentation re-exports, then
  run affected package dry-runs and archive size assertions.

## Hash harness

Expected unchanged across all 49 entries. Existing generated samples do not
call `Document::add_chart`.

## Implementation checklist

- [x] Move the common authoring data types, validation, and part serialization into `oxml-chart`.
- [x] Preserve the existing `rpptx` public paths through re-exports and shared calls.
- [x] Add the atomic inline `Document::add_chart` method.
- [x] Add bar, line, pie, cache, workbook, placement, and rollback regressions.
- [x] Run focused shared-chart, rdocx, rpptx, package, and unchanged-output checks.
- [x] Update exactly HLD 09.

## Open questions

None. Word's flow model makes inline placement the coherent default, while the
shared `ChartKind` and `ChartData` contract supplies the requested API parity.

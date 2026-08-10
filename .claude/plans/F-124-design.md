# F-124, add_chart

**Status**: approved
**Sprint**: S31
**Size**: L
**Depends on**: F-117, F-121

## Problem

The `rpptx` owning facade can add relationship-backed pictures, but it has no
chart authoring API. `Presentation::add_picture` in `crates/rpptx/src/lib.rs`
already stages package and slide mutations atomically, while
`SlideMut::add_table` can only append a package-independent graphic frame.
Chart creation needs both operations because one call must add a ChartML part,
an embedded workbook, two relationship scopes, two content types, and the
slide frame.

`rpptx-oxml` already preserves chart graphic data as raw XML, `rpptx-chart`
already owns all seven two-dimensional plot families, and `oxml-sml` already
writes the workbook. The missing work is the authoring bridge that derives all
of those artifacts from one validated data value.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and "The
  dependency rule".
- `docs/hld/04-opc-and-packaging.md`, part naming, relationship targets, and
  content types.
- `docs/hld/06-presentationml-model.md`, owning facade mutation and typed
  graphic frames.
- `docs/hld/09-charts-spec.md`, "Why a chart needs three parts", "oxml-sml,
  deliberately minimal", "The ChartML model", and "Authoring API".
- `docs/hld/12-testing-strategy.md`, external application acceptance evidence.
- `docs/hld/14-development-backlog.md`, "F-124, add_chart".

## Approach

Extend existing files only. Add `oxml-sml` and `rpptx-chart` as normal `rpptx`
dependencies. Add the missing chart and embedded-package OPC constants, and
add a `CT_GraphicFrame::new_chart` constructor that writes the required
`c:chart r:id` payload in schema order.

Expose the data-first authoring surface from `rpptx`:

```rust
pub enum ChartKind {
    Bar,
    Line,
    Pie,
    Doughnut,
    Area,
    Scatter,
    Radar,
}

pub struct ChartData {
    pub categories: Vec<String>,
    pub series: Vec<(String, Vec<f64>)>,
    pub number_format: Option<String>,
}

impl Presentation {
    pub fn add_chart(
        &mut self,
        slide_index: usize,
        kind: ChartKind,
        left: Emu,
        top: Emu,
        width: Emu,
        height: Emu,
        data: &ChartData,
    ) -> Result<ShapeRef<'_>>;
}
```

The owning `Presentation` method is the current facade equivalent of the
spec's historical `Shapes::add_chart` sketch. It is required because
`SlideMut` has no package access. The HLD authoring example will be aligned to
the concrete facade.

Validate nonempty categories and series, equal series lengths, finite values,
valid number formats, and chart-family requirements before mutating state.
Build one worksheet with a category column followed by numeric series columns.
Use its formula ranges for the ChartML caches so workbook and cache data share
one source. Build the corresponding typed plot and paired axes where the
family requires them.

Allocate `/ppt/charts/chartN.xml` and
`/ppt/embeddings/Microsoft_Excel_WorksheetN.xlsx` from the maximum occupied
numeric suffix plus one. Add the slide-to-chart relationship, the
chart-to-workbook package relationship, chart and workbook content-type
overrides, and the chart graphic frame. Stage package and slide values and
commit only after every serialization succeeds.

## Rejected alternatives

- Put `add_chart` on `SlideMut`. That borrow has no package access and would
  split one atomic mutation across two public surfaces.
- Accept a prebuilt chart part and workbook. That duplicates source data and
  cannot guarantee cache and workbook consistency.
- Add a new chart builder or module. The requested data value has three fields,
  and the existing crate roots already own the behavior.
- Write workbook ZIP bytes inside `rpptx`. `oxml-sml` is the existing concrete
  implementation and second-source logic would drift.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `created_chart_opens_and_edit_data_matches_source` | A saved chart opens without repair in pinned PowerPoint and Edit Data exposes the authored categories and values |
| unit | `add_chart_writes_complete_relationship_graph` | Slide, chart, workbook parts, both relationship scopes, and both content-type overrides resolve to exact targets |
| unit | `add_chart_uses_collision_free_part_numbers` | Existing sparse chart and embedding suffixes produce maximum suffix plus one |
| unit | `add_chart_caches_and_workbook_share_one_source` | Chart formulae, cached values, and worksheet cells agree for every series |
| negative | `add_chart_rejects_invalid_data_without_mutation` | Empty, ragged, nonfinite, or invalid format data returns an error and leaves package and slide bytes unchanged |
| round-trip | `authored_chart_graphic_frame_round_trips` | The chart relationship payload writes in schema order and reparses without losing unrelated frame XML |

The test gate is: a created chart opens in PowerPoint and "Edit Data" shows the
source values.

## HLD impact

- `docs/hld/09-charts-spec.md`

Replace the historical `Shapes` sketch with the concrete owning facade,
document validation and atomic package mutation, and record pinned acceptance
evidence.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add exact schema-order,
  fixed-prefix, round-trip, and raw-subtree preservation checks for the chart
  graphic frame.
- Crate dependency graph, a new `use` across families. Read HLD 03. Run the
  architecture dependency check and confirm no `oxml-*` crate gains an edge to
  an `rdocx-*` or `rpptx-*` crate.
- An external oracle comparison. Follow differential-testing guidance. Record
  the pinned PowerPoint version and build, bind the candidate deck to its SHA,
  and record whether open, repair, and Edit Data checks passed.

No published API, binding, feature, new file, unit-conversion behavior, or
baseline rider applies. The facade and supporting crates are unpublished.

## Hash harness

Expected unchanged. Chart authoring does not enter the Word sample generator
or its rendering path. All 28 hashes must match.

## Implementation checklist

- [ ] Add the required unpublished crate dependencies and OPC constants.
- [ ] Add the schema-ordered chart graphic-frame constructor.
- [ ] Add `ChartKind`, `ChartData`, validation, and typed ChartML construction.
- [ ] Write the workbook, chart part, relationships, content types, and frame
      atomically with collision-free names.
- [ ] Add focused graph, cache consistency, naming, rollback, and round-trip
      tests to existing test entrypoints.
- [ ] Produce SHA-bound pinned PowerPoint open and Edit Data evidence.
- [ ] Update exactly HLD 09.
- [ ] Run focused checks, routed checks, microscope, and worker preparation.

## Open questions

None. The owning facade is required by the existing package boundary, and the
current typed plot model supplies every requested two-dimensional family.

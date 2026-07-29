# 09, Charts spec

Owners: `rpptx-chart` for ChartML, `oxml-sml` for the embedded workbook.

Charts are the largest single subsystem in v1 and the reason the release spans
three OOXML formats. They were scoped in deliberately, and the estimate that
follows is recorded so it can be revisited rather than discovered.

## Why a chart needs three parts

A chart in a `.pptx` is not one thing:

```
/ppt/slides/slide1.xml
  p:graphicFrame > a:graphic > a:graphicData
      uri = ".../drawingml/2006/chart"
      c:chart r:id="rId2"                      -> the chart part

/ppt/charts/chart1.xml                          the ChartML definition
/ppt/charts/_rels/chart1.xml.rels
      package    -> /ppt/embeddings/Workbook1.xlsx
      colors     -> /ppt/charts/colors1.xml     (optional)
      style      -> /ppt/charts/style1.xml      (optional)

/ppt/embeddings/Workbook1.xlsx                  a complete .xlsx, nested
```

The workbook is what "Edit Data" opens. PowerPoint will render a chart whose
workbook is missing, but the file is malformed and editing is broken. This is
why the minimal SpreadsheetML writer is not optional.

## `oxml-sml`, deliberately minimal

A **writer only**, and never a general SpreadsheetML implementation. It emits:

- `[Content_Types].xml`, `_rels/.rels`, `xl/workbook.xml`,
  `xl/_rels/workbook.xml.rels`, `xl/worksheets/sheet1.xml`, and
  `xl/sharedStrings.xml` when strings are present.
- One worksheet. Numeric cells and inline or shared strings. No styles beyond a
  number format per column, no formulas, no charts of its own.
- A defined range per series so ChartML's `c:f` references resolve.

```rust
pub struct Workbook { sheet_name: String, columns: Vec<Column> }
pub enum Column { Text { header: String, values: Vec<String> },
                  Number { header: String, values: Vec<f64>, number_format: Option<String> } }
impl Workbook { pub fn to_xlsx_bytes(&self) -> Result<Vec<u8>>; }
```

**This crate must not grow into an `rxlsx` without a separate decision.** Its
scope is recorded in `02-scope-and-non-goals.md` as a permanent non-goal, and
its README says so.

## The ChartML model

```rust
pub struct CT_ChartSpace {
    pub chart: CT_Chart,
    pub sp_pr: Option<ShapeProperties>,
    pub tx_pr: Option<TextBody>,
    pub ext_lst: Option<Vec<u8>>,
    pub raw_children: OrderedRawChildren,
}

pub struct CT_Chart {
    pub title: Option<CT_Title>,
    pub auto_title_deleted: bool,
    pub plot_area: CT_PlotArea,
    pub legend: Option<CT_Legend>,
    pub plot_vis_only: bool,
    pub disp_blanks_as: DispBlanksAs,
}

pub enum Plot {
    Bar { direction: BarDirection, grouping: BarGrouping, gap_width: u16,
          overlap: i8, series: Vec<Series> },
    Line { marker: bool, smooth: bool, series: Vec<Series> },
    Pie { first_slice_ang: u16, series: Vec<Series> },
    Doughnut { hole_size: u8, series: Vec<Series> },
    Scatter { style: ScatterStyle, series: Vec<Series> },
    Area { grouping: Grouping, series: Vec<Series> },
    Radar { style: RadarStyle, series: Vec<Series> },
}

pub struct Series {
    pub index: u32, pub order: u32,
    pub name: Option<StringRef>,
    pub categories: Option<AxisData>,   // c:cat
    pub values: NumericData,            // c:val
    pub bubble_size: Option<NumericData>,
    pub sp_pr: Option<ShapeProperties>,
    pub data_labels: Option<CT_DLbls>,
    pub points: Vec<CT_DPt>,
}
```

Axes are `c:catAx`, `c:valAx`, `c:dateAx` and `c:serAx`, each carrying an id,
scaling, delete flag, position, gridlines, title, number format, tick marks,
label position and a `crossAx` back-reference. **Axis ids are arbitrary but must
pair consistently through `c:crossAx`.**

## Cached values are not optional

Every `c:cat` and `c:val` carries both a formula reference into the workbook and
a **cache** of the literal values:

```xml
<c:val>
  <c:numRef>
    <c:f>Sheet1!$B$2:$B$5</c:f>
    <c:numCache>
      <c:formatCode>General</c:formatCode>
      <c:ptCount val="4"/>
      <c:pt idx="0"><c:v>4.3</c:v></c:pt>
      ...
```

**The cache is what renders.** A consumer that cannot open the workbook, which
includes this renderer, draws from the cache. Writing a chart without one
produces an empty plot in most viewers. The cache and the workbook are written
from the same source data in one operation so they cannot diverge.

## Authoring API

```rust
pub struct ChartData {
    pub categories: Vec<String>,
    pub series: Vec<(String, Vec<f64>)>,
    pub number_format: Option<String>,
}

impl Shapes<'_> {
    pub fn add_chart(&mut self, kind: ChartKind, bounds: Rect, data: &ChartData)
        -> Result<GraphicFrame<'_>>;
}
```

`add_chart` writes the chart part, the workbook part, both sets of
relationships, both content-type overrides, and the `p:graphicFrame` on the
slide, then returns a handle for further styling. Part numbering follows the
`1 + max(existing suffix)` rule from `04-opc-and-packaging.md`.

## Rendering

`rpptx-chart` emits `PathElement`, `Text` and `Group` directly into the page
frame, so **no backend work is needed beyond what `08-rendering-spec.md`
already requires**. Bars, lines, pie wedges, areas and markers are all paths.
Gridlines and axis lines are strokes. Labels are glyph runs.

Scales are computed from the cached values: linear axes with a nice-number tick
algorithm unless `c:scaling` pins a minimum or maximum. Series colours come from
the chart's own `c:spPr` when present, otherwise from the theme's accent cycle
resolved through the same colour pipeline as everything else.

For a chart that was **preserved rather than authored**, draw the cached image
fallback if the file carries one, otherwise a labelled placeholder rectangle
with a diagnostic. This is the same fallback discipline used for SmartArt.

## What is not in v1

Combo charts mixing plot types on one axis pair, 3-D charts, bubble sizing
beyond the data model, trendlines, error bars, `c:view3D`, and chart styles from
`style1.xml`. All are preserved verbatim and rendered from the cache where
possible.

## Sizing

Roughly six to eight weeks, which is why this is its own milestone and why
`00-vision.md` records it as the dominant term in the overall estimate. It is
self-contained: nothing else in the plan depends on it, so it can move without
reshaping the rest.

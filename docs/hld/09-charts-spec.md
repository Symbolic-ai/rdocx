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

A **writer only**, and never a general SpreadsheetML implementation. The
implemented `Workbook` accepts one validated worksheet name and one or more
directly constructed `Column::Text` or `Column::Number` values. It rejects names
outside Excel's 31 UTF-16-code-unit and forbidden-character boundary, more than
16,384 columns, more than 1,048,575 data values in any column, and nonfinite
numbers. It also rejects a total shared-string reference count beyond the
SpreadsheetML unsigned 32-bit boundary. SpreadsheetML escape sequences
preserve XML control characters, carriage returns, reserved `_xHHHH_` text,
XML noncharacters, and XML metacharacters in headers, text values, worksheet
names, and number formats.

The writer emits:

- `[Content_Types].xml`, `_rels/.rels`, `xl/workbook.xml`,
  `xl/_rels/workbook.xml.rels`, `xl/worksheets/sheet1.xml`, and
  `xl/sharedStrings.xml`. Headers ensure that the shared-string part is always
  present for a nonempty workbook.
- One worksheet with shared-string headers and text values plus direct finite
  numeric cells. `xl/styles.xml` is present only when at least one numeric
  column requests a number format. Equal format codes share a deterministic
  custom format and cell-style index.
- A deterministic A1 formula range per nonempty column. Each range begins in
  row 2, ends at that column's last present value, and quotes the worksheet
  name only when formula syntax requires it. An empty or absent column returns
  no range.

```rust
pub struct Workbook { sheet_name: String, columns: Vec<Column> }
pub enum Column { Text { header: String, values: Vec<String> },
                  Number { header: String, values: Vec<f64>, number_format: Option<String> } }
impl Workbook { pub fn to_xlsx_bytes(&self) -> Result<Vec<u8>>; }
```

**This crate must not grow into an `rxlsx` without a separate decision.** Its
scope is recorded in `02-scope-and-non-goals.md` as a permanent non-goal, and
its README says so.

The native acceptance workbook has SHA-256
`8f8d12aa4ebe94f86c8164fd251cdb23845f985090be0fb6c77242aaa0fba329`.
Microsoft Excel 16.104, Info.plist build 16.104.25121423, opened its one
`Sales '24` worksheet without a repair warning and exposed the expected A1:B3
cells. LibreOffice Calc 26.2.5.2, build
`cd7284b4cbbfeb507e630c1aac019f4157393acb`, imported and re-exported the same
workbook without a conversion error, preserving that worksheet and cell range.

## The ChartML model

The implemented core owns `c:chartSpace`, `c:chart`, `c:plotArea`, title,
legend, chart flags, and DrawingML shape and text properties. Readers accept a
bound ChartML prefix. Writers use fixed `c:`, `a:`, and `r:` prefixes and emit
modelled children in schema order. Missing required chart and plot-area roots,
duplicate modelled children, malformed booleans, and unknown blank-display
values are errors.

```rust
pub struct CT_ChartSpace {
    pub chart: CT_Chart,
    pub sp_pr: Option<ShapeProperties>,
    pub tx_pr: Option<TextBody>,
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
```

`c:spPr` reuses `CT_ShapeProperties`. `c:txPr` uses the same concrete
`CT_TextBody` parser as `a:txBody`, with a caller-selected root local name and
the existing caller-selected writer tag. Title, plot area, and legend are
behavior-bearing shells. Their attributes and complete current children stay
in ordered raw slots. Plot variants, series, axes, extensions, and unsupported
root children remain byte-preserved at their schema boundaries until the story
that types each surface.

The pinned 50-deck corpus contains 26 chart parts across 9 decks. Every part
parses, serialises, reparses, and compares as the same core model. The inline
fixture keeps prefix, ordering, malformed-value, and raw-preservation coverage
available without the external corpus.

The typed series layer adds concrete formula-backed string and numeric caches.
`StringRef::new` receives one formula and one string vector.
`NumericData::new` receives one formula, one number format, and one numeric
vector. Writers derive `c:ptCount`, sequential `c:pt/@idx` values, and every
cached `c:v` from those vectors. Callers cannot supply independent cache
metadata that disagrees with the data.

```rust
pub struct StringRef {
    pub formula: String,
    pub values: Vec<String>,
}

pub struct NumericData {
    pub formula: String,
    pub format_code: String,
    pub values: Vec<f64>,
}

pub enum AxisData {
    String(StringRef),
    Numeric(NumericData),
}

pub struct Series {
    pub index: u32,
    pub order: u32,
    pub name: Option<StringRef>,
    pub categories: Option<AxisData>,
    pub values: NumericData,
    pub bubble_size: Option<NumericData>,
    pub sp_pr: Option<CT_ShapeProperties>,
}
```

Series readers require `c:idx`, `c:order`, and a formula-backed `c:val`. They
reject duplicate modelled children, empty formulae, missing caches and cache
metadata, invalid counts, duplicate or descending point indexes, indexes
outside the declared count, and nonfinite numbers. Producer caches may omit
blank points. Their strictly increasing sparse indexes and declared logical
count remain intact on round-trip. Newly authored caches are always dense and
sequential.

`c:tx`, `c:spPr`, `c:cat`, `c:val`, and `c:bubbleSize` write in schema order.
Markers, data points, labels, trendlines, extensions, unknown attributes, and
whitespace remain byte-preserved in their original series or reference slots.
Plot-area serialization continues to use its preserved plot bytes. Its
read-only series projection validates the common category-based plot payloads
without claiming ownership of a plot kind. Across the pinned corpus it parses
and reparses 66 series from the 26 chart parts.

The later typed plot surface expands the preserved plot-area slots to the
following model:

```rust

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

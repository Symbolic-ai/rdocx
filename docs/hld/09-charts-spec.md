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
    pub sp_pr: Option<CT_ShapeProperties>,
    pub tx_pr: Option<CT_TextBody>,
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

The example lists the public fields. Preservation state remains private, with
ordered raw children exposed for inspection through `raw_children()`.

`c:spPr` reuses `CT_ShapeProperties`. `c:txPr` uses the same concrete
`CT_TextBody` parser as `a:txBody`, with a caller-selected root local name and
the existing caller-selected writer tag. Title, plot area, and legend are
behavior-bearing shells. Their attributes and unsupported children stay in
ordered raw slots. Plot variants, extensions, and unsupported root children
remain byte-preserved at their schema boundaries until the story that types
each surface.

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
    pub data_labels: Option<CT_DLbls>,
}
```

Series readers require `c:idx`, `c:order`, and a formula-backed `c:val`. They
reject duplicate modelled children, empty formulae, missing caches and cache
metadata, invalid counts, duplicate or descending point indexes, indexes
outside the declared count, and nonfinite numbers. Producer caches may omit
blank points. Their strictly increasing sparse indexes and declared logical
count remain intact on round-trip. Newly authored caches are always dense and
sequential.

`c:tx`, `c:spPr`, `c:dLbls`, `c:cat`, `c:val`, and `c:bubbleSize` write in
schema order. Markers, data points, trendlines, extensions, unknown attributes,
and whitespace remain byte-preserved in their original series or reference
slots. Supported single-family bar and line plot areas own their series through
the typed plot model below. Unsupported plot areas continue to use preserved
plot bytes. Across the pinned corpus the model parses and reparses 66 series
from the 26 chart parts.

`CT_DLbls` models collection-level number format, position, separator, and the
six visibility flags for legend key, value, category name, series name,
percentage, and bubble size. Readers accept bound ChartML prefixes and reject
duplicate fields, unknown positions, malformed booleans, empty or XML-invalid
format codes, and XML-invalid separators. Writers use fixed prefixes and the
ChartML sequence. Individual `c:dLbl` overrides, leader lines, shape and text
properties, extensions, producer attributes, comments, and whitespace remain
byte-preserved in ordered raw slots.

`NumberFormat` is the shared value used by axes and data labels. Its
`format_value` method projects finite cached values through `General`, ordinary
zero-placeholder decimal precision, and percentage forms such as `0%`,
`0.0%`, and `0.00%`. Other valid producer codes remain available for
round-trip but return a contextual projection error. The implementation does
not claim the wider Excel format language or native label geometry.

The pinned corpus round-trips 34 data-label collections and 35 axis number
formats. The viewer gate changes only the chart part of `bar-chart.pptx`, sets
the typed cached values to `0.25`, and writes `showVal` with format `0%`.
LibreOffice 26.2.5.2 renders the SHA-bound candidate, then Poppler 26.01.0
extracts `25%` from the PDF. The reviewed candidate SHA-256 is
`4ba02faa8e4cff6cefa7a7dc73fc0eb0c08d62d180f83fa0d3fd56a7e4136242`.

The typed plot surface currently owns two-dimensional bar and line plots:

```rust
pub enum Plot {
    Bar {
        direction: BarDirection,
        grouping: BarGrouping,
        gap_width: u16,
        overlap: i8,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Line {
        grouping: Grouping,
        marker: bool,
        smooth: bool,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
}
```

`BarDirection` distinguishes horizontal bars from columns. `BarGrouping`
supports clustered, percentage-stacked, stacked, and standard modes.
`Grouping` supports the percentage-stacked, stacked, and standard line modes.
Bar gap width is from 0 through 500 and overlap is from -100 through 100. Both
plot kinds require at least one series and exactly two axis references. Line
marker and smooth values are typed booleans.

`CT_PlotArea` owns the typed plots and the axis collection. Each plot stores
only two `AxisId` references. Both identifiers must resolve in the plot-area
axis set, and the complete set must retain the reciprocal `crossAx` invariant.
`CT_PlotArea::new` validates authored plots and axes. `plots()`, `plots_mut()`,
and `axes()` expose the supported owned choice.

Readers accept any prefix bound to ChartML. Writers use fixed `c:`, `a:`, and
`r:` prefixes and the ChartML child sequence. Plot attributes, comments,
whitespace, extensions, `varyColors`, line decorations, bar series lines, and
unmodelled series payloads remain in ordered raw slots. Repeated series and
axis-id slots reconcile public insertions, edits, and reordering without moving
schema-leading content or losing between-item payloads.

Three-dimensional plots, stock, surface, `ofPie`, and other unsupported plot
families remain opaque. A plot area containing more than one plot family also
remains opaque, including a supported bar and line combination. The writer
does not partially rewrite an opaque choice or replace a parsed bar family
with line, or line with bar, while preserved family-specific payload remains.

The pinned corpus contains 12 bar plots and 3 line plots. The typed boundary
owns 11 bar plots and 2 line plots. One bar and line combination remains
opaque, which accounts for the remaining plot of each kind.

The external viewer gate rewrites only the chart part in representative bar
and line decks. LibreOffice 26.2.5.2 and Poppler 26.01.0 render originals and
candidates at 150 dpi. Normalized RGB mean absolute error has an exact threshold
of 0 and both observed values are 0. The SHA-256 bindings are:

- Bar original deck
  `79e1d218bfb2903e8dc8425a6b1997d9c1976f5a5f025bada85b0c47b5777969`,
  candidate deck
  `20a73449769a7e50c009d375cfda8da9beee7f367447caa905c233673f159dbf`,
  and both renders
  `97e9579bf906bca51b127683ca1e476c93545e0f95d40683ce21ce9c8c127529`.
- Line original deck
  `a2319540fb096629874e8c2baf91b9f8afd1386bfba411efff503b96dce9e9a1`,
  candidate deck
  `509bf5bba48ae3a39f9b207d989c3958316f00735ffd0fc835ddd620d4887769`,
  and both renders
  `e5a44127c31edde36e470ad5fa541206c5c9c2080d4618af8d5d0567d084cdc9`.

This viewer evidence covers plot serialization. Native bar, line, area, wedge,
and marker path generation remains in F-125.

`Axis` models the `Category`, `Value`, `Date` and `Series` roots with typed
`Scaling`, position, delete state, gridlines, title, `NumberFormat`, tick
marks, label position, shape properties, text properties and a `cross_axis`
reference. `AxisId` accepts the producer-compatible range from `i32::MIN`
through `u32::MAX`. Parsed identifier spellings are preserved when unchanged,
while equality and pairing use the normalized numeric value.

`CT_PlotArea::axes()` projects direct axis children and validates the complete
nonempty axis graph. Identifiers are unique, each cross reference resolves in
the same plot area, no axis crosses itself and every reference is reciprocal.
Parsing rejects missing required children, duplicate modelled children,
invalid enum and boolean values, nonfinite or inconsistent scaling bounds and
out-of-range identifiers. Writing uses fixed `c:`, `a:` and `r:` prefixes in
schema order. Unsupported type-specific children, extensions, attributes,
comments and whitespace remain in ordered raw slots. The pinned 50-deck corpus
currently exercises 40 axes across 26 chart parts.

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

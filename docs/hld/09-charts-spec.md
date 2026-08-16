# 09, Charts spec

Owners: `oxml-chart` for ChartML, `oxml-sml` for the embedded workbook.
`rpptx-chart` is a deprecated exact re-export shim.

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
ordered raw slots. Unsupported plot variants, extensions, and unsupported root
children remain byte-preserved at their schema boundaries.

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
slots. Supported single-family plot areas own their series through the typed
plot model below. Unsupported plot areas continue to use preserved plot bytes.
Across the pinned corpus the model parses and reparses 66 series from the 26
chart parts.

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

The typed plot surface owns all seven two-dimensional v1 plot families:

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
    Pie {
        first_slice_angle: u16,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
    },
    Doughnut {
        first_slice_angle: u16,
        hole_size: u8,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
    },
    Area {
        grouping: Grouping,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Scatter {
        style: ScatterStyle,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
    Radar {
        style: RadarStyle,
        series: Vec<Series>,
        data_labels: Option<CT_DLbls>,
        axis_ids: [AxisId; 2],
    },
}
```

`BarDirection` distinguishes horizontal bars from columns. `BarGrouping`
supports clustered, percentage-stacked, stacked, and standard modes.
`Grouping` supports the percentage-stacked, stacked, and standard line modes.
Bar gap width is from 0 through 500 and overlap is from -100 through 100. Line
marker and smooth values are typed booleans. Pie and doughnut first-slice
angles are from 0 through 360. Doughnut hole size is from 10 through 90.
Scatter and radar styles are closed enums. Every plot requires at least one
series.

Pie and doughnut plots are axis-free and reject both plot references and
plot-area axes. Bar, line, area, scatter, and radar plots each own exactly two
axis references. Both identifiers must resolve in the plot-area axis set, and
the complete set must retain the reciprocal `crossAx` invariant.

Scatter reuses the public `Series` cache model. Its numeric categories write
as `c:xVal` and its values write as `c:yVal`. Readers map those wrappers back
to the same two fields. String x values, missing x or y caches, and any mixture
of category/value and x/y wrappers are errors.

`CT_PlotArea` owns the typed plots and the axis collection.
`CT_PlotArea::new` validates authored plots and axes. `plots()`, `plots_mut()`,
and `axes()` expose the supported owned choice.

Readers accept any prefix bound to ChartML. Writers use fixed `c:`, `a:`, and
`r:` prefixes and the ChartML child sequence. Plot attributes, comments,
whitespace, extensions, `varyColors`, line decorations, bar series lines, and
unmodelled series payloads remain in ordered raw slots. Repeated series and
axis-id slots reconcile public insertions, edits, and reordering without moving
schema-leading content or losing between-item payloads.

Three-dimensional plots, stock, surface, bubble, `ofPie`, and other unsupported
plot families remain opaque. A plot area containing more than one plot family
also remains opaque, including a supported bar and line combination. The
writer does not partially rewrite an opaque choice or replace a parsed family
while preserved family-specific payload remains.

The pinned corpus contains 12 bar plots, 3 line plots, and 1 pie plot. The typed
boundary owns 11 bar plots, 2 line plots, and the pie plot. One bar and line
combination remains opaque, which accounts for the remaining plot of each
kind. The corpus contains no doughnut, area, scatter, or radar plot, so inline
fixtures provide non-vacuous parse, mutation, ordering, validation, and
round-trip coverage for those families.

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

The remaining-family viewer gate inserts typed pie, doughnut, area, scatter,
and radar candidates into the same representative chart deck. Each candidate
is bound to its SHA-256 before LibreOffice 26.2.5.2 exports it and Poppler
26.01.0 rasterises page one at 150 dpi. The asserted chart rectangle spans
pixels `[300,168)` through `[1200,956)`. A pixel is nonblank when any RGB
channel is below 245, and each candidate must contain at least 1,000 such
pixels in that rectangle. The observed evidence is:

- Pie candidate
  `74c3de0d605414ea640de7c999f6d48d5ee00f8c869474a354b1c04c300ac4eb`,
  render
  `0f35c6e6d621a160619063aecfb7e5547dc0b64e86679db5de908e2864409c1c`,
  309,502 nonblank pixels.
- Doughnut candidate
  `e41dcdb9403476ac829dc1f578da010c00e2318830ec22e3f881a84b383a7fda`,
  render
  `5aa52b1ad8485fb9e54aafe7a19cf007d2afb1af45ddfb98b54d2a0219bf00ed`,
  233,915 nonblank pixels.
- Area candidate
  `271188575c91900a1961cbb483ff57c33bac58b7f00f486a9b36b47f2d25a98d`,
  render
  `6a335d9a9517729664a7d6380cce27616fe7680eb2723c38c1a330534cec0a14`,
  308,569 nonblank pixels.
- Scatter candidate
  `8b8d9d248bce055799a5eefea216a81be482fba06cf273698c98398885a5645f`,
  render
  `89a6344aa62c47c7bbab922146cfdee88b74404064efb1316db4594e8f9be199`,
  9,865 nonblank pixels.
- Radar candidate
  `36b6ffe9ed5eea646d64aa151e513bbe0073a375bfe40745e83ba3aa90b69dad`,
  render
  `921a41e7c46c355a7bf0967429ff2f0cd066aae973313c075898436c1b33cb9f`,
  7,161 nonblank pixels.

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

The owning facade performs this mutation because the package parts and
relationships are not available through `SlideMut`. `add_chart` validates the
complete data value before mutation. Categories and series must be nonempty,
series lengths must match the category count, numeric values must be finite,
number formats must be valid, and both chart extents must be positive. Pie and
doughnut charts accept one series. Scatter categories must parse as finite
numeric values.

One call writes the typed chart part, one editable workbook part, the
slide-to-chart and chart-to-workbook relationships, both content-type
overrides, and the `p:graphicFrame` on the slide. Workbook cells and ChartML
caches are derived from the same `ChartData`. The package and slide changes are
staged and become visible together only after serialization succeeds. Chart
parts use `/ppt/charts/chartN.xml` and `/ppt/embeddings/WorkbookN.xlsx`.
Each numbered family independently takes the next positive suffix after its
greatest occupied suffix, following the allocation rule in
`04-opc-and-packaging.md`.

The native chart candidate has SHA-256
`e6e9f7eef1c774d0414c5d0c3f1202da1a28635b5d089e15455b7adc3f66cb00`.
Microsoft PowerPoint 16.104, Info.plist build 16.104.25121423, opened it without
a repair warning and recognized `Chart 4` as a native chart. Chart Design,
Edit Data exposed the authored `Category`, `Revenue`, and `Cost` columns with
rows `North, 12.5, 8.0`, `South, 19.0, 11.5`, and `West, 14.25, 9.75`.

## Rendering

`oxml-chart` emits `PathElement`, `Text` and `Group` directly into the page
frame, so **no backend work is needed beyond what `08-rendering-spec.md`
already requires**. Bars, lines, pie wedges, areas and markers are all paths.
Gridlines and axis lines are strokes. Labels are glyph runs.

The geometry entry point receives `&CT_Chart`, `Rect`, the effective
`&CT_OfficeStyleSheet`, and the effective `&ColorMap`, then returns
`Result<ChartGeometry>`. The labelled entry point receives the same theme and
colour-map inputs before its `FontManager`. Input and output coordinates are
typographic points. Geometry reserves 36 points on the left, 12 on the right
and top, and 28 on the bottom, then returns one identity group whose children
use chart-local point coordinates. Invalid or too-small bounds, opaque or
combination plots, and empty cached data return contextual errors.

Clustered bars derive their width from the category slot, `c:gapWidth`, series
count, and `c:overlap`. Stacked bars accumulate positive and negative values
separately, and percentage stacks normalise against the matching sign total.
Lines and areas use category-slot centres. Areas close against zero or the
previous stacked series. Scatter plots map numeric x and y caches directly.
Pie and doughnut slices use closed cubic wedges, including the first-slice
angle and doughnut hole size. Radar plots use closed radial polygons. Marker
paths follow their owning series path, and output order is plot order followed
by series order.

Category slot counts come from the caches' declared `c:ptCount`, and geometry
uses each preserved `c:pt/@idx` rather than collapsing sparse caches into dense
positions. Scatter x and y values pair only when their logical indexes match.
The private cache-layout accessor also gives newly authored dense caches their
sequential indexes without exposing parser preservation state publicly.

Sparse line, area, and scatter paths apply `c:dispBlanksAs`. `gap` creates
separate contiguous path segments, `zero` inserts baseline control points, and
`span` connects the present points. Zero control points are compressed at the
boundaries of missing runs, which preserves the same straight baseline without
allocating one point for every value of an untrusted declared count. Markers
remain limited to points present in the cache.

Geometry normalises domains after scaling by their largest finite magnitude,
so opposite finite extremes do not overflow their range. Stacked values,
percentage totals, pie totals, derived bounds, and every emitted path point
must remain finite. An overflow or nonfinite mutable cache returns a contextual
error before backend-neutral geometry is exposed.

Before geometry is lowered, each typed series receives one resolved colour.
A direct solid fill in its `c:spPr` wins, followed by a direct solid line fill.
A direct `a:noFill` is transparent. A present gradient, pattern, picture, or
colourless solid paint returns a contextual series projection error instead of
silently falling back. Without direct paint, series use accent1 through
accent6 in order and repeat after six. The semantic accent passes through the
effective colour map, concrete theme colour scheme, and ordered transform
stack in `oxml-drawing`. The deliberately naive Word tint and shade helper is
not part of this path.

The labelled entry point computes linear scales from cached values and targets
six ticks. The step is 1, 2, or 5 times a power of ten. Unpinned bounds expand
to enclosing step multiples, while a `c:scaling` minimum or maximum remains
exact. Ordinary bar and area value domains include zero. A 0 through 100 value
axis therefore emits 0, 20, 40, 60, 80, and 100. Constant, fractional,
negative, mixed-sign, and large finite domains produce increasing finite
ticks. Standard line and scatter domains use only their rendered values without
forcing zero in both labelled and geometry-only entry points. Scatter domains
use only logical indexes with a rendered x and y pair. Zero-valued bars retain
label anchors, and an all-zero bar chart remains renderable. `c:orientation`
reverses both tick coordinates and plot geometry.

Major gridlines emit first, followed by one clipped plot group, axis lines and
major tick marks, legend swatches, then text. Deleted axes emit none of those
annotations. Axis position controls the plot edge, tick direction, and default label side.
`c:tickLblPos` can move labels to the high or low side or suppress them.
Category tick and gridline positions cover the full logical cache count even
when category text is absent or sparse. Category text retains its cached
logical indexes and emits only at present slots. Annotation expansion is
limited to 16,384 logical categories. A larger declared count returns a
contextual error before allocating annotations. Radar charts instead emit
category spokes, category labels at distinct outer, inner, or next-to-spoke
radii, and concentric value gridlines. Radar value labels likewise use distinct
high, low, and next-to-axis positions. High category-label origins are clamped
to the label space reserved around the plot, including for narrow plots.

Category labels, numeric tick labels, requested data-label fields, and legend
series names are shaped into `GlyphRun` values by the caller's `FontManager`.
The fallback chart style is Carlito at 9 points with black text. Modelled axis
default-run properties override the typeface, point size, bold, and italic
values. Axis, numeric category, and data label values use the implemented
`NumberFormat` subset when a format is present, and deterministic General
formatting otherwise. The category-axis format overrides the numeric category
cache format. Unsupported effective projections return a contextual error. Data
labels project the supported series name, category name, value, percentage, and
bubble-size flags with the declared separator and position. Percentage totals
are checked only when an effective collection or point label requests them.
The effective number format controls percentage precision. Bubble sizes join
the value cache by preserved logical index. Label anchors come from the same
family geometry, scales, bounds, and orientations as the plotted marks.
Inside-base, inside-end, and outside-end positions follow the rendered bar
segment or radial slice geometry. Inside displacement is clamped to short bars
and thin rings. Bar retention uses the data endpoint, so an off-plot zero
baseline does not suppress an in-range value and an out-of-range endpoint does
not retain a label. Retained bar anchors derive from the clipped visible
segment. If clipping collapses that segment to a point, inside positions remain
at that point and outside-end keeps the original vertical or horizontal value
axis direction. A zero-radius radar anchor likewise keeps its category-spoke
direction. Radar default domains include zero and preserve negative values for
all-negative and mixed-sign caches. Radar points outside explicit normal or
reversed value bounds do not emit geometry or labels. A nonempty radar cache
whose points are all outside explicit bounds still emits its axes and other
annotations. Individual
`c:dLbl` delete, visibility, number-format, position, and separator overrides
are projected privately by logical index from their namespace-resolved raw
subtrees. Those subtrees remain byte-preserved as the only serialization source.
A present legend shell emits one resolved series-colour swatch and shaped
series name per row in the upper-right of the plot. Bars, line and radar
strokes, pie and doughnut wedges, areas, scatter and line markers, and legend
swatches all consume the same resolved series colour. Unsupported legend
placement children remain preserved and do not change this default layout.

Package assembly resolves chart relationships in separate slide, layout and
master scopes and parses internal targets as `CT_ChartSpace`. The resolver
passes supported charts, local frame bounds, the effective theme and colour map,
and the caller's font manager to the native chart entry point. It freezes the
returned group in `ResolvedContent`, so the ordinary presentation renderer can
lower it without a chart-specific backend path.

For a chart that was **preserved rather than authored**, the resolver draws the
immediate typed cached-picture fallback when the native projection is
unsupported, the embedded image resolves in the same source scope, its bytes
match the declared PNG or JPEG content type, encoded bytes and decoded scanline
or pixel storage stay within 16 MiB caps, JPEG uses the 8-bit three-component
layout shared by the raster and PDF backends, the stricter raster boundary
visibly decodes it at native pixel bounds, and the resolver accepts that
renderer content type. The corpus renderer and integration gates call the same
crate-local package rendering function.
Otherwise it emits a labelled placeholder rectangle. Both routes record the
stable chart diagnostic. Missing previews, unsupported preview formats,
malformed or invisible preview bytes, content-type mismatches, and missing,
external, missing-target or malformed chart relationships keep their more
specific source context. The preserved chart and alternate-content bytes remain
the sole serialisation source.

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

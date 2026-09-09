# oxml-chart

Model, edit, serialize, and render format-neutral ChartML with typed chart data
and geometry.

## Capabilities

- Typed axes, series, titles, legends, labels, and seven plot families.
- Chart XML authoring with an editable embedded workbook.
- Backend-neutral chart geometry for document-family renderers.
- Ordered preservation of unmodelled XML around typed edits.

## Use it when

Use this crate when an application needs ChartML models or chart rendering
without depending on DOCX or PPTX APIs. Use `rpptx` for charts inside a
complete presentation.

## Relationship

It uses focused one-sheet SpreadsheetML workbooks from `oxml-sml` and feeds
backend-neutral chart geometry into document-family renderers. It is not a
complete OOXML package editor or a general spreadsheet library.

## Example

```rust,no_run
use oxml_chart::AxisId;

let category_axis = AxisId::new(10_000_001)?;
let value_axis = AxisId::new(10_000_002)?;
assert_ne!(category_axis, value_axis);
# Ok::<(), oxml_chart::ChartError>(())
```

Add `oxml-chart = "0.11.0"` to your dependencies. See the [chart API](https://docs.rs/oxml-chart) for supported plot families.

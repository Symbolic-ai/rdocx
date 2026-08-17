# oxml-sml

`oxml-sml` writes the deliberately minimal SpreadsheetML workbooks embedded
behind editable OOXML charts. It supports one worksheet, text and numeric
columns, shared strings, and one number format per numeric column.

## Use it when

Use this crate when an OOXML chart needs an editable `.xlsx` data workbook.
Use a general spreadsheet library for standalone workbook applications.

## Relationship

`oxml-chart` uses this crate for chart data. It builds the workbook package on
the shared `oxml-opc` layer and does not depend on presentation code.

## Example

```rust,no_run
use oxml_sml::{Column, Workbook};

let workbook = Workbook::new(
    "Sales",
    vec![Column::Number {
        header: "Revenue".into(),
        values: vec![120.0, 150.0],
        number_format: Some("$0.00".into()),
    }],
)?;
let bytes = workbook.to_xlsx_bytes()?;
assert!(!bytes.is_empty());
# Ok::<(), oxml_sml::Error>(())
```

This crate is not a general spreadsheet library. It does not read workbooks or
provide formulas, multiple worksheets, charts, or arbitrary cell styling.

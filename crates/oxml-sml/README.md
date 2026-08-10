# oxml-sml

`oxml-sml` writes the deliberately minimal SpreadsheetML workbooks embedded
behind editable OOXML charts. It supports one worksheet, text and numeric
columns, shared strings, and one number format per numeric column.

This crate is not a general spreadsheet library. It does not read workbooks or
provide formulas, multiple worksheets, charts, or arbitrary cell styling.

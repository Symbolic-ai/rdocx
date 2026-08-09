# F-117, all, pass 1

**Reviewed**: working-tree diff, 7 files, 820 added lines and 0 removed lines. This includes 31 tracked additions and the 789 untracked lines in `crates/oxml-sml/Cargo.toml`, `crates/oxml-sml/README.md`, and `crates/oxml-sml/src/lib.rs`.
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, string inputs can produce malformed or changed SpreadsheetML

`crates/oxml-sml/src/lib.rs:109`

Construction validates numeric values but does not validate or encode headers,
text values, or number-format strings for the XML 1.0 and SpreadsheetML string
boundary. Those values are later passed directly to `BytesText::new` at line
423 or to an XML attribute at line 474. For example, a header containing NUL
causes `to_xlsx_bytes` to return `Ok` with a malformed shared-strings part.
A carriage return can also be normalized when the XML is read back instead of
preserving the caller's value. Reject XML-illegal scalar values and encode the
SpreadsheetML string cases that XML escaping alone cannot preserve. Add exact
escaping and rejection tests for headers, text values, and number formats.

### D2, column lengths can exceed the worksheet row limit

`crates/oxml-sml/src/lib.rs:212`

The maximum column length is not validated before this loop emits every value
after the row-one header. A column with 1,048,576 values therefore writes row
1,048,577 and `formula_range` returns the same out-of-range address, even
though SpreadsheetML ends at row 1,048,576. This produces a workbook outside
the promised editable boundary. Reject any column longer than 1,048,575 data
values and cover both the accepted boundary and the first rejected length.

### D3, the external viewer gate does not test either viewer

`crates/oxml-sml/src/lib.rs:750`

The gate named `generated_workbook_opens_cleanly_in_excel_and_libreoffice_calc`
only writes an `.xlsx` file. It never invokes Excel or LibreOffice Calc, checks
their pinned versions, binds observations to an artifact SHA, verifies the one
worksheet and expected cells, or fails on a repair or conversion error. It
therefore passes without proving the story's declared gate. Implement the
pinned two-viewer checks and retain SHA-bound evidence as required by the
approved plan.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in correctness, contract, panics, OOXML schema order,
namespace usage, relationship targets, deterministic allocation, tests, or
structure. The new crate remains within its authorized one-sheet scope and
uses only the concrete dependencies named by the plan.

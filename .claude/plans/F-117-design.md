# F-117, oxml-sml workbook writer

**Status**: completed
**Sprint**: S29
**Size**: L
**Depends on**: none

## Problem

Charts require an embedded workbook whose ranges are the source addressed by
ChartML formula references. The workspace has no `oxml-sml` member or
dependency entry at `Cargo.toml:3`, while the chart contract requires a
deliberately narrow SpreadsheetML writer at `docs/hld/09-charts-spec.md:32`.
Without a valid nested `.xlsx`, PowerPoint may render cached chart values but
its Edit Data operation is broken.

The existing `OpcPackage` can already construct deterministic ZIP packages,
content types, and relationship scopes. It does not know the SpreadsheetML
part graph or how to encode worksheet cells, shared strings, column formats,
and formula-addressable ranges.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Charts" and "Non-goals,
  permanently".
- `docs/hld/03-architecture.md`, "Three families, one workspace" and "The
  dependency rule".
- `docs/hld/04-opc-and-packaging.md`, "Part naming" and "The package".
- `docs/hld/09-charts-spec.md`, "Why a chart needs three parts" and
  "`oxml-sml`, deliberately minimal".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/14-development-backlog.md`, "F-117, oxml-sml workbook writer".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Create `crates/oxml-sml` as an unpublished workspace crate at version 0.0.0.
The story authorizes the minimal required `Cargo.toml`, `README.md`, and
`src/lib.rs`. The README and crate docs state that this is a chart-workbook
writer, not a general spreadsheet library. Register the crate as a workspace
member and workspace dependency. Its normal dependencies are concrete:
`oxml-opc` for the nested package, `quick-xml` for escaped schema-ordered XML,
and `thiserror` for one contextual error enum. No trait, generic parameter,
feature flag, builder, or forwarding wrapper is added.

Expose the HLD data surface with direct construction rather than a builder:

```rust
pub enum Column {
    Text { header: String, values: Vec<String> },
    Number {
        header: String,
        values: Vec<f64>,
        number_format: Option<String>,
    },
}

pub struct Workbook {
    sheet_name: String,
    columns: Vec<Column>,
}

impl Workbook {
    pub fn new(sheet_name: impl Into<String>, columns: Vec<Column>) -> Result<Self>;
    pub fn to_xlsx_bytes(&self) -> Result<Vec<u8>>;
    pub fn formula_range(&self, column: usize) -> Option<String>;
}
```

Validate the worksheet name, finite numeric values, nonempty column set, and
Excel column-address limit before serialization. Column lengths may differ.
Each formula range covers that column's present data rows, beginning after its
header, and quotes the sheet name according to SpreadsheetML rules.

Build exactly one worksheet with headers in row 1. Intern all text headers and
values into a deterministic shared-string table. Numeric cells write their
finite decimal value directly. When a numeric column requests a format, emit
the minimal `xl/styles.xml` and assign a stable style index to every cell in
that column. Omit the styles part when no format is requested.

Use `OpcPackage` to emit `[Content_Types].xml`, `_rels/.rels`,
`xl/workbook.xml`, `xl/_rels/workbook.xml.rels`,
`xl/worksheets/sheet1.xml`, optional `xl/sharedStrings.xml`, and optional
`xl/styles.xml`. Add the SpreadsheetML content-type and relationship constants
to `oxml-opc` because that format-neutral crate already owns the shared OPC
vocabulary. Output order and shared-string allocation are deterministic.

## Rejected alternatives

- Add spreadsheet reading, formulas, multiple worksheets, charts, or general
  cell styling. Those violate the permanent `oxml-sml` non-goal.
- Construct ZIP bytes directly in `oxml-sml`. That duplicates the tested OPC
  writer and creates a second source of package behavior.
- Add a builder for `Workbook`. The concrete value has two fields, so the
  structural rules prohibit a builder.
- Store only inline strings. The story explicitly calls for shared strings,
  and deterministic interning is small and concrete.
- Require equal column lengths. Each ChartML series owns its own formula range,
  so rectangular padding would invent source cells.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `formula_ranges_quote_sheet_names_and_track_column_lengths` | A1 addresses, sheet quoting, headers, empty data columns, and differing lengths are exact |
| unit | `invalid_workbook_inputs_fail_before_package_construction` | Invalid names, no columns, excess columns, and nonfinite values return contextual errors |
| integration | `workbook_package_has_the_minimal_editable_part_graph` | Content types, relationships, worksheet, shared strings, optional styles, and defined data ranges resolve after reopen |
| regression | `equal_strings_share_one_stable_shared_string_index` | Repeated headers and values reuse one index and repeated serialization is byte-identical |
| integration, gate | `generated_workbook_opens_cleanly_in_excel_and_libreoffice_calc` | The same SHA-bound `.xlsx` opens as one worksheet with expected cells and no repair or conversion error in pinned Excel and LibreOffice Calc |

The test gate is: the produced `.xlsx` opens in Excel and LibreOffice Calc.

## HLD impact

- `docs/hld/09-charts-spec.md`
- `docs/hld/15-build-and-toolchain.md`

Document the exact implemented part set, validation boundary, optional styles
part, formula-range behavior, unpublished crate status, and pinned viewer gate.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Write SpreadsheetML in
  schema order with fixed prefixes, reopen the OPC package, and assert exact
  XML escaping and part relationships.
- Crate dependency graph and a new cross-family use. Run
  `cargo tree -p oxml-sml --edges normal` and prove that the new `oxml-*` crate
  depends only on `oxml-opc` and external libraries, never an `rdocx-*` or
  `rpptx-*` crate.
- A new crate, module, or file. The explicit F-117 invocation authorizes the
  crate and its minimal manifest, README, and single source file. Run Cargo
  metadata and package-file inspection so no speculative file enters it.
- External oracle comparison. Record the exact Excel and LibreOffice Calc
  versions, bind both observations to one workbook SHA, and treat a repair
  prompt as failure.
- Version strings and packaging. Inspect the root manifest, new manifest,
  lockfile, HLD publishing text, and package contents. Require version 0.0.0,
  `publish = false`, an archive below 10 MiB, no release allowlist change, no
  tag, and no publication.

## Hash harness

Expected unchanged. The new unpublished SpreadsheetML crate is not consumed by
Word sample generation or rendering. All 28 hashes must match.

## Implementation checklist

- [x] Register the minimal unpublished `oxml-sml` crate and dependencies.
- [x] Add validated workbook and column values without a builder.
- [x] Generate deterministic workbook, worksheet, shared-string, and optional
      style XML.
- [x] Assemble and reopen the complete nested OPC package.
- [x] Add formula-range, negative, package-graph, determinism, and viewer tests.
- [x] Update exactly HLD 09 and HLD 15.
- [x] Run focused checks, dependency and package riders, the viewer gate,
      microscope, and worker preparation.

## Open questions

None. F-117 and the chart HLD explicitly authorize the minimal crate, API,
package graph, and Excel plus LibreOffice acceptance boundary.

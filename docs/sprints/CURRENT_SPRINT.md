# Current Sprint, S70

**Milestone**: M19 Advanced spreadsheets.

**Goal**: decide whether a material Rust spreadsheet lifecycle gap still
exists before opening M19 implementation. If the decision is affirmative,
establish the pinned compatibility corpus and the first loss-aware workbook
ownership model. If it is negative, archive M19 without implementing it.

## Spec references

- `docs/hld/02-scope-and-non-goals.md`, for the conditional expansion rule,
  the current `oxml-sml` boundary, and the spreadsheet execution capabilities
  that remain permanent non-goals.
- `docs/hld/03-architecture.md`, for the existing format-neutral workspace
  layers and the dependency direction a distinct spreadsheet family must
  preserve if M19 proceeds.
- `docs/hld/04-opc-and-packaging.md`, for deterministic OPC ownership,
  normalized relationships, content types, and loss-free retention of
  unsupported workbook parts.
- `docs/hld/12-testing-strategy.md`, for regression, round-trip, corpus,
  external-oracle, and source-built test boundaries.
- `docs/hld/14-development-backlog.md`, for the M19 go or no-go condition, the
  milestone capability classification, its end gate, and the F-184, F-204,
  and F-185 acceptance contracts.

## The wave

| F-ID | Title | Size | Status | Owner |
|------|-------|------|--------|-------|
| F-184 | Advanced spreadsheet go or no-go | S | pending | - |
| F-204 | Spreadsheet corpus and compatibility matrix | M | pending | - |
| F-185 | Workbook and worksheet model | L | pending | - |

## Sequencing note

Rows are listed in dependency order. F-184 is a true go or no-go gate and must
land first. F-204 and F-185 may begin only after an affirmative decision. If
F-184 finds that a credible maintained Rust library already provides the
required loss-aware lifecycle, M19 is archived and neither dependent story is
implemented.

## Definition of done for this sprint

- The maintained Rust spreadsheet ecosystem is reassessed against the complete
  M19 lifecycle, not simple read, write, or formula support alone.
- One non-contradictory decision and capability matrix classifies every
  scheduled spreadsheet feature as preserve, model and edit, or execute and
  refresh.
- If M19 proceeds, a pinned licensed XLSX corpus verifies every checksum,
  rejects unpinned workbooks, records Excel and LibreOffice identities
  separately, and declares the capability class of every advanced part.
- If M19 proceeds, the first workbook and worksheet model preserves workbook,
  sheet, row, column, cell, merged-range, defined-name, relationship, and
  unsupported package ownership through load and save.
- If M19 is archived, the decision record explains which maintained ecosystem
  surface satisfies the complete required lifecycle and no implementation
  story starts.
- Binary `.xls`, unsupported execution, proprietary cloud connectors, VBA,
  XLM, custom functions, Python cells, and Microsoft-hosted services remain
  outside the execution boundary.

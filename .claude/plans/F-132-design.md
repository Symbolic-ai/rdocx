# F-132, Python enums, units and exceptions

**Status**: completed
**Sprint**: S33
**Size**: M
**Depends on**: F-129, F-130

## Problem

The workspace has no Python unit, enum, or exception classes. The canonical
Rust `Length` is not a Python integer subclass and deliberately truncates its
floating constructors (`crates/oxml-core/src/length.rs:5`,
`crates/oxml-core/src/length.rs:11`). Rust alignment and formatting enums carry
no Python numeric compatibility contract (`crates/rdocx/src/paragraph.rs:15`),
and `rdocx::Error` is a Rust enum rather than the required catchable Python
hierarchy (`crates/rdocx/src/error.rs:5`).

## Spec reference

- `docs/hld/01-glossary.md`, "EMU" and "Twip".
- `docs/hld/10-bindings-spec.md`, "Python API shape" and "Packaging".
- `docs/hld/14-development-backlog.md`, "F-132, Python enums, units and exceptions".
- `docs/hld/15-build-and-toolchain.md`, Python package version alignment and binding exclusions.

## Approach

After the F-130 mixed Python package skeleton is integrated, implement
immutable pure-Python
`Length(int)` subclasses with exact EMU factors and `int(value * factor)` so
positive and negative fractional inputs match Rust truncation. Provide
`Length`, `Inches`, `Cm`, `Mm`, `Pt`, `Emu`, and `RGBColor`, with the documented
unit properties.

Implement pure-Python `IntEnum` shims for the S33 consumers:
`WD_ALIGN_PARAGRAPH`, `WD_TABLE_ALIGNMENT`, `WD_CELL_VERTICAL_ALIGNMENT`, and
`WD_UNDERLINE`. Export them at the package top level and through
python-docx-shaped `rdocx.shared`, `rdocx.enum.text`, and `rdocx.enum.table`
namespaces.

Define `RdocxError` as the base for `PackageError`, `XmlError`,
`StaleElementError`, and `LayoutError`. F-130 and later bindings raise these
registered classes through concrete mapping code. Keep all enum values as
checked literals. Do not add python-docx as a runtime dependency or an
unversioned test oracle.

The sprint wave records F-130 as the scaffolding prerequisite.

## Rejected alternatives

- Use Rust pyclasses for units or enums. Native-int inheritance conflicts with
  `abi3-py39`, and PyO3 enums are not Python `IntEnum` values.
- Change Rust constructors to round. Their truncation is pinned and affects
  document layout.
- Raise generic runtime errors. Callers need the specified hierarchy.
- Add every python-docx enum now. F-135 owns broad parity and will identify any
  remaining documented surface.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit, gate | `alignment_center_and_inches_match_python_contract` | `WD_ALIGN_PARAGRAPH.CENTER == 1` and `Inches(1) == 914400` |
| unit | `length_is_an_int_with_unit_properties` | Every approved constructor and property uses canonical EMU values |
| regression | `fractional_lengths_truncate_toward_zero` | Positive and negative fractional inputs match Rust behavior |
| unit | `approved_enums_have_exact_values_and_docs` | The bounded enum inventory has stable integer values |
| unit | `exceptions_have_the_required_hierarchy` | Every concrete binding error is catchable as `RdocxError` |
| integration | `binding_errors_raise_public_exception_classes` | F-129 and F-130 mappings use the package classes |

The first unit test is the verbatim backlog gate. Focused checks use the
approved Python test runner plus `cargo check -p rdocx-py --all-targets` after
F-130 is integrated.

## HLD impact

- `docs/hld/10-bindings-spec.md`
- `docs/hld/14-development-backlog.md`

The HLD must record pure-Python ownership, the bounded S33 inventory, and the
real F-130 scaffolding dependency.

## Risk routing

- Unit conversion. Preserve the pinned `as i64` behavior and test positive and
  negative truncation.
- WASM or PyO3 bindings. Retain binding exclusions and run the existing rdocx
  WASM target check.
- Crate dependency graph. Read `docs/hld/03-architecture.md`, "The dependency
  rule" and "Why these seams". Inspect
  `cargo tree -p rdocx-py --edges normal` and
  `cargo tree -p rdocx-py --edges normal,dev`. Confirm the test-only
  `rdocx-py -> oxml-layout` edge points inward, creates no format-family cycle,
  and does not leak PyO3 into format-neutral crates.
- New module or file. Obtain explicit approval for the package namespaces and
  one dedicated Python test file.

## Hash harness

Expected unchanged. Pure binding package additions do not affect Rust document
serialization or rendering.

## Implementation checklist

- [x] Add the pure-Python unit and RGB color types.
- [x] Add the bounded text and table `IntEnum` inventory.
- [x] Add the exception hierarchy and concrete binding mappings.
- [x] Export top-level and python-docx-shaped module paths.
- [x] Add unit, integration, and truncation regressions.
- [x] Run focused checks and every risk rider.

## Open questions

None. F-130 scaffolding order, the exact S33 inventory and import paths, and the
new pure-Python namespace and test files were approved together.

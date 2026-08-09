# F-117, all, pass 3

**Reviewed**: full working-tree implementation diff, 9 files, 1,140 added
lines and 5 removed lines. This includes all 1,072 untracked lines in
`crates/oxml-sml/Cargo.toml`, `crates/oxml-sml/README.md`, and
`crates/oxml-sml/src/lib.rs`. Pass 1, pass 2, and the progress notes were
inspected separately.
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

No findings remain in correctness, contract, panics, OOXML schema order,
namespace usage, package relationships, deterministic allocation, tests, or
structure.

Pass 2 D1 is resolved. The shared SpreadsheetML encoder handles tab, line
feed, carriage return, the remaining XML controls, U+FFFE, U+FFFF, and reserved
`_xHHHH_` input. It runs before shared-string text, worksheet-name attributes,
and number-format attributes are written, so XML attribute normalization
cannot change the encoded value. The exact test covers XML metacharacters,
leading and trailing whitespace, illegal raw control bytes, and each required
SpreadsheetML escape class.

Pass 2 D2 is resolved. `Workbook::new` validates the aggregate header and text
value reference count against `u32::MAX`, `SharedStrings::from_columns` repeats
that fallible boundary check, and unique-string interning returns an error if
its index is unrepresentable. The accepted boundary and first rejected count
are covered without constructing a multi-gigabyte workbook.

The seven automated tests and strict Clippy check pass. The normal dependency
tree contains only `oxml-opc`, `quick-xml`, `thiserror`, and their external
dependencies. The retained viewer artifact and the regenerated candidate both
match SHA-256
`8f8d12aa4ebe94f86c8164fd251cdb23845f985090be0fb6c77242aaa0fba329`,
which is the digest recorded for the Excel and LibreOffice Calc observations.

# F-X054, integration recovery, pass 1

**Reviewed**: complete staged squash integration against `dc9d53f`, 39 files
and 5,112 changed lines with 5,003 additions and 109 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Verification

- The expanded ODT regression builds retained numbering root attributes and
  XML, instance attributes and XML, abstract type metadata, abstract
  attributes and XML, and every diagnosed level-local loss at
  `crates/rdocx/src/odt.rs:5472`. It applies the same producer format to two
  body paragraphs and one table-cell paragraph at
  `crates/rdocx/src/odt.rs:5514`.
- The regression rejects every ODT list element at
  `crates/rdocx/src/odt.rs:5548`, then compares the complete ordered 15-item
  numbering diagnostic vector at `crates/rdocx/src/odt.rs:5557`. Exact vector
  equality proves one occurrence of every container and level loss after the
  three visits. Reopen assertions prove absent numbering for both body uses
  and the table-cell use at `crates/rdocx/src/odt.rs:5627`.
- The implementation identifies producer-defined levels before list-style
  allocation at `crates/rdocx/src/odt.rs:471`. It scans container and level
  losses at `crates/rdocx/src/odt.rs:1185` and
  `crates/rdocx/src/odt.rs:1248`, suppresses list projection at
  `crates/rdocx/src/odt.rs:1098`, and deduplicates by exact path and message at
  `crates/rdocx/src/odt.rs:1718`.
- The earlier EPUB integration fixes remain present. Producer-defined formats
  receive accurate loss diagnostics at `crates/rdocx/src/epub.rs:799`, and
  list detection returns no list at `crates/rdocx/src/epub.rs:2554`.
- The complete public merge retains EPUB, ODT, and SVG result exports together
  with every F-X054 ordered-reader export at `crates/rdocx/src/lib.rs:46`.
  Python retains `XmlError` for OOXML failures and the generic `RdocxError` for
  HTML and ODT failures at `crates/rdocx-py/src/lib.rs:66`.
- The HLD edits are limited to the four files named by the F-X054 plan. They
  record producer-defined numbering at `docs/hld/04-opc-and-packaging.md:165`,
  the ordered native facade at `docs/hld/10-bindings-spec.md:451`, and the
  source-built regression boundary at `docs/hld/12-testing-strategy.md:53`.
- The exact producer regression passed 1 test. The ODT filter passed 41 library
  tests and its public round trip. Its pinned LibreOffice oracle passed 1 test
  in the required unsandboxed rerun. The EPUB filter passed 34 ordinary tests,
  and the exact checksum-verified EPUBCheck 5.3.0 test passed separately.
- Layout numbering passed 14 tests, low-level OOXML numbering passed 51 tests,
  and the complete native regression binary passed 169 tests. Checks for all
  `rdocx` and `rdocx-py` targets passed, as did the focused Python error-mapping
  unit test.
- `cargo fmt --all --check`, staged prose checking, and staged diff checking
  passed with zero violations.

## Not found

No additional correctness, contract, panic, OOXML preservation, test-gate,
public-export, binding, HLD-discipline, or structural defect was found. The
earlier integration repairs continue to preserve modeled numbering behavior,
reject malformed visible text, replay retained namespace scope safely, expose
direct ordered facts without changing legacy flattened accessors, and keep
the combined F-180, F-181, F-182, and F-X054 native surfaces coherent.

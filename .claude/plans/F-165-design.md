# F-165, Repeating table rows and lists

**Status**: approved
**Sprint**: S50
**Size**: M
**Depends on**: F-164

## Problem

Generic structural repetition can clone typed rows and paragraphs, but Word
tables and numbered lists carry behavior in properties outside their visible
text. A repeated row group must retain table banding, grid spans, vertical
merge restarts and continuations, row properties, and preserved producer XML.
A repeated list item must retain one numbering definition and level so Word
continues the sequence instead of restarting it for each record.

`CT_Row` and `CT_Tbl` retain cloneable row, grid, property, raw XML, and content
control state at `crates/rdocx-oxml/src/table.rs:1420` and
`crates/rdocx-oxml/src/table.rs:1596`. Paragraph numbering is stored on the
paragraph properties and the facade already preserves numbering definitions
during ordinary document mutation.

## Spec reference

- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/06-presentationml-model.md`, "Preservation strategy".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".
- `docs/hld/14-development-backlog.md`, "Milestone 16, Document automation"
  and "F-165, Repeating table rows and lists".

## Approach

Extend F-164 row blocks so one loop may own multiple adjacent template rows.
The start and end markers are dedicated rows and are removed. Each iteration
deep-clones every row between them in source order, then renders nested blocks
and scalar tags within the clone. The table itself is not rebuilt, so its
style, look and banding properties, grid, raw XML boundaries, and relationships
remain unchanged. Cloned row and cell properties retain horizontal and vertical
merge semantics.

Treat a paragraph loop whose body contains numbered paragraphs as a repeated
list block. Clones retain the original `numId` and level. No new numbering
instance or abstract definition is allocated, so Word continues numbering over
every generated item. Mixed numbered and ordinary paragraphs remain in source
order. Invalid row-container structure and invalid numbering references fail in
preflight without mutating the document.

No new public method, type, trait, file, dependency, or feature flag is added.
The behavior remains part of `Document::render_template`.

## Rejected alternatives

- Rebuilding a table through the high-level facade is rejected because it would
  discard row, cell, merge, banding, and unmodelled XML state.
- Allocating a numbering instance per loop iteration is rejected because Word
  would restart the list for each generated item.
- Special row and list wrapper types are rejected because the existing typed
  structures already carry the required behavior.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | `three_template_rows_over_ten_records_produce_thirty_preserved_rows` | The F-165 test gate: ten records produce thirty data rows, table banding remains enabled, grid spans and vertical merges match the three-row template, and visible values follow record order. |
| regression | `repeated_numbered_items_keep_one_continuous_sequence` | Repeated list paragraphs retain one `numId` and level across every generated item and render as a continuous list after reopen. |
| round-trip | `repeated_rows_and_lists_preserve_properties_and_raw_xml` | Save and reopen retain row, cell, table, numbering, and unmodelled XML state in schema order. |

The test gate is **regression**. A three-row template over ten records produces
thirty rows with the banding and numbering intact.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- **Any parser or serialiser**. Read
  `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add the row and list round-trip test
  above and verify schema order and byte-for-byte preservation of unmodelled
  row and cell subtrees.
- **Public API of a published crate**. This story extends the documented
  behavior of the F-163 method. Read `docs/hld/10-bindings-spec.md` and the
  `CLAUDE.md` structural rules, run the full package dry-run, and assert every
  `.crate` remains within the 10 MiB limit.

## Hash harness

Expected to be unchanged. Table and list generation is opt-in and no sample
invokes it.

## Implementation checklist

- [ ] Extend row-loop evaluation to clone multi-row template groups in order.
- [ ] Preserve table, row, cell, merge, banding, content-control, and raw XML
  state through repetition.
- [ ] Preserve one numbering identity and level across repeated list items.
- [ ] Validate row containers and numbering references before mutation.
- [ ] Add the thirty-row and continuous-numbering regression fixtures.
- [ ] Add round-trip assertions for table, list, and unmodelled XML state.
- [ ] Update the HLD with table and list repetition invariants.

## Open questions

None. The consolidated sprint design approval selected dedicated removable
marker rows around multi-row groups and preservation of the source list
`numId` and level for continuous numbering.

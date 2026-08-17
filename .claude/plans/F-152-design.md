# F-152, Content control model

**Status**: completed
**Sprint**: S46
**Size**: L
**Depends on**: none

## Problem

Body parsing preserves `w:sdt` as one opaque raw element instead of traversable
content (`crates/rdocx-oxml/src/document.rs:600`). Paragraph parsing does the
same at run level (`crates/rdocx-oxml/src/text.rs:428`). Table cells retain raw
children at indexed boundaries, but their typed content inventory contains
only paragraphs and nested tables. The current model can therefore keep a
content control byte-for-byte or expose the content it wraps, but cannot do
both and cannot report its tag, alias, id, or type at all five required levels.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-152, Content control model".
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "Package integrity".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".

## Approach

Add one recursive WordprocessingML content-control model. `CT_Sdt` owns a typed
`CT_SdtPr`, an ordered content sequence, and raw slots for every property and
child not needed for reporting or later binding. `CT_SdtPr` parses tag, alias,
numeric id, the bounded control type marker, and an optional `w:dataBinding`
record while preserving all other properties in their original positions.

Represent the five legal placements explicitly in their existing parent
inventories:

- block controls in `BodyContent`
- row controls in the table row sequence
- cell controls in the row cell sequence
- paragraph controls in cell and block content sequences
- run controls in the paragraph run sequence

Traversal APIs recursively expose controls in document order and continue to
expose the ordinary paragraphs, rows, cells, and runs inside `w:sdtContent`.
Writing emits typed content at the same parent position and preserves
unmodelled attributes, properties, and children byte-for-byte. Empty or
malformed controls remain raw rather than being silently discarded.

Proposed new source file:

```text
crates/rdocx-oxml/src/content_control.rs
```

No trait, generic parameter, crate, dependency, or feature flag is added.

## Rejected alternatives

- Keep unwrapping controls only during table traversal. That loses their
  identity and prevents binding or mutation.
- Use one `RawXml` field for the whole control and parse metadata on every
  accessor. That creates two competing sources of truth after mutation.
- Add five unrelated control structs. One OOXML `CT_Sdt` grammar with explicit
  parent placement reduces the cases readers must consider.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `sdt_properties_report_tag_alias_id_type_and_binding` | Each typed property parses across alias prefixes and unmodelled properties keep order |
| round-trip | `controls_at_all_five_levels_round_trip_without_losing_content` | Block, row, cell, paragraph, and run controls reload at the same nesting level with the same metadata and visible content |
| round-trip | `unmodelled_sdt_properties_and_children_remain_byte_identical` | Producer attributes, whitespace, extensions, and unsupported control types retain exact bytes and positions |
| regression | `table_traversal_sees_rows_cells_and_paragraphs_inside_controls_once` | Existing table and paragraph iterators neither skip nor duplicate wrapped content |

The **test gate**, from the backlog, is round-trip. Controls at all five nesting
levels survive, and each reports its tag, alias, id, and type.

Tests stay in crate-local modules and existing integration entrypoints. No new
test binary is added.

## HLD impact

- `docs/hld/03-architecture.md`

Record the recursive content-control ownership and traversal contract in the
WordprocessingML and facade boundaries.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add schema-order,
  prefix-tolerant, malformed-control, recursive round-trip, and exact raw
  preservation coverage at every parent level.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  typed low-level model and traversal accessors are additive. Run affected
  package dry-runs and archive size assertions.
- A new module or file. The recursive content-control module needs explicit
  approval and replaces five copies of the same property parser.

## Hash harness

Expected unchanged across all 49 entries. The sample generator contains no
content controls, and ordinary table traversal must remain identical.

## Implementation checklist

- [x] Add `CT_Sdt`, typed properties, binding metadata, and ordered raw slots.
- [x] Integrate the model at block, row, cell, paragraph, and run levels.
- [x] Preserve ordinary traversal through each wrapped content sequence.
- [x] Add five-level, alias, malformed, traversal, and raw-byte tests.
- [x] Run focused checks plus the declared packaging rider.
- [x] Update exactly HLD 03 at completion.

## Open questions

None. The single recursive `crates/rdocx-oxml/src/content_control.rs` module is
approved.

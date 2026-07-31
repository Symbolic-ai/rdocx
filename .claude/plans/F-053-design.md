# F-053, OrderedRawChildren

**Status**: completed
**Sprint**: S12
**Size**: M
**Depends on**: none

## Problem

OOXML child order is schema-significant. The DrawingML contract warns that
putting a modelled child in the wrong position causes a PowerPoint repair
prompt, and requires unknown children to remain in their original slots at
`docs/hld/05-drawingml-model.md:38`. Existing raw capture can preserve one
subtree byte for byte at `crates/oxml-core/src/raw_xml.rs:11`, but it does not
record that subtree's position relative to known siblings.

## Spec reference

- `docs/hld/03-architecture.md`, "Crate-level conventions".
- `docs/hld/05-drawingml-model.md`, "Two traps that are silent until PowerPoint
  refuses the file" and "Preservation".
- `docs/hld/06-presentationml-model.md`, "Preservation strategy".
- `docs/hld/13-risks-and-open-questions.md`, "R5, schema child ordering".
- `docs/hld/14-development-backlog.md`, "F-053, OrderedRawChildren".

## Approach

Add `order.rs` to `oxml-drawing` with a concrete `OrderedRawChildren` type. Each
captured raw child stores its byte-for-byte XML and a caller-defined schema
boundary. Owning parsers record unknown children at the boundary between two
known schema positions. Writers query every boundary in schema order, including
boundaries around absent optional children, and emit the raw slices there.

Keep the helper concrete. It stores raw child positions and bytes only, while
the owning element captures through `oxml_core::raw_xml` and writes through its
existing quick-xml sink. Expose `push`, `at`, and `is_empty`, with `at` yielding
borrowed byte slices in insertion order. This avoids a writer generic, a new
generic model parameter with one current instantiation, and a callback trait
with one implementer. F-053 needs `oxml-core` and quick-xml only as test
dependencies for its test-local parent parser and writer.

## Rejected alternatives

- Append all unknown children after modelled children. That violates
  `xsd:sequence` and can make PowerPoint repair the file.
- Store a generic `OrderedChild<T>` union. Only one instantiation exists today,
  so a generic parameter would violate the structural rule.
- Use the count of currently present modelled children as the slot. Removing an
  optional known child could then move or orphan a raw sibling.
- Reparse unknown XML during write. It risks changing prefixes, attributes, or
  entity spelling instead of preserving the captured bytes.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `modelled_child_between_two_unmodelled_children_keeps_all_three_slots` | Raw child A, one modelled child, and raw child B are emitted in their original order. |
| round-trip | `multiple_raw_children_at_one_slot_preserve_document_order` | Two adjacent unknown siblings remain adjacent and ordered. |
| regression | `raw_subtrees_are_reemitted_byte_for_byte` | Prefixes, attributes, entities, comments, and nested markup are not normalised. |
| unit | `raw_children_after_the_last_modelled_child_are_not_dropped` | The final position is emitted even when no later known child exists. |

The **test gate** is: an element with a modelled child between two unmodelled
ones round-trips with all three in the original order.

## HLD impact

None. The helper implements the existing child-order and preservation contract.

## Risk routing

- **Any parser or serialiser**: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. The required reading supplies the OPC
  and PresentationML context. The extra checks prove schema order,
  prefix-tolerant recognition by owning parsers, fixed-prefix output, and
  byte-for-byte preservation through `capture_element` and
  `capture_empty_element`.
- **Crate dependency graph and a new module or file**: read
  `docs/hld/03-architecture.md` and the structural rules in `CLAUDE.md`. The
  extra checks are a dependency scan and the concrete non-generic API audit.
  F-053 explicitly authorises the `order.rs` module.

## Hash harness

Expected to be unchanged. The helper is isolated in the unpublished
`oxml-drawing` crate.

## Implementation checklist

- [x] Add the concrete schema-boundary and raw-byte representation.
- [x] Expose insertion-ordered `push`, `at`, and `is_empty` operations.
- [x] Exercise capture and emission through a test-local parent parser.
- [x] Add the three-child test gate and byte-preservation regressions.
- [x] Keep `oxml-core` and quick-xml as test dependencies only.
- [x] Prove no forbidden family dependency was introduced.

## Open questions

None. The concrete schema-boundary representation satisfies the story without a
speculative trait or generic parameter.

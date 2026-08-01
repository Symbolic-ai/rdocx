# F-068, presentation.xml

**Status**: approved
**Sprint**: S16
**Size**: M
**Depends on**: none

## Problem

No type owns `/ppt/presentation.xml`, even though it is the required main part
and the only source of slide order. The contract at
`docs/hld/06-presentationml-model.md:23` requires typed deck sizes, slide and
master identifiers, and default text style while preserving the rest of the
root verbatim.

The identifier constraints are file-validity rules rather than conveniences.
A slide id outside 256 through 2147483647 or a duplicate id can make
PowerPoint repair the deck.

## Spec reference

- `docs/hld/01-glossary.md`, "Units and coordinate spaces".
- `docs/hld/04-opc-and-packaging.md`, "The package".
- `docs/hld/06-presentationml-model.md`, "Parts", "presentation.xml", and
  "Preservation strategy".
- `docs/hld/12-testing-strategy.md`, "The deck corpus".
- `docs/hld/14-development-backlog.md`, "F-068, presentation.xml".

## Approach

Add a PresentationML module exporting `CT_Presentation`, `CT_SlideSize`,
`CT_SlideId`, and `CT_SlideMasterId`. `CT_Presentation::from_xml(&[u8])` reads
any root prefix, captures root attributes, validates required numeric values,
retains `p:sldIdLst` order exactly, and preserves all unsupported children at
their schema boundaries. `to_xml()` writes fixed `p:`, `a:`, and `r:` prefixes
in `CT_Presentation` schema order.

Deck and notes dimensions use `oxml_core::units::Emu` without changing its
truncating constructors. Relationship ids remain strings because their target
resolution belongs to OPC. `p:defaultTextStyle` uses the existing
`oxml_drawing::text::CT_TextListStyle` model. Root validation rejects duplicate
slide ids, out-of-range slide ids, missing required size fields, and malformed
integers without panicking.

The external corpus test established by F-067 parses, serialises, reparses, and
compares every presentation part structurally. Focused code-built fixtures cover
zero-slide templates, ordered multiple slides, alternate prefixes, unsupported
root children, and invalid ids.

## Rejected alternatives

- Keep the whole presentation part opaque. F-068 explicitly owns the fields
  needed for slide order and later editing.
- Resolve relationships while parsing XML. Relationship targets belong to the
  OPC package layer and must remain separate from the part model.
- Round or rescale EMU values. The repository pins direct integer storage and
  truncating constructors.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `presentation_reads_any_prefix_and_writes_fixed_prefixes_in_schema_order` | Typed children and raw boundaries survive with canonical prefixes |
| unit | `slide_ids_preserve_order_and_enforce_powerpoint_bounds` | Order, uniqueness, minimum, maximum, and malformed values are enforced |
| unit | `zero_slide_template_round_trips` | An empty slide-id list remains valid |
| round-trip | `every_corpus_presentation_part_round_trips_structurally` | Parse, serialise, reparse equality holds for every fetched deck |

The test gate is: every corpus deck's presentation part round-trips.

## HLD impact

None.

## Risk routing

- Unit conversion and `Emu`. Preserve direct EMU integers and the existing
  truncation contract. Declare the hash harness unchanged.
- Any parser or serialiser. Test fixed write prefixes, schema order, alternate
  read prefixes, and byte-for-byte preservation of unsupported children.
- Crate dependency graph and a new family `use`. Run
  `cargo tree -p rpptx-oxml` and keep all edges from `rpptx-oxml` toward
  `oxml-*` only.
- A new module or file. Obtain explicit approval for the PresentationML module
  before implementation.

## Hash harness

Expected to be unchanged. PresentationML parsing does not enter the released
Word facade, layout, or sample paths.

## Implementation checklist

- [ ] Add the PresentationML root and child value types.
- [ ] Parse typed children and retain unsupported attributes and child slots.
- [ ] Write fixed prefixes and schema child order.
- [ ] Validate slide-id uniqueness and bounds.
- [ ] Add focused fixtures and the all-corpus presentation gate.
- [ ] Run crate, dependency-tree, prose, and hash checks.

## Open questions

None. The user approved the PresentationML source module and the shared F-067
corpus and part-equality decisions.

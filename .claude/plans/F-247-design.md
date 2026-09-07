# F-247, Complete numbering level and instance model

**Status**: approved
**Sprint**: S71
**Size**: L
**Depends on**: F-243, F-249

## Problem

`ListLevel` exposes only format and start through its public constructor
(`crates/rdocx/src/document.rs:6808`). The underlying `CT_Lvl` already parses a
larger but incomplete projection, and `CT_Num` retains overrides primarily as
raw XML (`crates/rdocx-oxml/src/numbering.rs:2332` and
`crates/rdocx-oxml/src/numbering.rs:2943`). Callers cannot author all standard
formats, level text, style links, suffixes, alignment, marker properties,
restart controls, or typed instance overrides.

Capability row `DOCX-012` therefore remains partial
(`docs/hld/02-scope-and-non-goals.md:219`). Public-authored numbering must
round-trip in schema order without reporting its own properties as unmodeled.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, capability row `DOCX-012`.
- `docs/hld/03-architecture.md`, typed projection and facade conventions.
- `docs/hld/04-opc-and-packaging.md`, numbering fail-closed behavior and
  preservation.
- `docs/hld/08-rendering-spec.md`, "The renderer's input".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, test taxonomy and private authoring gate.
- `docs/hld/13-risks-and-open-questions.md`, R5 and R13.
- `docs/hld/14-development-backlog.md`, "F-247, Complete numbering level and
  instance model".

## Approach

Complete `ST_NumberFormat` and the public `ListNumberFormat` mapping for the
standard set, including `none`. Extend `ListLevel` with level text, paragraph
style link, suffix, alignment, indentation, marker run properties, legal
numbering, restart-after-level, and supported producer-extension values.

Type and preserve `w:pStyle` in this story, but defer its standalone public
mutation to the two-sided F-248 transaction. Add concrete public
`NumberingDefinition`, `NumberingInstance`, and
`NumberingLevelOverride` values plus document create, inspect, update, and
remove operations. Model `CT_NumLvl` overrides and start overrides directly in
the existing numbering file. Preserve unmodeled definition, level, instance,
and override children at their schema ranks.

Validate level ranges, placeholder references, definition and instance ids,
style references, and override ownership on a staged candidate before package
mutation. Serialize every standard child in XSD sequence and keep imported
extensions byte-identical.

## Rejected alternatives

- Continue accepting raw XML for advanced levels. That fails the public facade
  completion boundary.
- Collapse unknown number formats to decimal. The reader already preserves
  unknown values and must not change meaning.
- Treat each override as an independent mutation. Definition and instance
  consistency must publish atomically.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `all_public_numbering_level_properties_survive_reopen` | Every format, text, style link, suffix, alignment, indent, marker property, legal flag, and restart value returns through the facade. |
| round-trip | `numbering_instances_and_overrides_round_trip_in_schema_order` | Level and start overrides serialize at the correct sequence positions and reopen identically. |
| regression | `public_authored_numbering_reports_no_unmodeled_properties` | A source-built definition and instance are fully covered by typed diagnostics. |
| regression | `invalid_numbering_mutation_is_atomic` | Bad levels, ids, placeholders, links, and overrides return an error without changing bytes or allocation state. |
| round-trip | `imported_numbering_extensions_remain_byte_identical` | Unknown attributes and subtrees survive modeled mutation. |

The **test gate is round-trip**. Every typed level and override survives save
and reopen with schema-correct order and reports no unmodeled properties when
created solely through the public API.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/13-risks-and-open-questions.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serializer**. Test XSD child order, fixed write prefixes,
  prefix-tolerant reads, and byte-exact retention of every unmodeled subtree.
- **Public API of a published crate**. State additive pre-1.0 impact and run the
  verified workspace package dry-run with archive-size assertions.

## Hash harness

Expected unchanged for existing fixtures. Existing numbering serialization
and layout remain stable unless a caller uses the new typed properties.

## Implementation checklist

- [ ] Complete standard format enums and mappings.
- [ ] Extend typed level properties and their schema-ordered serializer.
- [ ] Model numbering instances, level overrides, and start overrides.
- [ ] Add public create, inspect, update, and remove operations.
- [ ] Validate the whole candidate and preserve producer extensions.
- [ ] Add exhaustive public round-trip and atomic-failure coverage.
- [ ] Run the full gate and routed package dry-run.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The user approved typed preservation with public style-link mutation
deferred to F-248, refusal of referenced instance removal, and the formal F-249
dependency.

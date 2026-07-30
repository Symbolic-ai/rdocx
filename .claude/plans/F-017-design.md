# F-017, App and custom properties

**Status**: completed
**Sprint**: S03
**Size**: M
**Depends on**: F-013

## Problem

`oxml-core` is specified to own extended application and custom document
properties, but neither model exists. The only metadata model today is
`CoreProperties` at `crates/rdocx-oxml/src/core_properties.rs:8`, so Word and
PowerPoint cannot share their `docProps/app.xml` or `docProps/custom.xml`
handling.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Presentation and slides".
- `docs/hld/03-architecture.md`, "Three families, one workspace" and
  "Crate-level conventions".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types".
- `docs/hld/12-testing-strategy.md`, "New tests the extracted crates need",
  subsection `oxml-core`.
- `docs/hld/14-development-backlog.md`, "F-017, App and custom properties".

## Approach

Add public `app_properties` and `custom_properties` modules to `oxml-core`.
`AppProperties` is one union struct whose common fields and Word-only and
PowerPoint-only scalar fields are public `Option` values. Cover application,
version, company, manager, template, total time, pages, words, characters,
characters with spaces, lines, paragraphs, presentation format, slides,
notes, hidden slides, multimedia clips, scale crop, links up to date, shared
document, and hyperlinks changed.

Parse by local name, retain the encountered child order, and capture unknown
children verbatim. Serialization replays parsed order and uses the schema's
canonical order for newly constructed values. Word-only fields remain `None`
for a PowerPoint fixture and PowerPoint-only fields remain `None` for a Word
fixture.

Model custom properties as `CustomProperties { properties }`,
`CustomProperty { fmtid, pid, name, value }`, and a concrete
`CustomPropertyValue` enum for text, signed integer, floating point, Boolean,
and file-time values. Preserve unsupported `vt:*` value subtrees as raw bytes
rather than discarding them. These existing schema variants justify the enum,
and no trait or generic parameter is introduced.

## Rejected alternatives

- Separate Word and PowerPoint application structs. Their overlapping schema
  would duplicate parsing and contradict the required union model.
- Parse every possible extended-property vector now. The story requires the
  scalar union and preservation lets unsupported fields survive safely.
- Store custom values as strings only. That loses their `vt:*` type and cannot
  round-trip faithfully.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `word_app_properties_round_trip_without_presentation_fields` | Word fields parse, PowerPoint fields stay `None`, and absent fields are not emitted |
| round-trip | `powerpoint_app_properties_round_trip_without_word_fields` | PowerPoint fields parse, Word fields stay `None`, and absent fields are not emitted |
| regression | `unknown_app_property_subtree_is_preserved_verbatim` | Unmodelled XML survives at its original sequence position |
| round-trip | `custom_property_value_types_round_trip` | Text, integer, float, Boolean, and file-time types retain metadata and value |
| regression | `unknown_custom_property_value_is_preserved_verbatim` | An unsupported `vt:*` subtree is not lost or reinterpreted |

The backlog test gate is the Word and PowerPoint `app.xml` fixtures parsing,
leaving the other format's fields `None`, and round-tripping without emitting
those absent fields.

## HLD impact

None. The scope, architecture, and testing documents already describe this
shared model.

## Risk routing

- Parser and serializer. Assert prefix-tolerant reads, fixed prefixes and
  schema child order on writes, plus byte-for-byte preservation of unknown
  subtrees.
- Public API of a published family. Treat the additions as semver-compatible,
  run `cargo package -p oxml-core`, and assert the archive is below 10 MiB.
- New modules and files. The sprint invocation supplies explicit authorization
  and the two current format fixtures exercise the shared implementation.

## Hash harness

Expected to remain unchanged. The new property models are not wired into
rdocx document rendering in this story.

## Implementation checklist

- [x] Add the application-properties union and ordered parse/write support.
- [x] Add the custom-property collection and concrete value enum.
- [x] Preserve unknown application children and custom value subtrees.
- [x] Add code-constructed Word, PowerPoint, and custom-property fixtures.
- [x] Run focused round-trip, package, and hash checks.

## Open questions

Resolved. Add the two modules and model text, signed integer, floating point,
Boolean, file time, and empty values. Preserve every unsupported `vt:*` value
as raw XML.

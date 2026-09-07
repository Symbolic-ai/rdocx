# F-244, Corpus settings and document properties

**Status**: approved
**Sprint**: S71
**Size**: L
**Depends on**: F-243, F-249

## Problem

The facade exposes only title, author, subject, keywords, automatic
hyphenation, and OfficeMath defaults (`crates/rdocx/src/document.rs:4318`). It
loads custom properties only for read access (`crates/rdocx/src/document.rs:1398`),
does not own application properties, and has no public mutation or removal
surface for document variables and compatibility settings. The settings parser
models only a narrow subset while retaining the source bytes
(`crates/rdocx-oxml/src/settings.rs:142`).

Capability rows `DOCX-005` through `DOCX-007` therefore remain partial or
unsupported (`docs/hld/02-scope-and-non-goals.md:212`). The corpus needs typed,
deterministic values whose removal deletes only the relationship-owned content.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, capability rows `DOCX-005` through
  `DOCX-007`.
- `docs/hld/03-architecture.md`, "Facade conventions" and atomic mutation.
- `docs/hld/04-opc-and-packaging.md`, relationship-owned properties and
  settings preservation.
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "The private from-scratch DOCX
  conformance corpus".
- `docs/hld/14-development-backlog.md`, "F-244, Corpus settings and document
  properties".

## Approach

Store relationship-resolved application properties and their part name beside
the existing core and custom models. Expose the existing concrete
`CoreProperties`, `AppProperties`, `CustomProperty`, and
`CustomPropertyValue` types through `rdocx`, with document getters, setters,
individual removal, and whole-part removal operations.

Extend `CT_Settings` in its existing file with typed `compatSetting`,
`defaultTabStop`, `characterSpacingControl`, document variables, and language
defaults required by the five anonymous corpus profiles. Leave the exhaustive
settings matrix to F-270. Expose
focused document methods such as `set_document_variable`,
`remove_document_variable`, `set_compatibility_setting`, and the corresponding
readers. Every mutator stages package, relationship, content-type, and typed
state together and removes only a facade-owned empty optional part.

Use fixed creation metadata rather than clocks. Keep parsed unmodeled children
and namespace bindings byte-identical around each rewritten modeled child.

## Rejected alternatives

- Expose raw settings XML. Public OXML leakage does not satisfy facade
  authoring.
- Replace parsed settings wholesale. That would discard producer extensions.
- Keep empty orphan parts after removal. The story requires owned package
  content to disappear.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `authored_settings_and_properties_survive_reopen` | Every typed corpus value returns through the public facade after save and reopen. |
| regression | `removing_one_property_family_prunes_only_its_owned_graph` | The selected value, empty owned part, relationship, and override are removed without changing unrelated content. |
| round-trip | `settings_mutation_preserves_unmodeled_children_in_schema_order` | Prefix aliases and unknown settings children survive byte for byte around rewritten typed children. |
| regression | `fresh_property_output_has_no_clock_or_host_input` | Equivalent authored metadata produces identical bytes. |

The **test gate is round-trip**. Every authored value survives save and reopen,
and removal deletes only its owned package content.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serializer**. Require fixed write prefixes, schema-positioned
  modeled children, prefix-tolerant reads, and byte-exact unmodeled subtree
  preservation.
- **Public API of a published crate**. Record the additive pre-1.0 surface and
  run the verified workspace package dry-run with archive-size assertions.

## Hash harness

Expected unchanged. Existing documents and unset optional properties retain
their current bytes and layout behavior.

## Implementation checklist

- [ ] Load and retain relationship-resolved application property state.
- [ ] Extend typed settings projections in the existing settings module.
- [ ] Add public read, set, and remove methods for each story-owned family.
- [ ] Make cross-part mutation fail closed and atomic.
- [ ] Prove deterministic creation, save-reopen, selective removal, and raw
  preservation.
- [ ] Run the full gate and routed package dry-run.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The user approved the bounded settings subset and the formal F-249
dependency.

# F-155, Document protection

**Status**: completed
**Sprint**: S48
**Size**: M
**Depends on**: none

## Problem

The OPC layer knows the settings relationship type, but the Word facade loads
only styles, numbering, notes, comments, and other behavior-bearing parts in
`Document::from_package` (`crates/rdocx/src/document.rs:421`). A settings part
therefore survives because the package retains opaque parts, but
`w:documentProtection` cannot be inspected through the typed model or public
facade.

This hides the author's read-only, comments-only, tracked-changes-forced, or
forms-only intent. It also leaves the password-verification metadata opaque,
so callers cannot report the recorded algorithm, spin count, hash, or salt even
though save already preserves the source bytes.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-155, Document protection".
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, package integrity, relationship-resolved
  parts, and preservation of unrelated producer XML.
- `docs/hld/10-bindings-spec.md`, native Word API stability and unchanged
  Python, WASM, and CLI surfaces.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and in-code fixtures.

## Approach

Add `rdocx-oxml::settings` with a concrete `CT_Settings` root and a typed
`DocumentProtection` projection. `ProtectionMode` recognizes `readOnly`,
`comments`, `trackedChanges`, and `forms`. The projection records enforcement,
formatting protection, provider type, algorithm class and type, algorithm SID,
spin count, hash, and salt as the values Word wrote. Unsupported enum values or
malformed numeric metadata leave the complete `w:documentProtection` element
opaque rather than reporting a misleading partial policy.

The settings parser is prefix-tolerant, captures unknown root attributes and
children in schema position, and retains the original protection element bytes
as the sole serialization source. The writer uses fixed `w:` prefixes for newly
constructed typed values and writes `w:documentProtection` in settings schema
order. F-155 exposes reading only, so opened producer bytes remain unchanged on
save and no password verification or enforcement behavior is invented.

Resolve the settings part through `rel_types::SETTINGS` in `Document::from_package`
and remember its actual target rather than assuming `/word/settings.xml`.
Expose `Document::document_protection() -> Option<&DocumentProtection>` as an
additive native Rust accessor. Re-export the concrete mode and metadata types
from `rdocx`. Python, WASM, and CLI retain their current surfaces and continue
to preserve the package normally.

## Rejected alternatives

- Search the settings bytes in the facade on every accessor call. That would
  duplicate namespace handling and leave schema order and preservation untested.
- Return only a Boolean protected flag. It erases the four author intents and
  the recorded verification metadata required by the story.
- Enforce editing restrictions in mutation methods. The setting records author
  intent rather than an access-control boundary, and the story explicitly
  prioritizes reading it.
- Put a second settings parser inside `document.rs`. Settings is a separate
  OOXML root part and should remain locally understandable in one low-level
  module.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `document_protection_modes_and_metadata_parse_through_aliases` | All four modes, both Boolean spellings, algorithm values, hash, and salt are reported through aliased Word prefixes |
| round-trip | `settings_keep_document_protection_and_unmodelled_children_byte_identical` | Each protection mode plus neighbouring producer XML writes unchanged after parse and facade save |
| regression, gate | `each_document_protection_mode_is_reported_with_its_recorded_hash` | The native public accessor reports read-only, comments, tracked changes, and forms with exact hash, salt, and spin metadata |
| regression | `malformed_document_protection_remains_opaque_and_unreported` | Invalid mode or numeric metadata survives byte-identically and returns no partial policy |
| integration | `settings_relationship_target_is_resolved_instead_of_assumed` | A nonstandard settings part target is loaded, inspected, and saved in place without an orphan part |

The **test gate**, from the backlog, is regression. Each protection mode
round-trips with its hash intact, and the mode is reported through the public
API.

Fixtures are assembled in code. Tests join the existing `rdocx` integration
and regression binaries rather than adding another test target.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`

Record settings-model ownership, relationship-resolved loading, opaque invalid
value preservation, the read-only protection accessor, and unchanged binding
surfaces.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias, fixed-prefix,
  schema-order, malformed-value, structural round-trip, and byte-preservation
  tests for settings and `w:documentProtection`.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  concrete mode and metadata types plus the borrowed native accessor are
  additive and story-required. Run affected package dry-runs and archive size
  assertions.
- A new module or file. Read the structural rules. Add
  `crates/rdocx-oxml/src/settings.rs` only after explicit approval. The separate
  settings root owns parsing and serialization, while `rdocx::Document` is the
  existing second consumer through the public projection.

## Hash harness

Expected unchanged across all 49 entries. Opened settings bytes remain the
serialization source, and existing samples do not change document protection.

## Implementation checklist

- [x] Add the approved settings root module with ordered raw preservation.
- [x] Parse the four supported protection modes and all recorded verification metadata.
- [x] Preserve malformed or unsupported protection elements as opaque XML.
- [x] Resolve and retain the existing settings relationship target in the facade.
- [x] Add the borrowed native protection accessor and re-export its concrete types.
- [x] Add parser, preservation, relationship-target, public API, and failure regressions.
- [x] Run focused checks plus parser, serializer, and published-package riders.
- [x] Update exactly HLD 03, HLD 04, and HLD 10 at completion.

## Open questions

None. The new `crates/rdocx-oxml/src/settings.rs` source module was explicitly
approved for the separate settings-part grammar and document-protection model.

# F-243, Word-compatible fresh package profiles

**Status**: approved
**Sprint**: S71
**Size**: L
**Depends on**: F-240

## Problem

`Document::new()` currently creates only a main document part and a styles
relationship (`crates/rdocx/src/document.rs:1842`). The document can serialize
as all four Word package classes, but that operation changes only the main-part
content type (`crates/rdocx/src/document.rs:2007`). Fresh output therefore does
not own the settings, theme, font table, properties, and metadata graph that a
Word-compatible blank package requires.

The public capability matrix consequently classifies blank profiles and fresh
DOCM, DOTX, and DOTM creation as partial (`docs/hld/02-scope-and-non-goals.md:210`).
Callers need a deterministic source-built package, while the existing small
package remains useful and must stay explicitly selectable.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Beyond v1" and capability rows
  `DOCX-003` and `DOCX-004`.
- `docs/hld/03-architecture.md`, "Facade conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package", "Generalising the
  constructors", and "Package integrity".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, "The private from-scratch DOCX
  conformance corpus".
- `docs/hld/14-development-backlog.md`, "F-243, Word-compatible fresh package
  profiles".

## Approach

Add a public `WordCreationProfile` enum with `Minimal(WordPackageClass)` and
`WordCompatible(WordPackageClass)` variants. Add
`Document::new_with_profile(profile) -> Document`. Keep package class identity
authoritative in the main-part content type rather than storing a second class
field. `Document::new()` delegates to the Word-compatible DOCX profile. The
current compact package remains available only through the explicit minimal
profile.

Build each Word-compatible profile from deterministic constants and typed
models. Create the main document, styles, settings, theme, font table, core and
application properties, and the exact required relationship and content-type
graph. Select the main-part content type from `WordPackageClass`. Do not invent
a VBA part for an empty macro-enabled package. Stage and validate the complete
candidate before returning it.

Extend the existing public conformance harness with normalized package-graph,
strict-validation, save-reopen, repeat-save, and optional pinned Word no-repair
checks. Compare semantic package trees, not producer ZIP bytes.

## Rejected alternatives

- Remove the compact profile. Existing callers still need an explicit small
  package option even though the compatible profile becomes the default.
- Copy a bundled blank template. The story requires from-scratch ownership.
- Add four constructors. One profile enum keeps package completeness separate
  from filename spelling.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `word_compatible_profiles_reopen_with_the_same_package_class` | DOCX, DOCM, DOTX, and DOTM profiles retain exact identity and required owned parts after save and reopen. |
| integration | `word_compatible_profiles_have_a_complete_normalized_package_graph` | Required content types and relationships are present once, point to existing parts, and pass strict validation. |
| regression | `minimal_profile_preserves_the_existing_document_new_contract` | `Document::new()` remains the explicit minimal DOCX package. |
| regression | `equivalent_fresh_profiles_serialize_identically` | Two equivalent constructions and repeated saves are byte-identical. |
| differential | pinned Word no-repair check | Every produced class opens without a repair prompt when the configured Word gate is available. |

The **test gate is round-trip**. Each profile saves, reopens with the same
identity, and passes strict package validation and the pinned no-repair check.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serializer**. Preserve schema child order, accept namespace
  aliases on read, emit fixed prefixes, and prove unrelated unmodeled XML is
  byte-identical after profile-aware reopen and save.
- **Public API of a published crate**. State the additive pre-1.0 surface and
  run the verified workspace package dry-run with archive-size assertions.
- **External oracle comparison**. Pin the Word gate identity, record whether it
  ran, and keep its output outside published crates and tracked fixtures.

## Hash harness

Expected unchanged across existing document XML and render entries. If the
approved default profile changes the harness package fingerprint, land the
exact expected package-only delta as the F-243 behavior commit.

## Implementation checklist

- [ ] Add the profile enum and profile-aware constructor in existing modules.
- [ ] Construct the required parts and relationship graph deterministically.
- [ ] Select all four exact Word main-part content types.
- [ ] Validate the staged graph before publishing a document.
- [ ] Add round-trip, strict package, determinism, and minimal-profile tests.
- [ ] Extend the public conformance fixture without adding a Rust test binary.
- [ ] Run the full gate and all routed checks.
- [ ] Update exactly the listed HLD files.

## Open questions

None. The user approved Word-compatible DOCX as the `Document::new()` default,
explicit compact profiles, omitted fresh timestamps, and macro-capable package
identity without a synthesized VBA project.

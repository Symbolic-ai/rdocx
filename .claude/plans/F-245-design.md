# F-245, Corpus themes, font tables, and embedded fonts

**Status**: completed
**Sprint**: S71
**Size**: L
**Depends on**: F-243, F-249

## Problem

The document loader discovers a related theme and embedded font bytes only
while building layout input (`crates/rdocx/src/document.rs:5821`). The Word
theme type is read-only (`crates/rdocx-oxml/src/theme.rs:11`), there is no font
table model, and callers cannot create theme, language, font relationships, or
embedded-font parts from the public facade.

Capability rows `DOCX-009` and `DOCX-010` remain partial
(`docs/hld/02-scope-and-non-goals.md:216`). The corpus needs deterministic
theme resolution and explicitly authorized font embedding with exact licensing
metadata, without observing system fonts.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, capability rows `DOCX-009` and
  `DOCX-010`.
- `docs/hld/03-architecture.md`, facade ownership and the shared DrawingML
  theme seam.
- `docs/hld/04-opc-and-packaging.md`, relationship and part ownership.
- `docs/hld/08-rendering-spec.md`, "The renderer's input".
- `docs/hld/10-bindings-spec.md`, "Native Word facade stability".
- `docs/hld/12-testing-strategy.md`, private authoring and Word fidelity gates.
- `docs/hld/14-development-backlog.md`, "F-245, Corpus themes, font tables,
  and embedded fonts".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and
  "Packaging".

## Approach

Re-export and accept the existing concrete shared DrawingML
`CT_OfficeStyleSheet` theme type. Add facade-owned `FontDefinition`,
`EmbeddedFont`, `EmbeddedFontKind`, and `FontEmbeddingLicense` concrete types.
Expose `Document::set_theme`, `theme`, `set_language_defaults`, `fonts`,
`set_font`, `remove_font`, `embed_font`, and `remove_embedded_font` operations.
Embedding requires caller-provided bytes, an explicit authorization value, the
OOXML font key, and the exact license identity. It never discovers host fonts.

Extend the existing shared DrawingML theme serializer and Word adapter. Add a
focused `rdocx-oxml` font-table module for prefix-tolerant parsing,
fixed-prefix schema-ordered writing, embedded-font relationship attributes,
and raw-child retention. Store resolved theme and font-table part names and
dirty state in `Document`. Stage every part, relationship, content type, and
layout input atomically.

Feed embedded bytes and authored theme aliases into deterministic layout ahead
of bundled fallback. Compare public-authored results against the pinned Word
and LibreOffice identities already owned by the conformance harness.

## Rejected alternatives

- Accept a system font path. It is host-dependent and does not prove caller
  authorization.
- Store a raw theme or font-table XML blob. That bypasses public typed
  authoring.
- Put font-table parsing in `document.rs`. A focused module keeps one schema
  owner in one file.
- Correct legacy `apply_tint_shade`. Its behavior is intentionally frozen.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `authored_theme_font_table_and_embedded_fonts_survive_reopen` | Typed theme, aliases, language defaults, font records, keys, relationships, and bytes return unchanged. |
| regression | `font_embedding_requires_explicit_authorization_and_license_identity` | Missing or inconsistent authorization fails before package mutation. |
| round-trip | `font_table_preserves_unknown_children_and_relationship_attributes` | Prefix aliases and unmodeled producer content survive byte for byte in schema order. |
| differential | `public_authored_theme_and_fonts_match_pinned_word_resolution` | Effective fonts and theme colors match the pinned structural oracle and deterministic render thresholds without system fonts. |
| regression | `embedded_font_parts_are_packaged_deterministically` | Equivalent operations select stable part names and repeated saves are byte-identical. |

The **test gate is differential**. Public-authored theme and font resolution
matches the pinned Word references in deterministic layout without system-font
input.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/05-drawingml-model.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- **Theme colour, tint, shade, and colour mapping**. Read the DrawingML color
  contract and keep the deliberately naive legacy tint and shade function
  unchanged.
- **Layout, pagination, line breaking, and text shaping**. Use deterministic
  font mode for every baseline and treat unexplained fallback as a failure.
- **Any parser or serializer**. Require schema order, fixed write prefixes,
  prefix-tolerant reads, and byte-exact unmodeled subtree preservation.
- **Bundled fonts**. Verify the authentic license identity and packaged asset
  inventory. Caller fonts remain document parts and do not alter the workspace
  bundled-font inventory.
- **Public API of a published crate**. State additive pre-1.0 impact and run the
  verified workspace package dry-run with archive-size assertions.
- **A new module or file**. Add exactly one focused font-table module after
  explicit approval.
- **External oracle comparison**. Pin Word and LibreOffice identities, compare
  parsed trees and stated render metrics, and keep private evidence ignored.

## Hash harness

Expected unchanged for existing fixtures. New authored theme and embedded-font
inputs are opt-in and deterministic.

## Implementation checklist

- [x] Add the approved focused font-table module and public facade types.
- [x] Add theme serialization without changing legacy tint and shade behavior.
- [x] Add licensed caller-authorized embedded-font ownership.
- [x] Resolve authored theme, aliases, and embedded bytes into deterministic
  layout.
- [x] Add round-trip, preservation, licensing, determinism, and differential
  coverage.
- [x] Run the full gate, package inventory, and F-245-specific public oracle
  riders.
- [x] Update exactly the listed HLD files.

## Open questions

None. The user approved the focused `font_table.rs` module, reuse of the shared
theme type, explicit caller authorization and license metadata, and the formal
F-249 dependency.

# F-083, The seven-step list style merge

**Status**: completed
**Sprint**: S20
**Size**: L
**Depends on**: F-081, F-064

## Problem

The nine list levels are typed at
`crates/oxml-drawing/src/text/list_style.rs:15`, but `a:defPPr` is preserved as
opaque XML at `crates/oxml-drawing/src/text/list_style.rs:126`. That element
occurs in the pinned corpus and contributes defaults to every level. Paragraph
and character properties expose the fields needed for merging at
`crates/oxml-drawing/src/text/paragraph.rs:581` and
`crates/oxml-drawing/src/text/paragraph.rs:820`, but no resolver combines the
seven documented sources.

## Spec reference

- `docs/hld/05-drawingml-model.md`, "Text".
- `docs/hld/06-presentationml-model.md`, "Text styles" and "Placeholders".
- `docs/hld/07-inheritance-and-resolution.md`, "The nine-level list style" and
  "The resolver".
- `docs/hld/14-development-backlog.md`, "F-083, The seven-step list style
  merge".

## Approach

Type `a:defPPr` as
`CT_TextListStyle.default_paragraph_properties`, parse it before level one,
and write it in the same schema position. Preserve all other raw children.

Add `text.rs` to `rpptx-layout` with concrete `EffectiveListStyle` and
`EffectiveTextProperties` values. Merge sources one through five across all
nine levels. Within each list-style source, apply `defPPr` to every level and
then its `lvlNpPr`. Select the master title style for title and centered-title
placeholders, body style for body, subtitle, and object placeholders, and
other style for every other placeholder class and plain text.

Merge paragraph properties per field. Merge nested default character
properties per field. Merge bullet colour, size, font, and choice
independently. Treat fills and each typeface slot as atomic properties. Do not
inherit raw XML or hyperlink actions.

Cache the prefix from presentation default, selected master style, master
placeholder, and layout placeholder by `Option<PlaceholderKey>`. Clone the
cached prefix before applying the shape's own list style so two shapes sharing
a placeholder key cannot contaminate each other. Select a level from direct
paragraph `pPr/@lvl`, defaulting to zero, then apply paragraph and run values as
sources six and seven. The same API accepts a field's typed paragraph and run
properties when a caller resolves a field.

## Rejected alternatives

- Cache the shape-owned style by placeholder key. Different shapes occupying
  the same placeholder can have different direct styles.
- Merge whole paragraph or run structs. Later partial values would erase
  unrelated inherited properties.
- Preserve `a:defPPr` as opaque XML. That omits a frequent, specified source
  from the seven-step result.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `seven_source_list_style_merge_resolves_run_size_and_typeface` | A run crossing all seven sources resolves the backlog size and typeface case |
| unit | `default_paragraph_properties_apply_before_each_level` | `defPPr` feeds every level and level-specific values win |
| unit | `all_nine_levels_merge_independently` | Each level keeps its own cascade |
| unit | `later_sources_win_per_property_without_erasing_other_fields` | Property-level overlay preserves unrelated values |
| unit | `bullet_components_merge_independently` | Bullet colour, size, font, and choice cascade separately |
| unit | `shape_owned_style_is_not_shared_by_placeholder_cache` | Cache reuse cannot leak direct shape formatting |
| round-trip | `default_paragraph_properties_round_trip_in_schema_order` | Typed `a:defPPr` retains order, fixed prefixes, and opaque siblings |
| corpus | `all_corpus_modelled_parts_reparse_structurally` | The existing required corpus gate remains green after the parser change |

The backlog test gate is named explicitly:
`seven_source_list_style_merge_resolves_run_size_and_typeface`.

## HLD impact

- `docs/hld/05-drawingml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`

## Risk routing

- Any parser or serialiser. Recheck `a:defPPr` schema order,
  prefix-tolerant reads, fixed-prefix writes, and exact preservation of
  unmodelled siblings. Run the required corpus structural round-trip gate.
- Layout and text shaping. No visual baseline changes in this story. Run the
  structural cascade tests in deterministic font mode if any render assertion
  is added, and require all 28 hashes unchanged.
- A new module or file. `src/text.rs` is justified by the current F-083
  implementation and requires the shared explicit approval recorded in F-081.

## Hash harness

Expected to be unchanged. The resolver does not alter the Word renderer.

## Implementation checklist

- [x] Type, parse, and write list-style `a:defPPr` in schema order.
- [x] Add concrete effective list and text property values in `text.rs`.
- [x] Implement the five-source, nine-level cached prefix.
- [x] Apply direct shape, paragraph, and run sources without cache leakage.
- [x] Merge nested character and bullet properties per field.
- [x] Add focused cascade, parser, and corpus regressions.
- [x] Update the two HLD files during sprint finalisation.

## Open questions

None. Field callers pass their already selected typed paragraph and run
properties through the same sources six and seven API.

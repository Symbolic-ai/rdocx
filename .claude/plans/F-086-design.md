# F-086, Draw order and the flattener

**Status**: approved
**Sprint**: S21
**Size**: L
**Depends on**: F-081

## Problem

`ResolveCtx` owns the complete slide, layout, and master hierarchy at
`crates/rpptx-layout/src/context.rs:18`, but it only resolves properties for an
ordinary shape. The three ordered shape trees remain separate, so a later
renderer would have to rediscover master visibility, placeholder suppression,
latent footer policy, and the four-pass draw order.

The PresentationML roots expose their shape trees and raw background XML at
`crates/rpptx-oxml/src/slide_parts.rs:39`, but root visibility attributes and
header-footer flags are still preserved only as opaque data. The flattener
cannot safely implement `showMasterSp` or latent date, footer, and slide-number
rules until those inputs are typed without weakening round-trip preservation.

## Spec reference

- `docs/hld/06-presentationml-model.md`, "Placeholders" and "Preservation
  strategy".
- `docs/hld/07-inheritance-and-resolution.md`, "Draw order", "The chains",
  and "The resolver".
- `docs/hld/14-development-backlog.md`, "F-086, Draw order and the flattener".

## Approach

Extend the existing slide-part roots in `slide_parts.rs` with presence-sensitive
`show_master_shapes` values on slides and layouts and a typed header-footer
policy for the layout and master. Parse alternate prefixes, preserve unknown
attributes and children in their existing positions, and write the modelled
values at their schema locations with fixed prefixes. Keep background payloads
opaque in this story and select the first present source in slide, layout,
master, then theme-fallback order.

Add the flattener to the existing `context.rs`. It returns a borrowed ordered
view whose source identifies background, master, layout, or slide and whose
shape-tree child remains available to F-087. Walk selected alternate-content
fallbacks and recursive groups in document order. Master and layout ordinary
placeholders are templates and are never emitted. Non-placeholder master
content obeys layout `showMasterSp`, non-placeholder layout content obeys slide
`showMasterSp`, and slide content is always emitted.

Treat date, footer, and slide-number placeholders as the documented latent
exception. Emit the deepest permitted occupied placeholder once, using the
existing index-first and type-fallback identity rules. Do not emit template
prompt text. Keep the implementation concrete in the existing files, with no
new trait, generic, module, or source file.

## Rejected alternatives

- Put draw-order policy in the renderer. That would duplicate inheritance
  decisions in every backend and contradict the resolver boundary.
- Clone layout placeholders onto the slide. Latent placeholders must remain
  layout or master content, and cloning them creates duplicate footers.
- Filter placeholders by text content. Prompt strings vary by locale, while
  typed placeholder identity is stable.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `flattener_omits_template_prompt_and_emits_master_logo_once` | The backlog gate has no master-title prompt and one master logo |
| unit | `flattener_emits_the_four_sources_in_draw_order` | Background, allowed master, allowed layout, and slide content remain ordered |
| unit | `show_master_shapes_suppresses_the_owned_pass` | Layout and slide visibility flags independently suppress the master and layout passes |
| unit | `slide_placeholder_suppresses_layout_and_master_matches` | A deeper occupied placeholder wins exactly once |
| unit | `latent_placeholders_obey_header_footer_flags` | Date, footer, and slide-number content appears only when permitted |
| round-trip | `visibility_and_header_footer_inputs_round_trip_in_schema_order` | Alternate prefixes parse, fixed prefixes write, and unmodelled data survives |
| corpus | `all_corpus_slide_layout_and_master_parts_reparse_after_visibility_typing` | All 50 decks retain structural round-trip coverage |

The backlog test gate is named explicitly:
`flattener_omits_template_prompt_and_emits_master_logo_once`.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/07-inheritance-and-resolution.md`

## Risk routing

- Any parser or serialiser. Recheck root attributes, `p:hf` schema positions,
  prefix-tolerant reads, fixed-prefix writes, and byte preservation of
  unmodelled attributes and children. Run focused round-trip tests and the
  required 50-deck structural corpus gate.
- Theme colour and colour mapping. Background selection must preserve the
  context's per-master colour map for F-087. Run the exact colour tests and
  require all 28 hashes unchanged.
- Public API in unpublished crates. State that `rpptx-oxml` and `rpptx-layout`
  remain version 0.0.0 with `publish = false`, and run workspace publication
  dry-run only as part of the full gate.

## Hash harness

Expected to be unchanged. The unpublished PowerPoint flattener does not change
Word rendering.

## Implementation checklist

- [ ] Type slide and layout master-shape visibility without losing raw data.
- [ ] Type layout and master header-footer policy in schema order.
- [ ] Select the effective background source.
- [ ] Emit allowed master, layout, and slide content in final draw order.
- [ ] Suppress template and occupied placeholders, including latent kinds.
- [ ] Add focused parser, flattener, and corpus regressions.

## Open questions

None. The existing HLD fixes the four-pass order, the two visibility controls,
and the latent-placeholder exception.

# F-112, Text frame mutation

**Status**: completed
**Sprint**: S27
**Size**: L
**Depends on**: F-109

## Problem

The facade can read ordinary shape text through `ShapeRef::text()` at
`crates/rpptx/src/lib.rs:1140`, but it cannot replace placeholder text or edit
its paragraphs and runs. `CT_TextBody` at
`crates/oxml-drawing/src/text/mod.rs:36` exposes only immutable paragraph
iteration, even though `CT_TextParagraph` and `CT_RegularTextRun` already own
typed paragraph, character, font, and bullet properties in
`crates/oxml-drawing/src/text/paragraph.rs`.

Mutation must preserve the required one-paragraph text body, placeholder
identity, ordered run choices, and unmodelled XML. It also needs borrowed
handles suitable for later Python property setters without leaking the entire
OOXML object graph.

## Spec reference

- `docs/hld/01-glossary.md`, "Geometry and text" and "The PowerPoint triangle".
- `docs/hld/05-drawingml-model.md`, "Text" and "Bullets".
- `docs/hld/06-presentationml-model.md`, "Public read facade", "The shape
  tree", and "Placeholders".
- `docs/hld/10-bindings-spec.md`, "Presentation API".
- `docs/hld/14-development-backlog.md`, "F-112, Text frame mutation".

## Approach

Build on F-109 `ShapeMut` with behavior-bearing borrowed handles and re-export
the existing typed formatting values used by their setters:

```rust
impl ShapeMut<'_> {
    pub fn set_text(&mut self, text: &str) -> Result<()>;
    pub fn text_frame(&mut self) -> Option<TextFrame<'_>>;
}

impl TextFrame<'_> {
    pub fn text(&self) -> String;
    pub fn set_text(&mut self, text: &str);
    pub fn paragraph_count(&self) -> usize;
    pub fn paragraph_mut(&mut self, index: usize) -> Option<TextParagraphMut<'_>>;
    pub fn add_paragraph(&mut self) -> TextParagraphMut<'_>;
}

impl TextParagraphMut<'_> {
    pub fn set_text(&mut self, text: &str);
    pub fn add_run(&mut self, text: &str) -> TextRunMut<'_>;
    pub fn set_properties(&mut self, properties: CT_TextParagraphProperties);
    pub fn set_bullet(&mut self, bullet: Option<TextBullet>);
}

impl TextRunMut<'_> {
    pub fn set_text(&mut self, text: &str);
    pub fn set_properties(&mut self, properties: CT_TextCharacterProperties);
    pub fn set_font(&mut self, font: Option<TextFont>);
}
```

`text_frame()` is available only for ordinary shapes with a text body.
`set_text()` creates a minimal text body when an ordinary shape lacks one,
preserves the existing body properties and list style when present, replaces
paragraph content with exactly one paragraph and one regular run, and never
changes placeholder metadata. Clearing text retains one empty paragraph to
satisfy DrawingML `minOccurs=1`.

Add only the narrow constructors and mutable accessors needed by these handles
to the existing `text/mod.rs` and `text/paragraph.rs` files. Paragraph and run
property replacement uses the already typed `CT_TextParagraphProperties`,
`CT_TextCharacterProperties`, `TextFont`, and `TextBullet` models, whose writers
own schema ordering. Existing field and line-break variants remain preserved
unless the caller explicitly replaces that paragraph's text.

## Rejected alternatives

- Expose `&mut CT_TextBody` directly. It leaks raw storage and lets callers
  violate the required paragraph invariant.
- Add builder-only formatting. Python property setters need non-consuming
  `set_*` methods, and the architecture already specifies setter twins.
- Flatten every paragraph into one string. That would discard run-level font
  properties, fields, breaks, bullets, and preserved XML.
- Add new facade or text modules. The existing files are sufficient and new
  modules require an explicit ask.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `setting_text_on_placeholder_round_trips_and_renders` | Placeholder text survives save plus reload and produces visible rendered output |
| regression | `clearing_text_preserves_required_paragraph` | Empty text retains one paragraph and `validate()` reports no `EmptyTextBody` issue |
| round-trip | `paragraph_run_font_and_bullet_properties_round_trip` | Paragraph properties, multiple runs, font properties, and every selected bullet value survive serialization |
| regression | `text_mutation_preserves_placeholder_identity` | Placeholder type and idx remain unchanged after text replacement |
| round-trip | `text_mutation_preserves_unmodelled_xml_and_schema_order` | Untouched raw attributes, children, fields, and breaks remain byte-identical and typed replacements stay in schema order |
| negative | `text_mutation_indices_and_shape_kinds_are_total` | Invalid paragraph indices return `None`, non-text shapes return no frame or a contextual error, and neither path panics |
| integration | `text_frame_handles_append_paragraphs_and_runs_in_order` | New paragraphs and runs preserve caller order through save, reload, and plain-text projection |

The backlog test gate is named explicitly: setting text on a placeholder
round-trips and renders.

## HLD impact

- `docs/hld/05-drawingml-model.md`
- `docs/hld/06-presentationml-model.md`

Document the mutable text-body invariant, paragraph and run formatting surface,
bullet ownership, placeholder preservation, and facade borrow handles.

## Risk routing

- Unit conversion for DrawingML centipoints and text spacing: read
  `docs/hld/01-glossary.md`, "Units", and `CLAUDE.md`, "Things that are
  deliberately wrong". Reuse typed integer values, retain truncating
  constructors, assert exact serialized units, and declare the hash harness
  unchanged.
- Any parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add prefix-tolerant, fixed-prefix,
  schema-order, required-paragraph, and raw-subtree preservation checks.
- Layout, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. Use deterministic font mode for the render
  acceptance check and do not record a system-font baseline.

`rpptx` and `oxml-drawing` remain unpublished. No new crate, module, file,
trait, generic, feature, or dependency edge beyond F-109 is planned.

## Hash harness

Expected to be unchanged. The new PresentationML text mutation API does not
alter the Word rendering fixtures or baselines.

## Implementation checklist

- [x] Add text-body construction and mutation accessors in existing DrawingML
  text files.
- [x] Add `ShapeMut::set_text` and behavior-bearing text-frame, paragraph, and
  run handles.
- [x] Preserve the required paragraph and placeholder identity on replacement.
- [x] Expose typed paragraph, character, font, and bullet setters.
- [x] Add round-trip, negative, preservation, and ordered-append tests.
- [x] Run the rendering gate in deterministic font mode.
- [x] Update exactly the two listed HLD files.

## Open questions

None. The approved scope uses typed formatting setters, one required paragraph,
and replacement semantics that preserve placeholder identity and body-level
state.

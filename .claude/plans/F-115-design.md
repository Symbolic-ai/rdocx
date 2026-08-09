# F-115, Slide and presentation properties

**Status**: approved
**Sprint**: S28
**Size**: S
**Depends on**: F-017

## Problem

The typed presentation root already parses slide size, slide roots preserve
producer content, and `oxml-core` already owns shared core properties. The
`rpptx` facade does not expose slide size, background, hidden state, core
properties, or slideshow output. Each mutation must round-trip without
discarding unmodelled XML or changing the caller's source package on failure.

## Spec reference

- `docs/hld/01-glossary.md`, `Emu` units.
- `docs/hld/04-opc-and-packaging.md`, content types, relationships, core
  properties, and package preservation.
- `docs/hld/06-presentationml-model.md`, presentation and slide roots.
- `docs/hld/10-bindings-spec.md`, presentation API.
- `docs/hld/14-development-backlog.md`, "F-115, Slide and presentation
  properties".

## Approach

Expose concrete facade methods and the existing shared property model:

```rust
pub use oxml_core::CoreProperties;

impl Presentation {
    pub fn slide_size(&self) -> Option<(Emu, Emu)>;
    pub fn set_slide_size(&mut self, width: Emu, height: Emu) -> Result<()>;
    pub fn core_properties(&self) -> Option<&CoreProperties>;
    pub fn core_properties_mut(&mut self) -> &mut CoreProperties;
    pub fn save_as_show(&self, path: impl AsRef<Path>) -> Result<()>;
}

impl SlideRef<'_> {
    pub fn hidden(&self) -> bool;
    pub fn has_explicit_background(&self) -> bool;
}

impl SlideMut<'_> {
    pub fn set_hidden(&mut self, hidden: bool);
    pub fn set_background(&mut self, fill: Fill) -> Result<()>;
    pub fn clear_background(&mut self);
}
```

Add narrow setters to the existing `CT_Presentation`, `CT_Slide`, and
background types. Slide-size mutation preserves the existing producer size
kind and all raw attributes and children while replacing only `cx` and `cy`.
It rejects non-positive dimensions before mutation.

Model root `p:sld/@show` as an optional boolean. The facade's hidden value is
its inverse, with missing `show` treated as visible. Write a fixed boolean
spelling and preserve unrelated root data.

Construct a direct slide background from the existing `Fill` model, serialize
it through the typed raw-preserving writer, and place it in schema order.
Clearing removes only a direct fill background. Existing `p:bgRef` theme
backgrounds remain untouched because authoring theme references is outside
this story.

Resolve the package core-properties relationship at open. Preserve the typed
model without dirtying its source part on immutable access. Mutable access
materializes a default model when absent and marks the part for serialization.
Save writes the relationship, content-type declaration, and part only when
needed, using the existing `CoreProperties` parser and writer.

`save_as_show` clones the staged package, changes only the main presentation
content type to the slideshow type, serializes with the normal deterministic
path, and leaves the in-memory presentation and ordinary `save` behavior
unchanged. It does not rename slide or presentation parts.

Hidden state in this story is package persistence and facade state. The current
render entrypoint does not assemble `RenderInput` from `Presentation`, so this
story does not invent a second facade-to-render pipeline.

No new module, file, trait, generic, feature, or dependency is introduced.

## Rejected alternatives

- Replace the entire slide-size node. That would discard size kind and
  producer extensions.
- Interpret hidden as a renderer-only flag. The source of truth is
  `p:sld/@show`.
- Author `p:bgRef`. Direct fill is the requested mutation surface and existing
  theme references must remain preserved.
- Store slideshow mode on `Presentation`. The operation is output-specific and
  should not change later ordinary saves.
- Create another core-properties model. `oxml-core::CoreProperties` is already
  the shared concrete implementation.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration, gate | `slide_and_presentation_properties_round_trip` | Slide size, direct background, hidden state, and core properties survive save and reopen |
| round-trip | `slide_size_mutation_preserves_kind_and_unmodelled_xml` | Only dimensions change and schema order remains valid |
| round-trip | `hidden_flag_uses_inverse_show_semantics` | Missing, true, and false values map correctly and emit fixed boolean spelling |
| round-trip | `background_set_and_clear_preserve_theme_references_and_raw_xml` | Direct fill changes without damaging an existing `bgRef` or extensions |
| package | `core_properties_are_loaded_lazily_and_written_with_valid_graph` | Immutable access does not rewrite bytes and mutable access creates or updates the correct part, relationship, and content type |
| package, gate | `save_as_show_changes_only_the_main_content_type` | `.ppsx` reopens and differs from ordinary save only at the main content type |
| negative | `invalid_slide_size_does_not_mutate_the_presentation` | Zero or negative dimensions return an error and preserve exact bytes |

## HLD impact

- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`

Document core-property graph ownership, output-only slideshow content type,
slide-size mutation, hidden inverse semantics, direct background behavior, and
preservation of theme background references.

## Risk routing

- Unit conversion and `Emu`: reuse the existing checked integer values and
  assert exact serialized dimensions.
- Parser or serialiser: assert prefix-tolerant input, fixed-prefix output,
  schema order, fixed booleans, and raw XML preservation.
- Dependency graph: reuse the existing lower-level `oxml-core` dependency and
  inspect `cargo tree -p rpptx --edges normal`.

The affected crates are unpublished. No external oracle, layout, new file,
module, crate, trait, generic, feature, or dependency rider applies.

## Hash harness

Expected unchanged. This story changes only unpublished PowerPoint package and
facade behavior. All 28 deterministic hashes must match.

## Implementation checklist

- [ ] Add slide-size facade access and raw-preserving mutation.
- [ ] Add hidden state through typed `p:sld/@show`.
- [ ] Add direct background set and clear while preserving `p:bgRef`.
- [ ] Resolve, expose, create, and save shared core properties.
- [ ] Add output-only `save_as_show` content-type conversion.
- [ ] Add property, negative, schema-order, and package-graph tests to existing
  binaries.
- [ ] Update exactly HLD 04 and HLD 06.
- [ ] Run focused checks, risk riders, `/verify --full`, and the hash harness.

## Open questions

None. The approved scope persists hidden state without adding a new render
assembly path, authors direct fill backgrounds only, and preserves theme
background references untouched.

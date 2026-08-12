# 06, PresentationML model

Owner: `rpptx-oxml`, with the facade in `rpptx`.

## Parts

| Part | Type | Required |
|---|---|---|
| `/ppt/presentation.xml` | `CT_Presentation` | yes |
| `/ppt/slides/slideN.xml` | `CT_Slide` | zero or more |
| `/ppt/slideLayouts/slideLayoutN.xml` | `CT_SlideLayout` | at least one per master |
| `/ppt/slideMasters/slideMasterN.xml` | `CT_SlideMaster` | at least one |
| `/ppt/theme/themeN.xml` | `CT_OfficeStyleSheet` | one per master |
| `/ppt/notesSlides/notesSlideN.xml` | `CT_NotesSlide` | optional |
| `/ppt/notesMasters/notesMaster1.xml` | `CT_NotesMaster` | required if any notes slide exists |
| `/ppt/tableStyles.xml` | `CT_TableStyleList` | conventional, not fatal if absent |
| `presProps.xml`, `viewProps.xml` | | conventional, not fatal if absent |

**Hard structural constraints.** Every slide has exactly one `slideLayout`
relationship. Every layout has exactly one `slideMaster` relationship. Every
master has a `theme` relationship and a `p:clrMap`. Every notes slide has
exactly one `notesMaster` relationship and exactly one `slide` relationship
back to its source slide. Every notes master has exactly one `theme`
relationship. A source slide has at most one `notesSlide` relationship. A deck
with zero slides is valid, and that is what a template is.

`CT_Slide` and `CT_SlideLayout` expose the presence-sensitive
`show_master_shapes` root attribute. An absent `showMasterSp` means true.
`CT_SlideLayout` and `CT_SlideMaster` expose their optional `p:hf` as
`CT_HeaderFooter`, with presence-sensitive slide-number, header, footer, and
date-time flags. Each absent flag means true. Boolean inputs accept both XML
boolean spellings and canonical output uses `1` or `0`.

The visibility attributes read through any PresentationML element prefix and
write at their fixed root locations. The modelled `p:hf` writes with the fixed
`p:` prefix in its schema position. Unmodelled root attributes, header-footer
attributes, and header-footer children retain their original payload and
relative positions.

`CT_Slide` also models the presence-sensitive root `show` attribute. Missing
`show` means visible, while the facade exposes its inverse as `hidden`. Boolean
input accepts both XML spellings and output uses fixed `1` or `0` values.
Unrelated root attributes and children remain preserved.

## Public facade

`rpptx::Presentation` opens a path or byte slice and owns the OPC package, the
typed presentation root, and the ordered slides resolved from `p:sldIdLst`.
Part names are never assumed. The main part, slides, and optional notes slides
are joined through their OPC relationships, including normalized relative
targets. Missing parts, missing or wrong relationship ids, external slide
targets, duplicate notes-slide links, and malformed typed roots return a
concrete facade error.

The presentation and slide property surface is concrete and borrowed:

```rust
Presentation::slide_size(&self) -> Option<(Emu, Emu)>;
Presentation::set_slide_size(&mut self, width: Emu, height: Emu) -> Result<()>;
Presentation::core_properties(&self) -> Option<&CoreProperties>;
Presentation::core_properties_mut(&mut self) -> &mut CoreProperties;
Presentation::save_as_show(&self, path: impl AsRef<Path>) -> Result<()>;
SlideRef::hidden(&self) -> bool;
SlideRef::has_explicit_background(&self) -> bool;
SlideMut::set_hidden(&mut self, hidden: bool);
SlideMut::set_background(&mut self, fill: Fill) -> Result<()>;
SlideMut::clear_background(&mut self);
```

Core properties use the package-level relationship described in
`04-opc-and-packaging.md`. Read access does not dirty the source part. Mutable
access materializes a default model when absent and writes its relationship,
content type, and part on save.

`slides()` and `slide(index)` expose borrowed `SlideRef` handles in producer
slide order. A slide exposes its producer id, optional `p:cSld` name, immediate
z-order shapes, recursive visible text, and optional speaker-note text. Indexed
access returns `Option` and does not panic.

The public facade also exposes total title and placeholder lookup and immutable
text-frame, paragraph and regular-run handles. Repeatable shape lookup covers
group children. Mutable shape, text and table handles have consuming nested
accessors so a path-based binding can retain one facade borrow across the
resolved operation without reaching into `rpptx-oxml`.

The owning facade edits the ordered slide collection through three atomic
methods:

```rust
pub fn remove_slide(&mut self, index: usize) -> Result<()>;
pub fn move_slide(&mut self, from_index: usize, to_index: usize) -> Result<()>;
pub fn duplicate_slide(&mut self, index: usize) -> Result<SlideRef<'_>>;
```

Every index is zero-based and must identify an existing slide. `to_index` is
the final index, and moving a slide to its current index changes nothing. A
duplicate is inserted immediately after its source. Collection changes keep
the facade records and `p:sldIdLst` synchronized.

Removal stages the complete change before replacing the live presentation. It
removes the selected slide id, presentation relationship, slide part,
relationship scope, and content-type override. An attached notes part and its
scope are removed with the slide. Matching `p:sld` entries are spliced out of
preserved `p:custShowLst` XML without changing its containers or unrelated
bytes.

Duplication also stages a complete graph. It allocates a new slide part,
producer slide id, presentation relationship, and destination relationship
scope. Internal targets are recomputed relative to the new part, external
target mode is retained, and numeric relationship ids are rewritten in typed
and preserved XML. Equal image bytes reuse the package-wide media part through
the normal `MediaStore`. Notes are copied to a new part whose slide
relationship points back to the duplicate. Custom-show membership is not
copied.

The table style part is typed for layout resolution. A caller may pass its
`CT_TableStyleList` to `ResolveCtx`, which uses either the table's explicit
style id or the part's default id. An absent part or unmatched id leaves direct
cell formatting and DrawingML defaults in force. Package parts outside the
typed model continue to use the normal preservation path.

`ShapeRef` normalizes the six typed shape-tree members to `ShapeKind`. Immediate
group members and the selected `mc:Fallback` view are exposed through child
iteration. Ordinary shapes return their text-body text. Table frames return
row-major cell text with tabs between cells and newlines between rows. Other
shape kinds have no direct text.

`slide_mut(index)` exposes a borrowed `SlideMut` handle. Its `shape(index)`
method retains read access, while `shape_mut(index)` returns a `ShapeMut` for an
immediate z-order child. `ShapeMut::child_mut(index)` recurses through group
children only. The selected `mc:Fallback` view remains read-only.

Position, size, rotation, and name setters support ordinary shapes, pictures,
graphic frames, groups, and connectors. Fill and line setters support ordinary
shapes, pictures, and connectors because those kinds own typed shape
properties. Adjustment mutation supports finite values on preset geometry.
Unsupported shape kinds and unsupported geometry return concrete facade
errors. Indexed access remains total and returns `Option`.

Ordinary shapes expose text mutation through behavior-bearing borrowed
handles:

```rust
ShapeMut::set_text(&mut self, text: &str) -> Result<()>;
ShapeMut::text_frame(&mut self) -> Option<TextFrame<'_>>;
TextFrame::paragraph_mut(&mut self, index: usize) -> Option<TextParagraphMut<'_>>;
TextFrame::add_paragraph(&mut self) -> TextParagraphMut<'_>;
TextParagraphMut::add_run(&mut self, text: &str) -> TextRunMut<'_>;
```

`TextFrame` also reads and replaces whole-frame text. Paragraph handles replace
text, paragraph properties, and bullets. Run handles replace text, character
properties, and the direct Latin font. The typed formatting values are
re-exported by `rpptx`. Structural append returns the newly inserted borrowed
item, and Rust's borrow rules prevent a live nested handle from being
invalidated by another structural mutation.

Whole-frame replacement creates a minimal body when needed and always retains
one paragraph. It preserves existing body properties, list style,
first-paragraph formatting, end properties, and placeholder metadata. Existing
fields and line breaks survive property-only edits. Explicit paragraph text
replacement removes its old run choices. Inserting absent paragraph properties
places them before preserved markup-compatibility run content, while later raw
boundaries and bytes remain unchanged. Unsupported shape kinds return the
normal contextual mutation error, and a shape without a text body returns no
text-frame handle.

Table graphic frames expose concrete borrowed `TableRef` and `TableMut`
handles through `ShapeRef::table` and `ShapeMut::table_mut`. Their cell access
is total and returns `Option`. Table handles expose row and column counts,
column widths, and the first-row, last-row, first-column, last-column,
horizontal-banding, and vertical-banding flags. Cell handles expose plain text,
typed text-frame mutation, direct fill, four optional margins, merge-origin and
continuation state, and span height and width.

Changing a column width uses a checked sum and synchronizes the graphic-frame
width. Merge accepts opposite rectangle corners in either order. It validates
the complete rectangle before changing state, rejects overlap with an existing
merge, migrates typed paragraphs in row-major order, and writes the DrawingML
origin and continuation pattern described in `05-drawingml-model.md`. Split is
valid only on a checked merge origin. It restores span one and clears
continuation flags without redistributing content. Fallible width, merge, and
split operations stage and serialize a table clone before committing it, so an
error leaves the table unchanged.

`SlideMut` also exposes the direct shape construction surface:

```rust
pub enum ConnectorType { Straight, Elbow, Curve }

pub fn add_textbox(
    &mut self, left: Emu, top: Emu, width: Emu, height: Emu,
) -> Result<ShapeMut<'_>>;
pub fn add_shape(
    &mut self, preset: &str,
    left: Emu, top: Emu, width: Emu, height: Emu,
) -> Result<ShapeMut<'_>>;
pub fn add_connector(
    &mut self, connector: ConnectorType,
    begin_x: Emu, begin_y: Emu, end_x: Emu, end_y: Emu,
) -> Result<ShapeMut<'_>>;
pub fn add_group_shape(&mut self) -> Result<ShapeMut<'_>>;
pub fn add_table(
    &mut self,
    rows: usize,
    columns: usize,
    left: Emu,
    top: Emu,
    width: Emu,
    height: Emu,
) -> Result<ShapeMut<'_>>;
```

The owning facade adds pictures because media parts and relationships belong to
the presentation package rather than to a borrowed slide handle:

```rust
pub fn add_picture(
    &mut self,
    slide_index: usize,
    image_data: &[u8],
    image_filename: &str,
    left: Emu,
    top: Emu,
    width: Option<Emu>,
    height: Option<Emu>,
) -> Result<ShapeRef<'_>>;
```

With neither extent supplied, `add_picture` probes the image and uses its
native size with a 72-DPI fallback. With exactly one extent supplied, it infers
the other from the pixel aspect ratio and truncates toward zero. With both
supplied, it does not require intrinsic metadata. Unsupported bytes, missing
intrinsic dimensions, out-of-range inference, and an invalid slide index
return contextual errors without changing the presentation.

Picture insertion reuses equal media bytes package-wide and creates or reuses
an internal image relationship in the target slide's own scope. Package,
media-store, and relationship changes remain staged until picture construction
succeeds. The picture receives a tree-wide allocated id and deterministic name,
then its canonical `p:nvPicPr`, relationship-backed `p:blipFill`, and typed
`p:spPr` shell append at top z-order.

An ordinary shape has canonical non-visual properties, a typed transform,
preset geometry, and a minimal text body. `add_shape` keeps the string API but
accepts only names in the generated table of all 187 ECMA preset shapes. An
unknown name returns a contextual error without changing the slide. A textbox
uses `rect`, sets `txBox="1"`, has `a:noFill`, and contains `a:bodyPr`,
`a:lstStyle`, and one required paragraph. An empty group contains the required
`p:nvGrpSpPr` and `p:grpSpPr` shells, with no invented transform or members.

A constructed connector is free-standing and uses `line`, `bentConnector3`,
or `curvedConnector3` for `Straight`, `Elbow`, or `Curve`. Its transform offset
is the componentwise minimum endpoint, its extents are the absolute endpoint
spans, and its horizontal and vertical flips retain endpoint direction. A
horizontal or vertical connector may have one zero extent. A span that cannot
fit in the signed EMU representation returns a contextual error.

A constructed table uses a canonical `p:graphicFrame` with deterministic name
`Table {id}`, a typed transform, the DrawingML table URI, and a rectangular
`a:tbl` payload. The constructor rejects invalid counts and extents before tree
mutation. It appends at top z-order like the other borrowed slide constructors.
The table model owns truncating dimension distribution and assigns each
remainder to the final row or column so the grid matches the frame extent.

Every shape constructor, including the owning picture operation, rescans the
tree immediately before allocation, derives a deterministic producer name from
the allocated id, and appends at top z-order. The append operation shifts
raw-child boundaries at the old trailing position before adding the typed
member. Preserved schema-final content such as `p:extLst` therefore remains
after the new member, while all raw subtrees retain their bytes and relative
positions.

The ignored integration gate
`all_shape_constructors_open_in_powerpoint_without_repair` builds all four
forms in a tree with preserved schema-final extension content. The generated
deck opens without repair in pinned Microsoft PowerPoint 16.104, bundle
16.104.25121423.

The picture native-size comparison uses pinned python-pptx 1.0.2. The ignored
integration gate `added_picture_validates_and_opens_without_repair` confirms
that a generated picture deck validates and opens without repair in the same
pinned PowerPoint bundle.

The text mutation gate
`setting_text_on_placeholder_round_trips_and_renders` clears and then replaces
the same placeholder, saves and reopens the deck, resolves it through the
normal layout path, and compares the blank and changed PNG outputs. It uses
`layout_presentation_deterministic`, so the observed pixel change comes from
bundled or presentation-embedded fonts and never from discovered system fonts.
Placeholder type and `idx` remain unchanged through the mutation.

`to_bytes()` clones the source package, serialises the owned presentation,
slide, and notes roots back to their relationship-resolved part names, and uses
the deterministic OPC writer. Typed edits retain unmodelled attributes and
children in their raw slots and preserve schema child order. Parts outside
those owned roots remain the exact source bytes.

## `presentation.xml`

Carries `p:sldSz` (deck dimensions in EMU), `p:notesSz`, `p:sldIdLst`,
`p:sldMasterIdLst`, and `p:defaultTextStyle` which is the base of the text
inheritance chain.

Slide-size mutation rejects non-positive dimensions before changing the model.
It replaces only `p:sldSz/@cx` and `@cy`, preserving the producer size kind,
unmodelled attributes, unmodelled children, and schema position. An absent
`p:sldSz` is materialized with only the validated dimensions.

**`p:sldIdLst` order is slide order.** There is no separate ordering mechanism.
`CT_Presentation` records the original relationship-id order and reconciles raw
list boundaries against surviving relationship ids during serialization.
Comments and unmodelled children therefore remain anchored when slides move,
are removed, or are inserted.

**`p:sldId/@id` must be at least 256 and at most 2147483647, and unique.** A
value below 256 is a guaranteed repair prompt, and it is the single most common
defect in naive `add_slide` implementations.

The `.pptx` versus `.ppsx` distinction lives entirely in this part's content
type: `presentationml.presentation.main+xml` against
`presentationml.slideshow.main+xml`. `Presentation::save_as_show()` changes
only the staged output package's main content type. It does not store slideshow
mode on the facade or affect later ordinary saves.

## Notes parts

A notes slide has this root sequence:

```
p:notes
  p:cSld       required
  p:clrMapOvr? optional
  p:extLst?    optional
```

A notes master has this root sequence:

```
p:notesMaster
  p:cSld       required
  p:clrMap     required
  p:hf?        optional, preserved raw
  p:notesStyle? optional, typed as CT_TextListStyle
  p:extLst?    optional
```

Both roots reuse `CT_CommonSlideData`. They read namespace aliases, write fixed
`p:`, `a:`, and `r:` prefixes, and retain unsupported attributes and children
in their schema slots.

`CT_NotesSlide::notes_text()` walks its shape tree in z-order, including
recursive groups and the selected rendering view of `mc:AlternateContent`.
It includes text only from a `p:sp` whose placeholder effective type is exactly
`body`. An absent `p:ph/@type` therefore qualifies. The placeholder matching
equivalence between `body`, `subTitle`, and `obj` does not broaden extraction.
Slide-image, slide-number, date, footer, header, and notes-master prompt text is
excluded.

Each included `p:txBody` contributes run and field text in document order.
`a:br` contributes a newline and paragraph boundaries contribute a newline.
Empty bodies are skipped, and multiple nonempty body placeholders are joined
with a newline.

## The shape tree

```
p:cSld
  p:bg?          slide background
  p:spTree
    p:nvGrpSpPr  required, even though it is nearly empty
    p:grpSpPr    required
    (p:sp | p:pic | p:graphicFrame | p:grpSp | p:cxnSp | mc:AlternateContent)*
```

Document order is z-order. `p:spTree` missing its own `p:nvGrpSpPr` or
`p:grpSpPr` is a repair prompt, as is any `p:sp` without `p:spPr`.

`CT_Background` retains the complete captured `p:bg` subtree as its sole
serialisation source. Its read-only rendering projection distinguishes a
`p:bgPr` DrawingML fill from a `p:bgRef` style index and colour. Projection
parsing uses the namespace bindings inherited from `p:cSld`, so aliased
prefixes remain valid even when their declarations live on the part root.
Attributes, effects, unsupported siblings, and the original child order remain
inside the retained raw subtree and round-trip byte-identically.

Slide background authoring accepts a direct DrawingML `Fill` and writes it as
canonical `p:bg/p:bgPr` before `p:spTree`. Replacing an existing direct fill
preserves the captured `p:bg` and `p:bgPr` attributes and all raw siblings while
changing only the fill subtree. Replacing a `p:bgRef` or unsupported background
choice is rejected. Clearing removes only a direct `p:bgPr` background, so theme
references and opaque producer payloads remain untouched.

An ordinary `CT_Shape` owns its required `p:spPr` as a public boxed
`CT_ShapeProperties`. The allocation keeps recursive group parsing within the
normal test-thread stack even when a master contains deeply nested shapes with
large text bodies. Field access remains direct through `Box` dereferencing.
Modelled transform, geometry, fill, line, and effect children write with fixed
prefixes in schema order. Unsupported attributes and children remain in the
shape-properties ordered raw slots.

The optional `p:style` is typed as `CT_ShapeStyle` behind `CT_Shape::style()`.
Its required line, fill, effect, and font references parse with any bound
prefix and write with fixed `p:` and `a:` prefixes after `p:spPr` and before
`p:txBody`. Root attributes and unsupported children remain in their ordered
raw slots. The style payload shares the shape's existing allocation so typing
it does not expand the recursive `ShapeTreeChild` value.

```rust
pub enum ShapeTreeChild {
    Shape(CT_Shape),
    Picture(CT_Picture),
    GraphicFrame(CT_GraphicFrame),   // tables, charts, SmartArt, OLE
    GroupShape(CT_GroupShape),       // recursive
    Connector(CT_ConnectionShape),
    AlternateContent(Box<CT_AlternateContent>),
}

pub struct CT_AlternateContent {
    raw_xml: Vec<u8>,
    chart_choice: Option<Box<CT_GraphicFrame>>,
    selected_fallback: Option<Vec<ShapeTreeChild>>,
}
```

`CT_AlternateContent` selects ordered typed members only from its one immediate
`mc:Fallback` child. It resolves the branch by namespace URI and uses the same
shape-tree dispatch as `p:spTree` and `p:grpSp`. An immediate `mc:Choice` stays
opaque except for a read-only projection of its first chart-bearing
`p:graphicFrame`. Only the schema-positioned
`p:graphicFrame/a:graphic/a:graphicData` payload can select that projection.
Descendant extension payloads stay opaque. The projection resolves the
`c:chart@r:id` attribute by namespace URI and pairs it with the first immediate
typed `p:pic` fallback.
Other choices are not evaluated. No fallback is valid and produces no
selection. An empty fallback produces an empty selection, while more than one
immediate MC fallback is invalid.

The captured `raw_xml` subtree is the only serialisation source. The selected
fallback is a read-only rendering view and is never written back in place of
the producer XML, so choices, comments, processing instructions, attributes,
entities, whitespace, and fallback content remain byte-identical.

The graphic-data payload is exposed read-only. Typed table payloads retain a
dedicated mutable accessor, while chart and opaque payload bytes cannot be
mutated independently of their relationship projection.

`CT_ConnectionShape` types the connector's required `p:spPr` and its optional
start and end connections. Each present `a:stCxn` or `a:endCxn` carries the
required unqualified shape `id` and connection-site `idx` as `u32` values.
Free-standing, start-only, end-only, and fully connected shapes therefore use
the same model. Unsupported connector locks, style, extensions, attributes,
and children remain in their ordered schema slots and round-trip without being
interpreted.

**`p:cNvPr/@id` must be unique within one `spTree`**, including inside nested
groups, preserved raw members, and every branch of `mc:AlternateContent`. A
`ShapeIdAllocator` retains typed recursive scanning and also performs a
namespace-resolved scan of the complete preserved tree. PresentationML aliases
are accepted, foreign `cNvPr` elements are ignored, and non-selected
compatibility choices still reserve their ids. Allocation starts at 2 because
the tree's own `p:nvGrpSpPr` takes 1, fills unused gaps, and reserves each
result.

## Placeholders

```rust
pub struct PlaceholderKey { pub ph_type: PhType, pub idx: Option<u32> }
```

Matching follows PowerPoint's actual rule, not the obvious one:

- Match on `@idx` when it is present on both sides.
- Otherwise match on `@type`, where **an absent `@type` means `body`**.
- `title` and `ctrTitle` form one equivalence class.
- `body`, `subTitle` and `obj` form a second.

**`idx` is the join key from a slide placeholder to its layout placeholder.**
Renumbering it severs position and formatting inheritance silently. Fresh `idx`
allocation is needed in exactly two other operations: adding a new placeholder to
a *layout*, and copying a slide between presentations whose layouts assign
different indices. Both are separate code paths.

`dt`, `ftr` and `sldNum` are **latent** placeholders. They are drawn from the
layout or master directly and are never cloned onto a new slide, because cloning
them produces duplicate footers.

## Preservation strategy

This is the scope control for a format that is otherwise unbounded.
**Parse only what is rendered or edited. Preserve everything else verbatim**
through `oxml_core::raw_xml::capture_element`.

Preserved as opaque bytes in v1: `p:timing`, `p:transition`, `p:custShowLst`,
`p14:sectionLst`, comments, ink, `p:contentPart`, SmartArt `dgm:` payloads, OLE
and ActiveX, and every `mc:AlternateContent` alternative.

An `mc:AlternateContent` model may inspect its selected fallback and an
immediate chart choice. The full captured subtree remains opaque for
serialisation and round-trips byte-identically. Malformed non-chart choices are
not parsed merely because they contain a graphic-frame element.

The gate on this is a full-corpus round-trip. Every modelled root parses,
serialises, reparses, and compares structurally. The expected package replaces
each modelled root with those exact canonical serialised bytes. After save and
reopen, every rewritten root must match its expected bytes, while every
unmodelled part must match its original bytes. Content types, relationships,
part names, and part counts must remain structurally unchanged.

## Relationship remapping

`rpptx-oxml/src/relmap.rs`, roughly 140 lines:

```rust
/// Re-serialise `raw`, rewriting every attribute in the `r:` namespace whose
/// value matches /^rId\d+$/ through `map`. Preserves everything else
/// byte-for-byte, including comments and processing instructions.
pub fn rewrite_rel_ids(raw: &[u8], map: &HashMap<String, String>) -> Result<Vec<u8>>;
```

Modelled fields such as `CT_Blip::embed` are easy. But `r:embed`, `r:id`,
`r:link`, `r:dm`, `r:lo`, `r:qs` and `r:cs` also occur *inside preserved raw
blobs*. Without this function, a duplicated slide's SmartArt or embedded video
silently points at the source slide's relationships. The preservation strategy
that makes round-tripping possible is exactly what makes deep copy dangerous,
and this is the mitigation.

## Adding a slide

**Synthesise, do not deep-copy.** This is what python-pptx does and it is
dramatically safer. A synthesised placeholder contains no `r:embed`, so **no
relationship remapping is needed at all** on the common path. It also carries no
stale `a:xfrm` overriding the layout, and no copied creation-id GUIDs.

The sequence:

1. Resolve the layout.
2. Allocate `/ppt/slides/slideN.xml` as `1 + max(existing suffix)`.
3. Build the `CT_Slide` shell. No copied name, no background,
   `p:clrMapOvr` containing `a:masterClrMapping`, and a `p:spTree` with its
   required `p:nvGrpSpPr` and `p:grpSpPr`.
4. Select cloneable layout placeholders: those with a `p:ph`, excluding the
   latent `dt`, `ftr` and `sldNum` types. Non-placeholder layout shapes are not
   cloned, they render through the layout pass.
5. Synthesise one minimal `p:sp` per placeholder. Copy `type` and `idx`
   **verbatim**. Empty `p:spPr` so position inherits. A `p:txBody` containing
   `a:bodyPr`, `a:lstStyle` and **at least one `a:p`**, because zero paragraphs
   violates `minOccurs=1`.
6. Create the slide's relationships: exactly one, to the layout, with a
   **relative** target computed rather than hardcoded, because a template may
   store layouts elsewhere.
7. Register the content-type override.
8. Add a `slide` relationship on `presentation.xml` and append a
   `p:sldId` with `id = max(existing).max(255) + 1`.
9. Return the slide handle.

Deep copy exists only for `duplicate_slide` and cross-deck copy, where it uses
`rewrite_rel_ids`, transfers media with content-hash deduplication, and assigns
fresh `p:cNvPr` ids across ordinary, grouped, and compatibility content. The
same map rewrites `a:stCxn` and `a:endCxn` endpoints so copied connectors target
the copied shapes. Slides and notes both use this narrow shape-tree behavior.

## Validation

`Presentation::validate()` returns issues rather than panicking, and runs
automatically under `debug_assertions` before `save`. It will save more
debugging time than any other three hundred lines in the project.

```rust
pub enum ValidationIssue {
    DuplicateShapeId { slide: usize, id: u32 },
    SlideIdOutOfRange { slide: usize, id: u32 },
    DuplicateSlideId { id: u32 },
    MissingContentTypeOverride { part: String },
    DanglingRelationship { part: String, r_id: String, target: String },
    UnreachableRelationshipTarget { part: String, target: String },
    EmptyTextBody { slide: usize, shape: usize },
    DuplicatePlaceholderIdx { slide: usize, idx: u32 },
    OrphanMedia { part: String },
    CustomShowReference { slide_id: u32 },
    MissingLayoutRel { slide: usize },
    MissingThemeRel { master: usize },
}
```

Mapped to the symptoms they prevent:

| Symptom | Cause |
|---|---|
| "PowerPoint found a problem and needs to repair" | `p:sldId/@id` below 256, or duplicated |
| "PowerPoint can't read this file" | missing content-type override for a new part |
| Slide opens blank, layout lost | relationship target with a leading slash, or the wrong relative depth |
| Repair | duplicate `p:cNvPr/@id` anywhere in one `spTree` |
| Repair | `p:txBody` with zero `a:p` |
| Repair | children out of schema sequence |
| Placeholder loses inheritance | `idx` was altered |

## The bundled template

`Presentation::new()` loads `crates/rpptx/assets/default.pptx` through
`include_bytes!`, behind a `default-template` feature. This differs from rdocx,
which builds an empty document from constructors, and the reason is arithmetic:
the Office theme's `a:fmtScheme` alone is roughly 250 lines of gradient and
effect XML, and each of the eleven standard layouts is another 120. Generating
that from Rust is about 2,500 lines of write-only code that nobody would ever
verify. python-pptx ships a binary template for the same reason.

Contents: 16:9 slide size, one master with the standard colour map and full
`p:txStyles`, the eleven standard layouts, a full theme, `presProps`,
`viewProps`, `tableStyles` defaulted to Medium Style 2 Accent 1, a notes master,
and **zero slides**.

The asset must live under the crate's own directory. A workspace-root `assets/`
compiles locally but is not included in the published `.crate`.

The PresentationML model crates remain version `0.0.0` with `publish = false`.

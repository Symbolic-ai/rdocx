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
| `presProps.xml`, `viewProps.xml`, `tableStyles.xml` | | conventional, not fatal if absent |

**Hard structural constraints.** Every slide has exactly one `slideLayout`
relationship. Every layout has exactly one `slideMaster` relationship. Every
master has a `theme` relationship and a `p:clrMap`. A deck with zero slides is
valid, and that is what a template is.

## `presentation.xml`

Carries `p:sldSz` (deck dimensions in EMU), `p:notesSz`, `p:sldIdLst`,
`p:sldMasterIdLst`, and `p:defaultTextStyle` which is the base of the text
inheritance chain.

**`p:sldIdLst` order is slide order.** There is no separate ordering mechanism.

**`p:sldId/@id` must be at least 256 and at most 2147483647, and unique.** A
value below 256 is a guaranteed repair prompt, and it is the single most common
defect in naive `add_slide` implementations.

The `.pptx` versus `.ppsx` distinction lives entirely in this part's content
type: `presentationml.presentation.main+xml` against
`presentationml.slideshow.main+xml`. `Presentation::save_as_show()` exposes it.

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
    selected_fallback: Option<Vec<ShapeTreeChild>>,
}
```

`CT_AlternateContent` selects ordered typed members only from its one immediate
`mc:Fallback` child. It resolves the branch by namespace URI and uses the same
shape-tree dispatch as `p:spTree` and `p:grpSp`. Every `mc:Choice` stays opaque
and is not evaluated. No fallback is valid and produces no selection. An empty
fallback produces an empty selection, while more than one immediate MC
fallback is invalid.

The captured `raw_xml` subtree is the only serialisation source. The selected
fallback is a read-only rendering view and is never written back in place of
the producer XML, so choices, comments, processing instructions, attributes,
entities, whitespace, and fallback content remain byte-identical.

`CT_ConnectionShape` types the connector's required `p:spPr` and its optional
start and end connections. Each present `a:stCxn` or `a:endCxn` carries the
required unqualified shape `id` and connection-site `idx` as `u32` values.
Free-standing, start-only, end-only, and fully connected shapes therefore use
the same model. Unsupported connector locks, style, extensions, attributes,
and children remain in their ordered schema slots and round-trip without being
interpreted.

**`p:cNvPr/@id` must be unique within one `spTree`**, including inside nested
groups and inside `mc:AlternateContent` fallbacks. A `ShapeIdAllocator` scans
the whole tree and hands out ids from 2, because the tree's own `p:nvGrpSpPr`
takes 1.

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

An `mc:AlternateContent` model may inspect its selected fallback, but the full
captured subtree remains opaque for serialisation and round-trips
byte-identically.

The gate on this is a full-corpus round-trip: parse, serialise, reparse, compare
structurally, and separately compare the saved package part-by-part against the
original bytes.

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
fresh `p:cNvPr` ids.

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

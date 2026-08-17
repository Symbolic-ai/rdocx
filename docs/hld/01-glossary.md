# 01, Glossary

Terms used precisely throughout this spec set. Where a word has a loose everyday
meaning and a strict OOXML meaning, the strict one is intended.

## The formats

**OOXML**, Office Open XML. The ECMA-376 family. A `.docx`, `.pptx` or `.xlsx`
is a ZIP archive of XML parts plus binary media.

**OPC**, Open Packaging Conventions. The container half of OOXML: the ZIP
layout, `[Content_Types].xml`, and the `_rels/` relationship files. Format
neutral. Everything in `oxml-opc` implements this and nothing above it.

**WordprocessingML**, the `w:` namespace. Word's document vocabulary. Lives in
`rdocx-oxml`.

**PresentationML**, the `p:` namespace. PowerPoint's vocabulary: presentations,
slides, layouts, masters, the shape tree. Lives in `rpptx-oxml`.

**SpreadsheetML**, the `x:` namespace. Excel's vocabulary. This repository
implements a deliberately minimal *writer* only, in `oxml-sml`, because every
chart embeds a workbook.

**DrawingML**, the `a:` namespace. The shared graphics vocabulary: transforms,
geometry, fills, lines, effects, colour schemes, and its own text model. Used by
all three formats. Lives in `oxml-drawing`. PowerPoint is roughly 90 percent
DrawingML, which is why that crate is the largest new body of work.

**ChartML**, the `c:` namespace. Charts. Lives in `oxml-chart`.

## The container

**Part**. One entry in the package, addressed by an absolute name such as
`/ppt/slides/slide1.xml`. In `OpcPackage` these are a `HashMap<String, Vec<u8>>`.

**Relationship**. A typed, directed edge from one part to another, or to an
external URI. Written in a sibling `_rels/<name>.rels` file and referenced from
the XML by an `r:id` such as `rId3`.

**Relationship scope**. The critical subtlety for PowerPoint. Each part has its
own relationship namespace, so `rId2` on a slide, `rId2` on its layout and
`rId2` on its master are three different targets. An `r:id` is therefore not a
valid global key for anything, which is why the renderer keys media by content
hash instead.

**Content type**. The MIME type of a part, declared either by a `Default` on the
file extension or by an `Override` on the exact part name. A missing override is
one of the most common causes of a file PowerPoint refuses to open.

## Units

OOXML uses a different unit almost everywhere. All of them live in `oxml-core`.

| Unit | Size | Used by |
|---|---|---|
| **EMU**, English Metric Unit | 1/914400 inch | DrawingML positions and sizes. The canonical internal unit of `Length` |
| **Twip** | 1/1440 inch | WordprocessingML measurements |
| **Half-point** | 1/2 point | `w:sz`, Word font size |
| **Centipoint** | 1/100 point | `a:rPr/@sz`, DrawingML font size. Not the same as a half-point |
| **60000ths of a degree** | | `a:xfrm/@rot`, `a:lin/@ang`. A right angle is 5400000 |
| **Thousandths of a percent** | | `a:lumMod val="75000"` means 75 percent |

EMU is chosen as the canonical unit because it divides evenly into both inches
and centimetres, and because it is what DrawingML already uses.

## The PowerPoint triangle

The single most important structural concept, and the one with no analogue in
Word.

**Slide master**. Defines the theme binding, the colour map, and the default
text styles for a family of layouts. A deck may have several.

**Slide layout**. Belongs to exactly one master. Defines the arrangement of
placeholders for one kind of slide, for example "Title and Content".

**Slide**. Belongs to exactly one layout.

**Placeholder**, `p:ph`. A shape carrying a `type` and an `idx`. A placeholder
on a slide inherits its position, size and text formatting from the matching
placeholder on its layout, which inherits from its master. **`idx` is the join
key.** Changing it severs inheritance silently, which is why `add_slide` never
renumbers it.

Placeholders on a layout or master are **templates and are never drawn**. A
renderer that draws them emits "Click to edit Master title style" into
production output.

**Shape tree**, `p:spTree`. The ordered list of shapes on a slide. Document
order is z-order. Its children are shapes, pictures, graphic frames, group
shapes and connectors.

## Colour

**Theme colour**. A named slot in `a:clrScheme`: `dk1`, `lt1`, `dk2`, `lt2`,
`accent1` through `accent6`, `hlink`, `folHlink`.

**Colour map**, `p:clrMap`. A per-master indirection from the semantic names
used in markup (`bg1`, `tx1`) to theme slots. `bg1` usually maps to `lt1`, but a
dark master inverts it. Resolution goes through the map *before* the theme.
Skipping this is the most common colour bug.

**Colour transform**. A modifier applied to a resolved colour, in document
order: `tint`, `shade`, `alpha`, `lumMod`, `lumOff`, `satMod`, `hueMod`, `gray`,
`comp`, `inv`. Office themes use `lumMod` and `lumOff` on nearly every
accent-derived colour, so omitting them makes an entire deck the wrong shade.

**Format scheme**, `a:fmtScheme`. Theme-level lists of fill, line and effect
styles that shapes reference by index through `p:style`. A `fillRef@idx` above
1000 means index `idx - 1000` into the background fill list.

## Geometry and text

**Preset geometry**, `a:prstGeom`. One of roughly 187 named shapes, each defined
in the spec as a parameterised path with adjustable values.

**Custom geometry**, `a:custGeom`. An explicit path using the same guide and
path machinery as the presets, which is why implementing presets properly costs
almost nothing extra once the evaluator exists.

**Guide**, `a:gd`. A named formula in a small stack language used to compute
path coordinates from the shape's width, height and adjust values.

**Text body**, `a:txBody`. DrawingML text: `a:bodyPr` for the frame,
`a:lstStyle` for nine levels of list formatting, then `a:p` paragraphs of `a:r`
runs of `a:t` text. Structurally parallel to `w:p`/`w:r`/`w:t` but a completely
different vocabulary.

**Autofit**. How text responds to a box that is too small. `a:noAutofit`
overflows, `a:spAutoFit` grows the shape, `a:normAutofit` shrinks the text and
**stores the scale it computed in the file**, which the renderer should trust
rather than recompute.

## Project vocabulary

**F-ID**. A story identifier, `F-001` onward. `F-X###` marks cross-cutting work
that belongs to no milestone.

**Test gate**. The smallest test that proves a story works. Every story has
exactly one. Nothing merges without it.

**Hash harness**. The output-stability check introduced in M1: a digest of each
sample document's XML parts and its rendered first page, compared before and
after every migration step. It exists because the extraction changes unit
conversion and text-shaping inputs, both of which alter output without failing
to compile.

**Deterministic font mode**. Rendering using only the bundled fonts, with system
font loading bypassed, so that a digest recorded on one machine matches one
recorded on another.

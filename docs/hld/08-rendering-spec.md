# 08, Rendering spec

Owners: `oxml-layout` for the types, `oxml-pdf` for the backends,
`rpptx-render` for the slide pipeline.

## The seam that makes this cheap

The staged `crates/oxml-pdf` backend consumes `oxml-layout::LayoutResult` and
uses `oxml-media` for image format and header metadata. Its writer, font,
image, and raster modules support the existing text, line, rectangle, image,
link, metadata, and outline paths. The released `crates/rdocx-pdf` backend
remains dependency-separate, but carries the approved F-039 global CTM writer
change until the F-046 cutover removes the duplicate. The shared contract is:

```rust
pub struct LayoutResult { pages: Vec<PageFrame>, fonts: Vec<FontData>,
                          metadata: Option<DocumentMetadata>, outlines: Vec<OutlineEntry> }
pub struct PageFrame { page_number: usize, width: f64, height: f64,
                       elements: Vec<PositionedElement> }
```

**A slide is a page with a fixed size.** Font subsetting, ToUnicode CMaps, JPEG
passthrough, PNG inflate, soft masks, PDF assembly and the tiny-skia rasteriser
all carry over unchanged. That is roughly 1,667 lines the presentation side does
not have to write.

`Group` recursively emits a saved graphics state, its local matrix, optional
clip and opacity, its children, and a matching restore. Group effects and the
raster path remain staged for their owning stories. `Path` emits geometry,
solid fill and solid stroke operators. Gradient and tile paints remain staged
for their resource-owning stories. The font, image, and link collection passes
use `walk` and depth-first leaf ordinals so nested content shares stable resource
identity with recursive emission. The published backend does not depend on this
staged crate before the shared-crate cutover.

## Extending `PositionedElement`

The obvious approach is to bolt `rotation` onto `Image`, `gradient` onto
`FilledRect`, `arrowhead` onto `Line` and so on. That is about ten new fields
across five arms, breaks every construction site in `rdocx-layout`, and **still
does not nest**, because `a:grpSp` transforms compose arbitrarily deep.

**Add exactly two arms instead.** Rotation, flips, group transforms, clipping,
all 187 presets and `custGeom` collapse into them, and rdocx's five existing
arms are untouched, so rdocx's output stays bit-identical.

```rust
/// 2x3 affine, row-major: x' = a·x + c·y + e ; y' = b·x + d·y + f.
/// Maps 1:1 onto the PDF `cm` operator and onto tiny_skia::Transform.
pub struct Transform { pub a: f64, pub b: f64, pub c: f64,
                       pub d: f64, pub e: f64, pub f: f64 }

pub enum PathCommand { MoveTo(Point), LineTo(Point),
                       CurveTo { c1: Point, c2: Point, to: Point }, Close }
pub enum FillRule { NonZero, EvenOdd }
pub struct Path { pub commands: Vec<PathCommand>, pub fill_rule: FillRule }

pub struct GradientStop { pub offset: f64, pub color: Color }

/// Content-addressed media handle. Replaces `embed_id`, which assumed one
/// global relationship namespace and is therefore invalid for pptx.
pub struct MediaId(pub u64);

pub enum Paint {
    Solid(Color),
    Linear { start: Point, end: Point, stops: Vec<GradientStop>, extend: (bool, bool) },
    Radial { center: Point, radius: f64, focal: Point,
             stops: Vec<GradientStop>, extend: (bool, bool) },
    Tile { image: MediaId, tile: Rect, transform: Transform },
}

pub struct Stroke { pub paint: Paint, pub width: f64,
                    pub cap: LineCap, pub join: LineJoin,
                    pub dash: Option<Vec<f64>> }

pub struct PathElement { pub path: Path, pub fill: Option<Paint>,
                         pub stroke: Option<Stroke> }

#[non_exhaustive]
pub enum Effect { OuterShadow { dx: f64, dy: f64, blur: f64, color: Color } }

pub struct Diagnostic { pub message: String }

pub struct GroupElement { pub transform: Transform, pub clip: Option<Path>,
                          pub opacity: f64, pub effects: Vec<Effect>,
                          pub children: Vec<PositionedElement> }

#[non_exhaustive]
pub enum PositionedElement {
    Text(GlyphRun), Line { .. }, FilledRect { .. }, Image { .. }, LinkAnnotation { .. },
    Path(PathElement),      // new
    Group(GroupElement),    // new
}
```

`PositionedElement` and `Effect` are the two non-exhaustive enums. `PageFrame`
gains `background: Option<Paint>`, and `LayoutResult` gains
`diagnostics: Vec<Diagnostic>`. The two structs are also non-exhaustive and use
constructors that take their previous required fields, defaulting the new
fields to `None` and an empty vector. This surface remains staged in unpublished
`oxml-layout` at version 0.0.0.

### Why `Group` is the whole design

A shape's `a:xfrm` rotation applies to its fill, its outline **and its text**
together, so a group is the semantically correct carrier. A nested `a:grpSp`
with `a:chOff` and `a:chExt` becomes a nested `Group` whose transform is:

```
translate(-chOffX, -chOffY)
  · scale(extX/chExtX, extY/chExtY)
  · translate(offX, offY)
  · rotate_about(rot / 60000, cx, cy)
  · scale(flipH ? -1 : 1, flipV ? -1 : 1) about the centre
```

`rpptx-layout` freezes this mapping before the renderer boundary. Every
`ResolvedShape` carries a backend-neutral `group_transform` accumulated from
its parent groups. The shape's own point bounds, rotation, and flips remain
separate, so the renderer can group the shape's geometry, paint, outline, and
text once and then apply the accumulated parent transform. Nested composition
applies the child transform before the parent transform. Missing group mapping
components leave that group's contribution as identity, while a zero child
extent cannot produce a non-finite matrix.

Rotated text, rotated pictures, rotated gradients and clipped picture frames all
follow for free. Arrowheads lower into extra closed, filled `Path` elements
using the line paint, so they need no backend support. The renderer selects the
first and last non-degenerate path tangents and reverses the first tangent for a
head endpoint. Small, medium and large endpoint width or length use 2, 3 and 5
times the stroke width. Triangle has a tip and straight base, stealth adds a
half-length inset notch, diamond places its widest points at half length, oval
uses four cubic segments, and arrow is a closed chevron with one-stroke-wide
arms. Degenerate geometry emits no endpoint.

## The recursion hazard

**Three passes in the PDF backend must visit every leaf nested inside a
`Group`:**

| Pass | Implementation | Symptom if missed |
|---|---|---|
| Font subsetting | `collect_glyph_usage` in `font.rs` | Grouped text renders with no font |
| XObject registration | Image collection in `write_pdf` | Grouped images vanish |
| Link annotations | Allocation, page assembly, and dictionary writing in `write_pdf` | Grouped hyperlinks are dead |

These failures appear **only for grouped content**, so the mitigation is one
helper in `oxml-layout` that flattens groups and accumulates the transform:

```rust
pub fn walk(elements: &[PositionedElement], f: &mut impl FnMut(&PositionedElement, &Transform));
```

All three passes use it. Image and annotation resources use the callback's
depth-first leaf ordinal, which recursive content emission matches. Link
rectangles use the accumulated transform before conversion to PDF page
coordinates. Each pass has an explicit nested-target regression test.

## Two remaining defects to fix

All are forced by pptx, and all improve rdocx:

| Defect | Location |
|---|---|
| `dash_pattern: _` means dashes are ignored in **all** PNG output today | `raster.rs:73` |
| Images keyed `Im{page}_{elem}`, no deduplication, and the full font dictionary is written into every page | `writer.rs` |

The last one matters at deck scale: a 200-slide deck would embed the master's
logo 200 times.

## The PDF backend

**One global CTM instead of a per-element flip.** Emit `q 1 0 0 -1 0 H cm` once
at the top of the content stream, then write everything in top-left, y-down
coordinates so group transforms compose naturally.

- Text: `Tm` becomes `[1 0 0 -1 x y]`, the negative `d` cancelling the outer flip
  so glyphs stay upright. Mathematically identical output.
- Images: `cm [w 0 0 -h x y+h]`. The `y+h` translation preserves the original
  image rectangle after PDF matrix concatenation.
- Link annotations live in `/Annots`, not the content stream, so that code is
  untouched.

**This is the single highest-risk change in the plan.** Gate it on golden-PNG
diffs of the existing `samples/` corpus, comparing **pixels, never PDF bytes**,
because the operator stream legitimately changes. Land it as its own reviewable
commit before any pptx code exists. The reviewed Poppler 26.01.0 baseline
contains four stroke-antialias pixel changes across `invoice` and `quote`. The
other five samples remain exact, and normal check mode requires exact equality
against that reviewed baseline.

`Path` is `m`/`l`/`c`/`h` followed by `f`, `f*`, `S`, `B` or `B*` by supported
fill, stroke and rule. Stroke state uses `w`, `J`, `j`, `M` and `d`. `Group` is
`q`, `cm`, optional clip via `W n`, optional `/GS gs` for opacity, recurse,
`Q`.

**Gradients** use `/Pattern cs /P scn` for fills and `/Pattern CS /P SCN` for
strokes. One private registry pre-scans path paint in depth-first order,
allocates one deterministic pattern per gradient occurrence, and gives each
page only the pattern names that it uses. A type 2 pattern dictionary composes
the accumulated element transform with the global page flip in `/Matrix` so
the gradient rotates with its shape in the same top-left coordinate space as
the path. It references a type 2 axial or type 3 radial shading and a **type 3
stitching function over type 2 exponential functions**, one per stop interval.
Stops are sorted and clamped, and the last stop at a repeated offset wins.
Single-stop gradients degrade to a solid at build time. **Stop alpha needs a
luminosity soft mask and is out of scope for v1**. The PDF fallback composites
each stop over white and emits opaque DeviceRGB values.

**Alpha** uses one document-wide `/ExtGState` per distinct normalized value.
Each state sets both `CA` and `ca`, and each page references only the states its
elements use. Text, lines, rectangles, solid paths, and group opacity share the
same private registry. A path whose fill and stroke alpha differ repeats its
geometry so each paint operation selects the right state.

## The rasteriser

Raster output uses one recursive renderer carrying the accumulated
`tiny_skia::Transform` and current clip mask. A group pre-concatenates its local
transform, intersects its optional path clip with the parent mask, and draws
children in order. Non-opaque group content is composited through one scratch
pixmap so overlapping children receive group opacity once rather than per
primitive.

Backend-neutral path commands map to `PathBuilder`, with non-zero and even-odd
fill rules passed to `fill_path`. Solid, linear, and radial paint can fill or
stroke paths. Gradient geometry follows the accumulated transform, and a
non-extended gradient domain is clipped before painting. Line and path dash
arrays use tiny-skia's phase-zero stroke dash support. Page background paint is
drawn over the white default before page elements.

Tile paint and group effects are not rendered by the raster backend. Picture
tiles are different. `rpptx-render` lowers them to bounded row-major image
elements before the shared backend sees them. In particular, the `blur` field
on an outer shadow has no raster approximation.

## Preset geometry

| Option | Cost | Coverage | Verdict |
|---|---|---|---|
| Hand-write the top ~20 | 1-2 days | ~85 percent by frequency, but the tail is *visibly broken* | insufficient |
| Generate from the official ECMA-376 shape definitions | 4-6 days | 187 preset shape definitions | **chosen** |
| Port LibreOffice's table | 2 days | ~100 percent | **rejected: MPL-2.0 file-level copyleft, incompatible with MIT OR Apache-2.0** |

**The decisive argument is that the marginal cost over hand-writing is near
zero**, because `a:custGeom` uses the identical guide and path machinery. The
evaluator gets written either way, so the presets become data rather than code.
Presets additionally carry the `<a:rect>` text rectangle needed to place text
inside non-rectangular shapes.

Mechanism: an offline generator under `tools/gen-presets/` emitting a
**checked-in** generated file. Not a `build.rs`, because checked-in generated
code gives reproducible builds, no build-time XML dependency, a clean
`cargo publish` and a reviewable diff.

The only permitted generator input is the official
[ECMA-376 fifth-edition Part 1 archive](https://ecma-international.org/publications-and-standards/standards/ecma-376/),
downloaded as `ECMA-376-1_5th_edition_december_2016.zip`. The source inside it
is `OfficeOpenXML-DrawingMLGeometries.zip/presetShapeDefinitions.xml`. It has
187 preset shape definitions and SHA-256
`2f7c868d857c1e3c4b5a6068759fe0e07d77ad58377a6618d1b02ba3507b6939`.
The generator refuses a different count or digest.

The [Ecma software policy](https://ecma-international.org/policies/by-ipr/ecma-international-policy-on-submission-inclusion-and-licensing-of-software/)
identifies XML data sets as source code and makes software incorporated in a
standard available under its three-clause BSD licence. F-090 must retain the
Ecma copyright notice, the three licence conditions, and the disclaimer beside
the checked-in generated output. The repository remains MIT OR Apache-2.0, with
the ECMA-derived table carrying its own third-party notice.

LibreOffice's preset table must not be used as generator input because its
MPL-2.0 file-level copyleft is outside this repository's licensing model. If
the official archive, exact count, exact digest, or required notice cannot be
reproduced, the fallback is deriving the table from the specification text.

Fallback for an unknown preset, in order: use `a:custGeom` if present, otherwise
emit the bounding rectangle **and still lay out the text inside it**, recording
a diagnostic. The invariant is *never lose a shape, never lose its text, only
ever lose its silhouette*.

## Text in a shape

Slide text is not flow text. It is a fixed box with anchoring, insets, optional
wrap, and autofit.

`line.rs` moves to `oxml-layout` and is decoupled from its four docx imports
(`CT_TabStop`, `ST_Jc`, `ST_TabJc`, `ST_Underline`) in favour of owned types,
with `LineSpacing` replacing the stringly-typed `line_rule: Option<String>`, and
one new behaviour: `wrap: false` never breaks except on an explicit break.

The algorithm: resolve `a:bodyPr`, compute the content box from the preset's
text rectangle minus insets, resolve each paragraph through the nine-level
chain, build inline items, stack lines from the box top, then anchor.

Top anchoring uses no translation, centre uses half the spare content height,
and bottom uses all of it. These offsets remain negative when the complete text
stack overflows. With positive spare height, justified anchoring divides the
space between line boxes, while distributed anchoring uses equal line gaps with
half a gap before the first line and after the last. Both forms centre a single
line and leave an overflowing stack undistributed. Anchoring never adds a clip.

**Vertical text** is laid out horizontally in a transposed box and wrapped in a
`Group` with a 90 degree rotation. `eaVert` upright stacking is not supported in
v1 and degrades to rotated vertical with a diagnostic.

**Bullets** become marker inline items, with `marL` and `indent` mapping onto
the existing left and hanging indent support. The marker font, colour, and
point or percentage size resolve independently. An omitted value uses the
paragraph's first effective run style.

Automatic numbering keeps one counter per text body and list level. The first
automatic paragraph at a level uses `startAt`, and the next paragraph at that
level with the same scheme increments it. Returning to a shallower level resets
deeper counters. A scheme change, character bullet, or paragraph without a
bullet resets the affected level. The renderer formats `arabicPlain`,
`arabicPeriod`, `arabicParenR`, `arabicParenBoth`, `alphaLcPeriod`,
`alphaUcPeriod`, `romanLcPeriod`, and `romanUcPeriod`. Any other schema scheme
uses a visible `arabicPeriod` marker.

The Wingdings trap is handled before font resolution. `a:buChar` U+F0B7 maps
to the visible Unicode bullet U+2022 instead of passing through the Wingdings
to Symbol alias as a private-use codepoint.

### Autofit

**PowerPoint stores its own computed answer in the file.** Trust it.

- `a:noAutofit`, the default: draw and overflow. **Do not clip.** Spilling looks
  less broken than truncation.
- `a:spAutoFit`: the stored `a:ext` is what PowerPoint last computed. Do not
  resize.
- `a:normAutofit fontScale="62500" lnSpcReduction="20000"`: apply verbatim. This
  is both cheapest and most faithful, because it reproduces exactly what the
  authoring application decided.
- Only a bare `<a:normAutofit/>` needs iteration, and then walk PowerPoint's own
  quantised 2.5 percent ladder rather than binary-searching a continuous scale,
  so the computed value matches what PowerPoint would have written. At most 31
  steps, typically one to three, and a shaping cache makes repeat passes nearly
  free.

## Performance

`Document` keeps normal-font and deterministic `LayoutResult` values in
separate `Mutex<Option<Arc<_>>>` caches. `render_page_to_png`,
`render_all_pages`, `to_pdf` and `layout_page` share the normal result. The
deterministic page renderer uses its own result, while caller-supplied font
layouts remain uncached because those fonts are not part of a stable cache key.

Every public document mutation and mutable-accessor entry point clears both
caches before changing or exposing content. Rendering every page through the
single-page entry point therefore performs one layout per font mode instead of
one layout per page. The presentation facade follows the same ownership model
when it is added.

```rust
pub fn layout_presentation(input: &RenderInput) -> Result<LayoutResult>;
pub fn layout_slide(input: &RenderInput, index: usize) -> Result<PageFrame>;
```

## The renderer's input

```rust
pub struct RenderInput {
    pub slides: Vec<ResolvedSlide>,
    pub media: HashMap<MediaId, MediaData>,   // deduplicated across the deck
    pub fonts: Vec<FontFile>,
    pub metadata: Option<DocumentMetadata>,
}
```

`RenderInput` is the rendering boundary. It contains only owned,
format-neutral `ResolvedSlide` values from `rpptx-layout`. Raw PresentationML
parts remain on the upstream assembly side of that boundary:

```rust

pub struct SlideBundle {
    pub slide: CT_Slide,
    pub layout: Arc<CT_SlideLayout>,   // ~5 layouts shared by 200 slides
    pub master: Arc<CT_SlideMaster>,
    pub theme:  Arc<CT_OfficeStyleSheet>,
    pub notes:  Option<CT_NotesSlide>,
    pub hidden: bool,
    pub relationships: RelScopes,
}

/// A slide, its layout and its master each have their OWN relationship
/// namespace: "rId2" means three different things.
pub struct RelScopes {
    pub slide: HashMap<String, ResolvedRel>,
    pub layout: HashMap<String, ResolvedRel>,
    pub master: HashMap<String, ResolvedRel>,
}

pub enum RelScope {
    Slide,
    Layout,
    Master,
}

pub struct ResolvedRel {
    pub target: String,
    pub relationship_type: String,
}

pub struct MediaData {
    pub bytes: Vec<u8>,
    pub content_type: String,
}
```

Relationship lookup requires an explicit `RelScope`, so equal identifiers in
slide, layout, and master parts never alias. Blips are resolved to bytes
**before** constructing `RenderInput` and keyed by `MediaId::from_bytes`. This
is why `embed_id` cannot survive the port, and it deduplicates a logo that
appears on every slide.

Upstream package assembly also constructs the source-neutral `ScopedMediaIds`
maps consumed by `ResolveCtx::resolve_slide_with_media`. Only embedded picture
relationships enter those maps. Linked media remains diagnosed and no renderer
performs network access.

An uncropped rectangular stretch picture that rotates with its shape lowers to
one image in local shape bounds. Source crop clamps each edge to the zero-to-one
range, rejects an empty retained width or height, expands the source image
rectangle, and clips it to the destination and picture geometry. When picture
content does not rotate with the shape, stretch and tile coverage expands to
the rotated geometry's axis-aligned bounds before inverse rotation, then clips
to the rotated picture path. Picture content is emitted before its outline.

Tile size starts from probed pixel dimensions. A positive declared blip DPI
overrides both embedded axes, embedded DPI overrides the 96 DPI fallback, and
the existing truncating native-size conversion supplies EMU before conversion
to points. Translation, fractional scale, alternating axis flips, nine-way
alignment, source crop, and `rotate_with_shape` are applied in presentation
lowering. Repeats use deterministic row-major order and are bounded by element
and cloned-media byte limits before allocation.

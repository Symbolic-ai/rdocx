# 08, Rendering spec

Owners: `oxml-layout` for the types, `oxml-pdf` for the backends,
`rpptx-render` for the slide pipeline.

## The seam that makes this cheap

The `crates/oxml-pdf` backend consumes `oxml-layout::LayoutResult` and
uses `oxml-media` for image format and header metadata. Its writer, font,
image, and raster modules support the existing text, line, rectangle, image,
link, metadata, and outline paths. `rdocx` uses this backend directly, and
`rdocx-pdf` contains only the deprecated `pub use oxml_pdf::*` compatibility
shim. The shared contract is:

```rust
pub struct LayoutResult { pages: Vec<Arc<PageFrame>>, fonts: Vec<FontData>,
                          metadata: Option<DocumentMetadata>, outlines: Vec<OutlineEntry>,
                          diagnostics: Vec<Diagnostic>,
                          structure: Option<DocumentStructure> }
pub struct FontData { id: FontId, family: String, data: Arc<[u8]>,
                      face_index: u32, bold: bool, italic: bool }
pub struct PageFrame { page_number: usize, width: f64, height: f64,
                       elements: Vec<PositionedElement>, background: Option<Paint> }
pub struct SourceNodeId(NonZeroU32)
pub struct SourceSpan { node: SourceNodeId, char_start: u32, char_end: u32 }
pub struct DocumentStructure { root: StructureId, nodes: Vec<StructureNode> }
pub struct StructureNode { id: StructureId, role: StructureRole,
                           children: Vec<StructureId>, alternate_text: Option<String> }
```

`TextSegment` and `GlyphRun` each carry `source: Option<SourceSpan>`. The node
is meaningful only within the result that allocated it. `char_start` and
`char_end` are exclusive Unicode-scalar offsets, not byte, UTF-16, grapheme, or
glyph-cluster positions. Word assigns spans before shaping. Its initial text
projection assigns one shaped segment to each formatting and provenance span.
The shared line breaker alone discovers UAX 14 opportunities, reshapes each
exact byte slice, and subdivides the scalar source range, so every wrapped
fragment retains an exact contiguous source range. Generated text uses `None`.

An empty Word paragraph contributes one `TextSegment` with empty text, zero
width, no glyph ids, and the resolved paragraph-mark font metrics. Its source
is the paragraph node at scalar range `0..0` when provenance is requested and
`None` otherwise. The original empty-line box remains unchanged, so the
carrier does not move later content. PDF font collection and emission skip
empty no-glyph runs, and raster draws no pixels for them.

**A slide is a page with a fixed size.** Font subsetting, ToUnicode CMaps, JPEG
passthrough, PNG inflate, soft masks, PDF assembly and the tiny-skia rasteriser
all carry over unchanged. That is roughly 1,667 lines the presentation side does
not have to write.

`Group` recursively emits a saved graphics state, its local matrix, optional
clip and opacity, its children, and a matching restore. `Path` emits geometry,
solid fill and solid stroke operators. The font, image, and link collection
passes use `walk` and depth-first leaf ordinals so nested content shares stable
resource identity with recursive emission.

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
    MarkedContent { structure: Option<StructureId>, children: Vec<PositionedElement> },
}
```

`PositionedElement` and `Effect` are the two non-exhaustive enums. `PageFrame`
gains `background: Option<Paint>`, and `LayoutResult` gains
`diagnostics: Vec<Diagnostic>`. The two structs are also non-exhaustive and use
constructors that take their previous required fields, defaulting the new
fields to `None` and an empty vector. This surface is published in
`oxml-layout` 0.1.2.

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

`rpptx-layout` freezes this mapping before the renderer boundary. For the
non-shearing path, the resolver factors accumulated group mappings into
child-coordinate scale and a remaining rigid transform. It materializes the
scale into every leaf's point bounds before resolving concrete geometry and
text boxes. Absolute line widths, font sizes, and effect metrics remain
unchanged. Every `ResolvedShape` carries only the remaining backend-neutral
rigid `group_transform`, which the renderer applies around the complete leaf.
Nested composition applies the child mapping before the parent mapping. Group
bounds do not create an implicit clip, and depth-first document order is
preserved. Missing group mapping components leave that group's contribution as
identity. A zero child extent uses finite unit scale on that axis and records a
stable diagnostic. A sheared or singular accumulated mapping records a
separate diagnostic and retains the previous affine fallback without claiming
general shear support.

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

The same walker recurses through `MarkedContent` without changing the
accumulated transform. Word pagination wraps exact emitted leaves in this
container. Empty no-glyph carriers produce no marked content. PDF collection
and field-substitution passes recurse into it, and compatibility assertions
that inspect leaf elements do the same.

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

**The writer is reproducible.** One layout produces one byte sequence, every
run and every process. It carries no creation date and no file identifier, and
every container it iterates to produce output is ordered rather than hashed:
the prepared fonts and their references by `FontId`, the glyph to Unicode table
by glyph id, and a page's image XObject names by element index. This is stated
rather than assumed because it was not true until S43. Three hashed maps were
iterated to write the file, so the same document wrote a different `/Font`
dictionary order, font object order and ToUnicode CMap line order on every run,
with no visible difference and no failing test.

Two writes of one document cannot detect that, since they reuse the same map
instances. The regression builds two documents and compares their bytes.

When `LayoutResult::structure` is present, the writer emits deterministic
`BDC` and `EMC` pairs with page-local MCIDs, `/StructParents`, one parent number
tree, `/StructTreeRoot`, `/MarkInfo`, an undetermined `/Lang`, and accessible
document title preferences. PDF/UA identification metadata is present only
when shown text contains no `.notdef` glyph. Word roles lower to headings,
paragraphs, nested `L` and `LI`, `Table`, `TR`, `TH`, `TD`, and `Figure`.
The PDF layer inserts the required `LBody` beneath each list item without
changing the backend-neutral tree. Informative figures carry source `/Alt`.
Table headers carry column scope. Decorative drawing and border paint emits as
artifacts. Malformed public structure graphs are rejected, and their marked
children recurse without BDC or orphan MCIDs. A result with no structure
retains the byte-compatible untagged path.

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

Tile paint is not rendered by the raster backend. Picture tiles are different.
`rpptx-render` lowers them to bounded row-major image elements before the
shared backend sees them. The raster backend supports the neutral outer-shadow
group effect. It renders the composited group subtree to a transparent layer,
uses the layer alpha as the shadow silhouette, applies a deterministic
device-scale box blur, offsets and tints the result, then composites the
unchanged group content above it. Resolved colour alpha is part of the shadow.
Group opacity applies once to the shadow and content together. PDF group
effects remain unsupported.

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

Presentation connectors reuse these generated definitions. `line` and
`straightConnector1` produce a single open segment. Bent and curved connector
presets retain their adjustment values and produce the generated ordered line
or cubic commands. Their direct stroke, dash, and endpoints use the normal
shape lowering path. A horizontal or vertical connector may collapse one
extent to zero. Its finite path and stroke remain visible rather than being
dropped as an empty shape.

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

Every shaped segment carries the selected font's ascent, descent, and `hhea`
line gap. A line's natural advance is the greater of its combined maximum
ascent and descent and the tallest complete run advance. Omitted line spacing
uses that natural advance, while percentage line spacing multiplies the largest
effective run point size. Exact point spacing remains exact, including when it
is smaller than the glyph box.
Non-negative leading is divided equally above and below the glyph box. A
below-natural exact line keeps its stated height and places no negative leading
before the baseline. Stored normal-autofit line-spacing reduction applies only
to explicit percentage line spacing.

An empty paragraph shapes a zero-width metrics carrier from its resolved
`a:endParaRPr` style. Its line height and font-relative paragraph spacing use
that resolved font and size. Nonempty paragraphs continue to use their first
effective run. `cap="all"` transforms run and field text to Unicode uppercase
before script-specific font selection and shaping. `cap="small"` has no render
projection and remains preserved as unmodelled XML. A paragraph with no visible
text or field emits no bullet, but it retains its empty line metrics.

`a:bodyPr/@spcFirstLastPara` controls only the text body's edge spacing. When
false, including when absent, the first paragraph's `spcBef` and the last
paragraph's `spcAft` do not contribute to the stack. Interior paragraph
spacing remains unchanged. A true value applies both edge values normally.

Top anchoring uses no translation, centre uses half the spare content height,
and bottom uses all of it. These offsets remain negative when the complete text
stack overflows. With positive spare height, justified anchoring divides the
space between line boxes, while distributed anchoring uses equal line gaps with
half a gap before the first line and after the last. Both forms centre a single
line and leave an overflowing stack undistributed. Anchoring never adds a clip.

**Vertical text** is laid out horizontally in a same-centre transposed box and
wrapped in a `Group`. `vert` rotates the group 90 degrees and `vert270` rotates
it -90 degrees. `eaVert` and `wordArtVert` use the `vert` path.
`mongolianVert` and `wordArtVertRtl` use the `vert270` path. Unsupported upright
or WordArt stacking stays visible and records, respectively,
`east Asian vertical text rendered as rotated vertical text`,
`Mongolian vertical text rendered as rotated vertical-270 text`,
`WordArt vertical text rendered as rotated vertical text`, or
`right-to-left WordArt vertical text rendered as rotated vertical-270 text`.

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

Slide-number fields substitute the one-based `PageFrame` number before text
shaping and carry `FieldKind::Page` into the emitted glyph run. An untyped field
that the resolver normalized from an effective `sldNum` placeholder follows the
same path. Other fields retain their stored display text.

An external URI frozen on `ResolvedRunStyle` is copied into each shaped
`TextSegment`. The line emitter creates one `LinkAnnotation` for every laid-out
run fragment. The PDF backend's recursive leaf walk applies the complete nested
group transform to each annotation rectangle. Unsupported actions never create
an annotation, but their visible text remains in the line.

## Tables

Table lowering derives cumulative row and column offsets from the resolved
grid. Right-to-left tables reverse visual column placement without changing
logical cell ownership. A merge continuation emits nothing. Its origin spans
the covered row heights and column widths, then emits one fill and one fixed-box
text body. Resolved cell margins define that text body's content box.

Borders are physical row or column segments rather than four strokes per cell.
The renderer maps every logical cell edge onto those segments and emits each
segment once. An adjacent-edge conflict selects the higher style-region
priority, then the wider stroke, then the deterministic side order top, left,
bottom, right. This keeps shared borders stable across merge origins and table
direction.

Each table origin draws its fill before its text. The table's unique border
segments draw after all cell fills and text so a neighbouring fill cannot cover
them. Table cells do not use the shape autofit algorithm.

Word table lowering also assigns logical ownership before pagination. A table
owns rows in source order. Repeating header rows use `TH`, ordinary cells use
`TD`, and each cell owns its paragraph nodes. A header repeated on another
page creates another marked-content occurrence for the same logical cell.

### Autofit

**PowerPoint stores its own computed answer in the file.** Trust it.

- `a:noAutofit`, the default: draw and overflow. **Do not clip.** Spilling looks
  less broken than truncation.
- `a:spAutoFit`: the stored `a:ext` is what PowerPoint last computed. Do not
  resize.
- `a:normAutofit fontScale="62500" lnSpcReduction="20000"`: apply verbatim. This
  is both cheapest and most faithful, because it reproduces exactly what the
  authoring application decided. Font scale applies to every effective run and
  bullet size. Line-spacing reduction multiplies explicit percentage spacing by
  `1 - lnSpcReduction`. Explicit percentage spacing remains based on the largest
  effective run point size. Omitted Single and exact point spacing are
  unchanged.
- Only a bare `<a:normAutofit/>` needs iteration, and then walk PowerPoint's own
  quantised 2.5 percent ladder rather than binary-searching a continuous scale,
  so the computed value matches what PowerPoint would have written. The ladder
  runs from 100 percent through a 25 percent floor, at most 31 candidates and
  typically one to three. A per-layout shaping cache makes repeat passes nearly
  free. If the 25 percent candidate still does not fit, draw it visibly without
  clipping.

## Performance

`Document` keeps normal-font and deterministic `WordLayoutResult` values in
separate `Mutex<Option<Arc<_>>>` caches. `layout`, option-taking accepted
layout, `render_page_to_png`, `render_all_pages`, `to_pdf`, and `layout_page`
share the normal result. The normal path also retains one synchronized
`rdocx-layout::Engine` after mutation invalidates that completed result. The
deterministic renderers use their own bundle. Tracked layouts remain uncached
and use the normal engine with a distinct revision-view paragraph identity.
Caller-supplied font layouts construct an isolated engine, remain uncached, and
cannot observe bundled or system fonts. Caller-font access returns an owned
bundle. The separate bundled-fallback caller-font mode retains one reusable
deterministic-base engine. Caller faces have highest priority, missing families
resolve from bundled faces, and system fonts remain unavailable. Its owned
result shares immutable pages and font bytes without caching the completed
bundle. Every PDF and raster path borrows its `LayoutResult` field from the same
bundle that owns the exact font data and Word source map. Cloning a completed
result shares each immutable page frame and font byte buffer.

Normal `FontManager` construction clones one process-lifetime snapshot of the
bundled and system face table. Installing, removing, or replacing a system font
therefore takes effect after process restart. File-backed faces share an
`Arc<[u8]>` by canonical file identity, including distinct TTC collection
indices, through a 256-entry and 128 MiB process cache. Poisoned process locks
recover by rebuilding the requested entry. Deterministic managers build from
bundled fonts only. Caller-font managers build from caller bytes only.

Each reusable font manager keeps exact shaping results keyed by `FontId`, owned
source text, and floating-point size bits. The memo is capped at 2,048 entries
and 16 MiB. Resolution, coverage misses, coverage fallbacks, and per-paragraph
font traces also have explicit bounds. Loading a different additional font set
rebuilds face identity and clears all dependent memo state.

The Word engine caches only ordinary context-independent body paragraphs and
tables made entirely from direct cache-safe paragraphs. It also caches safe
header and footer variants. Paragraph keys compare the complete typed
paragraph, content width, and revision view. A stable borrowed `u64` paragraph
fingerprint prefilters lookup candidates. The complete typed equality remains
authoritative after a fingerprint match, so collisions cannot serve the wrong
block. Hits neither clone the owned key nor refresh its queue position. Table
keys compare the complete typed table projection, content width, revision
view, and whether result-local provenance is present. Header and footer keys
compare the story kind, variant, complete section properties and geometry,
relationship identity, complete typed part, canonical resolved part bytes, and
whether result-local provenance is present. The retained-work context
separately compares styles, numbering, section properties, headers, footers,
media, charts, chart theme and colour map, core properties, hyperlinks, notes,
theme, additional fonts, page background, and the document-wide
wrapping-drawing state. Numbering, drawings including `AlternateContent`,
fields, hyperlinks, relationships, media, generated markers, nested tables,
content controls, preserved producer XML, and other traversal-sensitive input
bypass body reuse. Encountering any such body block disables later
retained-block reads for that layout, so inserting an earlier note or numbering
input cannot leave a later generated marker stale. Each entry owns its block,
diagnostics, and exact bounded font-resolution trace. Header and footer parts
with numbering, drawings, fields, hyperlinks, revisions, content controls,
generated markers, or other traversal-sensitive state bypass reuse. Supported
VML watermarks remain reusable because the part, complete section geometry,
resolved media bytes, and reusable context are all authoritative identity
inputs.

Retained block and restart state share a 4,224-entry and 64 MiB ceiling.
Paragraph blocks receive 4,096 entries and 56 MiB, table blocks receive 32
entries and 2 MiB, header and footer variants receive 64 entries and 4 MiB, and
aligned restart page and checkpoint slots receive 32 entries and 2 MiB. The published and
transaction-pending queues enforce the same partitions using retained-capacity
accounting for owned keys, resolved part bytes, rows, cells, paragraphs,
watermark media, diagnostics, font traces, and reflow buffers. Eviction follows
insertion order without hit-time queue maintenance. Oversized entries bypass
retention.

Cache publication is a whole-layout transaction. A late error publishes none
of the staged paragraph, table, header, or footer entries. A hit replays
diagnostics and the font trace without changing insertion order, then rebinds
scalar provenance to the source node allocated for the current
`WordLayoutResult`. After success, active faces are retained in first-resolution
order and stale faces and entries are removed. This makes warm and cold pages,
font ids, font bytes, diagnostics, revision view, and resolved source provenance
equal.

The engine also records restart checkpoints for one safe section containing
only one-line or two-line context-independent paragraphs. A checkpoint exists
only before a complete paragraph at an empty page boundary. It records the next
block, page count, and displayed header page number. Headers, footers,
backgrounds, notes, fields, headings, tables, drawings, multi-section content,
split paragraphs, keep constraints, and any unrepresented state use the full
paginator. A warm edit restarts at the last checkpoint before its first changed
block. It stops at an unchanged suffix only when the complete retained context,
font-resolution trace, page count, displayed header page number, and body
suffix all match exactly. The old raw pagination tail is then reused, and final
page frames are shared only after their canonicalized content compares equal.
If any equality or capacity check fails, pagination continues through the
normal full path.

The same restart record retains aligned pristine and field-substituted page
pairs. PAGE, NUMPAGES, and PAGEREF pages bypass reshaping only when exact page
content first recovers the retained pristine `Arc`, then page index, displayed
page number, total-page count, sorted bookmark targets, font trace, revision
view, and every substitution input compare equal. Field-free output shares its
pristine `Arc` directly. Field-bearing blocks still receive no pagination
checkpoint, so this optimization cannot widen the restart-safe region. Page
pairs and checkpoints occupy aligned slots inside the 32-entry partition, and
all pair payloads and exact inputs count toward its 2 MiB ceiling. A mismatch
reshapes the page. A bound failure drops the whole restart record.

`Document::transfer_reusable_layout_from` builds the receiver input before it
takes anything. It moves the source's normal engine only when that complete
retained-work context is compatible. Failure preserves both engines. Success
does not clear either document's completed result cache, and the next receiver
mutation can reuse unchanged safe paragraphs from the transferred engine.
`Document::transfer_reusable_bundled_fallback_layout_from` applies the same
checked move to the private deterministic-base caller-font engine. The caller
must provide the exact font slice used by the receiver. Font order and bytes,
revision view, and every other retained-work input participate in
compatibility, and rejection preserves both engines.

The low-level Word engine keeps its existing `LayoutResult` entry points and
discards provenance there. `layout_document_with_provenance` and
`layout_document_deterministic_with_provenance` allocate one immutable source
table before layout and return it with the positioned output as
`WordLayoutResult`. Repeated header and note layout reuses the same node rather
than allocating per page. Source metadata does not change shaping, pagination,
font selection, or rendered bytes.

### Word revision views

`LayoutInput::revision_view` selects the accepted or tracked projection before
Word text shaping. One ordered projection combines ordinary runs and typed
revision wrappers at their preserved boundaries, including nested wrappers and
revisions inside hyperlinks. The same projection applies to body, table,
header, footer, footnote, and endnote paragraphs. Accepted layout includes
insertions and move destinations and omits deletions and move sources. Tracked
layout includes both sides. It forces single underline on insertion and move
destination text and single strike on deletion and move source text while
retaining the remaining resolved formatting.

Provenance ranges are local to this selected projection. The field model's
`Field::projected_text` decision is the single owner of whether a field cache
advances a run's projection offset, which prevents repeated cached and literal
text from producing a plausible but false range. Field display glyphs remain
unattributed even when a parsed complex cache contributes to later offsets.

A tracked paragraph with visible revised content or a property-only revision
carries a changed marker into pagination. Every page fragment of that paragraph
draws one solid line outside the text margin, on the right for odd pages and on
the left for even pages. The marker does not affect text placement. Existing
render methods select accepted layout and retain the normal and deterministic
caches. Option-taking tracked renders are uncached.

### Word watermarks

Header `w:pict` content has a conservative renderer-only projection. A direct
VML shape with finite positive point bounds becomes a watermark only when it
contains one VML text path string or one relationship-namespaced image id.
Namespace aliases and shadows are resolved by expanded name. Unsupported,
ambiguous, malformed, or unrelated VML remains preserved but does not enter
layout. Named VML colours and six-digit RGB values lower to shared colour.
Unsupported colours and unresolved image relationships suppress that watermark
and add a stable diagnostic.

Text watermarks shape with the deterministic font manager and image watermarks
reuse the scoped relationship entry in the layout's collision-safe
`MediaRegistry`. Both lower to an opacity-bearing `GroupElement` centered in the
section margin rectangle, with rotation around the local watermark centre.
Pagination inserts the selected group before behind-body drawings, ordinary
header content, and body elements. PDF and raster backends therefore use their
existing recursive group paths without watermark-specific code.

Header selection follows Word's variants rather than content emptiness. A
title page selects its first variant even when that variant is blank. When the
decoded `w:evenAndOddHeaders` setting is active, displayed even page numbers
select the even variant even when it is blank. `w:pgNumType/@w:start` resets
that displayed parity for a section. Missing later variants inherit only the
same type from the preceding section. No selected first or even variant borrows
default header, footer, or watermark content.

Every public document mutation and mutable-accessor entry point clears both
completed result caches before changing or exposing content. It preserves the
normal engine so safe paragraph and shaping work can be reused after an edit.
Rendering every page through the single-page entry point therefore performs one
layout per font mode instead of one layout per page. The presentation facade
follows the same ownership model when it is added.

```rust
pub fn layout_presentation(input: &RenderInput) -> Result<LayoutResult>;
pub fn layout_presentation_deterministic(input: &RenderInput) -> Result<LayoutResult>;
pub fn layout_slide(input: &RenderInput, index: usize) -> Result<PageFrame>;
```

Both whole-presentation entry points use the same private page-lowering path.
The normal entry point retains system-font discovery. The deterministic entry
point starts from `FontManager::new_deterministic()`, then adds only explicit
font files already present in `RenderInput` before lowering every slide.

The `rpptx` facade assembles a presentation package into this input through its
deterministic render boundary. It rejects a source-to-resolved shape-count
difference and verifies that page count matches slide count before returning.
The CLI thumbnail path uses this same boundary for slide one. It derives DPI
from the rendered page width so the PNG is exactly 320 pixels wide while its
height remains proportional, then applies the normal per-slide pixel bound.
The corpus driver in `crates/rpptx/examples/render_deck.rs` calls that boundary
for all 50 pinned decks, renders every page at 150 dpi, and writes one TSV row
per slide. Each row records the positive-extent source leaf count, resolved
shape count, dropped count, diagnostics, and PNG path. A panic or missing page
fails the driver.

## Word bookmark field pagination

Word `REF` fields resolve the text of one uniquely correlated top-level
bookmark before shaping. A missing or ambiguous target keeps the stored field
display and emits a stable diagnostic. `PAGEREF` follows the same validation,
then shapes the fixed page placeholder and carries a format-neutral target id
through line breaking.

Only bookmarks named by a valid `PAGEREF` add a zero-width target marker to
page output. After the document paginates once, the existing field substitution
pass records the page containing each target, reshapes PAGE, NUMPAGES, and
target-page values, and removes the target markers. It does not run pagination
again. A field inside a laid-out table paragraph follows the same target
discovery and substitution path. Normal and deterministic layouts use this
identical algorithm, with the deterministic entry point restricted to bundled
and document-supplied fonts.

The native facade exposes a separate read-only field evaluator for automation
callers. `PAGE` and `NUMPAGES` always report pagination deferral. A valid
`PAGEREF` also reports deferral, while a missing or ambiguous target keeps its
stored display with a stable diagnostic. The evaluator does not replace the
single post-pagination substitution pass and does not trigger layout. `REF`
resolves the same unique top-level bookmark text used by layout, so pure
evaluation and rendering share the same target-validity boundary.

## Word chart pagination

The Word layout input owns document-scoped parsed chart targets, the effective
DrawingML theme, and the standard DrawingML colour map. The paragraph engine
renders a chart into a local `Group` with the same `FontManager` used for the
surrounding document. `InlineItem::Group` and `LineItem::Group` carry the
result through shared line breaking with the same width, height, ascent, and
descent rules as an image. Pagination translates the local group to the
resolved line position without changing its children.

An informative inline image or chart uses the additive `Figure` variant in
the same non-exhaustive inline and line enums. The variant carries alternate
text and the logical figure id around an unchanged `Image` or `Group`, then
pagination lowers it to `PositionedElement::MarkedContent`. Existing `Image`
and `Group` fields and direct constructors remain unchanged. A drawing with no
usable alternate text stays on the existing variant and becomes an artifact.

An anchored chart uses `AnchoredContent::Group`. The existing anchored drawing
path resolves its page rectangle, wrapping distances, and behind-text order,
then translates the local group to that rectangle. Missing, external, or
malformed chart targets and chart-renderer errors produce a visible placeholder
group plus a stable layout diagnostic. They do not disappear and do not move
chart logic into a PDF or raster backend.

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

Presentation page lowering constructs `LayoutResult` with `structure: None`.
Its PDF remains on the existing untagged path, and the added semantic carrier
does not alter its visible elements, resources, or raster pixels.

`ResolvedContent::Group` carries frozen backend-neutral chart output. The
ordinary shape lowering path inserts that group below the graphic-frame
transform, exactly as it does other local shape content. No PDF or raster path
contains chart-specific logic. Because the group may contain labels already
shaped by the resolver, `layout_presentation_with_font_manager` consumes the
same `FontManager` that produced the group and then shapes ordinary slide text.

`ResolvedSlide::background` is an optional `ResolvedBackground`, either a
backend-neutral `Paint` or a shared `ResolvedImage`. `ResolvedContent::Image`
and `ResolvedShape::image_fill` use the same `ResolvedImage` structure, which
carries media ID, source crop, stretch or tile placement, declared DPI, and
rotation policy. A background image lowers through the existing picture path
before every slide shape. A shape image fill lowers through that path and its
concrete geometry clip before the shape stroke and text. It stays separate from
`ResolvedContent`, so picture-filled text retains both layers. Paint backgrounds
remain in `PageFrame::background`.

Linear-gradient paints already contain their final local-space axis. The
resolver applies DrawingML angle scaling and rectangle projection before this
boundary. A `rotWithShape="0"` linear fill crosses that boundary only when the
effective shape transform is unrotated and unflipped, where its local axis is
identical to the shape-relative result. A rotated or flipped case remains a
resolver diagnostic until the neutral paint contract can represent its
independent transform. Backends consume concrete endpoints without
reinterpreting `scaled`, the source fill bounds, or rotation policy.

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
    pub target_mode: Option<String>,
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
maps consumed by `ResolveCtx::resolve_slide_with_media` and records package
content types by resolved media ID. Only embedded picture relationships enter
those maps. The target must be present, its declared content type must match its
sniffed PNG or JPEG format. JPEG inputs must use the 8-bit, three-component
layout supported by both the raster decoder and PDF pass-through path. Encoded
bytes and decoded scanline or pixel storage must remain within 16 MiB caps, and
the stricter raster backend must visibly
decode it at its native pixel bounds before it enters renderer input. Native
bounds retain sparse visible pixels that a one-pixel probe could miss. This
also covers the PDF backend, whose JPEG path is a less strict pass-through.
Missing, unsupported, malformed,
invisible or content-type-mismatched media stay unresolved so the owning shape
can retain its visible fallback and diagnostic. Linked media remains diagnosed
and no renderer performs network access. Direct background picture fills use
the scope of the slide, layout, or master that
supplied `p:bg`. Theme-referenced picture fills are rejected
precisely until a theme media scope exists. They never fall back to a same-named
identifier in another scope.

The same assembly step constructs `ScopedChartResources`. Each internal chart
relationship resolves against its producing part, parses the target as
`CT_ChartSpace`, and stays in the slide, layout or master map that owns its
identifier. External, missing-target and invalid resources remain typed
failures for contextual diagnostics. A same-named identifier in another scope
is never consulted. The resolver consumes these maps before `RenderInput` is
constructed and freezes either native group content, cached-picture content or
a labelled bounds fallback.

The corpus renderer and integration tests call the same crate-local package
rendering function. Relationship assembly, media admission, chart parsing,
resolution and ordinary renderer lowering therefore have one production path.

An OLE payload remains raw for serialization. Its optional standard fallback
`p:pic` is an upstream projection only. When that picture has an embedded PNG
relationship in the producing part scope, it crosses the renderer boundary as
ordinary `ResolvedContent::Image` and uses the graphic-frame bounds. The
diagnostic states that the static preview is visible while embedded OLE
interactivity is not rendered. Missing previews, linked or unresolved media,
and non-PNG media retain the visible bounds fallback. In particular, a WMF
preview is not passed to a backend that cannot decode it.

The same relationship scopes project hyperlink relationships with
`TargetMode="External"` into `ScopedHyperlinkTargets`. Internal package targets
never enter that map. The resolver uses the producing shape's scope so equal
relationship identifiers on a slide, layout, and master cannot alias.

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

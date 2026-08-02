# 07, Inheritance and resolution

Owner: `rpptx-layout`.

This is the highest-risk correctness work in the project. Getting it wrong
produces subtly wrong fonts, positions and colours on real templates, which
users notice immediately and cannot describe. It is scheduled as its own
milestone for that reason.

`crates/rdocx-layout/src/style_resolver.rs` is worth studying for its
`merge_from` cascade shape, but the structure differs: this walks a **part
hierarchy**, not a style graph.

## The output contract

```rust
pub struct ResolvedSlide {
    pub size: (f64, f64),                 // points
    pub background: Option<ResolvedFill>,
    pub shapes: Vec<ResolvedShape>,       // already flattened and in draw order
    pub diagnostics: Vec<Diagnostic>,
}

pub struct ResolvedShape {
    pub bounds: Rect,
    pub rotation_deg: f64,
    pub flip_h: bool, pub flip_v: bool,
    pub geometry: ResolvedGeometry,
    pub fill: ResolvedFill,
    pub line: Option<ResolvedLine>,
    pub shadow: Option<ResolvedShadow>,
    pub content: ResolvedContent,
    /// Set when we fell back: unknown preset, SmartArt, chart, ink.
    pub unsupported: Option<&'static str>,
}

pub enum ResolvedContent {
    None,
    Text(ResolvedTextBody),
    Image { media: MediaId, src_rect: Option<CropRect> },
    Table(ResolvedTable),
}
```

Every theme reference, colour transform, inherited property and list-style level
is **already collapsed to a concrete value**. The renderer consumes this and
nothing else, and never sees a `p:` or `a:` type.

**Freeze this contract when the resolver lands, or the resolver and renderer
tracks diverge.** It is versioned with the crate.

## Draw order

The single most common source of visibly wrong output. For each slide, in
order:

1. **Background.** Slide `p:bg`, else layout `p:bg`, else master `p:bg`, else
   the theme's first background fill style.
2. **The master's non-placeholder shapes**, if the layout's `showMasterSp` is
   not `0`.
3. **The layout's non-placeholder shapes**, if the slide's `showMasterSp` is not
   `0`.
4. **The slide's own `p:spTree`**, in document order.

**Placeholders on the layout and master are templates and are never drawn.**
They contribute inherited geometry and formatting only. A renderer that draws
them emits "Click to edit Master title style" into production output.

Two further suppressions while flattening passes 2 and 3:

- A master or layout placeholder is drawn only if nothing further down the chain
  occupies it. A layout placeholder suppresses the master placeholder it
  inherits from, and a slide placeholder suppresses the layout one with the same
  `idx`.
- `dt`, `ftr` and `sldNum` render only when the layout and master `p:hf` flags
  permit. Their text comes from the slide's own shapes if present, otherwise
  from the `a:fld` in the layout or master shape.

This logic belongs in the flattener, not the renderer.

## The chains

Six independent chains. Getting them separately right is the difference between
"renders" and "renders correctly".

### 1. Position and size

Shape's own `a:xfrm`, then the matching layout placeholder, then the matching
master placeholder, then none. A shape that resolves to no extent is skipped
rather than treated as an error.

### 2. Body properties

The shape's `p:txBody/a:bodyPr`, then the layout placeholder's, then the
master's, then the spec defaults: insets of 0.1 inch horizontally and 0.05 inch
vertically, anchor top, wrap square, horizontal text, no autofit.

### 3. The nine-level list style

Seven sources, merged in this order, later winning per property and per level:

1. `p:defaultTextStyle` in `presentation.xml`
2. The master's `p:txStyles`, selecting `p:titleStyle` for `title` and
   `ctrTitle`, `p:bodyStyle` for `body`, `subTitle` and `obj`, and
   **`p:otherStyle` for everything else including plain text boxes and table
   cells**
3. The master placeholder's `a:lstStyle`
4. The layout placeholder's `a:lstStyle`
5. The shape's own `a:lstStyle`
6. The paragraph's `a:pPr`
7. The run's `a:rPr`

Within each list-style source, `a:defPPr` is applied to all nine levels before
the matching `a:lvlNpPr`. Paragraph fields and nested default character fields
overlay independently. Bullet colour, size, font and choice are independent
properties, while each fill and typeface slot is atomic. Effective values do
not inherit opaque XML or hyperlink actions.

### 4. Style references

`p:style` carries `a:lnRef`, `a:fillRef`, `a:effectRef` and `a:fontRef`, each
with a **1-based** `idx` into the theme's `a:fmtScheme` lists.

**`fillRef@idx` above 1000 means index `idx - 1000` into `a:bgFillStyleLst`.**

Resolution takes the format-scheme entry, substitutes every
`a:schemeClr val="phClr"` with the reference's own colour, then layers the
shape's explicit `a:spPr` fill, line and effects on top.

### 5. Colour

Covered in `05-drawingml-model.md`. The order is colour map, then theme, then
the transform stack in document order. The colour map is per-master and a dark
master inverts `bg1` and `tx1`.

### 6. Fonts

`a:latin@typeface` of `+mn-lt` resolves to the theme's minor font,
`+mj-lt` to the major font. `+mn-ea` and `+mn-cs` select the East Asian and
complex-script faces. The corresponding `+mj-ea` and `+mj-cs` tokens select
the major collection. A caller may supply an ISO 15924 script tag. The first
matching `a:font@script` entry in the selected collection overrides the base
face. A missing or unknown script falls back to the token-specific base face.
Ordinary typefaces and unknown theme-like tokens pass through unchanged.

Script detection belongs to later text shaping. The resolver accepts the
script explicitly because the current run model does not infer one.

## The resolver

```rust
pub struct ResolveCtx<'a> {
    pub theme: &'a CT_OfficeStyleSheet,
    pub color_map: ColorMap,               // clrMap, overridden by clrMapOvr
    pub master: &'a CT_SlideMaster,
    pub layout: &'a CT_SlideLayout,
    pub slide:  &'a CT_Slide,
    pub default_text_style: &'a CT_TextListStyle,
    list_style_cache:
        RefCell<HashMap<Option<PlaceholderKey>, EffectiveListStyle>>,
}

impl ResolveCtx<'_> {
    fn placeholder_chain(&self, sp: &Shape) -> (Option<&Shape>, Option<&Shape>);
    pub fn effective_xfrm(&self, sp: &Shape) -> Option<Xfrm>;
    pub fn effective_body_pr(&self, sp: &Shape) -> BodyPr;
    pub fn effective_list_style(&self, sp: &Shape) -> EffectiveListStyle;
    pub fn effective_text_properties(
        &self,
        sp: &Shape,
        paragraph: Option<&CT_TextParagraphProperties>,
        run: Option<&CT_TextCharacterProperties>,
    ) -> EffectiveTextProperties;
    pub fn effective_fill(&self, sp: &Shape) -> Option<Fill>;
    pub fn effective_line(&self, sp: &Shape) -> Option<Line>;
    pub fn resolve_color(&self, c: &ColorChoice) -> Color;
    pub fn resolve_typeface(&self, tf: &str, script: Option<&str>) -> String;
}
```

Built once per slide, because the layout and master chain is constant across a
slide's shapes. The presentation default, selected master style, master
placeholder and layout placeholder form a prefix memoised by
`Option<PlaceholderKey>`. `effective_list_style` clones that prefix before
applying the shape's own list style. Shapes that share a placeholder key can
therefore reuse inherited formatting without sharing their direct formatting.

## Testing this

Structural unit tests are necessary but not sufficient, because the failure mode
is "subtly wrong" rather than "wrong". The milestone therefore gates on visual
differential tests against decks whose correct appearance can be eyeballed, plus
a table of roughly 40 theme-colour and transform pairs sampled from real
PowerPoint renders and asserted to resolve to exact RGB values.

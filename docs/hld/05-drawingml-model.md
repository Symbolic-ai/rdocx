# 05, DrawingML model

Owner: `oxml-drawing`. The largest body of genuinely new type work in the plan,
because PowerPoint is roughly 90 percent DrawingML and rdocx has almost none.

## What already exists

Of `crates/rdocx-oxml/src/drawing.rs`, 863 lines, almost all is `wp:`
WordprocessingDrawing: inline and anchor wrappers, text wrapping, relative
positioning. None of that has pptx value.

The one transferable piece is `write_graphic_element`, roughly 60 lines, which
emits `a:graphic > a:graphicData > pic:pic` with `a:blip`, `a:stretch`,
`a:xfrm` with `a:off` and `a:ext`, and `a:prstGeom prst="rect"`. That is about
80 percent of a pptx `p:pic`, and it seeds the picture writer.

`theme.rs`, 335 lines, is pure DrawingML but read-only, has no `a:fmtScheme`,
and its tint and shade maths follow Word's convention. It **stays in
`rdocx-oxml`** for this phase. `oxml-drawing` writes its own canonical theme and
rdocx adopts it later through the `From` adapter.

## Modules

| Module | Contents |
|---|---|
| `order.rs` | Schema child-ordering helper |
| `color.rs` | Colour choices, the transform stack, resolution against a theme and colour map |
| `xfrm.rs` | `a:xfrm`, offset, extent, child offset and extent, rotation, flips |
| `geometry.rs` | `a:prstGeom`, `a:custGeom`, guides, adjust values, path lists |
| `fill.rs` | `a:noFill`, `a:solidFill`, `a:gradFill`, `a:pattFill`, `a:blipFill` |
| `line.rs` | `a:ln`, width, dash, cap, join, head and tail ends |
| `effect.rs` | `a:effectLst`, outer shadow modelled, the rest preserved |
| `shape_props.rs` | `a:spPr` and the group equivalent |
| `style_ref.rs` | `a:lnRef`, `a:fillRef`, `a:effectRef`, `a:fontRef` |
| `table.rs` | `a:tbl`, properties, grid widths, rows, cells, merge spans and continuation flags |
| `text/` | `a:txBody`, `a:bodyPr`, `a:lstStyle`, `a:p`, `a:pPr`, `a:r`, `a:rPr`, `a:t`, `a:fld`, `a:br`, bullets |
| `theme.rs` | `CT_OfficeStyleSheet`, read and write, plus `office_default()` |

## Two traps that are silent until PowerPoint refuses the file

**Child ordering.** OOXML schemas are `xsd:sequence`, so element order within a
parent is part of the contract. Emitting `a:ln` before `a:solidFill` inside
`a:spPr` produces a repair prompt, not a warning. Every writer in this crate
emits in schema order, and `OrderedRawChildren` keeps unmodelled siblings in
their original slots rather than appending them at the end.

**`a:t` whitespace.** Leading and trailing whitespace is significant and needs
`xml:space="preserve"`. Cheap to guard, and infuriating to diagnose later.

## Colour, the part everyone gets wrong

Resolution is three stages, in this order:

1. **The colour map.** `a:schemeClr@val` names a semantic slot such as `bg1` or
   `tx1`. That is mapped through `p:clrMap`, which lives on the slide master and
   may be overridden by `p:clrMapOvr` on a layout or slide. `bg1` usually means
   `lt1`, but **a dark master inverts it.** These are per-master mappings, not
   constants.
2. **The theme.** The mapped slot indexes `a:clrScheme`. Both `a:srgbClr@val`
   and `a:sysClr@lastClr` forms occur.
3. **The transform stack**, applied in document order.

```rust
pub enum ColorTransform {
    Tint(Percent1000), Shade(Percent1000), Complement, Inverse, Gray,
    Alpha(Percent1000), AlphaOffset(Percent1000), AlphaModulation(Percent1000),
    Hue(Angle), HueOffset(Angle), HueModulation(Percent1000),
    Saturation(Percent1000), SaturationOffset(Percent1000),
    SaturationModulation(Percent1000), Luminance(Percent1000),
    LuminanceOffset(Percent1000), LuminanceModulation(Percent1000),
    Red(Percent1000), RedOffset(Percent1000), RedModulation(Percent1000),
    Green(Percent1000), GreenOffset(Percent1000), GreenModulation(Percent1000),
    Blue(Percent1000), BlueOffset(Percent1000), BlueModulation(Percent1000),
    Gamma, InverseGamma,
}
```

**`LumMod` and `LumOff` are mandatory for v1.** Office themes apply them to
virtually every accent-derived colour. Getting them wrong makes an entire deck
the wrong shade, which users notice immediately and cannot describe.

Per ECMA-376 Part 1 section 20.1.2.3, `a:tint` and `a:shade` operate in linear
gamma, and `a:lumMod` and `a:lumOff` operate on HSL luminance. Neither is a
plain sRGB interpolation.

The transform vector is evaluated from left to right. Tint, shade, inverse,
and the absolute, offset, and modulation forms of the red, green, and blue
channels operate in linear light. Hue, saturation, and luminance transforms
operate on HSL components. Gray uses PowerPoint's encoded-channel weights of
0.2126 red, 0.7152 green, and 0.0722 blue. Final channel conversion rounds
exact halves upward. A zero-alpha result is transparent black.

`ResolvedColor` is the observable PowerPoint RGBA contract used by the shared
renderer. For partial alpha, PowerPoint's native shape clipboard PNG first
quantises each channel to an 8-bit premultiplied value and then exposes an
8-bit unpremultiplied RGBA pixel. Resolution reproduces that final round trip
so exact RGBA agrees with the pinned PowerPoint oracle. The focused partial
alpha regression prevents this transport-specific boundary rule from leaking
into the individual colour operations.

### Do not touch the Word path

`rdocx_oxml::theme::apply_tint_shade` takes Word's 0-255 byte convention and
does a naive sRGB interpolation. It feeds `rdocx-layout`'s colour resolution and
therefore every rendered PDF and PNG. **Correcting its maths during an
extraction would silently change every theme-derived colour in rdocx's output,
in a way indistinguishable from a migration bug.**

`oxml-drawing` therefore adds *separate*, spec-correct entry points:

```rust
pub fn apply_tint_shade_pct(hex: &str, tint: Option<Percent1000>, shade: Option<Percent1000>) -> String;
pub fn apply_lum_mod_off(hex: &str, lum_mod: Option<Percent1000>, lum_off: Option<Percent1000>) -> String;
mod color { pub fn srgb_to_linear(f64) -> f64; pub fn linear_to_srgb(f64) -> f64; }
```

The Word function keeps its name and behaviour, documented as "legacy, matches
Word's observed behaviour", so nobody unifies them by accident. Correcting it is
a separate, separately-reviewed change with its own reviewed hash delta.

## Geometry

Preset and custom geometry share one evaluator, which is the decisive argument
for implementing presets properly rather than hand-writing the common twenty.
`a:custGeom` needs the guide machinery regardless, so once it exists the 187
presets are a data problem rather than a code problem.

```rust
pub struct Guide { pub name: String, pub op: GuideOp, pub args: Vec<GuideOperand> }
pub enum GuideOp {
    MulDiv, AddSub, AddDiv, IfElse, Abs, At2, Cat2, Cos, Max, Min,
    Mod, Pin, Sat2, Sin, Sqrt, Tan, Val,
}
```

The enum maps the 17 formula tokens `*/`, `+-`, `+/`, `?:`, `abs`, `at2`,
`cat2`, `cos`, `max`, `min`, `mod`, `pin`, `sat2`, `sin`, `sqrt`, `tan`, and
`val` with their DrawingML argument order. The environment owns guide names and
operands. It is seeded with `w`, `h`, `ss = min(w, h)`, the edges and centres,
the standard fractional width and height guides, and the standard circle-angle
constants. Declared adjust values fall back to their formulas and caller
overrides replace those defaults. Ordinary guides then evaluate in declaration
order. All arithmetic uses `f64`, and angles use 60000ths of a degree. Office
interoperability defines `mod x y z` as the Euclidean norm and `sqrt x` as
`sqrt(abs(x))`. Division by zero and every non-finite result are errors.

Input paths model move, line, cubic, close, and arc commands. Evaluation emits
only move, line, cubic, and close commands. **`a:arcTo` is flattened to cubic
Béziers inside the evaluator**, with each sweep split into segments no larger
than 90 degrees. Positive sweeps are clockwise in DrawingML's downward y-axis,
and the ellipse centre is derived so the arc begins at the current pen
position. A bounded segment count prevents untrusted guide values from forcing
an unbounded allocation. Neither renderer backend sees an arc.

Presets also carry an `<a:rect>` text rectangle, which is needed to place text
correctly inside a non-rectangular shape and would otherwise have to be invented.

The generated table and its provenance are covered in `08-rendering-spec.md`.

## Text body

```
a:txBody
  a:bodyPr     insets, anchor, wrap, vertical direction, autofit
  a:lstStyle   nine levels of default paragraph and run properties
  a:p*
    a:pPr      alignment, level, indents, spacing, bullet
    (a:r | a:br | a:fld)*
      a:rPr    size in centipoints, bold, italic, underline, strike, spacing,
               baseline, typefaces, fill, hyperlink
      a:t      the text
```

Structurally parallel to `w:p` / `w:r` / `w:t` but a different vocabulary, so
this is a rewrite rather than a reuse. Two details that differ meaningfully from
Word: font size is centipoints rather than half-points, and line spacing is a
plain percentage rather than 240ths of a line.

Bullets are `a:buChar`, `a:buAutoNum`, `a:buNone`, with `a:buFont`, `a:buSzPct`
or `a:buSzPts`, and `a:buClr`.

## Tables

`CT_Table` models optional table properties, the required column grid, and one
or more rows. Each row retains its stored height and ordered cells. Each cell
owns an optional `CT_TextBody`, defaults `rowSpan` and `gridSpan` to one, and
retains `hMerge` and `vMerge` independently. A merge origin is the
non-continuation cell whose row or grid span is greater than one. Continuation
cells stay explicit, including cells that continue a two-dimensional merge in
both directions.

Table properties expose right-to-left order, first and last row and column
flags, row and column banding, and the optional table style id. Unsupported
fills, borders, margins, effects, extensions, attributes, and child elements
remain raw XML at their schema boundary. Writers use fixed `a:` prefixes and
emit `a:tblPr`, `a:tblGrid`, then `a:tr`, with cell text before cell
properties. A caller that extracts `a:tbl` from a larger XML part passes the
ancestor namespace bindings to `from_xml_with_inherited_namespaces`, which
keeps opaque producer-prefixed content namespace-complete in standalone
output.

## Theme

`CT_OfficeStyleSheet` is read **and written**, because every `.pptx` requires
`ppt/theme/theme1.xml` and a master without one is invalid.

`a:fmtScheme` is modelled, unlike the current rdocx theme reader which omits it
entirely. Shapes reference it through `p:style`, and `07` covers how the indices
resolve. `CT_StyleMatrix.name` is optional because accepted producer themes in
the corpus omit `a:fmtScheme/@name`. Reading and writing preserve that absence
rather than inventing an attribute. A canonical `a:blip` with `r:embed` or
`r:link` declares the fixed relationship namespace locally, so a modelled fill
remains namespace-valid when its parent did not declare `r`.

`office_default()` constructs the standard Office theme. It is the correctness
floor for a template whose master lacks a theme relationship. It is *not* how
`Presentation::new()` works, which uses a bundled binary template for the
reasons given in `06-presentationml-model.md`.

## Preservation

Anything not in the list above is captured verbatim through
`oxml_core::raw_xml::capture_element` and re-emitted in place. That is not a
fallback, it is the design: parse what you render, preserve the rest.

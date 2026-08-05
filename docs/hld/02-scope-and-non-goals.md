# 02, Scope and non-goals

This document decides whether something is in v1. `03-architecture.md` decides
which crate owns it.

## The shape of v1

One release, containing:

1. The `oxml-*` infrastructure extracted from rdocx, with rdocx migrated onto it
   and released as 0.3.0.
2. `rpptx` at feature parity with `python-pptx`, including charts.
3. PDF and PNG rendering of slides.
4. Rust crates, CLIs, WASM modules and Python wheels for both rdocx and rpptx.

There are no partial-feature interim releases. This was a deliberate choice and
its cost is recorded in `00-vision.md`.

## In scope for rpptx v1

### Presentation and slides

| Capability | Notes |
|---|---|
| `Presentation::new / open / from_bytes / save / to_bytes` | `new()` uses a bundled template |
| Slide collection, iteration, indexing, lookup by id | |
| `add_slide(layout)` | Synthesises placeholders, does not deep-copy |
| `remove_slide`, `move_slide`, `duplicate_slide` | Beyond python-pptx |
| Slide size get and set | |
| Slide masters and layouts, layout lookup by name | Read |
| Core, app and custom properties | Shared with rdocx via `oxml-core` |
| Notes slides | Read and write |
| Slide background, follow-master-background | |
| Hidden slides | Skipped when rendering, preserved on save |

### Shapes

| Capability | Notes |
|---|---|
| `add_textbox`, `add_picture`, `add_table`, `add_shape`, `add_connector`, `add_group_shape` | |
| Shape id, name, type, rotation | |
| Position and size, with placeholder inheritance | `Option`-returning plus an `effective_bounds` accessor |
| Fill, line, shadow | Fill and line full, shadow read-only |
| Adjustment values, `a:avLst` | |
| Click actions and hyperlinks | |
| Placeholders by index and by type | |
| Picture crop and intrinsic size | Via `oxml-media` |
| Image deduplication by content hash on insert | |

### Text

| Capability | Notes |
|---|---|
| Text frame, paragraphs, runs, line breaks | |
| Alignment, level, line spacing, space before and after | |
| Font: bold, italic, underline, strike, size, name, colour, caps, language | |
| Bullets: character, auto-number, none, size percent, colour | python-pptx has no bullet API. This is beyond parity |
| Margins, vertical anchor, word wrap, auto-size | |
| Nine-level list style inheritance | |

### Tables

Rows, columns, cells, cell text and text frames, cell fill and margins,
`merge` and `split`, merge-origin and span queries, and the banding flags.

### Charts

`add_chart` with bar, line, pie, scatter, area, doughnut and radar plots.
Series, categories, axes, gridlines, legend, data labels and number formats.
Each chart writes its own part, its relationship, and an embedded workbook.

### Rendering

Preset and custom geometry, solid, gradient, pattern and picture fills, lines
with dash, cap, join and arrowheads, rotation, flips and nested groups, the full
inheritance chain, shape text with anchoring, insets, wrap, bullets and stored
autofit, tables, connectors, hyperlinks, slide-number fields and backgrounds.

### Distribution

`rpptx` and `rdocx` as crates, `rpptx-cli` and `rdocx-cli`, `rpptx-wasm` and a
rewritten `rdocx-wasm`, and `rdocx-py` and `rpptx-py` wheels on PyPI.

## Explicitly not in v1

Each of these is **preserved verbatim on round-trip**. Nothing in this list
causes data loss, only reduced fidelity when rendering.

| Area | v1 behaviour |
|---|---|
| Animations, transitions, `p:timing` | Preserved, irrelevant to static rendering |
| SmartArt, `dgm:` | Preserved. Rendered from its drawing fallback part, else its cached picture, else its bounding box |
| OLE objects, ActiveX | Preserved, rendered as the stored preview image |
| Video and audio | Preserved, rendered as the poster frame |
| 3-D, `a:scene3d` and `a:sp3d` | Preserved, rendered flat |
| Blur on shadows, glow, reflection, soft edges | Shadow renders as a hard offset silhouette. The rest are dropped |
| WordArt text warp, `a:prstTxWarp` | Rendered as plain unwarped text |
| EMF and WMF images | Outline placeholder. Writing an EMF interpreter is out of scope |
| `eaVert` upright stacked CJK | Falls back to rotated vertical text |
| `mongolianVert` upright stacking | Falls back to rotated vertical-270 text |
| `wordArtVert` and `wordArtVertRtl` glyph stacking | Fall back to rotated vertical and vertical-270 text respectively |
| Gradient stop alpha | Stop colour composited, alpha dropped |
| Justified text inside shapes | Treated as left-aligned |
| Table cell text autofit | Not attempted |
| Sections, `p14:sectionLst` | Preserved |
| Comments, ink, `p:contentPart` | Preserved |

Every one of these records a diagnostic, surfaced by `rpptx inspect --json` and
by the render API, so a user can tell approximation from fidelity.

## Non-goals, permanently

**`oxml-sml` is not a spreadsheet library.** It writes one worksheet with the
cells a chart needs. It is not a foundation for an `rxlsx` and should not grow
into one without a separate decision.

**Drop-in `python-docx` and `python-pptx` compatibility is not promised.** Those
libraries' real-world surface is inseparable from lxml, and a large fraction of
production code reaches through `._p`, `._r` and `qn()`. What is promised is
*source compatibility for the documented API*. Touching a private lxml-shaped
attribute raises a clear error naming the equivalent, rather than failing five
frames away.

**Not a PowerPoint clone.** The renderer targets business decks built from
stock or corporate templates. Decks that lean on 3-D, heavy effects or WordArt
will render legibly but not faithfully, and will say so.

## The measurable bar

For a business deck built from a stock Office template, with title and content
slides, bullets, tables, images, theme colours and a gradient title bar, a
150 dpi PNG should be indistinguishable from PowerPoint's own export at a
glance: text baselines within about one point, shape edges exact, colours exact.

Gated in CI against roughly 50 real decks compared with LibreOffice's render,
targeting at least 0.95 SSIM on at least 80 percent of slides, and 100 percent
of slides rendering without a panic or a dropped shape. LibreOffice is the CI
oracle only because PowerPoint is not scriptable on runners, so SSIM regressions
are review-required rather than automatic failures.

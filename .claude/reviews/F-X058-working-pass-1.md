# F-X058, working, pass 1

**Reviewed**: complete working-tree diff against
`f74420c3b6f553ab2e3e139eb9e4f54074496adf`, 26 tracked files with 2,213
insertions and 55 deletions, plus seven untracked font, licence, notice, and
subset-provenance assets
**Verdict**: 6 defects, 1 smell, 0 nitpicks

## Defects

### D1, DrawingML paragraph direction is parsed but discarded before layout
`crates/rpptx-layout/src/text.rs:154`

`merge_paragraph` copies the inherited paragraph properties through spacing and
alignment but never copies `right_to_left`. The resolved paragraph also has no
direction carrier at `crates/rpptx-layout/src/lib.rs:314`, and the PowerPoint
shaper always requests `LayoutTextDirection::Auto` at
`crates/rpptx-render/src/text.rs:485`. An `a:pPr rtl="1"` paragraph whose base
direction cannot be inferred from a strong character, such as a numeric or
punctuation-leading paragraph, therefore renders as automatic LTR instead of
the explicit RTL paragraph requested by the file. The parser-only regression
at `crates/oxml-drawing/src/text/paragraph.rs:1957` cannot detect the dropped
value.

### D2, rich shaping does not segment at font-coverage boundaries
`crates/oxml-layout/src/font.rs:852`

The fallback helper explicitly assumes that one run gets one font and chooses
the face covering the most missing characters. Multilingual range construction
splits only when script or bidi level changes at
`crates/oxml-layout/src/font.rs:1349`, then assigns exactly one font to the
whole range at `crates/oxml-layout/src/font.rs:1367`. A same-script span whose
characters are divided across a caller font and a bundled fallback is shaped
with one face and emits `.notdef` glyphs for the other portion. This contradicts
the approved coverage-segmentation contract and the HLD statement at
`docs/hld/08-rendering-spec.md:452`.

### D3, SVG silently drops rich per-glyph offsets for one-to-one glyph text
`crates/rdocx/src/svg.rs:178`

The SVG adapter converts every multilingual run to its horizontal legacy
projection, which discards x offsets, y offsets, and y advances. The legacy SVG
emitter reports an approximation only when scalar and glyph counts differ at
`crates/rdocx/src/svg.rs:273`. Arabic, Hebrew, or mark-positioned text can have
one glyph per scalar while still requiring nonzero offsets, so those runs are
silently painted at different positions from PDF and raster. The new test even
constructs nonzero offsets at `crates/rdocx/src/svg.rs:1127` but asserts neither
their rendering nor an approximation diagnostic.

### D4, the validating rich-segment constructor accepts values that panic the line breaker
`crates/oxml-layout/src/font.rs:121`

The constructor checks vector lengths and only each cluster's nonempty ranges
and glyph end. It accepts an invalid bidi level such as 255, cluster character
ranges beyond `base.text`, out-of-bounds glyph starts, and overlapping or
unordered clusters. `break_multilingual_into_lines` then recreates a Unicode
bidi level and calls `expect` at `crates/oxml-layout/src/line.rs:635`, so a value
successfully returned by this public validating constructor can panic on the
next public layout call. Out-of-range character clusters also violate the
promised logical cluster and source mapping without being rejected.

### D5, public multilingual output can panic PDF rendering on mismatched vectors
`crates/oxml-layout/src/output.rs:229`

Every glyph-positioning vector in `MultilingualGlyphRun` is independently
public and there is no validating constructor or backend preflight for their
relative lengths. The PDF writer indexes all four vectors by glyph index at
`crates/oxml-pdf/src/writer.rs:1521`. A public run with one glyph and an empty
offset or advance vector therefore panics in `render_to_pdf` instead of
returning or safely approximating output. Raster uses checked access at
`crates/oxml-pdf/src/raster.rs:984`, so the same supposedly backend-neutral
value has inconsistent failure behavior across consumers.

### D6, the named multilingual regressions do not prove their acceptance claims
`crates/oxml-layout/src/font.rs:2370`

The Arabic test checks only the script label and nonempty cluster ranges, so it
would pass with isolated, nonjoining glyph forms. The Thai test at
`crates/oxml-layout/src/font.rs:2402` calls a test-only boundary helper and
checks only that offsets are character boundaries, without shaping, line
breaking, or source spans. The bidi test accepts any permutation different from
logical order at `crates/oxml-layout/src/line.rs:1570`, rather than the exact UAX
9 visual sequence. The CJK test at `crates/oxml-layout/src/line.rs:1575` tests
three predicate calls rather than completed lines. These tests can remain green
with broken joining, wrong ICU opportunities, incorrect bidi ordering, or
prohibited punctuation at actual line edges, so the approved regression gate
is not established.

## Smells

### S1, a public forwarding helper exists only to shorten in-module tests
`crates/oxml-layout/src/font.rs:1423`

`shape_multilingual_seed` is a hidden public convenience that constructs a
default `TextSegment` and forwards to `shape_multilingual_text`. Its only callers
are the two tests at `crates/oxml-layout/src/font.rs:2373` and
`crates/oxml-layout/src/font.rs:2393`. It adds an unplanned published entrypoint
despite the approved plan and repository structure rule explicitly rejecting a
forwarding wrapper.

## Nitpicks

None.

## Not found

OOXML namespace handling, unknown attribute and child preservation, schema
child ordering, dependency direction, conditional-hyphen fitting and no-source
behavior, legacy Latin hash isolation, deterministic font ordering, authentic
Noto licence and notice coverage, reproducible CJK subset provenance, package
inventory, archive-size evidence, WASM routing, and the prohibition on new
modules, traits, generics, and feature flags produced no additional findings.

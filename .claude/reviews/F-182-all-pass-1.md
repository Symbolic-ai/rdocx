# F-182, all aspects, pass 1

**Reviewed**: the complete working-tree implementation diff, 7 files with
1,474 additions and 3 deletions, including the 1,155-line untracked SVG
backend
**Verdict**: 7 defects, 0 smells, 0 nitpicks

## Defects

### D1, the golden gate proves only filled rectangles

`crates/rdocx/src/svg.rs:1121`

The story's only pixel gate builds alternating opaque `FilledRect` stripes. It
contains no text, embedded font, image, path, gradient, group transform, clip,
opacity, effect, link, or diagnosed fallback. Removing or breaking almost the
entire SVG backend while leaving the rectangle arm intact therefore leaves the
0.99 SSIM gate green. The one-point perturbation calibrates that synthetic
stripe pattern, not a representative exported page. This does not satisfy the
golden gate's obligation to prove that the SVG page rasterises like the PNG
backend, and it hides the font-oracle mismatch below.

### D2, the resvg oracle does not bind the emitted synthetic font family

`crates/rdocx/src/svg.rs:235`
`crates/rdocx/src/svg.rs:1102`

SVG text requests the synthetic family `rdocx-font-<FontId>`, while the oracle
loads raw font bytes into resvg's font database under the font's internal
family names. No resolver or database alias maps the synthetic family to the
corresponding `FontId`. The CSS `@font-face` rule in the generated SVG does not
establish that mapping in resvg's explicit font database. A representative
text golden would therefore fall back, omit text, or use a different face
instead of comparing the exact `LayoutResult` font required by the approved
plan.

### D3, font collection face identity is discarded

`crates/rdocx/src/svg.rs:141`
`crates/rdocx/src/svg.rs:620`
`crates/oxml-layout/src/output.rs:379`

`FontData` identifies one face inside a font collection with `face_index`, but
the SVG emitter embeds the complete byte buffer and chooses its CSS format only
from the first four bytes. It never selects or extracts that face. Two layout
fonts backed by different faces in the same TTC can consequently emit two
rules that resolve to the same default collection face. Normal rendering may
receive TTC system fonts, so this changes glyph shapes and metrics without a
diagnostic and violates the exact-font contract.

### D4, gradient stops do not follow the shared normalization contract

`crates/rdocx/src/svg.rs:530`

The SVG backend emits stops in carrier order and merely clamps each offset.
The shared PDF and raster backends sort offsets, treat a NaN offset as zero,
and make the last stop at a repeated offset win. A descending stop list is
therefore rendered with different color transitions in SVG, repeated offsets
can select a different color, and a NaN produces an invalid `offset="NaN"`
attribute. `Paint` is public and does not require normalized input, so these are
valid backend inputs that break parseability or pixel equivalence.

### D5, the outer-shadow filter clips valid effects

`crates/rdocx/src/svg.rs:428`

Every shadow uses the fixed default object-bounding-box filter region from
-50 percent through 150 percent. A narrow child with an offset or blur larger
than half its bounds has its shadow clipped, and a horizontal or vertical
zero-height geometry can have a degenerate object bounding box. The shared
raster backend expands from the actual alpha bounds and clips only at the page
or parent mask. The SVG output can therefore omit part or all of a supported
`OuterShadow` without a diagnostic.

### D6, multiple outer shadows cascade through earlier shadows

`crates/rdocx/src/svg.rs:400`
`crates/rdocx/src/svg.rs:410`

Consecutive `feDropShadow` primitives omit `in`, so every primitive after the
first consumes the preceding filter result. Because that result already
contains the source graphic and its shadow, a second effect shadows the first
shadow as well as the source. The shared raster backend evaluates each
`OuterShadow` independently from the original group layer and then places the
unchanged content once. A group with two supported effects therefore gains
spurious shadow-of-shadow geometry.

### D7, searchable text can make the SVG invalid XML

`crates/rdocx/src/svg.rs:675`

The text escape helper replaces markup metacharacters but passes XML 1.0
forbidden control characters through unchanged. Native document construction
accepts Rust strings directly, so text containing a value such as U+000B can
reach layout and then produce an SVG string that an XML parser rejects. The
security regression covers `<`, `&`, and `>` but not XML-invalid scalar values.
The backend must reject, diagnose, or deterministically replace forbidden XML
characters while keeping permitted whitespace searchable.

## Smells

None.

## Nitpicks

None.

## Verification evidence

- `cargo test -p rdocx svg_ -- --nocapture` passed all six focused unit tests,
  the focused integration test, and the focused facade regression.
- The passing golden used only the stripe fixture described in D1.

## Not found

No additional defect was found in page bounds, normal and deterministic cache
selection, out-of-range handling, recursive child order, affine transform
emission, marked-content recursion, solid geometry, PNG and JPEG embedding,
safe-link scheme filtering, attribute escaping, ordered diagnostics, source
document preservation, public result shape, runtime and development dependency
placement, WASM surface containment, or the approved private-module structure.
The implementation did not touch HLD files or sprint ledgers before the
integrator-managed completion step. No trait, generic, feature, crate, or test
binary was added.

# F-225, correctness, pass 2

**Reviewed**: current working tree implementation across 15 files, 4,157 inserted lines and 7 deleted lines from `597a27c`
**Verdict**: 10 defects, 0 smells, 0 nitpicks

## Defects

### D1, text state and affine geometry are still not projected faithfully

`crates/rpptx/src/pdf.rs:1088`

The shaped advances are adjusted only by the ratio of the two axis lengths.
They do not receive `horizontal_scale`, character spacing, or word spacing,
even though those values contribute to the later text-matrix advance at lines
1091 to 1097. The preserved projection then emits a bare `GlyphRun` at line
1101 with no rotation or affine container. The editable projection retains
only one rotation angle and an axis-aligned box at lines 1115 to 1131. Rotated
text therefore stays horizontal in preserved mode, and shear or reflection is
lost in editable mode. Spacing and horizontal scaling can also disagree
between the visible glyphs and the origin of the next text show in both modes.
The remediation for pass 1 D2 is incomplete.

### D2, editable images still discard shear and reflection

`crates/rpptx/src/pdf.rs:1203`

The preserved layout now retains the four-corner affine transform, but the
editable representation reduces those corners to two axis lengths, a centre,
and the angle of the first axis at lines 1203 to 1217. It later inserts an
ordinary rectangular picture with only that rotation at lines 1577 to 1600.
A sheared unit square becomes a rotated rectangle, and a reflected second axis
loses its flip. The affine image bounds test checks the calculated bounds, not
the editable slide geometry or its render. The remediation for pass 1 D3 is
therefore only complete for preserved mode.

### D3, stroke metrics do not follow the CTM and valid zero values are rejected

`crates/rpptx/src/pdf.rs:660`

Path coordinates are transformed through the CTM before storage, but the
stroke keeps the untransformed user-space line width and dash lengths at lines
660 to 670. A uniform scale therefore enlarges the path without enlarging its
stroke or dash pattern. A nonuniform transform needs either faithful outline
handling or an explicit unsupported diagnostic, not a silent scalar stroke.
In addition, line width is required to be positive at line 688 and every dash
element is required to be positive at `crates/rpptx/src/pdf.rs:2217`. PDF
permits a zero hairline width and nonnegative dash elements when the entire
array is not zero. The pass 1 D5 state fields are now serialized, but their
effective geometry and valid input domain remain wrong.

### D4, repeated font-map decompression is outside the aggregate budget

`crates/rpptx/src/pdf.rs:994`

Embedded font discovery decompresses and charges each `ToUnicode` stream once
at lines 1914 to 1934, then discards the decoded map. Every later `Tj` or string
item in `TJ` calls `get_font_encoding_with_limit` again at lines 994 to 1004.
Those repeated decodes receive the current remaining allowance but are never
charged to `self.decompressed`. A single map can consequently consume the
allowed decompression work again for every text-show operation. The aggregate
limit still does not bound decompression work, so pass 1 D6 remains open.

### D5, semantic strictness accepts malformed operator state and extra operands

`crates/rpptx/src/pdf.rs:581`

`l`, `c`, `y`, and `h` append commands without requiring a current point or
subpath, while only `v` performs that state check. Streams such as `10 10 l S`
therefore reach layout and custom-geometry construction instead of failing as
malformed state. Separately, `numbers` accepts any operand count greater than
the declared arity and silently takes the first values at
`crates/rpptx/src/pdf.rs:2024`. Many zero-operand and single-operand operators
also ignore extras through their direct match arms and accessors. Strict
syntax decoding does not supply the semantic validation required by the
approved malformed-state boundary.

### D6, finite operands can produce unchecked non-finite geometry

`crates/rpptx/src/pdf.rs:106`

Individual PDF numbers are checked for finiteness, but matrix multiplication
and point transformation at lines 106 to 121 use unchecked floating-point
arithmetic. Repeated finite `cm` operands can overflow a CTM component to
infinity. Rectangle endpoint addition is similarly unchecked at lines 636 to
645. Preserved mode can place those derived values into `LayoutResult` and the
rasterizer before any point-to-EMU check. The design requires non-finite
geometry rejection and checked arithmetic, including values derived from
otherwise finite operands.

### D7, Identity encodings can fall back without the required diagnostic

`crates/rpptx/src/pdf.rs:1866`

`Identity-H` and `Identity-V` are classified as supported solely from the
encoding name at lines 1866 to 1878. A font using either encoding without a
valid `ToUnicode` map cannot provide the required Unicode mapping. The later
encoding helper is lenient for non-limit errors, so this case can fall back to
a different decoding while `show_text` takes its supported branch and emits no
encoding diagnostic at lines 982 to 1011. The pass 1 D9 remediation covers
unknown names but not invalid or missing maps for encodings that require one.

### D8, unsupported rendering state is applied as visible default content

`crates/rpptx/src/pdf.rs:828`

A nonzero `Tr` value records a diagnostic but no state that suppresses or
changes dependent text. The following `Tj` is always projected as ordinary
filled text at lines 839 to 842, so PDF text rendering mode 3, which is
invisible, becomes visible slide content. Unsupported colour-space and
extended-graphics-state operators follow the same diagnose-and-continue path
at lines 907 to 916, after which later paint operators use stale supported
state. This changes the affected content rather than isolating the unsupported
state as required by the approved subset boundary.

### D9, the approved editable round-trip contract remains under-asserted

`crates/rpptx/src/pdf.rs:3340`

The approved test named
`editable_pdf_text_images_paths_and_links_survive_save_and_reopen` is still
absent. The similarly named unit test does not save or reopen and checks only
text, shape count, two XML markers, and a relationship target at lines 3340 to
3362. The integration test saves and reopens, but it checks only decoded image
dimensions rather than exact projected image bytes, checks path feature
markers rather than exact commands, and does not associate the asserted link
rectangle with the hyperlink relationship at
`crates/rpptx/tests/integration.rs:297`. Image-byte, path-command, and link
overlay ownership regressions can therefore remain green. Pass 1 D10 is not
fully remediated.

### D10, the Poppler gate neither pins `pdftoppm` nor exercises the full source subset

`crates/rpptx/tests/integration.rs:204`

The oracle helper starts from the complete source but replaces its page
content at lines 208 to 239 with only a solid rectangle and one text glyph.
The embedded image is no longer drawn, and the declared cap, join, dash, and
stroke projection is absent from the oracle image. The differential can thus
pass while image placement or stroke fidelity regresses. The test verifies
only `pdfinfo` version at lines 353 to 360, then invokes `pdftoppm` at line 376
without checking that binary's required 26.01.0 identity. This does not meet
the approved oracle pinning and source-built image boundary, so pass 1 D11
remains incomplete despite the added editable SSIM and real perturbations.

## Smells

None.

## Nitpicks

None.

## Pass 1 disposition

- D1 is closed by corrected CTM concatenation at `crates/rpptx/src/pdf.rs:566`.
- D2 remains as pass 2 D1.
- D3 remains for editable mode as pass 2 D2.
- D4 is closed by explicit image semantic checks at `crates/rpptx/src/pdf.rs:2328`.
- D5 remains as pass 2 D3.
- D6 remains as pass 2 D4.
- D7 is closed by the per-page pixel accumulator at `crates/rpptx/src/pdf.rs:1177`.
- D8 is closed by recursive active-content scanning at `crates/rpptx/src/pdf.rs:1651` and fatal non-URI link rejection at `crates/rpptx/src/pdf.rs:1367`.
- D9 remains as pass 2 D7.
- D10 remains as pass 2 D9.
- D11 remains as pass 2 D10.

## Not found

No additional findings in input, page, object, operation, shape, diagnostic,
or aggregate image-pixel bounds, graphics-state stack balance, page-box and
rotation normalization, safe URI filtering, link relationship ownership,
OOXML child order, public feature gating, native API shape, transactional
candidate publication, panic paths, dependency direction, HLD file scope, or
structural rules. The implementation changes exactly the nine HLD files named
by the approved plan. It adds no unapproved trait, generic parameter, wrapper,
builder, feature flag, crate, module, file, or integration test binary.

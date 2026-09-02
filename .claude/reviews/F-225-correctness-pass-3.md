# F-225, correctness, pass 3

**Reviewed**: current working tree implementation across 16 files, 5,112 inserted lines and 21 deleted lines from `597a27c`
**Verdict**: 10 defects, 0 smells, 0 nitpicks

## Defects

### D1, the approved raw SSIM gate was replaced with a calibrated global blur

`crates/rpptx/tests/integration.rs:282`

The approved plan at `597a27c` required full-image luminance SSIM of at least
0.995. The current diff changes that contract to a 9 by 9 blur at
`.claude/plans/F-225-design.md:172` after the unblurred result measured only
about 0.966 at `.claude/scratch/F-225-progress.md:70`. The helper calls itself
edge-aware, but `box_blur` applies the same radius-four window to every pixel at
`crates/rpptx/tests/integration.rs:304`. Its comment says radius four was chosen
because radius three scored 0.994292 at
`crates/rpptx/tests/integration.rs:283`. Raw SSIM is now only printed at
`crates/rpptx/tests/integration.rs:508`. This is threshold calibration against
the implementation under test, not evidence that only rasterizer
antialiasing is excluded. It permits a result that fails the approved 0.995
gate by roughly 0.029 to pass after globally suppressing differences.

### D2, the one-pixel sensitivity test is rejected before SSIM is consulted

`crates/rpptx/tests/integration.rs:603`

The image is deliberately moved one pixel, then the final predicate receives
`f225_first_image_x_points(&image_shift) == 680.0` as its geometry fact at line
604. That fact is false by construction, so the conjunction in
`crates/rpptx/tests/integration.rs:24` rejects the mutation regardless of the
blurred SSIM value. The separately supplied point error is only 0.48 points and
therefore remains inside its one-point allowance. The test does not prove that
the new 9 by 9 metric notices a one-pixel image shift, despite the changed plan
claim at `.claude/plans/F-225-design.md:176`.

### D3, a charged ToUnicode map is reparsed under the remaining budget and a limit failure is downgraded

`crates/rpptx/src/pdf.rs:2223`

`embedded_fonts` decompresses and charges each map, then stores its decoded
bytes back into the stream at `crates/rpptx/src/pdf.rs:2348`. `parse_page`
nevertheless passes only the remaining aggregate allowance to
`get_font_encoding_with_limit` at `crates/rpptx/src/pdf.rs:459`. If the already
charged map is longer than that remainder, reparsing the now uncompressed
stream reports a memory-limit error. Lines 2243 to 2245 convert every such
error into an unsupported encoding instead of failing closed, so valid mapped
text is silently decoded through the fallback path. The new regression leaves
only six bytes beyond the charged content and map at
`crates/rpptx/src/pdf.rs:3827`, but it asserts only successful import and never
asserts mapped text. It therefore accepts the fallback instead of proving one
cached mapping parse.

### D4, editable mode still drops transforms that the approved subset supports

`crates/rpptx/src/pdf.rs:1360`

The contract includes affine transforms and basic text state at
`.claude/plans/F-225-design.md:106`. Editable text is omitted whenever the two
axis lengths differ, so a valid `Tz` horizontal scale alone drops otherwise
supported text. Sheared text is dropped by the same branch. Editable images
are likewise omitted whenever their axes are not orthogonal at
`crates/rpptx/src/pdf.rs:1581`. Preserved mode now retains the full affine
groups, but omission is not an exact editable projection and the plan never
excludes shear or nonuniform affine transforms from the supported subset.

### D5, omitted affine elements bypass the public shape limit

`crates/rpptx/src/pdf.rs:1702`

`ensure_shape_room` counts only `page.editable` and links. A sheared or
nonuniform text show first appends a preserved layout group at
`crates/rpptx/src/pdf.rs:1344`, then omits the editable element at line 1360.
Sheared images follow the same sequence at `crates/rpptx/src/pdf.rs:1567` and
line 1583. Repeating either operation can therefore grow `page.layout` while
the counter remains unchanged. The final check also counts only editable
elements and links at `crates/rpptx/src/pdf.rs:476`. A caller can set
`max_shapes_per_page` to one and still make the preserved candidate retain many
elements, so the declared public security bound does not fail closed.

### D6, unsupported text clipping state leaks into later graphics

`crates/rpptx/src/pdf.rs:926`

Text rendering modes 4 through 7 add glyphs to the clipping path. The importer
records a diagnostic and hides that text, but it does not taint
`graphics_supported` or otherwise isolate the resulting clip. Later paths and
images are consequently projected without the clip and become visible outside
the source clipping region. The new isolation regression exercises only mode
3 at `crates/rpptx/src/pdf.rs:3844`, which has no clipping side effect. This
leaves the plan requirement that unsupported state cannot leak into later
operators unproved and violated.

### D7, text-map collection is quadratic at the allowed operation limit

`crates/rpptx/src/pdf.rs:2285`

Every distinct text string is checked against all earlier strings with
`values.iter().any`, and the same linear deduplication is repeated for every
item in `TJ` at `crates/rpptx/src/pdf.rs:2313`. The default permits one million
operations at `crates/rpptx/src/pdf.rs:54`. A bounded content stream containing
many unique short `Tj` strings therefore performs quadratic byte comparisons
before interpretation. The input and operation caps do not make that work
practical, so adversarial input can consume excessive CPU inside the declared
bounded parser.

### D8, text advancement ignores the PDF font width table

`crates/rpptx/src/pdf.rs:1292`

The importer advances the PDF text matrix using `shaped.width` from the
embedded or substitute TrueType font at lines 1311 to 1327. `PageFont` retains
no `/FirstChar` or `/Widths` data at `crates/rpptx/src/pdf.rs:206`. PDF text
advancement is defined by the source font dictionary widths, which may differ
from the decoded Unicode font's shaping metrics. A following `Tj`, `TJ`, or
relative text move is therefore placed from the wrong origin. The Poppler
fixture explicitly assigns every character a width of 500 at
`crates/rpptx/tests/integration.rs:100`, but each fixture text object contains a
single glyph with its own absolute matrix, so the differential cannot expose
the accumulated placement error.

### D9, dash phase normalization creates an invalid odd pattern

`crates/rpptx/src/pdf.rs:2841`

For the valid dash state `[0 2] 2 d`, the modulo phase becomes zero. The loop
still skips the leading zero, chooses the gap entry, and emits `[0, 2, 0]`.
That odd-length result is not the phase-equivalent `[0, 2]` cycle. Preserved
rendering can reject the malformed dash and fall back to a solid stroke, while
editable lowering duplicates the odd sequence into a different six-element
cycle. The new regression covers `[0 2] 0 d` only at
`crates/rpptx/src/pdf.rs:3755`, so it misses the boundary where the phase equals
the cycle length.

### D10, malformed image filter objects are silently normalized

`crates/rpptx/src/pdf.rs:2901`

`filter_names` discards every non-name array member with `filter_map`, and a
wrong-type scalar `/Filter` becomes an empty filter list. `show_xobject` then
treats that empty result or a surviving `FlateDecode` name as supported at
`crates/rpptx/src/pdf.rs:1523`. Inputs such as `/Filter 7` and
`/Filter [/FlateDecode 7]` therefore reach image decoding as if their malformed
filter declarations were valid. The approved contract requires malformed
state rejection, not lossy repair of syntax that changes the image decoding
pipeline.

## Smells

None. Count: 0.

## Nitpicks

None. Count: 0.

## Pass 2 disposition

- D1 remains for editable affine text as pass 3 D4. Preserved text now carries
  its full transform at `crates/rpptx/src/pdf.rs:1344`.
- D2 remains for sheared editable images as pass 3 D4. Preserved images now
  carry their full transform at `crates/rpptx/src/pdf.rs:1567`.
- D3 is substantially remediated by finite uniform stroke scaling and explicit
  omission of unsupported outlines at `crates/rpptx/src/pdf.rs:1085`. Dash
  normalization still has the boundary defect in pass 3 D9.
- D4 no longer decompresses a map for every text show, but the budget and
  fallback failure remains as pass 3 D3.
- D5 is closed by exact operator arity and current-path checks at
  `crates/rpptx/src/pdf.rs:566` and `crates/rpptx/src/pdf.rs:604`.
- D6 is closed by derived-value finiteness checks at
  `crates/rpptx/src/pdf.rs:2528`.
- D7 is closed for missing Identity maps by the explicit requirement at
  `crates/rpptx/src/pdf.rs:2200`. The reparsing failure is pass 3 D3.
- D8 is closed for invisible mode 3 and ordinary unsupported graphics state,
  but text clipping still leaks as pass 3 D6.
- D9 is closed by the exact named save and reopen assertions beginning at
  `crates/rpptx/src/pdf.rs:4117`.
- D10 is closed for source-fixture breadth and both binary pins at
  `crates/rpptx/tests/integration.rs:121` and
  `crates/rpptx/tests/integration.rs:443`. The replacement oracle metric and
  its sensitivity proof are new pass 3 D1 and D2.

## Not found

No additional findings in page-box and rotation normalization, path command
state, finite derived geometry, graphics-state stack balance, URI validation,
link relationship ownership, OOXML child order, relationship publication,
public feature gating, native API shape, transactional candidate publication,
panic paths reachable from imported bytes, dependency direction, HLD file
scope, or structural rules. The implementation changes exactly the nine HLD
files named by the approved plan and adds no unapproved trait, generic
parameter, wrapper, builder, feature flag, crate, module, or integration test
binary.

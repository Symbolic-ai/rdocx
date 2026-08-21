# 14, Development backlog

Solo-developer build plan. Ordered by dependency, biased toward small,
incrementally-testable slices so something verifiable lands every few days.

## How to read this

- **Milestones (M1, M2, ...)**, each ends with a concrete, testable gate. Pause
  at any milestone boundary and the workspace is coherent.
- **Stories (F-001, F-002, ...)**, each sized for a solo dev. `S = 1d`,
  `M = 2-3d`, `L = 4-5d`, `XL = split me`.
- **Depends on**, hard dependencies. If unstated, the story can start as soon as
  its milestone begins.
- **Test gate**, the smallest test that proves the story works. Every story has
  exactly one. Nothing merges without it.
- `F-X###` marks cross-cutting work belonging to no milestone.

## Velocity assumption

### v1, as planned and as delivered

The v1 backlog was **162 stories**, roughly **408 developer-days**, forecast at
17 to 18 months solo. It delivered as **182 stories** across 43 sprints, the
extra 20 being cross-cutting `F-X###` work that no milestone predicted: an
external contribution, four releases, two dependency events and the defect
follow-ups each of those filed.

The forecast held at the story level and failed at the sprint level. Sprints
came in far under their estimates whenever a story arrived with its cause
already written up by the sprint that filed it, and the escalation record
carries the variance for each.

### Post-v1, M14 through M20

**56 stories, roughly 190 developer-days**, or about 38 weeks solo. By
milestone, in days: M14 28, M15 12, M16 31, M17 23, M18 26, M19 44, M20 27.

M19 is 44 of those 190 and supersedes a recorded permanent non-goal, so it is
the one milestone that is a business decision rather than a scheduling one. The
roadmap is ordered so that everything before it stands alone: stopping after M18
leaves a coherent product, and so does stopping after M15.

Three ways to compress, all available without reworking the order:

- **M15 first.** Twelve days, and it is the only milestone whose engine already
  exists and already sits on the format-neutral side of the crate graph.
- **Drop M18.** Format breadth is the least coupled block. Nothing else depends
  on it.
- **A second developer.** M14, M17 and M19 barely touch each other. M16 depends
  on M14 and M20 depends on nothing.

---

## Milestone 1, Preparation and safety net (about 2 weeks)

**Goal**: rdocx behaves identically to today, but every future change is
measurable. Nothing has moved yet.

**Why first**: the extraction changes unit conversion and text-shaping inputs,
both of which alter output silently. Without a byte-level baseline, every later
step is unverifiable.

**End-of-milestone gate**: `cargo test --workspace` green, the hash harness
records a baseline that reproduces on a second machine, and `v0.4.1` is tagged.

### F-001, Deterministic font mode (M)
Add `FontManager::new_deterministic()` using bundled fonts only, bypassing
`load_system_fonts()` at `crates/rdocx-layout/src/font.rs:93`.
**Test gate**: rendering the same document twice with system fonts installed and
absent produces identical PNG bytes.

### F-002, rust-toolchain.toml (S)
Pin 1.97.1 with `rustfmt`, `clippy` and the `wasm32-unknown-unknown` target.
**Depends on**: none.
**Test gate**: `rustup show` reports the pinned channel in a clean clone, and
the MSRV job still pins 1.93 separately.

### F-003, Output-stability hash harness (L)
Digest each sample's `document.xml`, `styles.xml`, `numbering.xml` and page-one
PNG at 150 dpi in deterministic font mode. Store the baseline. Provide a
`--update` mode that requires an explicit reason string.
**Depends on**: F-001.
**Test gate**: the harness passes on an unmodified tree and fails when a
whitespace change is injected into a writer.

### F-004, Caladea licence and the false OFL claim (S)
Add `LICENSE-Caladea` plus NOTICE. Correct `bundled_fonts.rs:12`, which claims
all bundled fonts are SIL OFL when Caladea is Apache-2.0.
**Test gate**: a test asserts a licence file exists for every distinct font
family in `fonts/`.

### F-005, Fix the image counter (S)
`crates/rdocx/src/document.rs:135-138` counts matching parts instead of parsing
the maximum suffix.
**Test gate**: `next_image_name_uses_the_highest_existing_index_not_the_part_count`,
asserting `image1` + `image5` yields `image6`, and `image1,2,4` yields `image5`.

### F-006, Fix the JPEG standalone-marker walk (S)
`crates/rdocx-pdf/src/image.rs:51` treats every marker as length-bearing.
**Test gate**: a JPEG with an `RST` marker before the `SOF` still reports correct
dimensions, and a truncation loop over the file panics nowhere.

### F-007, Resolve core properties through the relationship (S)
Replace the hardcoded `/docProps/core.xml` lookup with
`rel_types::CORE_PROPERTIES`.
**Test gate**: a package storing core properties at a non-standard path round-trips
with its metadata intact.

### F-008, Non-consuming setter twins (M)
Add `set_*` siblings for every consuming builder in `paragraph.rs`, `run.rs`,
`table.rs`, with the builders delegating.
**Test gate**:
`doc.paragraph_mut(0).unwrap().add_run("text").set_bold(true)` compiles and has
the same effect as the builder form.

### F-009, Cache the layout result (M)
Separate `Mutex<Option<Arc<LayoutResult>>>` caches for normal and deterministic
font modes on `Document`, invalidated before public mutation and mutable access,
plus a cloned `layout_page` entry point. Caller-supplied font layouts remain
uncached.
**Test gate**: rendering all pages of a 20-page document performs exactly one
layout, asserted with a counter.

### F-010, Reserve crate names (S)
Publish `0.0.0` placeholders for every `oxml-*` and `rpptx*` name.
**Test gate**: `cargo info` resolves each name.

### F-011, Pin unit truncation behaviour (S)
Tests locking the current `as i64` truncation in every `Length`, `Twips` and
`Emu` constructor, before anyone changes it to rounding.
**Test gate**: the pinning tests, which must fail if truncation becomes rounding.

### F-012, Tag v0.4.1 (S)
A known-good published state immediately before the churn.
**Depends on**: F-003 through F-011.
**Test gate**: the release tag builds and publishes from a clean clone.

---

## Milestone 2, Shared infrastructure extraction (about 2 weeks)

**Goal**: `oxml-core` and `oxml-opc` exist as isolated staged crates, and no
released rdocx dependency or behaviour changes.

**End-of-milestone gate**: hash harness unchanged, and `OpcPackage` opens a real
`.pptx` in a test.

### F-013, Create oxml-core (M)
Copy `units.rs`, `raw_xml.rs`, `xml_text.rs`, the generic half of
`namespace.rs`, `core_properties.rs`, `error.rs`, plus
`crates/rdocx/src/length.rs`. Leave released rdocx consumers unchanged. Make
`xml_text` public. Consolidate the duplicate `local_name` and `get_attr`
helpers in the staged crate.
**Test gate**: the moved tests pass unchanged in their new crate.

### F-014, New unit types (M)
`Centipoints`, `Angle` in 60000ths of a degree, `Percent1000`, `Length::mm`.
**Depends on**: F-013.
**Test gate**: round-trip assertions including `Angle::from_degrees(90.0).0 == 5_400_000`.

### F-015, rdocx-oxml becomes a facade (S)
`rdocx-oxml` re-exports the shared modules, error surface and namespace helpers
from published `oxml-core` 0.1.2. Existing public paths and all internal call
sites remain source compatible. `Cargo.lock` records the one-way dependency.
**Depends on**: F-013, F-X005.
**Test gate**: the crate-local diff changes only `lib.rs`, `namespace.rs` and
`Cargo.toml` plus five deletions. The workspace tests and hash harness pass.

### F-016, Length re-export (S)
Delete `crates/rdocx/src/length.rs`, re-export from `oxml-core`.
**Depends on**: F-013, F-X005.
**Test gate**: workspace compiles with no call-site changes.

### F-017, App and custom properties (M)
`AppProperties` as a union struct with `Option` fields, plus `CustomProperties`.
Neither exists today.
**Depends on**: F-013.
**Test gate**: a Word `app.xml` and a PowerPoint `app.xml` each parse, leave the
other format's fields `None`, and round-trip without emitting them.

### F-018, Create oxml-opc (M)
Copy `rdocx-opc` into an isolated staged crate. Replace `new_docx` with
`with_main_part` and `ContentTypes::minimal` without changing `rdocx-opc`.
**Test gate**: the 11 moved tests pass, with the two docx-specific ones rebuilt
on a local fixture helper.

### F-019, PresentationML relationship and content types (S)
Add the package-namespace, extended and custom property, and PresentationML
constants, plus a `content_types` constants module.
**Depends on**: F-018.
**Test gate**: a table test asserting every constant is unique and well-formed.

### F-020, oxml-opc reads a pptx (M)
A pptx-shaped package fixture built in code: package rels to `presentation.xml`,
slide rels to `slide1.xml`, a layout one directory up.
**Depends on**: F-019.
**Test gate**: `main_document_part()` resolves `/ppt/presentation.xml`, and
`resolve_rel_target("/ppt/slides/slide1.xml", "../slideLayouts/slideLayout1.xml")`
resolves correctly.

### F-021, Zip-slip hardening tests (S)
Part names escaping the package root, and absolute-path entries.
**Depends on**: F-018.
**Test gate**: `../../etc/passwd` is clamped to the root, and an absolute entry is
normalised.

### F-022, rdocx-opc deprecation shim (S)
`pub use oxml_opc::*` with a deprecation note, description updated, consumers
flipped to `oxml_opc` directly.
**Depends on**: F-018, F-X005.
**Test gate**: workspace compiles, and `rdocx::Error::Opc` wraps the new type.

---

## Milestone 3, Media (about 1 week)

**Goal**: one isolated staged crate owns everything about an image byte string.

**End-of-milestone gate**: the staged crate passes its tests and the hash
harness remains unchanged. F-027 later proves sniffed content types with a
focused package regression.

### F-023, oxml-media format sniffing (M)
`ImageFormat::sniff`, `from_extension`, `extension`, `content_type`, `resolve`.
**Test gate**: every supported format sniffs from magic bytes, and a `.png` that
is really a JPEG resolves to JPEG.

### F-024, Image probing and DPI (L)
`probe() -> ImageInfo` for PNG, JPEG, GIF, BMP and WebP, including `pHYs` units
0 and 1, JFIF density units 1 and 2, EXIF before the SOF, and progressive JPEG.
**Depends on**: F-023.
**Test gate**: dimension and DPI assertions per format, plus a truncation loop
`for n in 0..data.len()` that panics nowhere.

### F-025, MediaNamer (S)
`scan` parses the maximum existing suffix rather than counting.
**Test gate**: the naming assertions from F-005, now in the shared crate.

### F-026, native_size with explicit DPI (S)
`native_size(default_dpi) -> Option<NativeSize>` returns dependency-free EMU
dimensions. Declared finite positive DPI wins per axis, otherwise the explicit
caller default applies. Conversion truncates toward zero, and an invalid
effective DPI returns `None`. Callers use 72 for python-docx parity against
Word's 96.
**Depends on**: F-024.
**Test gate**: a 96 dpi PNG probed at `default_dpi = 72` yields the expected EMU.

### F-027, rdocx adopts oxml-media (M)
`rdocx::Document` uses `MediaNamer` for scanned collision-free allocation and
shared byte-first format resolution for package metadata, HTML, and layout
inputs. The facade has no local image numbering, extension, or MIME helper.
**Depends on**: F-023, F-025, F-X005.
**Test gate**: a mislabelled image is stored with its sniffed extension and
content type, naming remains collision-safe, and the hash harness is unchanged.

### F-028, add_picture_auto (S)
`Document::add_picture_auto` probes and sizes image bytes at a 72 DPI caller
default before mutation, converts the shared EMU dimensions with `Length::emu`,
and delegates successful insertion to the existing `add_picture` path. This is
an additive API, so the explicit-size signature and its existing callers stay
unchanged. Unavailable dimensions return a typed error carrying the filename
without adding a part, relationship, drawing, or paragraph.
**Depends on**: F-026, F-027.
**Test gate**: a picture added with no explicit size has exact 72 DPI EMU
dimensions before and after round-trip, while unavailable dimensions fail
atomically.

---

## Milestone 4, Layout primitives (about 2 weeks)

**Goal**: the format-neutral layout types live in `oxml-layout` and can express
a rotated, clipped, gradient-filled shape.

**End-of-milestone gate**: hash harness unchanged. This is the milestone where
that matters most.

### F-029, Create oxml-layout (M)
Copy `output.rs`, `font.rs`, `bundled_fonts.rs` and `fonts/`, `error.rs` into an
isolated staged crate. Move `FontFile` within that staged implementation and
leave `rdocx-layout` unchanged.
**Test gate**: the copied tests pass in `oxml-layout`, and the existing
`Document::load_fonts_from_dir` remains unchanged.

### F-030, Decouple line.rs (L)
In staged `oxml-layout`, replace the four docx imports with `TabStop`, `Align`,
`TabAlign`, `Underline` and `LineSpacing`. Add `wrap: bool`. The rdocx-side
converter waits for the deferred consumer cutover.
**Depends on**: F-029.
**Test gate**: `line.rs`'s 11 tests rewritten on the new types pass, and the hash
harness is unchanged.

### F-031, Transform (M)
The 2x3 affine, `rotate_about`, `then`, `apply`, `is_identity`,
`transform_rect_bbox`.
**Depends on**: F-029.
**Test gate**: composition order matches the PDF `cm` operator, verified against
a hand-computed matrix.

### F-032, Path and PathCommand (M)
Four command variants, fill rule, `bounds()` documented as conservative
control-point bounds, plus `rect`, `round_rect` and `ellipse` constructors.
**Depends on**: F-029.
**Test gate**: an ellipse path's bounds contain the ellipse and lie within its
control hull.

### F-033, Paint and Stroke (M)
Solid, linear, radial and tile paints. Stroke width, cap, join and dash.
**Depends on**: F-032, F-036.
**Test gate**: a single-stop gradient degrades to solid at construction time.

### F-034, Path and Group arms (M)
Add both `PositionedElement` variants, `PageFrame::background`,
`LayoutResult::diagnostics`, and `#[non_exhaustive]` on `PositionedElement`,
`Effect`, `PageFrame` and `LayoutResult`, with constructors on the two structs.
**Depends on**: F-031, F-033.
**Test gate**: the staged `oxml-layout` construction sites compile, and the hash
harness is unchanged.

### F-035, The walk helper (S)
`walk(elements, &mut f)` flattening groups and accumulating the transform.
**Depends on**: F-034.
**Test gate**: a three-deep nested group yields every leaf exactly once with the
correct accumulated transform.

### F-036, MediaId (S)
Content-addressed media handles replacing `embed_id` as the renderer's key.
**Depends on**: F-029.
**Test gate**: the same image bytes inserted twice produce one `MediaId`.

---

## Milestone 5, PDF backend (about 2 weeks)

**Goal**: staged `oxml-pdf` renders rotated, clipped, gradient-filled paths and
nested groups. Released rdocx keeps its dependency graph and publication state,
with only the F-039 global CTM source change mirrored into `rdocx-pdf` before
the F-046 cutover.

**End-of-milestone gate**: golden-PNG diffs of the whole sample corpus show zero
pixel changes.

### F-037, Create oxml-pdf (S)
Copy `rdocx-pdf` into an isolated staged `oxml-pdf`, rewire the copy to
`oxml-layout` and `oxml-media`, and delete duplicated header parsers from the
copy. Leave the `rdocx-pdf` dependency cutover and publication until F-046.
F-039 is the only approved mirrored source change before that cutover.
**Depends on**: F-029, F-024.
**Test gate**: the eight moved tests pass.

### F-038, Golden-PNG harness (M)
Render the sample corpus to PNG and compare pixels. Distinct from the hash
harness, and specifically for F-039.
**Depends on**: F-037, F-001.
**Test gate**: passes on an unmodified tree, fails on an injected one-pixel offset.

### F-039, Global CTM flip (L)
Replace the per-element Y flip with one `q 1 0 0 -1 0 H cm`. Text `Tm` becomes
`[1 0 0 -1 x y]`, images `cm [w 0 0 -h x y+h]`.
**Depends on**: F-038.
**Test gate**: the old manifest differs only at the four declared Poppler
26.01.0 antialias pixels in `invoice` and `quote`, then all seven buffers match
the reviewed manifest exactly.

### F-040, Group rendering (M)
`q`, `cm`, optional clip via `W n`, optional `/ExtGState` for opacity, recurse,
`Q`. Effects and raster group support remain owned by later renderer work.
**Depends on**: F-039.
**Test gate**: `q`/`Q` counts balance in the content stream for a three-deep
nesting.

### F-041, Path rendering (M)
`m`, `l`, `c`, `h` then `f`, `f*`, `S`, `B` or `B*`. Stroke state via `w`, `J`,
`j`, `M`, `d`. This story renders solid paint components. Gradient shading
dictionaries remain owned by F-043.
**Depends on**: F-039.
**Test gate**: fill-only emits `f`, stroke-only `S`, both `B`.

### F-042, Rewrite the three collection passes on walk (M)
Font subsetting, XObject registration and link annotations use `walk`.
Depth-first leaf ordinals align resources with recursive emission, and link
rectangles apply the accumulated group transform.
**Depends on**: F-035, F-040.
**Test gate**: three tests, one per pass, each with the target nested inside a
group. This is the R3 regression gate.

### F-043, Gradient shading dictionaries (L)
Type 2 axial and type 3 radial, with a type 3 stitching function over type 2
exponentials, deterministic occurrence names, page-local pattern resources,
and an accumulated `/Matrix` so gradients rotate with their shape. Fill and
stroke pattern operators preserve the supported solid half of mixed paint.
**Depends on**: F-041.
**Test gate**: a rotated linear gradient renders with its axis rotated, asserted
on sampled raster pixels at 72 dpi with Poppler 26.01.0.

### F-044, ExtGState alpha (S)
One document-wide state per distinct normalized alpha, with page-local resource
references. Differing fill and stroke alpha paint the path in two operations.
**Depends on**: F-039.
**Test gate**: a 50 percent alpha fill over white rasterises to the midpoint colour.

### F-045, Rasteriser: groups, paths, gradients, dashes, background (L)
The raster backend recursively composes group transforms, intersects clip
masks, composites group opacity, translates path geometry and paint to
tiny-skia, honours line and path dashes, and paints supported page backgrounds.
**Depends on**: F-040, F-041, F-043.
**Test gate**: a rotated rectangle at 72 dpi has a filled interior pixel and an
empty corner, and a dashed line has gaps.

---

## Milestone 6, shared publication and rdocx cutover (after PowerPoint development)

**Goal**: after PowerPoint development is complete, the shared crates are
published through an approved release plan and rdocx moves onto them.

**End-of-milestone gate**: `cargo publish --dry-run` passes for every crate and
the `.crate` sizes are under the limit.

### F-046, rdocx layout and PDF cutover (M)
Move `rdocx-layout` onto the published `oxml-layout` types through its retained
flow-model facade, add the `rdocx-pdf` deprecation shim over published
`oxml-pdf`, and install the rdocx-side conversion boundary deferred from F-030.
**Depends on**: F-030, F-037, F-047 through F-050, F-X005.
**Test gate**: the workspace compiles, `rdocx::Error::Layout` wraps the new
type, and the hash harness is unchanged.

### F-047, Packaging include and size gate (M)
`include` on `oxml-layout`, drop `--no-verify`, assert `.crate` size in CI.
**Depends on**: F-037.
**Test gate**: `cargo package --list` contains every TTF and the licence files,
and the archive is under 10 MiB.

### F-048, Automate split-family release preparation (M)
Add `cargo-release` preparation for the stable and incubating tag namespaces.
**Test gate**: a dry-run bump of the workspace version updates
`[workspace.package]` and every `[workspace.dependencies]` pin, and touches no
README prose.

### F-049, Extend publish.yml to the extracted workspace (M)
Publish the expanded dependency graph and support both release tag namespaces.
**Depends on**: F-048.
**Test gate**: a dry-run publish of the full workspace succeeds in dependency
order.

### F-050, CI matrix additions (S)
`--no-default-features` for `oxml-layout`, the wasm check job, the prose gate.
**Test gate**: all new jobs pass on a clean tree.

### F-051, CHANGELOG and migration notes (S)
Document the crate moves, the deprecations, and the eventual breaking cutover.
**Depends on**: F-015, F-016, F-022, F-027, F-028, F-046, F-X005.
**Test gate**: every renamed crate is named in the CHANGELOG with its replacement.

---

## Milestone 7, DrawingML (about 4 weeks)

**Goal**: `oxml-drawing` models enough of the `a:` namespace to describe any
shape a business deck contains.

**End-of-milestone gate**: every `a:txBody` and `a:spPr` in the deck corpus
parses, serialises and reparses to a structurally equal value. F-067 executes
this carried gate at S16 entry after it creates the external corpus harness.

### F-052, Create oxml-drawing and namespace constants (S)
**Test gate**: crate compiles, namespace URIs match the spec.

### F-053, OrderedRawChildren (M)
The schema child-order helper that keeps unmodelled siblings in their slots.
**Test gate**: an element with a modelled child between two unmodelled ones
round-trips with all three in the original order.

### F-054, Colour choices (M)
`a:srgbClr`, `a:schemeClr`, `a:sysClr`, `a:prstClr`.
**Test gate**: each form parses and round-trips.

### F-055, The colour transform stack (L)
All transforms, applied in document order, with RGB-to-HSL conversion and
linear-gamma tint and shade per ECMA-376 20.1.2.3.
**Depends on**: F-054, F-014.
**Test gate**: a table of 40 (theme colour, transform) pairs sampled from real
PowerPoint renders resolves to exact RGB.

### F-056, Colour map resolution (M)
`p:clrMap` and `p:clrMapOvr` applied before the theme lookup.
**Depends on**: F-055.
**Test gate**: a dark master inverting `bg1` and `tx1` resolves correctly.

### F-057, a:xfrm (M)
Offset, extent, child offset and extent, rotation, flips.
**Test gate**: a nested group transform composes to the hand-computed matrix.

### F-058, Guide evaluator (L)
The full `GuideOp` set, the seeded environment, adjust values, and `a:arcTo`
flattened to cubics.
**Depends on**: F-014.
**Test gate**: a hand-written `custGeom` with guides produces the expected path
coordinates.

### F-059, a:custGeom (M)
Path lists, adjust value lists, guide lists, the text rectangle.
**Depends on**: F-058.
**Test gate**: a corpus `custGeom` shape round-trips and evaluates to a closed path.

### F-060, Fills (L)
`a:noFill`, `a:solidFill`, `a:gradFill` with linear and path variants,
`a:pattFill`, `a:blipFill` with stretch, tile and `a:srcRect`.
**Depends on**: F-054.
**Test gate**: each fill form round-trips, and a gradient's stops are ordered.

### F-061, Lines (M)
`a:ln` with width, dash presets, cap, join, head and tail ends.
**Depends on**: F-054.
**Test gate**: every `ST_PresetLineDashVal` maps to a dash array.

### F-062, Effects (S)
`a:effectLst` with outer shadow modelled, everything else preserved.
**Test gate**: a shape with a glow round-trips with the glow intact as raw XML.

### F-063, Shape properties and style references (M)
`a:spPr`, and `a:lnRef` / `a:fillRef` / `a:effectRef` / `a:fontRef` including the
`idx > 1000` background-fill rule.
**Depends on**: F-060, F-061.
**Test gate**: `fillRef@idx = 1001` resolves to background fill style 1.

### F-064, DrawingML text model (XL, split at implementation)
`a:txBody`, `a:bodyPr`, `a:lstStyle` with nine levels, `a:p`, `a:pPr`, `a:r`,
`a:rPr`, `a:t`, `a:fld`, `a:br`, and the bullet family.
**Depends on**: F-053.
**Test gate**: every `a:txBody` in the corpus round-trips structurally, and
`a:t` whitespace survives via `xml:space`.

F-064 is split into the four implementation stories below. The parent closes
only after every child closes.

### F-064a, Text body properties and shell (M)
`a:txBody` ownership and `a:bodyPr` insets, anchoring, wrapping, vertical
direction, and autofit forms.
**Depends on**: F-053.
**Test gate**: every `a:bodyPr` autofit form round-trips in schema order with
unmodelled children preserved.

### F-064b, Text paragraphs and runs (L)
`a:p`, `a:pPr`, `a:r`, `a:rPr`, `a:t`, `a:fld`, and `a:br`, including the
DrawingML centipoint and percentage conventions.
**Depends on**: F-064a.
**Test gate**: leading and trailing `a:t` whitespace survives a structural
round-trip through `xml:space="preserve"`.

### F-064c, Text bullets (S)
`a:buChar`, `a:buAutoNum`, `a:buNone`, `a:buFont`, `a:buSzPct`, `a:buSzPts`,
and `a:buClr` on paragraph properties.
**Depends on**: F-064b.
**Test gate**: every modelled bullet form round-trips with colour, font, and
size children in schema order.

### F-064d, Nine-level list styles (M)
`a:lstStyle` with nine level-specific paragraph property slots, completing the
modelled `a:txBody` hierarchy.
**Depends on**: F-064b, F-064c.
**Test gate**: a schema-valid `a:txBody` fixture using all nine list levels
serialises, reparses, and remains structurally equal.

### F-065, Theme read and write (L)
`CT_OfficeStyleSheet` including `a:fmtScheme`, plus `office_default()`.
**Depends on**: F-060, F-061.
**Test gate**: the Office theme generated by PowerPoint 16.104 build
16.104.25121423 round-trips structurally, and `office_default()` produces a
theme that the same pinned build opens without repair.

### F-066, The rdocx Theme adapter (S)
`impl From<&CT_OfficeStyleSheet> for rdocx_oxml::theme::Theme`, leaving the Word
tint and shade path untouched.
**Depends on**: F-065.
**Test gate**: the hash harness is unchanged.

---

## Milestone 8, PresentationML (about 4 weeks)

**Goal**: open any deck in the corpus, model what will be rendered, preserve the
rest verbatim, and save it byte-comparably.

**End-of-milestone gate**: all 50 corpus decks round-trip, and every one opens in
PowerPoint without a repair prompt.

### F-067, Create rpptx-oxml and the corpus harness (M)
Crate skeleton, corpus fetch script, and a raw open-and-save test treating every
part as opaque. Once the corpus is present, execute the carried M7 DrawingML
structural gate before beginning M8 model work.
**Test gate**: the carried M7 DrawingML gate passes, and all 50 decks round-trip
byte-identically with no XML modelling.

### F-068, presentation.xml (M)
`CT_Presentation`, `p:sldSz`, `p:notesSz`, `p:sldIdLst`, `p:sldMasterIdLst`,
`p:defaultTextStyle`.
**Test gate**: every corpus deck's presentation part round-trips.

### F-069, Slide, layout and master parts (L)
`CT_Slide`, `CT_SlideLayout`, `CT_SlideMaster`, `p:cSld`, `p:clrMap`,
`p:clrMapOvr`, `p:txStyles`.
**Depends on**: F-064.
**Test gate**: every corpus slide, layout and master round-trips structurally.

### F-070, The shape tree (L)
`p:spTree`, `p:nvGrpSpPr`, `p:grpSpPr`, and the six-variant child union.
**Depends on**: F-063.
**Test gate**: a deck with nested groups round-trips with tree shape preserved.

### F-071, Placeholders (M)
`p:ph`, `PhType`, `PlaceholderKey` and its matching rule.
**Depends on**: F-070.
**Test gate**: matching by idx, by type, absent type defaulting to body, and both
equivalence classes.

### F-072, Pictures (M)
`p:pic`, `p:blipFill`, `a:srcRect` crop.
**Depends on**: F-060.
**Test gate**: a cropped picture round-trips with its crop rectangle.

### F-073, Graphic frames (M)
`p:graphicFrame` and the `a:graphicData` uri dispatch for tables, charts,
SmartArt and OLE.
**Depends on**: F-070.
**Test gate**: each payload kind is recognised and its unmodelled forms preserved.

### F-074, DrawingML tables (L)
`a:tbl`, `a:tblPr`, `a:tblGrid`, `a:tr`, `a:tc`, merges and banding flags.
**Depends on**: F-064.
**Test gate**: a table with merged cells round-trips with merge origins intact.

### F-075, Connectors (S)
`p:cxnSp` with start and end connections.
**Test gate**: a corpus connector round-trips.

### F-076, mc:AlternateContent (M)
Preserved verbatim, with the fallback branch selected for rendering.
**Depends on**: F-070.
**Test gate**: a deck with `AlternateContent` round-trips byte-identically in that
subtree.

### F-077, Notes slides and notes master (M)
**Depends on**: F-069.
**Test gate**: notes text extracts, and a deck with notes round-trips.

### F-078, relmap rewrite_rel_ids (M)
**Depends on**: F-067.
**Test gate**: a preserved blob containing `r:embed`, `r:link` and `r:dm` has all
three rewritten, and everything else is byte-identical.

### F-079, The rpptx read facade (L)
`Presentation::open`, `from_bytes`, `to_bytes`, `slides`, `text`, plus the
`*Ref` handle types and shape iteration.
**Depends on**: F-069, F-070.
**Test gate**: a `dump_deck` example printing every slide's shapes and text
matches python-pptx's output on the corpus.

### F-080, Modelled round-trip gate (M)
Parse, serialise, reparse, compare structurally, plus part-by-part byte
comparison of the saved package.
**Depends on**: F-079.
**Test gate**: all 50 decks pass, and each opens in PowerPoint without repair.

---

## Milestone 9, Inheritance resolver (about 2 weeks)

**Goal**: a `ResolvedSlide` in which every inherited and theme-derived value is
already concrete.

**End-of-milestone gate**: the contract is frozen and published to the render
track.

### F-081, ResolveCtx skeleton and placeholder chain (M)
**Depends on**: F-071.
**Test gate**: a slide placeholder resolves to its layout and master counterparts.

### F-082, Effective transform and body properties (M)
**Depends on**: F-081, F-057.
**Test gate**: a slide placeholder with no `a:xfrm` inherits the layout's position.

### F-083, The seven-step list style merge (L)
**Depends on**: F-081, F-064.
**Test gate**: a run inheriting from `p:defaultTextStyle` through five
intermediate levels resolves to the expected size and typeface.

### F-084, Format scheme reference resolution (M)
Including `phClr` substitution and the `idx > 1000` rule.
**Depends on**: F-063, F-065.
**Test gate**: a shape with `p:style` resolves to the theme's fill with its own
colour substituted.

### F-085, Typeface resolution (S)
`+mn-lt`, `+mj-lt`, `+mn-ea`, `+mn-cs` and per-script overrides.
**Depends on**: F-065.
**Test gate**: `+mn-lt` resolves to the theme's minor Latin typeface.

### F-086, Draw order and the flattener (L)
Background resolution, the master and layout non-placeholder passes,
`showMasterSp`, the placeholder suppression rules, and latent placeholder
handling.
**Depends on**: F-081.
**Test gate**: a rendered slide contains no "Click to edit Master title style",
and a master logo appears exactly once.

### F-087, ResolvedSlide contract (M)
The full type set, frozen and documented.
**Depends on**: F-082 through F-086.
**Test gate**: a corpus slide resolves with no unresolved theme references
remaining anywhere in the output.

### F-088, Visual differential tests (M)
Decks whose correct appearance can be eyeballed, plus the 40-pair colour table.
**Depends on**: F-087.
**Test gate**: the colour table resolves exactly, and the differential decks are
reviewed once manually.

---

## Milestone 10, Renderer (about 4 weeks)

**Goal**: a deck renders to PDF and PNG at the quality bar in
`02-scope-and-non-goals.md`.

**End-of-milestone gate**: the pinned 50-deck SSIM harness renders every slide
without panic, missing output, dimension mismatch, or a dropped bounded shape,
retains the 0.95 SSIM on 80 percent trend result, and has an accepted native
PowerPoint representative review.

### F-089, Resolve the preset geometry licensing question (S)
Settle Q1 from `13-risks-and-open-questions.md` before writing the generator.
**Test gate**: a written decision recorded in the HLD with its licence basis.

### F-090, Preset table generator (L)
`tools/gen-presets/` emitting a checked-in generated file.
**Depends on**: F-089, F-058.
**Test gate**: the generated table covers every preset name in the corpus, and
the file regenerates byte-identically.

### F-091, Preset evaluation and fallback (M)
**Depends on**: F-090.
**Test gate**: an unknown preset emits its bounding box, keeps its text, and
records a diagnostic.

### F-092, rpptx-render skeleton and RenderInput (M)
`RelScopes`, `SlideBundle`, media resolution to `MediaId`.
**Depends on**: F-087, F-036.
**Test gate**: a slide, layout and master each using `rId2` for different targets
all resolve correctly.

### F-093, Shape geometry, fills and lines (L)
**Depends on**: F-091, F-092.
**Test gate**: a slide of solid, gradient and outlined shapes rasterises with
correct colours at sampled pixels.

### F-094, Rotation, flips and groups (M)
**Depends on**: F-093, F-031.
**Test gate**: a rotated shape's corners land at hand-computed coordinates.

### F-095, Arrowheads (S)
Lowered into filled paths.
**Depends on**: F-093.
**Test gate**: a line with a triangular tail end emits an extra filled path.

### F-096, Pictures with crop and tile (M)
**Depends on**: F-092, F-072.
**Test gate**: a cropped picture renders only its crop region.

### F-097, Backgrounds (S)
**Depends on**: F-086.
**Test gate**: a slide inheriting a master gradient background renders it.

### F-098, Shape text layout (XL, split at implementation)
`bodyPr`, insets, anchoring, wrap, the content box from the preset text
rectangle.
**Depends on**: F-083, F-030.
**Test gate**: text anchored bottom-centre in an inset box lands at the computed
baseline.

F-098 is implemented through the four stories below. F-098a owns content-box
geometry, F-098b owns shaped inline resolution, F-098c owns line stacking, and
F-098d owns horizontal and vertical anchoring. The parent is complete only when
all four child gates pass together in deterministic font mode.

### F-098a, Text content box (M)
Use the preset or custom geometry text rectangle, falling back to the shape
bounds, then apply the resolved body insets without producing negative extents.
**Depends on**: F-083, F-030.
**Test gate**: a preset text rectangle minus four unequal insets produces the
hand-computed content box.

### F-098b, Paragraph inline resolution (L)
Resolve concrete run style into shaped inline items and preserve explicit line
breaks without introducing a second text model.
**Depends on**: F-098a.
**Test gate**: resolved text runs emit glyph items with the expected font size,
colour, style, and explicit break boundaries.

### F-098c, Line stacking (M)
Break paragraphs against the content width, apply paragraph indents and spacing,
and stack their lines in shape-local coordinates.
**Depends on**: F-098b.
**Test gate**: wrapped paragraphs stack at hand-computed baselines while
`wrap="none"` breaks only at explicit line breaks.

### F-098d, Text anchoring (S)
Lower stacked line items to glyph runs, apply horizontal paragraph alignment,
and place the complete block through the resolved vertical anchor.
**Depends on**: F-098c.
**Test gate**: text anchored bottom-centre in an inset box lands at the computed
baseline.

### F-099, Bullets (M)
Character, auto-number with the eight common schemes, none, size, colour, and
the Wingdings codepoint table.
**Depends on**: F-098d.
**Test gate**: a Wingdings `F0B7` bullet renders as a visible bullet glyph, not
a missing-glyph box.

### F-100, Autofit (M)
Stored `normAutofit` applied verbatim, `spAutoFit` trusted, `noAutofit`
overflowing without clipping, and the 2.5 percent ladder for the bare case.
**Depends on**: F-098d.
**Test gate**: a stored `fontScale` of 62500 renders at exactly 62.5 percent.

### F-101, Vertical text (S)
Transposed layout wrapped in a rotated group, with `eaVert` degrading.
**Depends on**: F-098d.
**Test gate**: vertical text renders rotated and records a diagnostic for `eaVert`.

### F-102, Table rendering (L)
**Depends on**: F-074, F-098.
**Test gate**: a banded table with merged cells renders with correct fills and no
duplicated borders.

### F-103, Hyperlinks, fields and diagnostics (M)
Link annotations, slide-number fields reusing the existing field machinery, and
the diagnostic surface.
**Depends on**: F-092.
**Test gate**: a slide-number field renders the correct number and a hyperlink
emits an annotation.

### F-104, SSIM fidelity harness (L)
Corpus renders compared with LibreOffice.
**Depends on**: F-102.
**Test gate**: all pinned corpus slides render without panic, missing output,
dimension mismatch, or a dropped bounded shape. The harness records 0.95 SSIM
on 80 percent as a trend, and the native PowerPoint representative review is
accepted.

---

## Milestone 11, Write API (about 3 weeks)

**Goal**: build and edit decks, and produce files PowerPoint accepts.

**End-of-milestone gate**: a generated 10-slide deck opens clean in PowerPoint,
Keynote, Google Slides and LibreOffice.

### F-105, Bundled default.pptx (M)
The 16:9 template with one master, eleven layouts, a full theme, and zero slides.
**Depends on**: F-065.
**Test gate**: `Presentation::new()` produces a deck PowerPoint opens without
repair.

### F-106, ShapeIdAllocator and MediaStore (M)
Tree-wide id scanning, and content-hash media deduplication.
**Depends on**: F-070, F-036.
**Test gate**: ids are unique across nested groups and `AlternateContent`, and the
same image inserted twice creates one part.

### F-107, add_slide (L)
The nine-step synthesise recipe.
**Depends on**: F-105, F-106.
**Test gate**: a deck with three added slides opens without repair, and every
`p:sldId/@id` is at least 256 and unique.

### F-108, validate() (M)
Every `ValidationIssue` variant, run under `debug_assertions` before save.
**Depends on**: F-107.
**Test gate**: one deliberately corrupted deck per variant is detected, and the
whole corpus validates clean.

### F-109, Shape mutation facade (L)
Position, size, rotation, name, fill, line, adjust values.
**Depends on**: F-079.
**Test gate**: every setter round-trips through save and reload.

### F-110, add_textbox, add_shape, add_connector, add_group_shape (M)
**Depends on**: F-109.
**Test gate**: each produces a shape PowerPoint opens without repair.

### F-111, add_picture (M)
Owning-facade picture insertion uses 72-DPI native sizing, truncating one-axis
aspect inference, package-wide media deduplication, and slide-scoped image
relationships. Every fallible operation completes before package or shape-tree
state is committed.
**Depends on**: F-106, F-026.
**Test gate**: a picture added with no explicit size uses its native dimensions.

### F-112, Text frame mutation (L)
Text frame, paragraphs, runs, font properties, bullets.
**Depends on**: F-109.
**Test gate**: setting text on a placeholder round-trips and renders.

### F-113, Table facade (L)
`add_table`, cells, merge and split, banding, column widths.
**Depends on**: F-074, F-109.
**Test gate**: merge then split restores the original grid.

### F-114, remove_slide, move_slide, duplicate_slide (M)
Including deep copy through `rewrite_rel_ids` and media transfer.
**Depends on**: F-078, F-107.
**Test gate**: a duplicated slide's images resolve to the new slide's own
relationships.

### F-115, Slide and presentation properties (S)
Slide size, background, hidden flag, core properties, `save_as_show`.
**Depends on**: F-017.
**Test gate**: each property round-trips.

### F-116, Cross-viewer acceptance (M)
**Depends on**: F-107 through F-115.
**Test gate**: a generated 10-slide deck exercising every feature opens clean in
all four viewers.

---

## Milestone 12, Charts (about 7 weeks)

**Goal**: create and render charts.

**End-of-milestone gate**: a chart created by rpptx opens in PowerPoint, its
data is editable, and it renders.

### F-117, oxml-sml workbook writer (L)
One worksheet, numeric and string cells, shared strings, defined ranges.
**Test gate**: the produced `.xlsx` opens in Excel and LibreOffice Calc.

### F-118, ChartML core types (L)
`CT_ChartSpace`, `CT_Chart`, `CT_PlotArea`, `CT_Title`, `CT_Legend`.
**Depends on**: F-063.
**Test gate**: a corpus chart part round-trips.

### F-119, Series and data references (L)
`c:ser`, `c:cat`, `c:val`, string and numeric references, and the caches.
**Depends on**: F-118.
**Test gate**: a chart written with a cache and a formula reference has both
consistent with one source of data.

### F-120, Axes (L)
`c:catAx`, `c:valAx`, `c:dateAx`, `c:serAx`, scaling, gridlines, tick marks,
label position, and paired `crossAx` ids.
**Depends on**: F-118.
**Test gate**: axis id pairing is consistent, and a corpus chart's axes round-trip.

### F-121, Bar and line plots (M)
**Depends on**: F-119, F-120.
**Test gate**: each round-trips and renders.

### F-122, Pie, doughnut, area, scatter and radar plots (L)
**Depends on**: F-121.
**Test gate**: each round-trips and renders.

### F-123, Data labels and number formats (M)
**Depends on**: F-119.
**Test gate**: a percentage-formatted label renders with the correct text.

### F-124, add_chart (L)
Writes the chart part, the workbook, both relationship sets, both content-type
overrides, and the graphic frame.
**Depends on**: F-117, F-121.
**Test gate**: a created chart opens in PowerPoint and "Edit Data" shows the
source values.

### F-125, Chart rendering: geometry (L)
Bars, lines, wedges, areas and markers as paths.
**Depends on**: F-121, F-093.
**Test gate**: a bar chart rasterises with bars at computed positions.

### F-126, Chart rendering: axes, gridlines and labels (L)
Nice-number tick selection, axis lines, tick labels, legend.
**Depends on**: F-125, F-098.
**Test gate**: a chart with a 0 to 100 value axis produces the expected tick set.

### F-127, Chart colour resolution (M)
Series colours from `c:spPr` or the theme accent cycle.
**Depends on**: F-125, F-055.
**Test gate**: an unstyled four-series chart uses accent1 through accent4.

### F-128, Preserved chart fallback (S)
Cached image if present, else a labelled placeholder with a diagnostic.
**Depends on**: F-125.
**Test gate**: a 3-D chart renders its cached image and records a diagnostic.

---

## Milestone 13, Bindings and tooling (about 4 weeks)

**Goal**: both libraries ship as crates, CLIs, WASM modules and Python wheels.

**End-of-milestone gate**: wheels install and pass the parity suites on every
target platform.

### F-129, oxml-py-support (M)
Word `ContentPath` and `PathSeg` values, the revision counter, the Rust
`StaleElementError`, and canonical `Length` conversion helpers. Presentation
path variants wait for F-136.
**Test gate**: a stale path raises the named error with both revisions in the
message.

### F-130, rdocx-py core (L)
`PyDocument`, lazy collections, paragraph and run handles.
**Depends on**: F-129, F-008.
**Test gate**: `doc.paragraphs[3]` held across `remove_content(1)` raises
`StaleElementError` rather than reading the wrong paragraph.

### F-131, rdocx-py formatting and tables (L)
Path-only font and paragraph-format sub-handles expose the bounded S33
formatting inventory with tri-state clearing. Lazy table, row, cell and nested
paragraph handles cover table style, alignment and width, plus cell text,
width and vertical alignment. Public facade accessors re-resolve every path.
**Depends on**: F-130, F-132.
**Test gate**: `r.font.bold` returns `None` when unset, not `False`.

### F-132, Python enums, units and exceptions (M)
The bounded `IntEnum` shims for paragraph alignment, table alignment, cell
vertical alignment and underline, pure-Python `Length` and `RGBColor` values,
the package exception hierarchy, and concrete mapping from Rust binding errors.
The types are top-level exports and retain the `rdocx.shared`,
`rdocx.enum.text` and `rdocx.enum.table` compatibility paths.
**Depends on**: F-129, F-130.
**Test gate**: `WD_ALIGN_PARAGRAPH.CENTER == 1` and `Inches(1) == 914400`.

### F-133, rdocx-py rendering with allow_threads (S)
**Depends on**: F-130.
**Test gate**: four concurrent `to_pdf` calls from a thread pool complete faster
than serial execution.

### F-134, Type stubs and py.typed (M)
Both mixed packages ship hand-written native-extension stubs and `py.typed`
markers. Strict installed-wheel smoke programs cover concrete handles,
collections, overloads, iterators, path-like inputs, byte outputs, and optional
values without duplicating inline-typed pure-Python modules. Bounded enums and
Length returns retain their semantic types, and factory-only native handles
remain non-constructible at type-check time.
**Depends on**: F-131, F-136.
**Test gate**: exact `mypy==2.3.0 --strict` and `stubtest` both pass against
freshly installed cp39-abi3 wheels.

### F-135, python-docx parity suite (M)
**Depends on**: F-131.
Pin and assert python-docx 1.2.0. Execute an explicit manifest of all executable
documentation examples inside the completed S33 surface from stable v1.2.0
tagged sources. Sixteen examples change only the import namespace. The exact
Quickstart held-row example uses the minimal public row re-fetch required by
strict global revision before its second cell assignment. Author the approved
structure with both writers, read both outputs with both libraries, and compare
normalized public records rather than package bytes. Preserve relative float
line spacing separately from absolute lengths and compare explicit table style
after save and reopen.
**Test gate**: `documented_s33_examples_run_with_declared_transformations`
passes for the exact seventeen-entry manifest, and the two-way normalized
differential agrees.

### F-136, rpptx-py (L)
An unpublished abi3-py39 mixed-layout binding over `Presentation`, using lazy
path-only slide, shape, text and table handles. The bounded surface includes
pure-Python presentation units and required shape enum values.
**Depends on**: F-129, F-116.
**Test gate**: the seven python-pptx 1.0.2 Getting Started workflows run with
the package namespace changed and minimal public re-fetches after structural
writes. Both readers agree on each writer, and normalized structures from the
two writers agree directly with that exact oracle version.

### F-137, wheels.yml (M)
Build `rdocx` and `rpptx` with maturin as abi3-py39 wheels for
manylinux_2_28 x86_64 and aarch64, musllinux_1_2 x86_64, macOS x86_64 and
arm64, and Windows x86_64. Build one source distribution per package. Every
compatible wheel is installed and tested in a fresh environment. A separate
job collects the exact twelve wheels and two source distributions and receives
PyPI OIDC authority only for the `py-v*` tag namespace. Manual dispatch never
publishes.
**Depends on**: F-134, F-136.
**Test gate**: the local exact-product contract and its negative mutations
pass, both native wheels and source distributions build, and both native wheels
install and pass their compatible package, typing, and stub gates. The first
reviewed hosted dispatch supplies cross-platform execution evidence.

### F-138, PR-time Python job (S)
`maturin develop && pytest`.
**Depends on**: F-137.
**Test gate**: the job fails when a binding test fails.

### F-139, Rewrite rdocx-wasm (L)
Wrap `rdocx::Document` and keep the existing JavaScript method names. The
default-on `system-fonts` feature is forwarded through `rdocx-layout` and
`rdocx`, while `rdocx-wasm` disables it and retains unconditional bundled font
data. An inline Node regression exercises the same package-preserving contract
as the native gate.
**Depends on**: F-029.
**Test gate**: a document with images, headers and numbering round-trips through
`fromBytes` and `toDocxBytes` with every part intact. This is the R-class
regression gate.

### F-140, wasm CI job (S)
**Depends on**: F-139, F-142.
**Test gate**: locked `cargo check --target wasm32-unknown-unknown` and
`wasm-pack test --node` run for both WASM packages on PRs.

### F-141, to_pdf in the browser (M)
**Depends on**: F-139, F-001.
**Test gate**: a wasm-pack node test produces a non-empty PDF with embedded fonts.

### F-142, rpptx-wasm (M)
Wrapping the real facade, in two feature profiles.
**Depends on**: F-116.
**Test gate**: the default profile is under 1 MB gzipped and round-trips a deck.

### F-143, oxml-cli-support (S)
Range parsing, output-path defaulting, the versioned JSON envelope.
**Test gate**: `2,4-6` parses to the expected set, and the envelope carries
`"schema": 1`.

### F-144, rpptx-cli (L)
`inspect`, `text`, `convert`, `diff`, `replace`, `validate`, `render`.
**Depends on**: F-143, F-116, F-104.
**Test gate**: `validate` exits non-zero on a corrupted deck and zero across the
corpus.

### F-145, rpptx-cli thumbnail and outline (M)
**Depends on**: F-144.
**Test gate**: `thumbnail` produces a proportional 320-pixel-wide PNG of slide
one, and `outline` prints each title once followed by the recursive paragraph
tree with stable level indentation.

### F-146, npm publication (S)
`@tensorbee/rdocx-wasm` and `@tensorbee/rpptx-wasm` build as release bundler
packages under exact checksum-pinned wasm-opt 125. Pull-request CI packs and
installs both tarballs locally without registry credentials or publication
authority.
**Depends on**: F-140, F-142.
**Test gate**: `npm pack` produces an installable tarball for each, and both
installed packages retain their exact metadata, WASM, JavaScript glue,
TypeScript declaration, and import.

---

## Milestone 14, Word collaboration layer (about 4 weeks)

**Goal**: the parts of a document that exist because more than one person
touched it. All four are preserved verbatim today and none is addressable.

Commercial libraries treat this as the dividing line. Aspose.Words, Spire.Doc
and GemBox all sell revision and comment APIs, and `python-docx` has offered
neither in a decade of requests. Nothing in the Rust ecosystem has any of it.

**End-of-milestone gate**: a document carrying tracked changes, comments,
content controls and bookmarks round-trips byte-identically in the parts this
milestone does not model, and every one of the four is readable and writable
through the public API.

### F-147, Comment model and part (M)
`word/comments.xml`, `CT_Comment` and `CT_Comments`, plus the
`w:commentRangeStart`, `w:commentRangeEnd` and `w:commentReference` anchors in
the body. Today the part survives because `OpcPackage` writes every part it
holds, which means a comment is never lost and never reachable.
**Depends on**: none.
**Test gate**: round-trip. A document with three comments, one spanning two
paragraphs, reloads with every anchor in the same place and saves byte-identical.

### F-148, Comment API (M)
`Document::comments`, `add_comment` over a run range, `reply_to`, `resolve` and
`remove`. Replies use `w:commentsExtended` and the paragraph-id linkage, which
is what Word itself reads.
**Depends on**: F-147.
**Test gate**: regression. A comment added over a range, replied to and resolved
opens in Word with the thread intact.

### F-149, Revision model (L)
`w:ins`, `w:del`, `w:delText`, `w:moveFrom`, `w:moveTo`, and the property-change
elements `w:rPrChange`, `w:pPrChange`, `w:tblPrChange` and `w:sectPrChange`.
These are captured as raw XML today, listed in the modelled-children exclusions
in `numbering.rs` and `text.rs`.
**Depends on**: none.
**Test gate**: round-trip. Every revision element survives a load and save
unchanged, and each is reported with its author, timestamp and kind.

### F-150, Accept and reject revisions (L)
`accept_all`, `reject_all`, and the same two scoped to an author, a date range
or a single revision id. Rejecting an insertion removes content, rejecting a
deletion restores it, and a property change reverts to the recorded prior value.
**Depends on**: F-149.
**Test gate**: regression. Accepting every revision produces the document Word
produces from the same input, compared as normalised body XML.

### F-151, Revision display in the renderer (M)
Rendering shows insertions underlined, deletions struck through, and a change
bar in the margin, or renders the accepted view. The choice is a render option
and the default is the accepted view, because that is the document a reader
means when they ask for a PDF.
**Depends on**: F-149.
**Test gate**: golden. Both views of one document render, and the accepted view
is pixel-identical to the same document with revisions accepted and removed.

### F-152, Content control model (L)
`w:sdt`, its `w:sdtPr` properties and `w:sdtContent`, at block, row, cell,
paragraph and run level. `table.rs` already unwraps these to find rows and
cells, so the traversal exists and the model does not.
**Depends on**: none.
**Test gate**: round-trip. Controls at all five nesting levels survive, and each
is reported with its tag, alias, id and type.

### F-153, Content control binding (M)
Read and write a control's value by tag or alias, and bind a control set to a
key-value map in one call. Includes the `w:dataBinding` XPath into a custom XML
part, which is how document-assembly products drive Word.
**Depends on**: F-152.
**Test gate**: regression. A control set bound to a map produces the expected
text, and a bound custom XML part updates both the part and the display text.

### F-154, Bookmarks and cross-references (M)
`w:bookmarkStart` and `w:bookmarkEnd`, a bookmark collection, insertion over a
range, and `REF` and `PAGEREF` targets resolved against them.
**Depends on**: none.
**Test gate**: regression. A bookmark inserted over a range is listed, its text
is readable, and a cross-reference to it resolves to the right page after
pagination.

### F-155, Document protection (M)
`w:documentProtection` in settings: read-only, comments-only, tracked-changes-
forced and forms-only, with the hash and salt Word writes. Reading the setting
matters more than enforcing it, because a consumer needs to know the author's
intent.
**Depends on**: none.
**Test gate**: regression. Each protection mode round-trips with its hash
intact, and the mode is reported through the public API.

---

## Milestone 15, Charts beyond PowerPoint (about 2 weeks)

**Goal**: one chart engine, two document families. `oxml-chart` owns the
format-neutral model and renderer. `rpptx-chart` remains an exact deprecated
re-export for existing consumers.

`python-docx` has no chart API at all. The standard workaround is rendering a
chart to PNG and pasting it, which loses every bit of editability. Apache POI
and docx4j both have native Word charts, and so does every commercial library.

**End-of-milestone gate**: a Word document gains a native chart that opens
editable in Word, and renders identically to the same chart in a deck.

### F-156, Extract oxml-chart (L)
Move `rpptx-chart` to `oxml-chart` with no behaviour change. A pure rename and
re-export, with the deprecation shim pattern F-015 and F-022 already
established.
**Depends on**: none.
**Test gate**: regression. The hash harness is byte-identical across the move,
and every existing chart test passes against the new path. This is a file move,
so folding any behaviour change into it is forbidden.

### F-157, Word chart part and embedded workbook (M)
The chart part, its relationship from `document.xml`, and the embedded
`.xlsx` workbook Word requires. `oxml-sml` already writes exactly the one
worksheet a chart needs, which is the whole reason it exists.
**Depends on**: F-156.
**Test gate**: round-trip. A document with a chart part saves with the part, its
relationship, its content type and its embedded workbook, and Word opens it
without repair.

### F-158, Document::add_chart (M)
The Word-side authoring API, matching the shape of `rpptx`'s `add_chart` so a
reader who knows one knows the other.
**Depends on**: F-157.
**Test gate**: regression. A bar, line and pie chart added to a document carry
the series, categories and number formats they were given.

### F-159, Chart rendering in the Word paginator (M)
An anchored or inline chart lays out and renders through the same path as an
image, delegating to the chart renderer for its content.
**Depends on**: F-158.
**Test gate**: golden. A chart in a Word document renders pixel-identical to the
same chart on a slide at the same size.

---

## Milestone 16, Document automation (about 5 weeks)

**Goal**: generate documents from data rather than editing them by hand. This
is the largest commercial category. Aspose sells a LINQ reporting engine,
docxtpl is one of the most-used Python packages in the space, and every
document-assembly product is built on fields, content controls and merges.

`rdocx` already has `replace_text`, `replace_regex`, `replace_all` and
`replace_many_in_chart_xml`, which covers substitution and nothing structural.

**End-of-milestone gate**: a template with loops, conditionals and a repeating
table row produces a correct document from a JSON data model, and every field in
it evaluates to the value Word computes.

### F-160, Field instruction parser (L)
`w:fldSimple` and the `w:fldChar` plus `w:instrText` run sequence, parsed into a
field name, arguments and switches. `text.rs` already extracts `w:instr` for the
simple form.
**Depends on**: none.
**Test gate**: unit. Every field form in the corpus parses, including nested
fields and instructions split across runs, which is how Word actually writes
them.

### F-161, Field evaluation engine (L)
`IF`, `REF`, `PAGEREF`, `SEQ`, `DOCPROPERTY`, `DOCVARIABLE`, `STYLEREF`,
`INCLUDETEXT`, `DATE`, `TIME`, `FILENAME`, `AUTHOR` and `MERGEFIELD`, plus the
formatting switches. `PAGE` and `NUMPAGES` already evaluate during pagination.
**Depends on**: F-160, F-154.
**Test gate**: regression. Each supported field evaluates to the value Word
computes for the same document, checked against a pinned expected set.

### F-162, Field update policy (M)
Update on demand, update on save, and leave alone, with the dirty flag Word
sets. A field whose result is cached must not be silently recomputed, because a
document may legitimately carry a stale result on purpose.
**Depends on**: F-161.
**Test gate**: regression. Each policy produces the expected result cache, and
an unsupported field keeps its cached result rather than blanking.

### F-163, Template syntax (L)
A tag syntax over the existing placeholder machinery, resolving inside runs that
Word has split mid-tag, which is the failure every naive implementation hits.
**Depends on**: none.
**Test gate**: unit. A tag split across five runs with different formatting
resolves, and the surrounding formatting is preserved.

### F-164, Loops and conditionals (L)
Block-level repetition and inclusion over a data model, at paragraph, row and
section granularity.
**Depends on**: F-163.
**Test gate**: regression. A template with a nested loop and a conditional
produces the expected document from a fixture data model.

### F-165, Repeating table rows and lists (M)
The two structures that need their own handling: a row that repeats keeps its
formatting and its merged cells, and a repeated list item keeps its numbering
continuous.
**Depends on**: F-164.
**Test gate**: regression. A three-row template over ten records produces thirty
rows with the banding and numbering intact.

### F-166, Mail merge (M)
A record set driving `MERGEFIELD`, with one document per record or one document
with a section per record.
**Depends on**: F-161, F-164.
**Test gate**: regression. A merge over a fixture record set produces the
expected documents, and an absent field renders empty rather than failing.

### F-167, Document comparison (L)
Compare two documents and express the difference as tracked revisions, scoped to
body text, tables and list structure. Formatting-only differences are recorded
as a diagnostic rather than a revision, which keeps this one story instead of
three.
**Depends on**: F-149.
**Test gate**: regression. Comparing a document with its edited copy produces
revisions that, when accepted, reproduce the edited copy exactly.

### F-168, Watermarks (S)
Text and image watermarks through the header `w:pict` shape Word uses, readable
and writable, and rendered.
**Depends on**: none.
**Test gate**: golden. A watermark renders behind body text on every page.

---

## Milestone 17, Security and compliance (about 3 weeks)

**Goal**: files an enterprise or a public body can accept. Encryption and
signatures are table stakes in commercial libraries and absent from every open
source Office library in Python and Rust. Apache POI is the only open
implementation of OOXML agile encryption worth reading.

Tagged PDF is a legal requirement for public-sector documents in the EU and the
United States, and a LibreOffice-based pipeline cannot produce it well. The PDF
backend here is ours, so it can.

**End-of-milestone gate**: an encrypted document opens with its password, a
signed document verifies, and a rendered PDF passes a PDF/UA structure check.

### F-169, Agile encryption, read (L)
ECMA-376 Part 4 agile encryption: the `EncryptionInfo` stream, key derivation,
and AES decryption of the package. This is the difference between opening a
protected file and telling the user to go and find Word.
**Depends on**: none.
**Test gate**: regression. A password-protected document produced by Word opens
with the right password and fails cleanly with the wrong one.

### F-170, Agile encryption, write (M)
Save with a password, using the same parameters Word writes, so the result opens
in Word rather than only in this library.
**Depends on**: F-169.
**Test gate**: round-trip. A document encrypted here decrypts here, and the
parameters match a Word-encrypted reference byte for byte where the spec fixes
them.

### F-171, Digital signature verification (L)
Read `_xmlsignatures`, verify the signature over the declared part set, and
report which parts a signature actually covers, since a signature over a subset
is the usual attack.
**Depends on**: none.
**Test gate**: regression. A validly signed document verifies, and a document
modified after signing fails with the changed part named.

### F-172, Digital signature creation (M)
Sign a package with a supplied key and certificate.
**Depends on**: F-171.
**Test gate**: round-trip. A document signed here verifies here and in Word.

### F-173, Tagged PDF structure tree (L)
Emit `/StructTreeRoot`, marked content, heading levels, list structure, table
headers and alternate text from the document's own semantics, which the layout
engine already knows because `audit_accessibility` reads them.
**Depends on**: none.
**Test gate**: regression. A rendered PDF carries a structure tree whose heading
and list nesting matches the source document.

### F-174, PDF/A conformance (M)
PDF/A-2b and PDF/A-3b output: embedded fonts already, plus the output intent,
metadata and the prohibited-feature checks.
**Depends on**: F-173.
**Test gate**: regression. A rendered PDF passes a conformance check for the
declared level.

### F-175, Redaction (M)
Remove text and its traces rather than drawing a black box over it, covering the
body, comments, revisions, metadata and the embedded workbook of any chart.
**Depends on**: F-147, F-149.
**Test gate**: regression. Redacted text is absent from every part of the saved
package, checked by scanning the raw zip rather than the model.

---

## Milestone 18, Format breadth (about 5 weeks)

**Goal**: read and write the formats users actually have, rather than the one
format we prefer. Aspose.Words converts between roughly twenty. The gap that
costs real users is inbound: a library that cannot read RTF or HTML cannot be
put in front of a corpus nobody curated.

Rendering is already format-neutral below the facade, so every writer here is a
new front end onto a layout engine that exists.

**End-of-milestone gate**: each format round-trips at its declared fidelity
level, and every lossy conversion records a diagnostic naming what it dropped.

### F-176, RTF reader (L)
The control-word grammar, destinations, code pages and the subset of RTF that
Word itself writes. Scoped to text, formatting, tables, lists and images.
**Depends on**: none.
**Test gate**: differential. An RTF file converted to docx here matches the same
file opened and saved as docx by the pinned oracle, compared structurally.

### F-177, RTF writer (M)
The inverse, at the same scope.
**Depends on**: F-176.
**Test gate**: round-trip. A document written to RTF and read back preserves
text, formatting, tables, lists and images.

### F-178, HTML import (L)
HTML and CSS to a Word document, scoped to the subset a browser copy-paste and a
CMS export actually produce. This is the most requested inbound conversion in
every library's issue tracker.
**Depends on**: none.
**Test gate**: regression. A fixture set of HTML documents produces the expected
paragraph, run, table and list structure, with unsupported CSS recorded as a
diagnostic.

### F-179, ODT reader (L)
OpenDocument Text, which European public-sector procurement frequently mandates
and which no Rust library reads.
**Depends on**: none.
**Test gate**: differential. An ODT converted here matches the pinned
LibreOffice conversion structurally.

### F-180, ODT writer (L)
The inverse.
**Depends on**: F-179.
**Test gate**: round-trip. Text, formatting, tables, lists and images survive.

### F-181, EPUB export (M)
Reflowable EPUB 3 from the document outline, which the heading and outline APIs
already produce.
**Depends on**: none.
**Test gate**: regression. A generated EPUB passes epubcheck and its spine
matches the document outline.

### F-182, SVG page export (M)
A rendered page as SVG, from the same `PageFrame` the PDF and PNG backends
consume. Text stays text, so the output is searchable and scalable.
**Depends on**: none.
**Test gate**: golden. An SVG page rasterises to the same pixels as the PNG
backend at the same dpi, within the recorded tolerance.

### F-183, Image export options (S)
Multi-page TIFF, JPEG quality, transparent PNG backgrounds, and a page range on
every image entry point.
**Depends on**: none.
**Test gate**: regression. Each option produces the declared output and a page
range selects exactly the requested pages.

---

## Milestone 19, Spreadsheets (about 10 weeks)

**Goal**: `rxlsx`, the third family.

**This milestone supersedes a recorded permanent non-goal.**
`docs/hld/02-scope-and-non-goals.md` states that `oxml-sml` is not a spreadsheet
library and must not grow into one without a separate decision. F-184 is that
decision, and nothing else in this milestone may start before it lands.

The economics changed after v1 shipped. OPC, DrawingML, the chart engine, the
layout engine and the PDF backend all exist and are format-neutral. Of the three
Office formats, xlsx carries the highest volume in data engineering, and Rust's
coverage is fragmented: `calamine` reads, `rust_xlsxwriter` writes, neither
renders, and `umya-spreadsheet` does both thinly. Nothing in any language reads,
writes, recalculates and renders on one foundation.

**End-of-milestone gate**: a workbook round-trips, its formulas recalculate to
the values Excel computes, and a sheet renders to PDF.

### F-184, Supersede the spreadsheet non-goal (S)
The decision record. Amend `02-scope-and-non-goals.md`, state what changed and
why, and define the boundary between `oxml-sml` as chart support and `rxlsx` as
a library.
**Depends on**: none.
**Test gate**: regression. The scope document states the superseding decision,
and a test asserts the two statements do not contradict each other.

### F-185, Workbook and worksheet model (L)
Workbook, sheets, rows, columns, cells, cell types, merged ranges and defined
names.
**Depends on**: F-184.
**Test gate**: round-trip. Every element survives a load and save unchanged.

### F-186, Shared strings, styles and number formats (L)
The three tables that make xlsx compact and make naive implementations wrong:
the shared string table, `styles.xml` with its indexed formats, and the built-in
plus custom number format codes.
**Depends on**: F-185.
**Test gate**: round-trip. A workbook with every built-in format and twenty
custom ones preserves each cell's displayed value.

### F-187, Reader (L)
Streaming read of the sheet XML, because a spreadsheet is the one Office format
that is routinely too large to hold in memory as a tree.
**Depends on**: F-186.
**Test gate**: regression. A 100 MB fixture reads within a bounded memory
ceiling, asserted rather than assumed.

### F-188, Writer (L)
Streaming write, with the same ceiling.
**Depends on**: F-187.
**Test gate**: round-trip. A generated workbook opens in Excel without repair.

### F-189, Formula parser (L)
The A1 and R1C1 grammars, operators, ranges, cross-sheet and cross-workbook
references, and the shared-formula compression Excel writes.
**Depends on**: F-185.
**Test gate**: unit. Every formula in the corpus parses and re-serialises
identically.

### F-190, Calculation engine (L)
Dependency graph, evaluation order, cycle detection, and the function set that
covers the overwhelming majority of real sheets: maths, statistics, text,
logical, date and lookup.
**Depends on**: F-189.
**Test gate**: differential. Recalculated values match the values Excel stored
in a pinned corpus, cell for cell, with unsupported functions listed rather than
silently wrong.

### F-191, Charts in spreadsheets (M)
The chart part on a worksheet, reusing `oxml-chart` for the third time.
**Depends on**: F-156, F-185.
**Test gate**: round-trip. A chart on a sheet saves, reopens and renders.

### F-192, Conditional formatting and data validation (M)
Both are widely used and both are commonly dropped by libraries that claim
round-trip fidelity.
**Depends on**: F-186.
**Test gate**: round-trip. Every rule type survives with its ranges and
priorities.

### F-193, Pivot table preservation (M)
Preserve the pivot cache and definition verbatim, and report the pivot's source
range and fields. Recalculating a pivot is out of scope and stated as such.
**Depends on**: F-185.
**Test gate**: round-trip. A workbook with three pivots saves byte-identical in
the pivot parts and reports each pivot's source.

### F-194, Sheet rendering (L)
Page setup, print areas, repeating rows and columns, scaling, and the grid
itself, through the existing layout and PDF backends.
**Depends on**: F-186, F-191.
**Test gate**: golden. A rendered sheet matches the pinned oracle render within
the recorded SSIM threshold.

### F-195, rxlsx distribution (L)
The facade, `rxlsx-cli`, `rxlsx-wasm` and the Python wheel, following the shape
M13 established for the other two families.
**Depends on**: F-188, F-194.
**Test gate**: regression. The parity suite passes on every target platform.

---

## Milestone 20, Fidelity at scale (about 3 weeks)

**Goal**: prove the Word renderer against documents nobody here wrote.

PowerPoint fidelity is measured against 50 fetched decks with an SSIM harness.
Word fidelity rests on seven samples this project generates itself, so it can
catch a regression against its own output and can never catch a disagreement
with how Word actually renders. That asymmetry is the largest untested surface
in the workspace.

**End-of-milestone gate**: the Word corpus renders at the declared SSIM
threshold, and text shaping is correct for the scripts the corpus contains.

### F-196, Word corpus (M)
A pinned, fetched document corpus with the same provenance and licence
discipline as the deck corpus, covering business letters, reports, forms, legal
documents with revisions, and multi-script text.
**Depends on**: none.
**Test gate**: regression. The fetcher verifies every checksum and refuses a
corpus that does not match.

### F-197, Word SSIM harness (L)
The analogue of `pptx_ssim_harness.py`, comparing rendered pages against the
pinned oracle with the same trend-reference and hard-gate split.
**Depends on**: F-196.
**Test gate**: regression. The harness reports per-page SSIM, and a deliberate
layout change moves it.

### F-198, Hyphenation (L)
Liang hyphenation with language-specific patterns, which changes line breaking
and therefore every subsequent line. Word hyphenates and this renderer does not,
so any hyphenated document currently differs from the first hyphenated line
onward.
**Depends on**: F-197.
**Test gate**: golden. A hyphenated document matches the oracle's line breaks
within the recorded tolerance, and the harness delta is declared.

### F-199, Complex script shaping (L)
Arabic joining and shaping, Indic reordering and clusters, Thai breaking, and
CJK line-breaking rules. The shaper handles these and the line breaker does not
know their rules.
**Depends on**: F-196.
**Test gate**: golden. Multi-script corpus pages match the oracle within the
recorded threshold.

### F-200, Vertical and bidirectional text (M)
Right-to-left paragraph direction, mixed-direction runs, and the vertical text
directions the deck renderer currently approximates.
**Depends on**: F-199.
**Test gate**: golden. A bidirectional document renders with the correct visual
order.

### F-201, Large document performance (L)
A bounded memory ceiling and a stated throughput floor for a thousand-page
document, with the paginator and the renderer both measured.
**Depends on**: none.
**Test gate**: regression. A thousand-page fixture paginates and renders within
the asserted ceiling and floor.

### F-202, Incremental layout (L)
Re-lay out only what a mutation invalidated, rather than the whole document. The
layout cache added in F-009 is all or nothing, which is what makes an editing
session quadratic.
**Depends on**: F-201.
**Test gate**: regression. Editing one paragraph of a thousand-page document
re-lays out a bounded number of pages, asserted by counting layout invocations.

### F-203, Reader compatibility corrections (M)
Namespace-aware table-cell property recognition and schema-slot preservation for
numbering-level raw XML. Foreign same-local-name elements remain opaque,
byte-identical XML, and raw content before `w:suff` remains before that typed
element after a round trip.
**Depends on**: none.
**Test gate**: regression. Foreign `tcW` XML remains unmodelled and
byte-identical, and an `isLgl` raw child stays before `suff` after parse and
write.

---

## Cross-cutting

### F-X001, rdocx-cli tests (M)
The published binary has one compiled-executable integration test for each of
its seven subcommands in a single test binary. Fixtures are constructed in
code. Text extraction preserves document order, and both render branches use
bundled-font deterministic output.
**Test gate**: all seven named command integration tests pass, and the text,
validation, and deterministic-render sensitivity mutations fail.

### F-X002, README example correctness (S)
All six root README Rust examples use `rust,no_run` and compile against the
current `rdocx` rlib without executing filesystem writes. The read example uses
the total indexed `row_count`, `row`, `cell_count`, and `cell` APIs.
**Test gate**: `python3 scripts/readme_doctests.py` compiles all six examples.

### F-X003, Deduplicate the sample generators (S)
`generate_all_samples.rs` and `generate_samples.rs` overlap substantially.
**Test gate**: one generator produces every sample the harness needs.

### F-X004, Fix the shared temp path in the test suite (S)
`integration_test.rs` writes to a fixed, non-unique temp path shared across
concurrent runs.
**Test gate**: two concurrent `cargo test` runs both pass.

### F-X005, Tag rpptx-v0.1.2 (S)
Retain complete registry metadata after the immutable partial 0.1.0
publication, remove the CI-only tool dependency exposed by the 0.1.1 workflow,
prepare the exact incubating family at 0.1.2, and publish it through a newly
reviewed release tag before released rdocx consumers cut over.
**Depends on**: F-047 through F-050.
**Test gate**: all 12 incubating packages resolve from crates.io at 0.1.2 with
the expected owner, and the GitHub release targets the newly reviewed sprint
SHA.

### F-X006, Tag the expanded rpptx family (S)
Prepare the complete 14-package incubating family at 0.1.3, including
`oxml-cli-support` and `rpptx-cli`, then publish it only through
`/release rpptx-v0.1.3` after the command's separate final approval. The
complete family is published at 0.1.3. The immutable `rpptx-v0.1.2` tag and
its 12 published packages remain unchanged.
**Depends on**: F-143, F-144, F-145.
**Test gate**: all 14 incubating packages resolve from crates.io at 0.1.3 with
the expected owner, and the GitHub release targets the reviewed sprint SHA.

### F-X007, Integrate PR 25 and stable crate documentation (L)
Integrate Jon Stokes's PR 25 through the sprint branch, retaining contributor
credit in the GitHub merge record. The public Word facade gains custom list
definitions, per-paragraph numbering, composable hard line breaks and
hyperlinks, and fixed table-column widths. Rejected list updates remain
side-effect free, and fixed table geometry keeps the table width, grid, and
spanning cell widths consistent. Every stable crate has a package README that
states when to use it, links to its API documentation, and includes a current
example or a clear deprecation path. The README examples are compile-checked.
Typed numbering edits preserve unsupported attributes and child XML in schema
order across namespace aliases and collisions. Repeated tab stops carry public
source-occurrence provenance so edits, insertions, removals, and explicit
clears retain producer ownership in deterministic linear work. The public tab
parser tracks namespace scopes and accepts both empty and expanded tab-stop
elements. Preservation carriers extend one expanded-name `mc:Ignorable`
attribute without duplicating it, using the actual property ancestor scope
rather than a document-wide declaration list. Style, body, table-cell, header,
footer, footnote, and endnote paragraph properties retain established aliased
and default WordprocessingML parsing. Nested tab namespace scope has a normal
64-element depth bound. These public model additions set the stable release
boundary at 0.5.0.
**Test gate**: the merged PR's focused round-trip suite passes against current
`main`, the two rejected-state and table-geometry regressions pass, and every
stable crate README example compiles against its packaged crate. Numbering
round trips cover schema order, foreign namespace collisions, nested property
markup, provenance-only replacement, repeated occurrence ownership, explicit
clear carriers, namespace shadows, and expanded tab elements. The hash harness
remains 28 of 28. The gate also covers direct style and paragraph boundaries,
table cells, headers, notes, foreign same-local negatives, property-local
compatibility scope, and bounded deep tab aliases. Stable package archives stay
below 10 MiB, and the public migration examples compile.

### F-X008, Tag v0.5.0 (S)
The stable workspace package, nine internal pins, and eleven inherited
lockfile packages are 0.5.0 after F-X007. The exact seven stable crates.io
packages are published at 0.5.0 from the reviewed `v0.5.0` tag. The two Python
project versions and `rdocx-wasm` inherit 0.5.0 without gaining publication
authority. All 15 incubating manifests remain at 0.1.3, with exactly 14 in the
incubating crates.io family and `rpptx-wasm` unpublished. `publish.yml` runs the
exact stable and incubating metadata preflights before its patched 21-package
workspace dry run. No incubating, WASM, Python, or npm package is part of the
stable publication.
**Depends on**: F-X007.
**Test gate**: the stable metadata regression proves the workspace version,
nine pins, eleven lock entries, two Python versions, WASM literals, README
requirements, exact stable publication set, and unchanged incubating 0.1.3
state. The workflow contract, 12 README examples, 28-entry hash harness, exact
patched 21-package dry run, seven stable archive inventories, and `cargo deny`
pass. All seven stable packages resolve independently from crates.io at 0.5.0
under owner `mantissaman`, the GitHub release targets the reviewed sprint SHA,
and the PR 25 contributor credit and merge note remain visible on GitHub.

### F-X009, README coverage for every workspace crate (L)
Every one of the 26 Cargo workspace packages declares a README. Each document
states what the crate owns, when it should be used directly, its relationship
to adjacent packages, and provides a concrete Rust, CLI, Python, or JavaScript
example appropriate to that package. Internal and unpublished packages are
labelled honestly and gain no publication authority. The README runner checks
the exact workspace package set, required sections, manifest wiring, examples,
and archive inventory.
**Test gate**: `python3 scripts/readme_doctests.py` verifies exact README
coverage for all 26 workspace packages, compiles 26 Rust examples, validates
the CLI, Python, and JavaScript snippets, and proves all 21
publishable archives contain the byte-identical declared README.

### F-X010, Tag v0.6.0 (S)
Prepare the complete stable train at the next minor version, 0.6.0. The eleven
workspace-version packages move together, including the exact seven crates.io
packages and the four unpublished Python and WASM support packages. Stable
README dependency examples, metadata regressions, lock entries, Python project
versions, and WASM contract literals move to 0.6.0. The incubating train remains
at 0.1.3. The reviewed `/release v0.6.0` workflow publishes only the exact
seven stable crates after full verification, a clean microscope, a clean
sprint review, and separate immediate approval. No PyPI, npm, WASM, Python, or
incubating publication is authorized.
**Depends on**: F-X009.
**Test gate**: the stable release regression proves the eleven-package train,
nine internal pins, exact seven-package publication set, README requirements,
lock entries, Python project versions, WASM literals, and unchanged incubating
train. The exact 21-package dry run, README compilation and archive inventory,
28-entry hash harness, and supply-chain gate pass. All seven crates resolve at
0.6.0 under owner `mantissaman`, each crates.io README is present, and the
annotated `v0.6.0` tag targets the reviewed sprint SHA.

### F-X011, Tag rpptx-v0.2.0 (S)
The complete incubating train is published at the next minor version, 0.2.0. The
fourteen publishable `oxml-*` and `rpptx-*` packages move together with
unpublished `rpptx-wasm`, their root dependency pins, lock entries, README
dependency examples, source assertions, workflow regressions, and local WASM
package version. The completed stable train remains at 0.6.0. Incubating 0.2.0
was published only after full verification, a clean sprint review, and
separate immediate approval. `/release rpptx-v0.2.0` published only the exact
fourteen incubating crates. No npm, PyPI, Python, WASM, or stable package was
published.
**Depends on**: F-X010.
**Test gate**: the incubating release regression proves the fifteen-package
preparation group, fourteen internal pins, exact fourteen-package publication
set, README requirements, lock entries, source and workflow assertions, and
unchanged stable train. The exact 21-package dry run, README compilation and
archive inventory, 28-entry hash harness, WASM package gate, and supply-chain
gate pass. All fourteen crates resolve at 0.2.0 under owner `mantissaman`, each
crates.io README is present, and the annotated tag targets the reviewed sprint
SHA used by the successful GitHub release workflow.

### F-X012, Restore pinned CI toolchains (M)
Hosted CI installs the reviewed Poppler 26.01.0 rendering oracle from its exact
source archive and SHA-256 rather than a moving package-manager version. The
shared installer bounds download and streaming extraction resources, rejects
unsafe archive members and populated prefixes, builds only the three required
tools, and verifies each runtime identity. Test, MSRV, both Python binding rows,
and Presentation fidelity invoke it unconditionally before use. The WASM job
verifies the official Binaryen 125 Linux archive and exact
`wasm-opt version 125 (version_125)` release identity. Product code, package
versions, published artifacts, and rendering baselines remain unchanged.
Test and MSRV also install exact uv 0.10.2 through the reviewed official setup
action, isolate its cache, and run their corpus tests with an explicit 8 MiB
Rust test-thread stack.
They run on Ubuntu 24.04 and install LibreOffice 26.2.5.2 from the reviewed official Linux x86-64
archive before the full workspace suite. The shared installer verifies SHA-256
`2f03bfb2ac9f33ea7c77331b4b7a23300fb0ed7443566046bf8b5bc51c1bed1e`,
uses bounded streaming extraction, refuses populated prefixes, and checks the
exact reviewed build identity before the three `rpptx-chart` viewer gates run.
The installer also declares the exact Ubuntu runtime-library package set needed
to execute that official build.
**Test gate**: behavioral regressions execute every source, resource, runtime,
and prefix guard. Workflow mutations reject missing, conditional,
failure-tolerant, or successfully short-circuited installer steps and reject a
weakened Binaryen checksum or identity gate. They also reject uv action,
version, cache, or stack drift. The same contract rejects LibreOffice version,
checksum, bound, runtime, ordering, or consumer-step drift. Full verification
and a hosted pull-request CI run at the reviewed SHA pass with all 28 hashes
unchanged.

### F-X013, Footnote and endnote placement (M, split at design)
Carries the footnote half of the external PR 2 contribution, whose
anchored-drawing half was superseded by F-X007 and the M7 anchor work. Split
into three children at design time, when fixing endnote placement and splitting
oversized notes were both taken into scope. The parent closes when every child
closes.

### F-X013a, Footnote line advance (S)
Footnote text advances horizontally across the segments of a line rather than
drawing every segment at the same indent. A footnote assembled from several
runs, which is what any footnote carrying mixed formatting produces, no longer
collapses into an unreadable stack at a single x. The advance accumulates the
segment width that line breaking already computed, so the fix introduces no new
measurement.
**Test gate**: regression, named as a sentence describing the failure it
prevents. A footnote built from several differently formatted runs renders its
segments at strictly increasing x, and a single-segment footnote is unmoved. The
hash harness carries an expected delta for every baseline holding a
multi-segment footnote, stated and justified in the commit.

### F-X013b, Footnote reservation and splitting (L)
Pagination reserves the height a page's notes occupy before body content fills
the text area, so body text and the note area no longer overlap. A page reserves
the separator offset once and each distinct note referenced by a line placed on
that page once, which keeps a note with the page carrying its reference rather
than with the paragraph that owns it. A note too tall for the space remaining
splits at a line boundary and continues on the next page, so an oversized note
can neither starve a page of body content nor stall pagination. Notes are laid
out once into a shared height map that the reservation and the rendering pass
both consume, so a reserved height and its rendered height cannot diverge.
**Depends on**: F-X013a.
**Test gate**: regression, named as sentences describing the failures they
prevent. A page whose body fills the text area leaves the reserved note area
clear. A note taller than its remaining space continues on the following page
without repeating its marker. A page carrying two references to one note
reserves that note once. The hash harness carries an expected delta for every
baseline holding a note, stated and justified in the commit.

### F-X013c, Endnotes at the document end (M)
Endnote references stop rendering their note at the foot of the page that
carries the reference. Endnotes collect into a document-end sequence rendered
after the final body page in reference order, while footnotes keep their
per-page placement. The layout carries the two note streams distinctly rather
than a single identifier that a footnote and an endnote of the same number both
match, which today resolves to whichever the footnote part happens to define.
**Depends on**: F-X013b.
**Test gate**: regression, named as sentences describing the failures they
prevent. A document mixing footnotes and endnotes places each stream in its own
region. A footnote and an endnote sharing a number resolve to their own note
rather than both to the footnote. The hash harness carries an expected delta for
every baseline holding an endnote, stated and justified in the commit.

### F-X017, Notes broken to their own section's width (S)
A note is line-broken to the width of the section that references it rather
than to the final section's width. `NoteRegistry` is built once ahead of
pagination against one content width, which is correct for every document whose
sections share a page size and wrong for any that does not. Note positioning is
already per-section, so this closes the half F-X013b left open.
**Depends on**: F-X013b.
**Test gate**: regression. A document whose two sections differ in page width
breaks each note to the measure of the section holding its reference, and a
single-section document is byte-identical to before.

### F-X014, Kashida justification values (S)
`ST_Jc` accepts `lowKashida`, `mediumKashida` and `highKashida`, mapping each to
justified alignment instead of rejecting the value.

The consequence is larger than the alignment. `CT_PPr::from_xml` propagates the
rejection with `?`, and that error travels all the way out of
`CT_Document::from_xml`, so a document carrying one of the three Arabic
justification settings **fails to open at all**. This is a load failure, not a
layout inaccuracy.
**Test gate**: regression, named as a sentence describing the failure it
prevents. A document whose paragraph carries each kashida value opens and lays
out justified, and the existing rejection still holds for a genuinely unknown
string. The hash harness is unchanged, since no recorded baseline carries a
kashida value.

### F-X024, Move the theme adapter into rdocx-oxml (M)
`oxml-drawing` hosts `impl From<&CT_OfficeStyleSheet> for
rdocx_oxml::theme::Theme`, which is the single documented exception to the rule
that nothing in `oxml-*` depends on `rdocx-*`. That one edge makes the two
publication trains mutually dependent: `rdocx-layout` depends on `oxml-layout`
and `oxml-drawing` depends on `rdocx-oxml`, so neither train can publish first
once both carry breaking changes.

The adapter moves to `rdocx-oxml`, which the orphan rule permits because `Theme`
is local there and `CT_OfficeStyleSheet` is the foreign type. The edge inverts
to stable depending on incubating, the architecture rule loses its exception and
becomes absolute, and train-at-a-time publication works in one fixed order
forever: incubating, then stable.

`rdocx-oxml` gains a dependency on `oxml-drawing`, so a Word-only consumer now
compiles DrawingML. That is the accepted cost, chosen over deleting an adapter
that exists so `rdocx-layout`'s `LayoutInput.theme` does not churn when
PresentationML themes reach Word layout.
**Depends on**: F-X020.
**Test gate**: regression. The conversion produces the same `Theme` from the
same `CT_OfficeStyleSheet` as before the move, `cargo tree` shows no `oxml-*`
package depending on any `rdocx-*` or `rpptx-*` package, and the workspace
still builds with all 28 hashes unchanged.

### F-X022, Tag rpptx-v0.3.0 (S)
The complete incubating train moves to the next minor version, 0.3.0, because
S41 broke its public API rather than merely extending it. `oxml-layout` renamed
`TextSegment::footnote_id` and `GlyphRun::footnote_id` to `note`, changing the
type from `Option<i32>` to `Option<NoteRef>`, and added two fields to
`LineBreakParams`. Under semver a 0.x minor bump is the correct response.

The fifteen packages carrying an explicit 0.2.0 move together, their root
dependency pins, lock entries, README dependency examples and the local
`rpptx-wasm` version with them. Exactly fourteen are published: `rpptx-wasm`
stays unpublished. The stable train stays at 0.6.0 during this story, and its
pins on the incubating crates move to 0.3.0 so the later stable release can
resolve against a published 0.3.0.

This story prepares and, through `/release rpptx-v0.3.0`, publishes. Publication
happens only after full verification, a clean microscope, a clean sprint review
and separate immediate approval at the reviewed SHA. No npm, PyPI, Python, WASM
or stable package is authorized.
**Depends on**: F-X020.
**Test gate**: the incubating release regression proves the fifteen-package
preparation group, the fourteen internal pins, the exact fourteen-package
publication set, README requirements, lock entries and the unpublished
`rpptx-wasm` literal. The patched workspace dry run, archive inventory under
10 MiB, README compilation and `cargo deny` pass, and all 28 hashes stay
unchanged.

### F-X023, Tag v0.7.0 (S)
The complete stable train moves to 0.7.0, because S41 broke its public API.
`rdocx-oxml` added `note_type` to `CT_Footnote`, six fields to `CT_Anchor` and
four variants to `WrapType`, each of which breaks an exhaustive match or a
struct literal. `rdocx-layout` added fields to `ParagraphBlock` and
`AnchoredDrawing`. The `rdocx` facade's own public API is unchanged, and it
moves with its train regardless.

The eleven workspace-version packages move together: the exact seven crates.io
packages plus the four unpublished Python and WASM support packages. README
dependency examples, metadata regressions, lock entries, the two Python project
versions and the WASM contract literals move to 0.7.0. The incubating train
remains at 0.3.0.

`/release v0.7.0` publishes only the exact seven stable crates, after full
verification, a clean microscope, a clean sprint review and separate immediate
approval. No PyPI, npm, WASM, Python or incubating publication is authorized.
**Depends on**: F-X022. The stable crates depend on `oxml-layout`, so the
incubating train has to be resolvable at 0.3.0 on crates.io before the stable
train that pins it can publish. This is the reverse of the S39 order, where only
one train moved.
**Test gate**: the stable release regression proves the eleven-package train,
the nine internal pins, the exact seven-package publication set, README
requirements, lock entries, Python project versions, WASM literals and the
unchanged incubating train at 0.3.0. The patched workspace dry run, archive
inventory, README compilation and `cargo deny` pass, and all 28 hashes stay
unchanged.

### F-X025, /verify must run the release regressions (S)
`/verify --full` runs formatting, lints, the workspace suite, the hash harness,
the prose rules, the no-default-features path, the WASM targets, docs, packaging
and the supply-chain check. It does not run
`python3 -m unittest scripts.test_sprint_workflow`, which holds the release
family preflights that `.github/workflows/publish.yml` invokes by name as the
publication gate.

S42 demonstrated the gap rather than theorised it. F-X022 moved every version
carrier under `crates/`, passed the entire local gate, and still left the
incubating preflight and the `ci.yml` WASM literal asserting the old version. It
would have failed in CI at publication time.
**Test gate**: regression. A deliberately stale version literal in
`scripts/test_sprint_workflow.py` or a workflow file fails `/verify --full`,
and a clean tree passes it.

### F-X026, CI must run the release regressions too (S)
`/verify` step 6 runs `python3 -m unittest scripts.test_sprint_workflow` after
F-X025, so the release family preflights no longer run for the first time on a
tag. `.github/workflows/ci.yml` does not. Its `prose` job runs the sprint's other
two standard-library checks, `prose_check.py` and `sync_agent_skills.py --check`,
and not this one, so a contributor who does not run `/verify` can move a version
carrier and see a green pull request.

Filed by the S43 sprint review, `.claude/reviews/S43-sprint-review-pass-1.md`,
finding N1. It is narrower than the defect S42 hit, since F-X022 was authored
through the local gate, which is why it was not fixed inside F-X025.
**Depends on**: F-X025.
**Test gate**: regression. The module runs in a named CI job, asserted the way
the other job contracts are, and a stale version literal fails that job.

### F-X027, Wire the golden-PNG gate into something (S)
`scripts/golden_png_harness.py` generates deterministic PDFs, rasterises page one
at 150 DPI with the pinned Poppler oracle, and compares decoded pixels against
`scripts/golden_pixel_manifest.json`. `docs/hld/12-testing-strategy.md` describes
it in full. It appears in no `/verify` step and no CI job, so it runs only when
somebody remembers it, and a recorded manifest nothing checks is not a gate.

Filed by the S43 sprint review, finding N2. Pre-existing rather than caused by
S43. It surfaced because F-X021 went looking for what watches PDF output. The
story decides where it belongs, given that it needs `pdftoppm` and a pinned
Poppler build and so cannot sit in the same place as the hash harness.
**Depends on**: none.
**Test gate**: regression. A deliberate rendering change fails the gate wherever
the story puts it, and a clean tree passes it.

### F-X028, Repair the agent-facing documentation drift (M)
`CLAUDE.md` opens by stating that its instructions override default behaviour,
so an error in it propagates into every future session. Five claims in it are
false today, and two more sit in the command surface and the spec set.

`CLAUDE.md:159-170`, "Known defects being carried", lists three defects and says
"Do not 'fix' these". All three were fixed in M1. `MediaNamer::scan` takes the
maximum occupied suffix, `Document` holds `layout_cache` and
`deterministic_layout_cache`, and Caladea ships `LICENSE-Caladea` and
`NOTICE-Caladea` with `bundled_fonts.rs` correctly recording Apache 2.0. The
entry claiming a false licence notice ships today is the most serious, because
it tells an agent to leave a legal defect alone that does not exist.

`CLAUDE.md:15` puts the `rdocx-*` family on crates.io at 0.2.0. It is 0.7.0.

`CLAUDE.md:41` and `:163` place the bundled fonts at `crates/rdocx-layout/fonts/`.
They live in `crates/oxml-layout/`.

`CLAUDE.md:41`, `CLAUDE.md:60` and `docs/hld/10-bindings-spec.md:249` name a
`bundled-fonts` feature. No manifest defines one. Bundled fonts are compiled in
unconditionally and `system-fonts` is the optional feature, so the wheel-building
instruction in the bindings spec names a flag that cannot be set.

`docs/hld/15-build-and-toolchain.md:229-236` states in the present tense that
the shared-version group "is at 0.6.0", that the Python project and rdocx WASM
literals "are also 0.6.0", and that the incubating manifests are "prepared at
explicit version 0.2.0". The trains are at 0.7.0 and 0.3.0. This is the same
paragraph family F-X025 corrected two sentences of, found while confirming the
WASM publication position for F-X030.

`.claude/commands/verify.md:55-57` runs `cargo test -p rdocx-layout
--no-default-features` and tells the reader to rename the package when the
extraction lands. It landed. `CLAUDE.md`, `AGENTS.md` and the CI matrix all name
`oxml-layout`. Both invocations work and neither is a no-op, 87 tests against
62, so this is one gate document disagreeing with every other record rather than
a broken gate.

F-X025 corrected two instances of the same class in the spec set. These are the
third through the twelfth, which is what makes this a story rather than another
one-off patch. Three of them were found while doing something else, which is the
argument for the test gate below rather than another manual sweep.
**Depends on**: none.
**Test gate**: regression. A test asserts that every path, version and feature
name `CLAUDE.md` and `.claude/commands/verify.md` cite resolves against the
workspace, so the next stale claim fails the gate rather than surviving 40
sprints.

### F-X029, Path-filtered CI jobs (M)
`.github/workflows/ci.yml` defines thirteen jobs and no `paths` filter, so every
job runs on every change. A commit that touches only `docs/` currently runs the
workspace test suite, the MSRV suite, both WASM targets, the Python bindings,
the packaging archive build and the pinned-render fidelity job.

The filters that pay: `presentation-fidelity` needs the PowerPoint and shared
crates, `python-bindings` needs the binding crates, `supply-chain` needs the
manifests and the lockfile, `hash-harness` needs anything that can reach the
sample generator, and `prose` needs only tracked Markdown.

**The trap is required status checks, and this story exists to get it right.**
A job skipped by a `paths` filter never reports, so a required check waits
forever and the pull request can never merge. The fix is a gate job that always
runs and reports on behalf of the filtered set, rather than filtering the
required jobs directly. A story that adds filters without handling this converts
a slow pipeline into a stuck one.

Filters must also fail safe. A filter that is too narrow silently stops running
a gate, which is the same class of defect as F-X021 and F-X025: an instrument
reporting green because it never ran.
**Depends on**: none.
**Test gate**: regression. A test asserts, for each filtered job, a changed path
that must trigger it and a changed path that must not, so narrowing a filter by
mistake fails the suite. A docs-only change reports every required check.

### F-X030, Decouple the npm package versions from the Rust family version (S, archived)

**Archived without being started. Its premise was wrong.**

The story claimed that a JavaScript-only fix to `@tensorbee/rdocx-wasm` or
`@tensorbee/rpptx-wasm` could not ship without versioning a Rust family that had
not changed. There is no shipping. Neither package is published anywhere.

`scripts/test_sprint_workflow.py:1337-1349` asserts that the WASM CI job
contains none of `npm publish`, `npm login`, `npm adduser`, `npm token`,
`wasm-pack publish`, `NODE_AUTH_TOKEN`, `NPM_TOKEN`, `--registry`, `id-token:`,
`git tag` or `gh release`. The job packs a bundler tarball and install-tests it
locally, and that is the whole of it.
`docs/hld/15-build-and-toolchain.md` says the same in prose: registry
publication is "unconfigured and unauthorized", and no WASM or npm package
gained publication authority from any release.

So the version inheritance costs nothing. It would begin to cost something on
the day npm publication is authorised, and not before. Recorded in
`02-scope-and-non-goals.md` as a deliberate position rather than an oversight,
so the next reader does not refile this.

**Do not reopen this without first authorising npm publication.** If that
happens, the work is the version split plus the `ci.yml` assertions at
`scripts/test_sprint_workflow.py:1317-1319` and the lockfile package set the
stable preflight asserts.

### F-X031, Require the CI gate in branch protection (S)

F-X029 creates an always-reporting `ci-gate` that represents the result of the
path-filtered CI graph. S44 deliberately stops at the tracked workflow because
changing GitHub branch protection is an external repository mutation. The gate
names and planned product surface continue to evolve through the roadmap, so
the required-check configuration is parked at its final boundary.

In S62, inspect the reviewed workflow at the sprint head, confirm that
`ci-gate` is still the one stable aggregate check, and configure the repository
ruleset or classic branch protection to require that exact check. Do not remove
existing protections without an explicit reviewed decision. Bind the evidence
to the repository, branch pattern, ruleset or protection identifier, and the
reviewed sprint SHA.

**Depends on**: F-X029.
**Test gate**: integration. A docs-only pull request reports a successful
required `ci-gate` while the filtered expensive jobs stay skipped, and a
selected failing job makes the required gate fail.

### F-X032, Expose complete Word layout results (S)

Expose the cached normal-font `WordLayoutResult` and an uncached caller-font
`WordLayoutResult` from `Document` so third-party renderers can consume
positioned pages together with the exact `FontData` and Word source map used by
layout. Accepted-default and `RenderOptions` variants use the existing layout
paths. Cache-backed access returns `Arc<WordLayoutResult>`, caller-font access
returns an owned result, and no new layout engine or font-set cache is
introduced.

**Depends on**: F-009, F-151, F-X037.
**Test gate**: regression. Every emitted glyph-run font id resolves to returned
font data, repeated default calls share the accepted layout cache,
caller-provided fonts appear in the owned result, and tracked layout does not
populate the accepted cache.

### F-X033, Integrate PR 36 ordered body items (S)

Integrate Pedro Assumpcao's PR 36 through the active sprint branch while
retaining the contributor commit and GitHub pull-request record. The additive
native `Document::body_items` reader returns direct document-body children in
source order as paragraph, table, body-level content-control, or preserved
unsupported XML views. Existing recursive paragraph and table accessors retain
their current semantics. Python, WASM, and CLI surfaces remain unchanged.

The submitted checks ran against an older base. Retarget the pull request to
the integrated sprint branch, run current-base GitHub CI, and merge it with a
GitHub merge commit. Maintainer hardening and documentation remain separate
from the contributor commit.

**Depends on**: none.
**Test gate**: integration. An in-code document with interleaved body
paragraphs, tables, content controls, and unmodelled XML opens through the
public facade and `body_items` reports every direct child once in exact source
order. Current-base GitHub CI, the submitted focused test, the full package
gate, and the unchanged hash harness also pass.

### F-X034, Reviewed release notes for every release (S)

Every release tag carries reviewed, human-written release notes rather than
only GitHub's generated commit summary. A canonical `/release-notes TAG`
ceremony reads the release plan, completed delivery records, relevant commits,
and contributor history, then prepares the versioned `CHANGELOG.md` section
with highlights, user-visible additions and fixes, compatibility or migration
guidance, and contributor credit. Its generated agent skill keeps the ceremony
identical across tools. Release preflight rejects a missing, empty, or
placeholder section. The publish workflow renders that reviewed section and
passes it to `gh release create` without replacing it with
`--generate-notes`.

**Depends on**: F-X025.
**Test gate**: regression. The custom command prepares complete notes from the
reviewed release record, its generated skill is in sync, release-note
extraction returns the exact versioned changelog section for both tag families,
missing or incomplete notes fail, and the publish workflow can create a GitHub
release only from that reviewed output.

### F-X035, Tag rpptx-v0.4.0 (S)

Prepare and publish the complete incubating family at 0.4.0. This is the first
incubating release containing `oxml-chart`, which is required by the current
stable `rdocx` graph and was not published at 0.3.0. All 15 crates.io packages
move together, `rpptx-wasm` remains unpublished, and the reviewed release notes
name the chart addition, compatibility position, and contributors.

**Depends on**: F-X034, F-X037.
**Test gate**: release. The incubating metadata regression, full verification,
22-package dry run, archive inventory, supply-chain gate, and unchanged hash
harness pass. After separate final approval, all 15 crates resolve from
crates.io at 0.4.0 and the GitHub release uses the reviewed notes at the exact
sprint SHA.

### F-X036, Tag v0.8.0 (S)

Prepare and publish the complete stable family at 0.8.0 after the incubating
0.4.0 dependency graph is available. The minor boundary covers the intentional
pre-1.0 low-level revision and field model changes plus the additive document
automation, complete-layout, and ordered-body facade APIs. Only the exact seven
stable crates publish. Python, WASM, npm, PyPI, and incubating publication stay
unauthorised. The reviewed release notes describe the new APIs, fixes,
compatibility boundary, and contributor credit.

**Depends on**: F-166, F-167, F-168, F-X032, F-X033, F-X035.
**Test gate**: release. The stable metadata regression, full verification,
22-package dry run, archive inventory, supply-chain gate, and unchanged hash
harness pass. After separate final approval, all seven stable crates resolve
from crates.io at 0.8.0 and the GitHub release uses the reviewed notes at the
exact sprint SHA while PR 36 credit remains visible.

### F-X037, Trace Word glyphs to source paragraphs (M)

Carry format-neutral source spans through shaping, both line-splitting stages,
pagination, and positioned glyph output. `rdocx-layout` returns a typed
`WordLayoutResult` whose result-local side table resolves each source node to a
document, table, nested-table, header, footer, footnote, or endnote paragraph
path. Character ranges use Unicode scalar indices in the selected revision
projection. Generated markers, dynamically evaluated fields, and text whose
display transformation cannot preserve an exact source slice remain
unattributed rather than reporting a false location.

The existing `layout_document` functions retain their `LayoutResult` return
type and discard provenance. New provenance variants return the Word-specific
bundle. F-X032 exposes that bundle through cached and caller-font facade paths,
so external renderers receive pages, fonts, and source resolution together.

This is an intentional low-level pre-1.0 source break for exhaustive
`TextSegment` and `GlyphRun` literals and belongs in the planned 0.4.0 and
0.8.0 release notes. It does not add rendering support for content that the
current layout engine skips.

**Depends on**: F-009, F-151.
**Test gate**: regression. Every attributed glyph run resolves to one exact
Word paragraph path and Unicode-scalar range whose projected text equals the
run text across ASCII, CJK, wrapping, tables, nested tables, headers, footers,
footnotes, endnotes, and both revision views. Both splitting stages preserve
contiguous ranges. Generated markers, evaluated fields, and non-bijective text
transformations remain unattributed. Caller-font and cached layouts carry the
same complete source map, packaged crates remain below 10 MiB, WASM checks
pass, and all 49 hash entries remain unchanged.

### F-X021, The hash harness should cover PDF output (M)
The output-stability harness records `page1.png` and three `word/*.xml` parts
for each of the seven samples, and no PDF. PDF is a first-class output of this
workspace, produced by a different code path from the PNG: `oxml-pdf` writes
glyph positions, embedded font subsets and compressed streams, none of which the
rasterised PNG exercises. That path can therefore drift with no gate noticing.

F-X020 demonstrated the gap rather than theorised it. A routine
semver-compatible dependency refresh changed all seven sample PDFs while every
PNG stayed byte-identical and the harness reported 28 of 28. The change was
benign, and it was found by hand rather than by the gate that exists to find it.

Recording a PDF byte hash directly would be brittle, since a PDF carries a
creation date and object ordering that need not be stable. The story therefore
decides what a stable PDF fingerprint is, likely extracted text plus page
geometry plus glyph positions, before recording one.
**Depends on**: none.
**Test gate**: regression. A deliberate change to the PDF writer moves the new
entries and leaves the PNG entries untouched, and a re-run with no change
reproduces every entry exactly.

### F-X020, Refresh the dependency lockfile (S)
Every semver-compatible dependency update outstanding at the start of the sprint
is taken, and its effect on rendered output is measured rather than assumed.
Sixteen updates are pending and none is a security fix: `cargo audit` reports
zero vulnerabilities across 152 dependencies and `cargo deny check advisories`
passes. Two of the sixteen, `font-types` and `zune-core`, sit in the font and
image decoding path, which is why the hash harness is this story's real gate
rather than a formality.

The `ttf-parser` unmaintained advisory, RUSTSEC-2026-0192, is unaffected. It is
allowlisted in `deny.toml` with a documented reason, and clearing it needs the
`fontdb` to `fontique` swap rather than a lockfile refresh.
**Test gate**: the full workspace suite and the hash harness. A delta is
expected only if a font or image dependency moved rendering, and any delta names
the dependency that caused it and is reviewed before the baseline is re-recorded.
A delta traced to no dependency in the rendering path blocks the story.

### F-X019, Paragraph-relative drawings in later blocks should wrap (M)
Text flows around a wrapping drawing anchored to a later paragraph even when
that drawing is positioned relative to its own paragraph rather than to the
page or a margin. F-X016 looks ahead only for absolutely framed drawings,
because a paragraph-relative one has no position until its own paragraph is
placed, and resolving that needs the paginator to run twice. No sample or corpus
document hits the gap today.
**Depends on**: F-X016.
**Test gate**: regression. A paragraph-relative wrapping drawing anchored to a
later paragraph pushes earlier text aside, and a document with no such drawing
paginates in a single pass exactly as before.

### F-X018, Unknown enumerated values should not fail a document open (M)
Nine value parsers in `rdocx-oxml/src/shared.rs` and `styles.rs` return an error
for any string they do not enumerate, and several are reached through `?` from
paragraph, table and numbering property parsing. A document using a
spec-valid value the model does not yet list therefore fails to open rather
than losing one property. F-X014 fixes the three kashida values because they
were reachable from a real contribution. This story decides the general rule,
which is that an unmodelled enumerated value falls back to the element's default
and the surrounding properties survive.
**Depends on**: F-X014.
**Test gate**: regression. A document carrying an unmodelled value for each of
the nine enumerations opens, keeps every sibling property, and renders with the
default for the unmodelled one.

### F-X015, Anchored drawing wrap and alignment model (M)
`CT_Anchor` carries the wrap mode, the four text-distance attributes and the
optional horizontal and vertical alignment children, and `AnchoredDrawing`
carries them into the layout model. `wrapSquare` and `wrapTopAndBottom` parse to
distinct wrap modes rather than collapsing into `None`, which is what the
currently parsed-but-unread `WrapType` does today. `distT`, `distB`, `distL` and
`distR` round-trip through the serialiser. A `positionH` or `positionV` that
names an alignment records that alignment alongside its offset. Placement and
rendering are deliberately unchanged, so this story adds only the model surface
that F-X016 consumes.
**Test gate**: round-trip. Wrap modes, the four distances and both alignment
axes survive a parse and serialise cycle, including a prefix-tolerant read. The
hash harness is unchanged, which is what proves the story is model-only.

### F-X016, Floating drawing placement and text wrapping (L)
An anchored drawing whose position names an alignment resolves against its
`relativeFrom` frame by that alignment rather than by a zero offset. Body text
flows around a `wrapSquare` drawing, reserving the frame width plus the relevant
text distance on the lines the drawing spans, and clears a `wrapTopAndBottom`
drawing by starting below it. Reserved width is taken from the drawing frame and
its `distL` or `distR`, not from a scan of image pixels, since pixel extents
describe `wrapTight` and `wrapThrough` rather than `wrapSquare`. Line breaking
gains a per-line width reservation that the paginator can vary once it knows
where on the page the paragraph landed.
**Depends on**: F-X015.
**Test gate**: golden. A paragraph beside a left-aligned square-wrapped drawing
breaks its lines at the reserved width, a right-aligned one reserves from the
line end, and a top-and-bottom drawing pushes the following text below its
bottom edge plus `distB`. Unwrapped and `wrapNone` drawings lay out exactly as
before, which the hash harness proves by leaving every baseline without a
wrapped drawing unchanged.

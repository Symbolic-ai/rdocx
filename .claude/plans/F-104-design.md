# F-104, SSIM fidelity harness

**Status**: completed
**Sprint**: S25
**Size**: L
**Depends on**: F-102

## Problem

The M10 acceptance contract requires the full pinned corpus to render without
panic, missing output, dimension mismatch, or a dropped bounded shape. It also
records the 0.95 SSIM on 80 percent reference as a trend and requires a native
PowerPoint representative review. The repository began with deterministic PNG
machinery but no whole-deck render command, LibreOffice comparison harness, or
CI fidelity job. The normal `rpptx-render::layout_presentation` also constructs
the system-font manager, which is unsuitable for a recorded comparison.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "The measurable bar".
- `docs/hld/08-rendering-spec.md`, the renderer pipeline and visible fallback
  invariant.
- `docs/hld/12-testing-strategy.md`, "The render fidelity gate".
- `docs/hld/14-development-backlog.md`, "F-104, SSIM fidelity harness" and the
  M10 end-of-milestone gate.
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering".
- `.claude/skills/differential-testing.md`, external oracle rules.

## Approach

Add `layout_presentation_deterministic` beside the normal renderer entry point.
It uses the bundled-font-only `FontManager` and otherwise shares the same
private page-lowering implementation, so the fidelity path cannot silently
diverge from normal rendering.

Add `crates/rpptx/examples/render_deck.rs` as the concrete corpus driver. It
opens each package, resolves every slide, relationship scope, media item,
theme, and table-style list into one `RenderInput`, renders every page at 150
dpi, and writes deterministic PNGs plus a tab-separated manifest containing
source leaf counts, resolved shape counts, diagnostics, and output paths. It
fails on any panic, missing page, or dropped bounded source shape.

Add `scripts/pptx_ssim_harness.py` as the orchestrator and metric owner. It
verifies the 50-deck manifest, invokes the Rust corpus driver once, records and
checks the exact LibreOffice version, renders the oracle through LibreOffice
and a pinned 150 dpi PDF raster path, decodes PNGs with the existing
golden-harness decoder, and computes a documented deterministic SSIM score.
The script reports per-slide scores and corpus totals, fails if any slide is
missing or dropped, and records whether 0.95 on at least 80 percent was met.
Embedded unit tests pin identical, changed, dimension-mismatch, completeness,
and trend-classification behavior.

Add a dedicated CI job that fetches the pinned corpus, installs the exact
recorded LibreOffice and raster tools, and runs the harness in required mode.
Record a one-time M10 PowerPoint spot-check over representative low, median,
and high SSIM outputs. LibreOffice differences remain review-required rather
than becoming an unexplained allowlist.

## Authorized remediation expansion

The initial remediation expansion on 2026-08-05 used 0.95 SSIM on 80 percent as
an automatic target. The later native calibration and user decision recorded
below supersede that hard interpretation. Remediation stays inside existing
files unless a later design revision receives explicit approval for a new file,
module, trait, or generic.

The fresh `evidence-2` baseline contains 50 decks, 421 slides, 2,154 bounded
source shapes, 2,154 resolved shapes, and zero dropped shapes. It has 12 slides
at or above 0.95 SSIM, or 2.850 percent, with minimum -0.177170506, median
0.172346895, and maximum 1.0. The first in-scope correction removed spurious
black fallback borders from no-geometry text boxes on 300 diagnosed slides.
The refreshed post-correction manifest and Rust renders at
`/private/tmp/f104-fidelity-current` contain the same 50 decks, 421 slides,
2,154 bounded source shapes, 2,154 resolved shapes, and zero dropped shapes.
Scoring them against the unchanged `evidence-2` oracle produces 15 passing
slides, or 3.563 percent, with minimum -0.177170506, median 0.198522927, and
maximum 1.0. It proved the defect but did not materially close the gate.

Remediation proceeds in evidence-ranked batches with a distinguishing
regression and a full corpus re-score after each batch:

1. Fix raster JPEG decoding before layout changes. The current PNG raster path
   passes compressed JPEG bytes to the RGB pixmap constructor. Visual evidence
   shows black or corrupt image regions on slides which carry no diagnostic,
   including `at.ecodesign...pptx` slide 22 at SSIM 0.004535185. This is the
   first proposed batch because 173 slides have no diagnostic, only 12 pass,
   and their median is 0.404902958. The accepted implementation adds workspace
   dependency `zune-jpeg` 0.5.15 with minimal features, decodes explicitly to
   RGB or RGBA for raster output, and preserves PDF DCT pass-through. The full
   re-score at `/private/tmp/f104-fidelity-after-jpeg` has 16 passing slides,
   or 3.800 percent, with minimum -0.177170506, median 0.235102028, maximum
   1.0, and zero dropped shapes. The focused defect is fixed, but the batch
   added only one passing slide and does not materially close the gate.
2. Correct absent-background semantics before resolving new background paint
   forms. An absent slide, layout, and master background means no background
   paint and must not implicitly select `bgFillStyleLst[0]`. Explicit `p:bgRef`
   continues to resolve its indexed theme fill. Then resolve background picture
   media and supported gradient geometry. Picture backgrounds affect 104
   slides. Background and shape gradient diagnostics affect 46 distinct slides.
   Every affected background slide is below 0.95. The absence-only re-score at
   `/private/tmp/f104-fidelity-after-background-absence` remains 16 passing
   slides, or 3.800 percent, with minimum -0.177170506, median 0.235102028,
   maximum 1.0, and zero dropped shapes. The corrected contract does not change
   the aggregate corpus result because affected theme defaults match the white
   raster default. The media-capable background sub-batch then reuses one
   `ResolvedImage` model and the existing picture lowering path for direct
   slide, layout, and master `p:bgPr` blips. Theme-referenced blips remain
   precisely diagnosed until theme relationship scope exists. Its re-score at
   `/private/tmp/f104-fidelity-after-background-media` has 26 passing slides,
   or 6.176 percent, with minimum -0.177170506, median 0.506613068, maximum
   1.0, and zero dropped shapes. This adds 10 passing slides and raises median
   SSIM by 0.271511040. The next sub-batch corrects supported linear-gradient
   geometry on non-square bounds, including absent or false `scaled` semantics
   and projection-covering endpoints. A background ignores
   `rotWithShape="0"` because it has no enclosing shape rotation. Before any
   path-gradient implementation, diagnostics distinguish circle, rectangle,
   shape, tile, and flip variants. Circle becomes radial paint only if the
   current neutral model represents it exactly. The completed gradient
   re-score at `/private/tmp/f104-fidelity-after-gradient` has 27 passing
   slides, or 6.413 percent, with minimum -0.177170506, median 0.532719562,
   maximum 1.0, and zero dropped shapes. This adds one passing slide and raises
   median SSIM by 0.026106494.
3. Resolve shape picture fills and remaining supported image placements.
   Picture-fill diagnostics affect 31 slides, 30 below the threshold. This
   batch reuses source-scoped media and the existing image lowering path. An
   ordinary shape gains an optional resolved image fill separate from its
   content, so a shape can retain both picture fill and text. Direct blip fills
   resolve in the flattened slide, layout, or master producer scope. A blip
   selected from the theme style matrix remains precisely diagnosed until
   theme relationship scope exists and cannot consume a colliding slide,
   layout, or master relationship ID. Rendering lowers the image before the
   shape stroke and text and clips it through the shape's concrete geometry.
   The completed re-score at `/private/tmp/f104-fidelity-after-shape-fill` has
   28 passing slides, or 6.651 percent, with minimum -0.177170506, median
   0.532719562, maximum 1.0, and zero dropped shapes. This adds one passing
   slide while the median remains unchanged.
4. Correct group transforms, clipping, and ordinary shape placement using the
   low native representative and corpus grouping decks. The native low slide
   confirms a wrong background plus missing or misplaced grouped geometry even
   though the resolver reports zero dropped shapes. The pinned corpus contains
   63 groups in 12 decks, including 18 nested groups and seven rotated groups.
   Its observed mappings are identity or non-shearing scale, translation,
   rotation, and reflection. Materialize accumulated child-coordinate scale
   into each leaf's bounds before concrete geometry and text-box resolution,
   while retaining only the rigid group mapping around the rendered leaf.
   Absolute line widths, font sizes, and effect metrics therefore remain in
   points. Nested mappings apply inner before outer, group bounds never imply a
   clip, and depth-first sibling order remains unchanged. A zero child extent
   uses finite unit scale on that axis, maps its child origin to the group
   origin, and emits a stable diagnostic. A genuinely sheared or singular
   accumulated mapping emits a separate stable diagnostic and retains the
   existing affine fallback rather than claiming general shear support. The
   completed group-batch re-score at
   `/private/tmp/f104-fidelity-after-groups` has 29 passing slides, or 6.888
   percent, with minimum -0.095159989, median 0.537355079, maximum 1.0, and
   zero dropped shapes. `sample_pptx_grouping_issues.pptx` improved from
   -0.177170506 to 0.992427070. Native inspection confirms that the page stays
   white and the complete grouped geometry closely matches the oracle.
5. Correct connector, supported custom geometry, and table border fidelity.
   These diagnostics cover 49 slides, 47 below the threshold, with median SSIM
   0.366.

   The retained manifest contains 91 resolved connector-geometry fallbacks.
   A direct package audit finds 85 connector sources. Of those, 72 are on 17
   slides in four decks and 13 are inherited from layouts or masters. The
   source forms are 44 `straightConnector1`, 37 `line`, two
   `bentConnector3`, and two `curvedConnector3`. There are no connector custom
   geometries. All four presets already have generated definitions and use the
   existing preset evaluator, including the stored bend and curve adjustment
   values. The smallest first sub-batch therefore reuses
   `concrete_preset_geometry` for connector shape properties, preserves the
   existing transform, direct line, fill, arrowheads, and visible fallback,
   and retains a precise diagnostic only for absent, unknown, or failed
   geometry. Eighty-three of the 85 source connectors already carry a direct
   line. The remaining two rely on a `p:style` line reference, which the
   connector model still preserves as raw XML. Keep a visible default line and
   a separate diagnostic for those two until typed connector style resolution
   is approved. Distinguishing tests cover straight, bent, curved, zero-extent,
   adjustment, arrowhead, and unknown-preset behavior without a new model,
   trait, module, or file. The completed connector evidence at
   `/private/tmp/f104-fidelity-after-connectors` contains 421 slides, 2,170
   source shapes, 2,170 resolved shapes, and zero dropped shapes. It reaches
   31 passing slides, or 7.363 percent, with minimum -0.097130275, median
   0.546938731, and maximum 1.0. Relative to the shadow baseline, 43 of 48
   changed slides improve and five degrade. Two slides cross the threshold and
   none cross below it. The largest gain is the aascu deck slide 29 at
   0.421063266. Connector geometry diagnostics fall from 91 to zero. One
   style-only connector retains its visible-default diagnostic.

   The next custom-geometry audit finds 18 evaluation fallbacks on five
   slides. One is inherited by `ArtisticEffectSample.pptx`, five are direct
   shapes on `customGeo.pptx` slide 6, and four inherited shapes repeat on
   three `themes.pptx` slides. Five direct paths omit both optional path
   coordinate dimensions. One inherited path omits its width. DrawingML uses
   the containing shape coordinate space for an omitted dimension, but the
   custom evaluator currently requires both dimensions and the layout scaler
   treats omission as coordinate size 1. The smallest first custom-geometry
   sub-batch passes the concrete shape size as the evaluator default and uses
   unit scale only on each omitted path axis. Focused tests distinguish both
   omitted dimensions, one omitted dimension, explicit dimensions, and a real
   evaluation error. The fallback diagnostic includes the stable evaluation
   error so any remaining unsupported geometry can be audited precisely. A
   direct evaluator run identifies the exact split as six missing-dimension
   errors and 12 `duplicate guide: connsiteX0` errors. The latter four source
   geometries contain ordered, repeated connection-site guide blocks and are
   accepted by PowerPoint. Guide formulas are an ordered name map, so a later
   ordinary guide replaces the earlier value. Keep duplicate rejection for
   adjustment declarations and keep unknown override rejection unchanged.
   Add a focused test in the existing evaluator module that proves a later
   ordinary guide definition is visible to path evaluation.
   Preserve the existing custom text-rectangle contract by evaluating it in
   the first path's effective coordinate space. A focused case uses an
   explicit 21,600 by 10,800 path inside a 10 by 20 point shape and verifies
   the scaled rectangle independently from the new omitted-axis behavior.
   The completed custom-geometry evidence at
   `/private/tmp/f104-fidelity-after-custom-geometry` contains 421 slides,
   2,170 source shapes, 2,170 resolved shapes, and zero dropped shapes. All 18
   evaluation fallbacks are eliminated. It remains at 31 passing slides, or
   7.363 percent, with minimum -0.097130275, median 0.546938731, and maximum
   1.0. Relative to the connector baseline, exactly five slides change and all
   five improve. The largest gain is `ArtisticEffectSample.pptx` slide 1 at
   0.001137326. There are no threshold crossings.

   The next gradient audit finds 99 `rotWithShape="0"` diagnostics. They are
   three template rectangles on each of 33 slides across `45541_Footer.pptx`,
   `45541_Header.pptx`, and `45545_Comment.pptx`. Direct package inspection
   finds five source rectangles per deck, with three on the master and two on
   the selected layout. All 15 are non-placeholders with zero own rotation,
   zero ancestor-group rotation, and no flips. Their fills are otherwise
   supported linear gradients at zero or 90 degrees. On an effectively
   unrotated and unflipped shape, independent and shape-relative rotation are
   identical. The smallest batch therefore passes these fills through the
   existing linear-gradient route without a new paint form. A nonzero own or
   group rotation, or a flip, retains the precise unsupported fallback.
   Focused layout tests distinguish an unrotated accepted shape from a rotated
   rejected shape before full corpus and unchanged-oracle evidence.
   The completed gradient-rotation evidence at
   `/private/tmp/f104-fidelity-after-gradient-rotation` contains 421 slides,
   2,170 source shapes, 2,170 resolved shapes, and zero dropped shapes. All 99
   diagnostics are eliminated. It remains at 31 passing slides, or 7.363
   percent, with minimum -0.097130275, median 0.554969615, and maximum 1.0.
   Relative to the custom-geometry baseline, exactly 33 slides change and all
   33 improve. The largest gain is `45545_Comment.pptx` slide 2 at
   0.096834794. There are no threshold crossings.

   The following OLE audit finds 37 unsupported instances on 36 slides across
   seven decks, backed by 16 unique source graphic frames. Four sources carry
   a standard fallback `p:pic`. Three embedded PNG previews cover 24 instances
   on 23 slides and can use the existing source-scoped image pipeline. One WMF
   preview covers one instance but is not supported by the current raster
   backend. Twelve source frames, covering the remaining 12 instances, have no
   preview. The bounded batch parses an optional `CT_Picture` next to retained
   raw OLE bytes in `rpptx-oxml`, while serialization continues to emit only
   the raw bytes. Layout renders a preview only when its embedded relationship
   resolves in the producing slide, layout, or master scope and the resolved
   media content type is PNG. A rendered preview emits a precise diagnostic
   that embedded OLE interactivity is not rendered. Missing previews, linked
   media, unresolved relationships, and unsupported media keep the existing
   visible bounds fallback and diagnostic. Focused tests cover raw round-trip,
   preview presence and absence, relationship scope, matching frame placement,
   and unsupported media. This batch adds no file or dependency. The completed
   evidence at `/private/tmp/f104-fidelity-after-ole` contains 421 slides,
   2,170 source shapes, 2,170 resolved shapes, and zero dropped shapes. All 24
   PNG-backed runtime instances render on 23 slides. The remaining 13 instances
   keep the bounds fallback, comprising 12 without previews and one WMF. All 23
   changed slides improve, none degrade, and there are no threshold crossings.
   The largest gain is `alterman_security.pptx` slide 1 at 0.102123705. The
   corpus remains at 31 passing slides, or 7.363 percent, with minimum
   -0.097130275, median 0.554969615, and maximum 1.0. All 421 oracle PNG hashes
   match the preceding unchanged-oracle evidence.

   A following tab-stop audit is deferred because its 19-slide, eight-deck
   source-chain count overstates visual reach. Only three slides contain tab
   characters, and their current SSIM scores are 0.4180, 0.4236, and 0.6750.
   Only `customGeo.pptx` slide 46 uses an explicit custom stop. The other two
   use a default interval. All 26 corpus custom stops have a position and
   alignment, are strictly increasing, and comprise 14 centre, 11 left, and
   one decimal alignment. DrawingML currently preserves `defTabSz` and
   `tabLst` only as raw XML. The neutral line breaker also adds its placeholder
   tab width before resolving the stop, resolves from the wrong horizontal
   position, and leaves the recorded line width at the placeholder value.
   Centre, right, and decimal stops currently behave like left stops. Any later
   tab batch must type whole-list replacement and scalar default-tab
   inheritance, split literal tabs before shaping, keep positions relative to
   the paragraph left margin, and correct neutral width accounting before
   connecting the existing tab-stop value. No tab code changes in this batch.

   The explicit-break audit is also deferred. It finds 106 direct-slide breaks
   on 40 slides and 128 unique selected-source breaks on 42 slides. All 106
   direct breaks carry `a:rPr`, 90 have visually relevant properties, and 88
   of those match the preceding direct run. The only two visual differences
   are a typeface declaration on `60810.pptx` slide 28 and a leading 3 point
   break on `smartart-simple.pptx` slide 1. Twenty-seven slides have a leading,
   consecutive, or trailing break that could affect empty-line metrics. A
   later batch may retain a resolved break style solely for those blank-line
   metrics. It must not cascade that style into an existing following run.

   The broader line-metric audit supersedes a break edit. Direct paragraph
   line spacing across unique slide, layout, and master parts is 9,445 omitted,
   501 percentage, and 96 point values. Direct slides contain 2,896 omitted,
   471 percentage, and 51 point values. Their selected-chain upper-bound
   presence is 421, 79, and 14 slides respectively. The deterministic Calibri
   substitute Carlito has 2,048 units per em, ascent 1,536, descent 512, and
   line gap 452. The old natural line advance discarded that 0.220703 em gap.
   Carry each font's line gap through the existing neutral segment and line
   structures. Use the tallest complete run advance for mixed-font lines.
   Omitted spacing uses the resulting natural advance. Percentage spacing uses
   the largest effective run point size, while exact point spacing stays exact.
   Divide non-negative leading equally before and after the glyph box, without
   clamping an exact height below natural. Apply stored `lnSpcReduction` only to
   explicit percentage spacing.

   The isolated `60810.pptx` representative changes 21 of 28 PNGs against the
   accepted OLE baseline. Twenty improve, one degrades by 0.010747790, and
   seven are byte-identical. Slides 3, 4, and 5 improve by 0.108262105,
   0.166818196, and 0.423360356. There are no threshold crossings. The row
   starts on slide 4 change from 157, 206, through 606 to 170, 231, through 720,
   while the oracle uses 184, 244, through 724. This proves both the corrected
   interval and the half-leading origin adjustment before full evidence.

   The completed natural-line evidence at
   `/private/tmp/f104-fidelity-after-line-gap` contains all 421 slides, 2,170
   source shapes, 2,170 resolved shapes, and zero dropped shapes. Relative to
   the accepted OLE baseline, exactly 366 slide PNGs change. Of those, 321
   improve and 45 degrade, while the remaining 55 corpus slides are
   byte-identical. Mean SSIM rises from 0.537943284 to 0.593093796 and median
   SSIM rises from 0.554969615 to 0.628401090. One slide crosses 0.95 upward
   and one crosses downward by 0.005747951, leaving 31 passing slides and
   7.363 percent coverage. Four slides lose at least 0.05 while 131 gain at
   least 0.05. All 421 regenerated oracle PNG hashes match the accepted
   evidence. The broad improvement and lack of a net threshold loss retain
   this batch.
6. Re-score and rank text, bullet, and line-layout differences on the remaining
   no-diagnostic slides. `60810.pptx` slide 18 demonstrates materially shifted
   title and bullet placement without a diagnostic. The first broad triage
   found that wrapped runs with non-one-to-one shaping used independently
   rounded glyph starts and lengths for each line-break chunk. Adjacent chunks
   could overlap or omit ligature glyphs. Reshape every chunk with the already
   selected font and size, retaining character spacing and exact glyph order.
   A direct-slide Carlito audit finds 282 potentially affected slides, including
   178 without diagnostics and 175 of those below 0.95. The same triage found
   one picture placeholder whose slide picture has media and crop but inherits
   its bounds from the layout. Resolve picture bounds in slide, layout, then
   master order while keeping media in the producing slide scope. The completed
   combined re-score at `/private/tmp/f104-fidelity-after-text-picture` remains
   29 passing slides, or 6.888 percent, with minimum -0.095068105, median
   0.543961644, maximum 1.0, and zero dropped shapes across 2,155 bounded and
   resolved shapes. The aascu representative improves from 0.024733089 to
   0.049620742 with glyph corruption removed, but a separate wrap-width
   mismatch remains. `layouts.pptx` slide 9 improves from 0.375543292 to
   0.931478945 after its large cropped image becomes visible.
   The next approved existing-file text batch resolves two broad typed gaps.
   First, carry `a:endParaRPr` through the existing paragraph style cascade and
   use its resolved size and font metrics for empty paragraphs and paragraph
   sizing instead of hardcoded fallback sizes. Preserve normal nonempty
   paragraph behavior. Second, parse, write, merge, and resolve
   `a:rPr/@cap="all"`, then uppercase its text before shaping. A corpus audit
   finds 203 slides with end-paragraph properties on empty paragraphs and 38
   slides with all-caps runs, all below 0.95. Audit `cap="small"` separately and
   do not claim support unless it is implemented. Shadows remain outside this
   batch. Initial combined evidence exposed an incorrect large bullet on an
   empty placeholder paragraph. The corrected renderer suppresses a bullet
   when a paragraph has no visible text or field while retaining its
   end-paragraph line metrics. The end-paragraph-only evidence at
   `/private/tmp/f104-fidelity-isolate-end-final` changes 141 slides, with 101
   improvements and 40 degradations but no threshold crossings. The blank
   `60042.pptx` slide 2 becomes an exact 1.0 match. The final combined evidence
   at `/private/tmp/f104-fidelity-after-end-caps-final` has 29 passing slides,
   or 6.888 percent, with minimum -0.095168692, median 0.566122775, maximum
   1.0, and zero dropped shapes across 2,155 bounded and resolved shapes.
   All-caps changes 37 slides relative to end-paragraph-only evidence. Of
   those, 27 improve and 10 degrade, with no threshold crossing. The pinned
   corpus has no `cap="small"` character properties, so that value remains
   preserved as unmodelled XML.

   A native PowerPoint 16.104 check of `aascu.org_workarea_downloadasset...`
   slide 17 confirms the corrected horizontal break after `with`. Rust scores
   0.07992 against PowerPoint while LibreOffice scores 0.04962 against Rust,
   and LibreOffice scores only 0.18671 against PowerPoint. The native result
   rejects adding a second 92.5 percent width contraction. Red-pixel row starts
   place Rust's first line 21 pixels, or 10.1 points, below PowerPoint. The
   first-paragraph difference is approximately the inherited 3.6 point top
   inset plus the first paragraph's 5.92 point `spcBef`. DrawingML controls
   that edge spacing with `a:bodyPr/@spcFirstLastPara`, whose absent default is
   false. Model, merge, and resolve that attribute. Suppress only first
   `spcBef` and last `spcAft` when false.

   Stored `lnSpcReduction="20000"` applies only to explicit percentage line
   spacing. Omitted Single and exact point spacing remain unchanged. A direct
   corpus audit finds 73 normal-autofit slides and none currently pass. Do not
   alter horizontal autofit scale.

   The combined spacing evidence at
   `/private/tmp/f104-fidelity-after-spacing` retains 29 passing slides, or
   6.888 percent, with minimum -0.095168692, median 0.540497119, maximum 1.0,
   and zero dropped shapes across 2,155 bounded and resolved shapes. Relative
   to the end-paragraph and caps baseline, 51 of 212 changed slides improve and
   161 degrade against LibreOffice, with no threshold crossing. The edge-only
   isolation at `/private/tmp/f104-fidelity-isolate-edge-only` has the same 29
   passing slides and zero dropped shapes, with median 0.543961644. It changes
   205 slides, of which 47 improve and 158 degrade. The stored line-spacing
   reduction changes another 27 slides, of which 10 improve and 17 degrade.
   This is a measured oracle disagreement rather than authority to discard the
   DrawingML rules. On the native PowerPoint 16.104 representative, Rust SSIM
   improves from 0.0799218 to 0.086834487 with edge suppression alone and to
   0.143245202 with both corrections. Against LibreOffice, the same sequence
   improves from 0.049620742 to 0.085430120 and then 0.136086151. Retain both
   standards-backed corrections and continue ranking other defects. Neither
   correction materially closes the automatic gate.

   The next approved fidelity batch lowers the already resolved outer-shadow
   effect into the existing neutral group effect list and implements its PNG
   raster approximation. It introduces no new schema or resolver model. The
   raster backend renders the group subtree once to a transparent layer, uses
   its composited alpha as the shadow silhouette, applies a deterministic blur
   at device scale, offsets it in the transformed coordinate space, tints it
   with the resolved colour and alpha, and composites it behind the unchanged
   group content. Group opacity applies once to both shadow and content. The
   PDF backend remains unchanged in this PNG fidelity batch. Focused tests pin
   effect propagation, behind-content order, offset, colour, blur extent, and
   group opacity.

   The completed shadow evidence at
   `/private/tmp/f104-fidelity-after-shadow` contains 421 slides, 2,155 bounded
   shapes, 2,155 resolved shapes, and zero dropped shapes. It retains 29
   passing slides, or 6.888 percent, with minimum -0.095168692, median
   0.543858773, and maximum 1.0. Relative to the retained spacing baseline, 41
   slides change. Of those, 39 improve and two degrade, with no threshold
   crossing. `customGeo.pptx` slide 6 has the largest gain at 0.014170722. The
   only losses are `60810.pptx` slides 11 and 25 at 0.006414395 each. The batch
   is strongly directional but does not materially close the gate.

   A resumed combined audit trialled natural-advance percentage spacing because
   `60810.pptx` slides 3 and 4 improved by 0.129136794 and 0.185882765, with no
   other change in that 28-slide deck. The full trial evidence at
   `/private/tmp/f104-fidelity-after-percent-natural.gY40Yo` contains 421
   slides, 2,170 bounded shapes, 2,170 resolved shapes, and zero dropped
   shapes. It reaches 31 passing slides, or 7.363 percent, with minimum
   -0.097206897, median 0.628401090, and maximum 1.0. The evidence records
   LibreOffice 26.2.5.2 and pdftoppm 26.01.0 exactly. ISO/IEC 29500-1 section
   21.1.2.2.11 defines percentage spacing from the largest run point size. The
   trial was rejected and reverted as a LibreOffice-only optimisation. It does
   not become accepted gate evidence.

   A native feasibility audit then re-exported the 34-slide ecodesign deck with
   the pinned Microsoft PowerPoint 16.104 application and pdftoppm 26.01.0.
   Slide 25 reproduces the recorded native PNG hash
   `85610d4b6778432355ab498f2a5da3bce6831cf502703d08caae70988307a49c`,
   which validates the existing native pipeline. Comparing native PowerPoint
   directly with the unchanged LibreOffice oracle produces zero of 34 slides
   at or above 0.95 SSIM, with minimum 0.207778837, median 0.650406194, and
   maximum 0.940934972. The restored Rust renderer against native PowerPoint
   also produces zero of 34 passing slides, with minimum 0.085036829, median
   0.462063463, and maximum 0.948686222. This does not excuse remaining
   renderer defects. It proves that 0.95 SSIM against LibreOffice cannot be a
   PowerPoint-conformance threshold because native PowerPoint fails every page
   in the representative deck.

   The user resolved the acceptance contract on 2026-08-08. SSIM remains a
   recorded trend at the existing 0.95 and 80 percent reference values. It is
   not a hard pass condition. The hard automatic gate requires all 50 decks and
   every slide to render without panic, missing output, dimension mismatch, or
   a dropped bounded shape. Exact LibreOffice and pdftoppm versions remain
   mandatory so the trend stays comparable. Native PowerPoint review remains a
   hard recorded acceptance gate and takes precedence when LibreOffice
   disagrees. The CI job runs the full comparison, writes its evidence, and
   succeeds below the trend reference when all hard conditions pass.

   The post-contract exact-tool evidence at
   `/private/tmp/f104-final-trend-check-20260808` contains all 50 decks and 421
   slides with zero dropped shapes. It records 30 slides at or above 0.95 SSIM,
   or 7.126 percent, with minimum -0.097206897, median 0.622465305, maximum 1.0,
   and `trend_target_met` false. The command exits successfully. Both
   evidence-backed external assertions execute and pass against the retained
   gate JSON.
7. Do not enter chart or SmartArt implementation under F-104. Those graphics
   categories overlap work already assigned beyond the M10 contract. Keep
   unsupported content visible and diagnosed for its owning story.

## Rejected alternatives

- Use system fonts for convenience. The resulting scores would not be
  reproducible across machines.
- Commit oracle PNGs. The corpus is external, binary fixtures are prohibited,
  and LibreOffice output belongs to the pinned executable version.
- Put the harness inside a published crate. The external oracle is test
  infrastructure and must not become a crate dependency.
- Compare only the first slide of each deck. The gate is defined over every
  corpus slide.
- Fail CI only because fewer than 80 percent of slides reach 0.95 SSIM. Native
  PowerPoint reaches zero percent against the pinned LibreOffice oracle on the
  34-slide calibration deck, so that policy would enforce oracle-specific
  pixel parity rather than PowerPoint fidelity.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `identical_images_have_ssim_one` | The metric gives exactly 1.0 for equal decoded pixels |
| unit | `pixel_changes_lower_ssim` | A controlled difference produces a stable lower score |
| regression | `dimension_mismatch_is_a_hard_failure` | Misaligned oracle output cannot be scored or hidden |
| unit | `trend_target_classifies_eighty_percent` | The recorded trend uses the exact 0.95 and 80 percent reference boundaries |
| unit | `missed_ssim_trend_is_recorded_without_changing_completeness` | A missed trend is evidence, while zero dropped shapes remains independently hard |
| integration | `all_corpus_slides_render_without_panic_or_dropped_shape` | The hard half of the backlog gate over the required 50-deck corpus |
| differential | `corpus_render_fidelity_records_ssim_trend` | Every slide receives a score and the evidence records whether the trend reference was met |
| manual | Pinned PowerPoint representative review | Native output opens without repair and arbitrates material LibreOffice disagreements |

The automatic hard gate is complete 50-deck rendering without panic, missing
output, dimension mismatch, or a dropped bounded shape. The SSIM reference is
at least 0.95 on at least 80 percent of slides and is recorded as a trend. The
manual hard gate is the pinned native PowerPoint representative review.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/07-inheritance-and-resolution.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout, line breaking, text shaping: read
  `docs/hld/08-rendering-spec.md`. The corpus driver uses deterministic fonts
  exclusively, and no system-font baseline is recorded.
- External oracle comparison: read
  `.claude/skills/differential-testing.md`. Pin and record the exact
  LibreOffice version, 150 dpi raster tool version, metric definition, and
  triage for every known divergence.
- New files: read the structural rules in `CLAUDE.md`. The two proposed files
  have distinct existing consumers today, the run-sprint gate and the CI
  fidelity job. Explicit approval is required before creation.
- Crate dependency graph: read `docs/hld/03-architecture.md`. Keep the oracle
  outside every crate and add only existing one-way development dependencies
  needed by the unpublished example.

Extra checks are the harness unit tests, required 50-deck corpus run, exact
oracle-version assertion, deterministic-font assertion, and the M10 manual
PowerPoint spot-check.

## Hash harness

Expected to be unchanged. The existing 28-entry Word harness does not consume
PresentationML rendering.

## Implementation checklist

- [x] Add a deterministic whole-presentation layout entry point without
  changing normal font discovery.
- [x] Add the approved whole-deck corpus renderer and source-to-resolved shape
  accounting.
- [x] Add the approved version-pinned LibreOffice SSIM harness and unit tests.
- [x] Record 150 dpi, 0.95 SSIM, and 80 percent coverage as the comparable trend
  reference, while enforcing 100 percent complete rendering.
- [x] Add the required CI trend and completeness job with pinned tools and
  corpus.
- [x] Triage below-reference slides, retain specification-correct remediation,
  and classify material LibreOffice disagreements against native PowerPoint.
- [x] Record the representative M10 PowerPoint spot-check.

## Open questions

None. The two named harness files are approved, as are the exact pinned
LibreOffice and 150 dpi raster tool versions. The user selected trend-only SSIM
paired with hard complete-rendering and recorded native PowerPoint acceptance.

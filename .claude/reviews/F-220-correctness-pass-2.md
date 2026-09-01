# F-220, correctness, pass 2

**Reviewed**: claim-base `3d87d827351508b38ffb22103f26351c3f18c0ca` through exact source head `4cfcd95b470ed1248bba821aa40cb3a7717ba69e`, 6 files and 3,040 changed lines (2,986 additions, 54 deletions), plus the untracked pass-1 review, approved revised plan, cited HLD sections, F-219 contract, progress record, and full tracked and untracked state
**Verdict**: 6 defects, 0 smells, 0 nitpicks. Not ready because the defects remain and the external PowerPoint completion evidence is absent.

## Defects

### D1, a cached drawing failure blocks the supported native layout path
`crates/rpptx/src/diagram.rs:171`

When a data model advertises a cached drawing id, package assembly stores a
missing, wrong-type, or malformed drawing as an error at
`crates/rpptx/src/lib.rs:6942`. `render_group` propagates that error before it
validates or lays out the data, layout, style, and colour parts. A supported
diagram with four usable native resources therefore becomes the labelled
unsupported fallback merely because its optional cache is unavailable. The
approved differential contract says cached `dsp:drawing` is not required and
remains outside the render path, so this is still a render-path dependency
rather than the pass-1 remediation.

### D2, accepted selectors and algorithm trees are flattened without being evaluated
`crates/rpptx/src/diagram.rs:264`

`validate_layout_projection` accepts every `forEach` whose attribute names are
in a broad allow-list, but no later code consumes `projection.selectors`.
Algorithm parameters are likewise searched across the flattened algorithm
vector and the last matching name wins at `crates/rpptx/src/diagram.rs:1182`,
without retaining or validating the algorithm or selector that owns it. For
example, a `forEach ptType="asst"` around an allowed algorithm is accepted and
then applied to every presentation node. Multiple allowed algorithms with
different scoped parameters are combined into one plausible family layout.
Those trees must either be mapped with their declared selection and ownership
semantics or fail closed.

### D3, absence of a presentation graph silently re-enables data-node layout
`crates/rpptx/src/diagram.rs:334`

If the data model contains no presentation points, `presentation_graph`
returns ordinary data and assistant points plus their `parOf` edges as the
render graph. The revised contract requires the data and presentation graphs
to be validated separately, with `presOf` mapping authoritative data text to
the presentation shape owner and missing ownership failing closed. A data-only
model therefore renders a plausible supported diagram instead of the required
visible unsupported fallback.

### D4, style labels remain detached from their schema owners
`crates/rpptx/src/diagram.rs:221`

The OXML projection appends every nested `layoutNode@styleLbl` to one flat
vector at `crates/rpptx-oxml/src/diagram.rs:1174`. The renderer then assigns
those labels by presentation-node index, repeats the last label for remaining
nodes, and chooses a connector label by a name substring. A root label and a
child or assistant label under a selector are therefore applied by point-list
position rather than by the layout node that selected the shape. Quick-style
and colour resolution now use the shared shape engine, but pass-1 D4's
schema-owner selection requirement is not fixed.

### D5, constraints and rules ignore their target semantics
`crates/rpptx/src/diagram.rs:819`

All flattened constraints and rules are processed through the same four-field
numeric mutation and applied to every rectangle. Target attributes such as
`for`, selector ownership, reference type, and operation are retained by the
projection but ignored here. In addition, both `sp` and `sibSp` always shift
the x coordinate at `crates/rpptx/src/diagram.rs:862`, including a vertical
list. An accepted schema-owned record can consequently modify the wrong nodes
or axis, or fall back for out-of-bounds geometry, instead of applying its
declared semantics or being rejected as unsupported.

### D6, transient expansion discards the authoritative frame rotation and flips
`crates/rpptx/src/diagram.rs:1113`

`frame_bounds` reads only the frame offset and extent. `render_group` replaces
the graphic frame with a new empty identity group at
`crates/rpptx/src/diagram.rs:174`, and every generated child receives an
absolute axis-aligned transform. A supported SmartArt frame with `rot`,
`flipH`, or `flipV` therefore renders unrotated and unflipped. This violates
the explicit contract that the outer graphic-frame transform remains
authoritative.

## Smells

None.

## Nitpicks

None.

## External completion evidence

The missing six-family PowerPoint oracle is not a defect, smell, or nitpick in
the current harness. Ordinary mode intentionally skips with the exact absent
manifest path at `crates/rpptx/tests/integration.rs:207`, while required mode
fails closed at `crates/rpptx/tests/integration.rs:211`. The progress record
accurately states that no trusted manifest or six PowerPoint artifacts exist at
`.claude/scratch/F-220-progress.md:64`. Under the differential-testing policy,
this is an external feature-completion blocker. It supplies no evidence that
the six formulas match pinned PowerPoint within the approved 1 point and 0.99
SSIM thresholds, so this review does not infer that fidelity from the green
skip or manufacture a source finding for the absent files.

## Not found

- **Panics and bounds**: Pass-1 D6, D7, and D8 are corrected. Sub-point resize,
  dense geometry, non-finite values, node and connection counts, graph depth,
  and layout-projection depth now fail closed. No reachable panic from the
  reviewed untrusted XML paths was found.
- **OOXML namespaces, schema positions, and raw preservation**: Alias prefixes,
  fixed-prefix shadows, foreign lookalikes, direct schema positions, raw-byte
  serialization, colour choice kinds, ordered transforms, and bounded
  projection traversal remain covered. The focused OXML projection regressions
  passed 3 tests.
- **Six-family bounded mechanics**: List, hierarchy assistant lanes, cycle
  span and direction, relationship cardinality, matrix title bands, and
  weighted pyramid tiers have dedicated checked paths. The 3 private diagram
  unit tests passed. No additional arithmetic or containment defect was found
  beyond D2 and D5. PowerPoint fidelity remains unproven for the evidence
  reason above.
- **Shared text and colour engines**: Complete `CT_TextBody` values and ordinary
  PresentationML style references now enter the existing resolver. No separate
  text shaper or colour-transform implementation remains in the SmartArt path.
- **Producing scope and facade reuse**: Slide, layout, and master resource maps
  remain separate. Static, timeline, media, and animation assembly reuse the
  transiently expanded clones. No cross-scope relationship-id alias was found.
- **Public API and structure**: Pass-1 D9 is corrected. SmartArt resource
  carriers and expansion are private to `rpptx`, the unapproved
  `rpptx-layout` surface is removed, and no new trait, generic, feature, crate,
  dependency, dynamic dispatch, or integration binary was added.
- **Differential harness sensitivity**: Pass-1 D10's row schema, exact slide,
  diagnostics, hashes, and artifact completeness checks are present. Pass-1
  D11 is corrected because the negative test now perturbs actual resolved
  SmartArt geometry and a rendered PNG through the shared comparator. The
  focused sensitivity test passed.
- **Validation evidence**: `cargo check -p rpptx --all-targets` passed with
  three pre-existing F-221 test warnings. `cargo test -p rpptx --lib
  diagram::tests` passed 3 tests. The focused `rpptx-oxml` projection run passed
  3 tests. The SmartArt negative-sensitivity test passed. The ordinary
  differential test passed only by its documented missing-manifest skip, and
  the same exact test with `RDOCX_PPTX_CORPUS_REQUIRED=1` failed at the absent
  manifest before comparing any family. `git diff --check` passed.

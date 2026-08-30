# F-214, all, pass 1

**Reviewed**: complete working tree against the F-214 worker base, 12 files,
2,153 additions and 28 deletions, including both approved untracked timeline
modules
**Verdict**: 13 defects, 0 smells, 0 nitpicks

## Defects

### D1, scale and spin use the page origin and do not compose

`crates/rpptx-layout/src/timeline.rs:345`

Scale changes only the diagonal coefficients and spin replaces the complete
timeline transform with a rotation about `(0, 0)`. The resolver appends that
matrix after the ordinary shape translation at
`crates/rpptx-layout/src/context.rs:3011`, so a scale or spin moves the shape
around the page origin instead of preserving the target shape centre. Spin
also discards an earlier scale or motion transform on the same target. Parallel
supported emphasis effects therefore do not compose into the required finite
shape-local transform.

### D2, motion paths ignore their coordinate origin

`crates/rpptx-layout/src/timeline.rs:417`

The evaluator parses raw path coordinates and writes them directly into the
point-valued translation coefficients. It never reads `TimingMotionPath::origin`
and never resolves the coordinates against slide or target geometry. A path
whose coordinates are relative to the layout or target is therefore treated as
an absolute point offset. The endpoint test uses already point-like values, so
it does not exercise the OOXML coordinate conversion required by the plan's
unit-routing rider.

### D3, a shape wipe is rendered as a fade

`crates/rpptx-layout/src/timeline.rs:372`

The effect evaluator accepts any filter containing `wipe`, then applies the
same opacity interpolation used by fade at lines 399 to 405. It ignores the
wipe direction and never produces a geometric reveal. This claims support for
the approved wipe entrance and exit slice while rendering a different effect.

### D4, alternative start conditions are added together

`crates/rpptx-layout/src/timeline.rs:176`

Every finite delay in `start_conditions` is added to the start time. A timing
condition list supplies alternative triggers, so the active start is selected
from the condition that fires rather than the sum of all listed delays. A node
with two supported delayed conditions consequently starts later than either
declared trigger.

### D5, morph does not interpolate shape extents

`crates/rpptx-render/src/timeline.rs:275`

Compatibility checks only the geometry enum discriminant and finite bounds.
The composed element then clones the incoming group's children and interpolates
only the top-level group transform at lines 297 to 301. Width and height are
already materialized in those child paths and text boxes, so they jump to the
incoming extent at progress zero instead of interpolating from outgoing to
incoming bounds as the approved morph contract requires.

### D6, hidden timeline shapes corrupt morph identity mapping

`crates/rpptx-render/src/timeline.rs:221`

Morph assumes the final `identities.len()` page elements correspond one for one
with every identity. Timeline rendering removes invisible shapes at
`crates/rpptx-render/src/lib.rs:368`, while the identity and resolved-shape
vectors retain them. Once any incoming shape is invisible, the computed offset
and subsequent identity index can select the wrong rendered element, omit a
visible element, or pair one shape with another.

### D7, unmatched incoming morph names have no diagnostic

`crates/rpptx-render/src/timeline.rs:312`

Unused incoming shapes are crossfaded without checking whether their explicit
name starts with `!!` and without emitting a diagnostic. An explicit incoming
morph candidate with no outgoing match therefore violates the contract that an
unmatched shape crossfades and emits a stable diagnostic.

### D8, the facade drops resolver diagnostics

`crates/rpptx/src/lib.rs:780`

The returned diagnostic list includes timing-state and transition-composition
messages only. `incoming.slide.diagnostics`, which carries group, geometry,
media, chart, and text resolution approximations for the actual evaluated
slide, is not included. The static assembly has those messages in its
`LayoutResult`, but that result is private to this method. A timeline caller
therefore receives a page without the diagnostics that explain visible
fallbacks on it.

### D9, invalid numeric timing values escape in public state

`crates/rpptx-layout/src/timeline.rs:431`

Rust floating-point parsing accepts non-finite spellings such as `NaN`, and the
evaluator stores the resulting opacity or transform in `state.shapes`. The
resolver checks a parallel copied value at
`crates/rpptx-layout/src/context.rs:3010` and can reset the rendered copy, but it
does not sanitize or reject the public `EvaluatedFrameState`. The facade can
therefore return a non-finite evaluated state even when rendering ignored it,
contrary to the finite-state contract.

### D10, the geometry tolerance gate trusts a generated constant

`crates/rpptx/tests/integration.rs:5410`

The oracle generator writes `geometry_error_pt` as literal zero for every
frame. The differential test reads that manifest field and checks it against
one point at `crates/rpptx/tests/integration.rs:5546`, but never measures Rust
geometry against PowerPoint geometry. Any implementation, including one with a
large bounds error, passes this part of the declared gate as long as the
manifest retains the generated zero.

### D11, the generator does not verify the claimed AppleScript build

`crates/rpptx/tests/integration.rs:5317`

Oracle generation calls `assert_powerpoint_build`, whose implementation at
`crates/rpptx/tests/integration.rs:6369` checks only the Info.plist version and
bundle build. It never queries PowerPoint's AppleScript `build` value, yet the
manifest records the constant app build `1214`. The generated evidence can
therefore claim the approved second build identity without having run that
identity.

### D12, the required evaluator tests do not exercise their named cases

`crates/rpptx-layout/src/timeline.rs:542`

The sequence test contains no parallel container and no `WithEffect` node even
though its name and the approved test plan require parallel, sequence, click,
with-previous, and after-previous intervals. The boundary test at
`crates/rpptx-layout/src/timeline.rs:575` checks only the exact end timestamp.
It does not cover exact start, after end, remove fill, appear, wipe direction,
or a multi-segment motion path. These omissions leave the central scheduling
and boundary contract unproved and allow D1 through D4 to pass the focused
suite.

### D13, F-214 expands an unapproved published crate surface

`crates/rpptx-oxml/src/shape_tree.rs:60`

`ShapeTreeChild::non_visual_name` is a new public method in the published
`rpptx-oxml` crate. The approved plan limits F-214's public additions to
`rpptx-layout`, `rpptx-render`, and `rpptx`, and its package riders likewise
name only those three crates. This extra public API is outside the reviewed
surface and has no corresponding semver or package evidence in the contract.

## Smells

None.

## Nitpicks

None.

## External evidence blocker

The PowerPoint oracle directory has source and failed-attempt presentation
artifacts but no `manifest.tsv` or pinned PNG frames. The ignored generator's
PowerPoint movie export failed externally, so the exact differential gate has
not run and must not be reported as passed. Required-corpus mode failing on the
missing manifest is correct fail-closed behavior. This external evidence gap is
separate from D10 and D11, which are defects in the gate even after artifacts
become available.

## Not found

No additional panic path was found in production code. The two production
`unreachable!` sites are guarded by their callers, and the motion-path
`expect` is protected by the parsed point-count check. No schema write-order,
namespace replay, raw-XML preservation, crate dependency, new trait, generic,
feature flag, backend animation variant, runtime oracle dependency, binary
fixture, or static-path coupling defect was found. Target ids use
`p:cNvPr/@id`, group lineage is retained for slide descendants, and layout and
master identities are excluded from the slide timing id scope. The ordinary
static methods remain on the no-timeline branch of the shared assembly path.

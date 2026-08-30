# F-214, all, pass 2

**Reviewed**: complete current working implementation diff against the F-214
worker base, 7 source and test files, 2,856 additions and 16 deletions,
including both approved untracked timeline modules, plus the pass 1 review
record
**Verdict**: 10 defects, 1 smell, 0 nitpicks

## Defects

### D1, the default parent motion origin is resolved against the target bounds

`crates/rpptx-layout/src/timeline.rs:647`

The default `parent` origin and the non-schema `target` token both become
`MotionOrigin::Target`. The resolver then multiplies the path coordinates by
the leaf target width and height at lines 92 to 98. PowerPoint's `parent`
origin is relative to the slide parent, while `layout` uses the object-relative
motion convention. A normal path with omitted origin therefore moves by a
fraction of the target size instead of the slide size. The focused regression
at `crates/rpptx-layout/src/context.rs:3737` asserts this incorrect target-size
result, so it cannot detect the defect. The pass 1 D2 motion-origin issue is not
closed.

### D2, relative motion path commands are evaluated as absolute coordinates

`crates/rpptx-layout/src/timeline.rs:686`

The parser removes uppercase and lowercase `M` and `L` tokens, then groups every
remaining pair as an absolute point. Lowercase `m` and `l` commands are
relative in the OOXML motion-path grammar. A path such as
`M 0 0 l .2 .1 l .2 .1 E` should finish at `(.4, .2)`, but this evaluator
finishes at `(.2, .1)`. The approved line and polyline slice therefore produces
wrong endpoints for valid relative paths.

### D3, `ppt_x` and `ppt_y` position animations resize the target

`crates/rpptx-layout/src/timeline.rs:475`

The generic animate evaluator maps `ppt_x` and `ppt_y` into `scale_x` and
`scale_y`. Those attributes describe the shape's horizontal and vertical
position, not its scale. Valid position animations consequently resize a shape
around its centre and never translate it. If position animation is outside the
bounded first slice, these attributes must be diagnosed as unsupported rather
than assigned different semantics.

### D4, unsupported condition targets are accepted as supported triggers

`crates/rpptx-layout/src/timeline.rs:290`

`TimingTarget::Unsupported` is classified as supported. The F-213 projection
uses that same variant for an omitted target and for actual unsupported
targets, including sound, ink, and runtime-node targets. An unsupported
shape-event condition can therefore start an animation without the stable
diagnostic required by the plan. This also makes the supported-sibling
regression unable to distinguish a targetless condition from an unsupported
targeted condition.

### D5, finite animate endpoints can still produce non-finite public state

`crates/rpptx-layout/src/timeline.rs:465`

The endpoint check rejects literal non-finite values, but interpolation uses
`from + (to - from) * progress` without checking the result. Opposite finite
extremes overflow `to - from`, and `from + by` can overflow at line 462. The
public `evaluate_timeline` function returns the resulting state directly at
line 180. Resolver-side sanitisation cannot protect direct callers, so the pass
1 D9 finite-state contract is not fully closed.

### D6, group-target geometry is applied independently around each leaf

`crates/rpptx-layout/src/context.rs:3059`

The resolver copies every containing-group state into each descendant, then
calculates scale, rotation, motion, and clip against that descendant's original
bounds. A group scale or spin therefore transforms each leaf around its own
centre instead of transforming the descendants around the group coordinate
space. Parent-relative group motion can also move differently sized leaves by
different distances, and a group wipe is intersected leaf by leaf instead of
revealing the group. The focused group test at
`crates/rpptx-layout/src/context.rs:3644` covers opacity only, so it does not
exercise the required group transform semantics.

### D7, ordinary static resolution now executes timeline-only identity work

`crates/rpptx-layout/src/context.rs:449`

Every existing static `resolve_slide*` path now calls the identity-producing
resolver and discards the identities. For every resolved shape this performs a
group-lineage search and serialises the source shape to recover its name at
lines 538 to 565. The approved contract requires every existing method to stay
independent of timeline execution, not merely to render the same pixels. The
static regression at `crates/rpptx/tests/integration.rs:5483` compares output
only and cannot detect this coupling or its per-shape cost.

### D8, morph compatibility treats every pair of custom paths as compatible

`crates/rpptx-render/src/timeline.rs:280`

Compatibility compares only the `ResolvedGeometry` enum discriminant and
finite positive bounds. All custom geometries share one discriminant even when
their path counts, commands, fill rules, and text rectangles differ. Such a
pair bypasses the required incompatible-geometry crossfade and diagnostic,
then clones and scales the incoming path group as though its geometry matched
the outgoing shape.

### D9, the outgoing morph state and outgoing page describe different frames

`crates/rpptx/src/lib.rs:4607`

The facade resolves the outgoing timeline state at timestamp zero, but obtains
the outgoing page from the static assembly page at lines 761 to 763. Morph then
uses bounds from the evaluated outgoing state with transforms and children
from the static outgoing page. If an outgoing shape has an entrance, scale,
spin, motion, or wipe active at zero, interpolation begins from mismatched
geometry and visual content. Progress zero is therefore not a coherent
outgoing endpoint.

### D10, `none` and invalid effect transitions are treated as entrances

`crates/rpptx-layout/src/timeline.rs:503`

The evaluator distinguishes only the exact string `out`. Every other value,
including the valid `none` token and an unrecognised token retained by the raw
projection, follows the entrance branches at lines 518 to 553. Unsupported
transition modes therefore hide the target before the effect and animate it in
without a diagnostic.

## Smells

### S1, morph-name recovery serialises and string-parses OOXML

`crates/rpptx-layout/src/context.rs:2948`

Name recovery serialises each complete shape, searches the byte string for a
`p:cNvPr` tag, slices a quoted attribute manually, and implements a second XML
entity decoder. This duplicates XML parsing rules outside the established
namespace-aware pull parser and silently converts any serialization or decode
failure into an absent morph name. It is both a structural breach of the OOXML
boundary and a fragile failure mode for explicit `!!` correlation.

## Nitpicks

None.

## Pass 1 closure

- D1, D3 through D8, and D10 through D13 are materially addressed in the
  current code. Scale and spin compose, wipe has shape-local clips, condition
  alternatives select a minimum, morph interpolates extents, hidden slots are
  retained, incoming explicit names diagnose, resolver diagnostics reach the
  facade, the oracle measures pixels and queries the AppleScript build, the
  named regressions were expanded, and the unapproved `rpptx-oxml` public method
  was removed.
- D2 is only partially addressed because D1 above resolves `parent` with the
  wrong coordinate space and D2 above mishandles relative path commands.
- D9 is only partially addressed because literal non-finite values are rejected
  and the resolver sanitises copied state, but D5 above still permits arithmetic
  overflow in the public evaluator.
- Focused checks passed: 4 `rpptx-layout` timeline unit tests, the target
  geometry regression, 4 `rpptx-render` timeline unit tests, the hidden-slot and
  local-clip regression, and 3 filtered `rpptx` timeline integration tests. No
  broad test command was run. The pinned PowerPoint differential was not among
  these checks.

## External evidence blocker

The PowerPoint oracle directory still has no `manifest.tsv` or pinned PNG
frames. GUI automation failed, so the exact PowerPoint differential gate has
not run and must not be reported as passed. Required-corpus mode correctly
fails closed when the manifest is absent. The non-GUI 50-deck SSIM rider was
interrupted and is also unclaimed. These missing external results are separate
from the implementation defects above.

## Not found

No additional production panic path was found. The new `unreachable!` and
`expect` sites are guarded by their callers. No schema write-order, namespace
replay, retained raw-XML, crate dependency, new trait, generic, feature flag,
backend animation variant, runtime oracle dependency, binary fixture, or
unapproved public-surface defect was found in the current diff. Slide targets
still use `p:cNvPr/@id`, group lineage is retained, and master and layout
identities remain excluded from the slide timing scope. The two timeline module
files remain the approved structural additions.

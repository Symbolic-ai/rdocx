# F-214, all, pass 4

**Reviewed**: complete stable pass 3 remediation diff against the F-214 worker
base, 14 implementation files, 4,075 additions and 35 deletions, including both
approved untracked timeline modules, the pass 3 review, the approved OXML name
accessor context, and the current progress record
**Verdict**: 6 defects, 0 smells, 0 nitpicks

## Defects

### D1, container end boundaries do not constrain their children

`crates/rpptx-layout/src/timeline.rs:336`

Parallel and sequence nodes evaluate every child against its own natural
interval before `declared_end` calculates the container's finite duration or
end condition. The calculated container end is returned to the surrounding
scheduler, but it is never passed into the child evaluation. A parallel node
that ends at 200 ms with a 1,000 ms fade child therefore still evaluates that
child at 500 ms, even though the containing active interval has ended. Pass 3
D2 is closed for leaf nodes but remains open for container nodes.

### D2, targetless conditions change the completed F-213 public projection

`crates/rpptx-oxml/src/timing.rs:1228`

`crates/rpptx-oxml/src/timing.rs:1280`

The F-213 model represented a condition with no target as
`TimingTarget::Unsupported`. F-214 now publishes `TimingTarget::Slide` for the
same XML, and the two existing integration expectations were changed to accept
that new value. Restoring the `TimingCondition::target` field type does not
restore its observable contract. The approved F-214 plan says to consume the
exact integrated F-213 values, while the user's only approved new public OXML
surface is `ShapeTreeChild::non_visual_name`. Pass 3 D5 therefore remains open
as a public model behavior change rather than a type change.

### D3, group wipes lose their oriented boundary for rotated descendants

`crates/rpptx-layout/src/context.rs:3316`

The group reveal rectangle is mapped into each descendant's local space and
then reduced with `transform_rect_bbox`. When a descendant has its own rotation
relative to the group, the mapped group rectangle is not axis aligned in that
shape's local space. Its bounding box includes pixels outside the group wipe,
so the shape is revealed early. The focused regression rotates the group but
not either child, which lets the shared group rotation cancel and cannot expose
this case. Pass 3 D4 is closed for the group extent, centre, scale, spin, and
motion, but not for the required oriented group clip.

### D4, compatibility-wrapped charts have no timeline target or morph identity

`crates/rpptx-layout/src/context.rs:597`

`crates/rpptx-oxml/src/shape_tree.rs:55`

The static resolver renders a chart-bearing `mc:AlternateContent` through its
selected `CT_GraphicFrame`, but timeline identity is taken from the outer
`ShapeTreeChild::AlternateContent`. That wrapper returns no non-visual id and
no name even when the rendered chart choice owns both. A timing node targeting
the chart's `p:cNvPr/@id` therefore has no matching resolved leaf, and an
explicit `!!` chart name cannot participate in bounded morph matching. Group
lineage around the wrapper is retained, but the leaf identity required by the
contract is dropped.

### D5, invalid transition directions execute supported fallback semantics

`crates/rpptx-render/src/timeline.rs:165`

`transition_direction` forwards any producer string. Wipe and push treat every
unrecognised value as the left case, while zoom treats every value other than
`out` as `in`. No diagnostic is emitted. For example, a typed wipe with
`dir="diagonal"` executes a supported wipe instead of remaining an unsupported
parameter with a stable diagnostic. The focused transition regression supplies
no parameters and asserts only that two finite groups exist, so it cannot
detect this semantic fallback.

### D6, the differential oracle does not cover timeline evaluation

`crates/rpptx/tests/integration.rs:5384`

`crates/rpptx/tests/integration.rs:5661`

The PowerPoint generator captures exactly one fade transition frame and one
morph frame. Its source deck contains no timing tree, and the gate requires
only one case name containing `transition` and one containing `morph`. A
manifest with those two rows can pass without exercising appear, exit, wipe,
opacity, scale, spin, motion, sequences, parallel groups, clicks, fill, push,
zoom, or exact timing boundaries. This is the story's declared differential
test gate, so it must be capable of detecting errors in the timeline evaluator
as well as the two compositors it currently samples.

## Smells

None.

## Nitpicks

None.

## Pass 3 closure

- Pass 3 D1 is closed. Parent motion places the target centre in slide space,
  and layout motion adds slide-percentage offsets from the target centre.
- Pass 3 D2 remains partial as D1. Leaf end conditions are effective and
  unsupported end triggers diagnose, but container ends do not bound children.
- Pass 3 D3 is closed. An unresolved indefinite leaf advances a sequence to
  `u64::MAX`, so later sequence children do not start.
- Pass 3 D4 remains partial as D3. Explicit group extents and composed nested
  coordinate spaces are used, but rotated descendant wipes still reduce an
  oriented clip to an axis-aligned bounding box.
- Pass 3 D5 remains partial as D2. The exact public field type is restored, but
  targetless XML now yields a different public enum value.
- Pass 3 D6 is closed. No `quick-xml` production edge or lockfile change
  remains.
- Pass 3 D7 is closed. Compatible object morphs retain and crossfade both
  endpoint contents, including the exact outgoing endpoint at progress zero.
- Pass 3 D8 is closed. `byWord` and `byChar` crossfade with stable diagnostics.
- Pass 3 D9 is closed for valid `in` and `out` zoom values. Invalid values are
  the adjacent defect D5.
- Pass 3 D10 is closed within the fixed facade contract. The outgoing slide is
  evaluated at the terminal position rather than timestamp zero.
- Pass 3 D11 is closed. Outgoing resolver and evaluator diagnostics are
  returned beside incoming and compositor diagnostics.
- Pass 3 D12 is closed. Shape effect filters match only exact supported tokens.
- Pass 3 S1 is closed. Name recovery performs no timeline serialization or XML
  reparsing.

## Focused evidence

- The OXML name parse and mutation regression passed.
- Both focused F-213 condition-target integration regressions passed with the
  changed expectations described in D2.
- Seven evaluator unit tests passed.
- Three resolver regressions for group targeting, motion origins, and explicit
  group geometry passed.
- Eight transition and morph unit tests passed.
- Five facade regressions passed for timeline lowering, terminal outgoing
  morph state and diagnostics, static isolation, and oracle source round trip.
- Optional oracle mode returned only because `manifest.tsv` is absent.
  Required mode failed closed at that exact missing path, as required.
- `git diff --check` passed. No broad test command ran.

## External evidence blocker

The PowerPoint oracle directory still has no `manifest.tsv` and no PNG frames.
The prior GUI automation attempts did not produce a movie or a slide-show
capture. The required pinned PowerPoint differential has not run and is not
passed. The full 50-deck SSIM rider was interrupted and remains unclaimed.
These external evidence gaps are separate from D1 through D6.

## Not found

No additional production panic path was found. No schema child-order,
namespace binding, retained raw-XML, reverse dependency, new dependency,
unapproved trait, generic, feature flag, crate, backend animation variant,
runtime oracle dependency, or binary fixture defect was found. Ordinary static
resolution still avoids timeline evaluation and identity recovery. The new
name cache is decoded during normal parsing, initialized by the ordinary
constructors, synchronized by every existing `set_name` method, preserved by
clone and round trip, and exposed through exactly the approved
`ShapeTreeChild::non_visual_name(&self) -> Option<String>` OXML addition. Slide
targets remain isolated from layout and master identity scopes, and ordinary
group lineage is retained. Valid transition and morph endpoint composition is
finite. The required oracle mode is fail-closed, but the absent artifacts
provide no fidelity evidence.

# F-214, all, pass 5

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 4,753 additions and 30 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 4 review, and the two explicit OXML API approvals
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, parent group clips inherit descendant animations

`crates/rpptx-layout/src/context.rs:3063`

`crates/rpptx-layout/src/context.rs:3090`

`crates/rpptx-render/src/lib.rs:390`

A group-target wipe is converted into the original leaf's local coordinates,
then the leaf's direct motion, scale, and spin and every descendant group
animation are applied to that leaf. The renderer finally installs the saved
clip inside the already animated leaf group. The clip therefore moves, scales,
or rotates with the descendant instead of remaining fixed in the targeting
group's coordinate space. The new regression proves the polygon is oriented
for a statically rotated child, but it does not combine a parent wipe with a
child animation or an animated nested group. Such combinations reveal pixels
outside the parent wipe boundary. Pass 4 D3 is closed for static descendant
orientation but not for supported composed animations.

### D2, the differential gate does not bind the fetched source to the approved generator

`crates/rpptx/tests/integration.rs:5800`

`crates/rpptx/tests/integration.rs:5906`

The gate trusts the source path and hash supplied by the fetched manifest, then
renders that same source for the Rust candidate. It never compares the fetched
source with `powerpoint_timeline_oracle_source_bytes()` or a pinned expected
hash. The final coverage check validates only the 17 case labels. A stale or
substituted source with no timing tree can therefore self-authenticate through
its manifest and pass with 17 static frames carrying the required names. The
expanded source generator and focused model test do not close pass 4 D6 until
the differential gate proves that its frames came from that exact source-built
deck.

### D3, invalid transition diagnostics disappear at the terminal boundary

`crates/rpptx-render/src/timeline.rs:60`

`compose_transition` returns the incoming page for every transition at progress
one before validating the direction or handling `TransitionEffect::Other`.
Invalid wipe, push, and zoom directions therefore produce the new stable
diagnostic at intermediate timestamps but no diagnostic at the exact end
boundary. Pass 4 D5 is closed for active composition, but the diagnostic
contract remains timestamp-dependent at a required exact boundary.

### D4, an invalid outgoing slide index is accepted as if no slide was supplied

`crates/rpptx/src/lib.rs:761`

The facade reports an out-of-bounds incoming index as an error, but a supplied
outgoing index that resolves to no slide falls through to `None`. With no
transition this succeeds silently. With an active transition it succeeds with
the generic `transition requires an outgoing slide` diagnostic, which is also
the result for an intentionally omitted outgoing slide. The documented
zero-based outgoing slide index therefore has no bounds contract and caller
input errors cannot be distinguished from absence.

## Smells

None.

## Nitpicks

None.

## Pass 4 closure

- Pass 4 D1 is closed for the supported container removal behavior. Finite
  container durations and supported end conditions now bound parallel and
  sequence child evaluation, and the focused boundary regression passes.
- Pass 4 D2 is closed. Targetless conditions retain the F-213 public
  `TimingTarget::Unsupported` projection, while the approved presence query
  privately distinguishes them from explicit unsupported targets.
- Pass 4 D3 is closed for statically oriented descendants. D1 is the adjacent
  composed-animation defect.
- Pass 4 D4 is closed. A selected compatibility-wrapped chart delegates both
  its existing non-visual id projection and the approved decoded name getter.
- Pass 4 D5 is closed for active wipe, push, and zoom composition. D3 is the
  remaining exact-end diagnostic defect.
- Pass 4 D6 remains partial as D2. The generator now creates five slides and
  the manifest gate requires the exact 17 case names, but the gate does not
  prove those frames belong to the approved generated source.

## Approved OXML boundaries

The OXML diff adds exactly the two approved public methods:

- `ShapeTreeChild::non_visual_name(&self) -> Option<String>`
- `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) -> Option<bool>`

All per-shape name helpers remain crate-private. Names are decoded during
ordinary shape parsing, initialized by existing constructors, synchronized by
each existing `set_name` mutation, preserved by clone, and delegated through a
chart-bearing `mc:AlternateContent` choice. The timing presence cache is built
namespace-aware during ordinary `CT_Timing` parsing. It follows the typed
start and end condition list indexes, respects direct expanded-name boundaries,
is cloned with the model, and is rebuilt by `set_node_duration` through the
ordinary replacement parse. Neither cache serializes or reparses XML on the
timeline evaluation path.

## Focused evidence

- The two F-213 condition projection and target-presence integration
  regressions passed, including duration mutation cache rebuilding.
- The OXML and resolver regressions for compatibility-wrapped chart id and name
  identity passed.
- The finite container boundary regression passed.
- The static rotated-descendant group polygon regression passed.
- The active invalid transition direction regression passed.
- The source-built five-slide oracle model regression passed.
- The terminal outgoing morph regression, zoom direction regression, and
  ordinary static timeline-isolation regression passed.
- Optional oracle mode returned only because `manifest.tsv` is absent.
  Required mode failed closed at that exact missing path.
- `git diff --check` passed. No broad test command ran.

## External evidence blockers

The PowerPoint oracle directory still has no `manifest.tsv` and no PNG frames.
The GUI automation did not produce the required artifact set. The required
pinned PowerPoint differential has not run and is not passed. The full 50-deck
SSIM rider was interrupted and remains incomplete and unclaimed. These external
evidence gaps are separate from D1 through D4.

## Explicit zero categories

No additional production panic path was found. No schema child-order,
namespace binding, retained raw-XML, reverse dependency, new dependency,
unapproved trait, generic, feature flag, crate, backend animation variant,
runtime oracle dependency, or binary fixture defect was found. No public OXML
surface beyond the two exact approvals was added. No ordinary static-path
execution or identity-cache dependency was found. No slide versus layout or
master target-scope leak was found, and ordinary group lineage is retained.
Apart from D1, no additional target mapping or group composition defect was
found. Apart from D3, no additional timing start, end, hold, remove, transition,
or morph endpoint defect was found. Morph matching remains limited to explicit
`!!` names and compatible resolved geometry, with finite crossfade fallbacks.
The required oracle mode is fail-closed for absent artifacts, but D2 and the
external blockers mean the differential gate is not passed.

# F-214, all, pass 7

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 4,940 additions and 30 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 6 review, and the two explicit OXML API approvals
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, one movie timestamp is used as both the clicked and unclicked oracle state

`crates/rpptx/tests/integration.rs:89`

`crates/rpptx/tests/integration.rs:98`

`crates/rpptx/tests/integration.rs:5580`

`crates/rpptx/tests/integration.rs:5904`

The shared matrix assigns slide 0 at movie timestamp zero to both
`timeline-appear-start` with click count zero and `timeline-click-start` with
click count one. The fill-removal cases likewise request click count one from
the same single linear movie used for the automatic sequence. The generator
selects an oracle image only by movie timestamp. It does not provide a click
state to PowerPoint or AVFoundation. One frame in one exported movie therefore
cannot represent both independent click states. If the movie has not advanced
the click, the click cases are wrong. If it has advanced the click, the
unclicked case at the same timestamp is wrong. The gate then compares those
frames with Rust states that do use the distinct manifest click counts, so the
declared click activation and fill-removal differential cannot be valid.

### D2, oracle PNGs are not bound to the approved movie timestamps

`crates/rpptx/tests/integration.rs:5390`

`crates/rpptx/tests/integration.rs:5602`

`crates/rpptx/tests/integration.rs:5896`

`crates/rpptx/tests/integration.rs:5900`

The shared tuple contains a movie timestamp, but the approved-case query drops
that field and the manifest has no machine-readable movie-timestamp column.
The generator writes the timestamp only into free-form classification text.
The gate accepts a PNG hash supplied by that same manifest and requires only
that the classification be nonempty. It never verifies the recorded movie
timestamp or binds a PNG hash to the tuple's capture time. A stale or
substituted frame from another movie time can therefore carry the correct case,
slide, local timestamp, and click count and pass the repaired row check. The
pass 6 coordinate repair does not yet prove that each compared PowerPoint image
came from its approved capture point.

## Smells

None.

## Nitpicks

None.

## Pass 6 closure

Pass 6 D1 is closed for manifest-row relabelling. One 17-row table now drives
the source-built generator and the gate's expected slide, slide-local
timestamp, and click count. Duplicate and unknown case names remain rejected,
the complete set is required, and the focused regression rejects relabelled
transitions plus substituted slides, timestamps, and click counts. D1 and D2
are adjacent defects in the table's capture semantics and image binding.

## Prior closure verification

- Parent group clips remain fixed in the targeting group's animated page space
  while nested-group motion and direct descendant spin are applied. The focused
  combined regression passes.
- Unsupported transition effects and invalid wipe, push, and zoom directions
  diagnose before cut and exact-end returns. The focused boundary regression
  passes.
- A supplied outgoing slide index outside the deck returns
  `UnknownSlideIndex`. The focused facade regression passes.
- Exact source-deck equality rejects a substituted source, and the source-built
  deck still round-trips through the F-213 model and evaluator.
- Targetless conditions retain the F-213 projection while the approved private
  presence cache distinguishes explicit unsupported targets. Its parse,
  namespace, clone, list-index, duration-mutation, and evaluator paths remain
  coherent.
- Compatibility-wrapped chart identity still delegates the existing id and the
  approved decoded name. Static rendering remains isolated from timeline
  execution.

## Approved OXML boundaries

The OXML public diff remains exactly the two user-approved methods:

- `ShapeTreeChild::non_visual_name(&self) -> Option<String>`
- `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) -> Option<bool>`

All concrete per-shape name helpers remain crate-private. No additional OXML
public method, type, module, field, or constant was added. Name caches remain
synchronized across parsing, construction, clone, existing mutation, group
dispatch, and compatibility-wrapped chart selection. The condition-presence
cache remains namespace-aware and rebuilds through ordinary parsing after
duration mutation. Neither cache serializes or reparses XML during timeline
evaluation.

## Focused evidence

- The repaired oracle row-binding regression passed.
- The exact source substitution regression and source-built timeline model
  regression passed.
- The parent-group clip isolation, invalid terminal transition direction,
  outgoing-index bounds, and ordinary static-path regressions passed.
- The F-213 target projection and approved presence-cache regression passed,
  including duration mutation.
- The compatibility-wrapped chart id and name regression passed.
- Optional oracle mode returned only because `manifest.tsv` is absent.
  Required mode failed closed at
  `crates/rpptx/tests/integration.rs:5843` for that exact missing manifest.
- `git diff --check` passed. No broad test command ran. The focused `rpptx`
  tests emitted only the three unrelated existing F-221 dead-code warnings.

## External evidence blockers

The PowerPoint oracle directory has no `manifest.tsv` and no PNG frames. The
GUI automation did not produce the required artifact set. The required pinned
PowerPoint differential has not run and is not passed. The full 50-deck SSIM
rider was interrupted and remains incomplete and unclaimed. These absent
external results are separate from D1 and D2.

## Explicit zero categories

No correctness defect outside D1 and D2 was found. No production panic path,
schema child-order defect, namespace binding defect, retained raw-XML defect,
reverse dependency, new dependency, unapproved trait, generic, feature flag,
crate, backend animation variant, runtime oracle dependency, or binary fixture
defect was found. No public OXML surface beyond the two exact approvals was
added. No name-cache or condition-presence-cache invariant defect was found. No
ordinary static-path execution or identity-cache dependency was found. No
slide versus layout or master target-scope leak, group-lineage loss, target
mapping defect, or group composition defect was found. No additional timing
start, end, click evaluation, hold, remove, transition, morph matching,
geometry interpolation, or endpoint defect was found. Morph matching remains
limited to explicit `!!` names and compatible resolved geometry, with finite
crossfade fallbacks. Required mode is fail-closed for missing artifacts, but D1,
D2, and the external blockers mean the differential gate is not passed.

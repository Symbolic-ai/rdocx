# F-214, all, pass 8

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 5,054 additions and 30 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 7 review, and the two explicit OXML API approvals
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the later click rows compare different slide-local sequence states

`crates/rpptx/tests/integration.rs:96`

`crates/rpptx/tests/integration.rs:5321`

`crates/rpptx/tests/integration.rs:5683`

`crates/rpptx/tests/integration.rs:6018`

The click rows now use movie times 3000 through 3501, but they still come from
the same exported movie and the same slide as the automatic sequence. That
sequence runs appear, wipe, opacity, parallel emphasis and motion, then exit
from movie time zero through 3000. At movie time 3000 the automatic target has
completed its exit. The gate instead renders the click-start Rust frame at
slide-local elapsed time zero, when the automatic target is at its appear
start, and the three fill-removal frames at local times 499 through 501. A
different movie timestamp avoids reusing one frame for two click counts, but it
does not isolate the click state from the concurrent automatic timeline. The
focused regression proves only that the integer movie times differ. It does
not prove that each PowerPoint frame represents the same full slide-local state
as the Rust candidate. Pass 7 D1 therefore remains open.

### D2, manifest metadata still self-authenticates each oracle image

`crates/rpptx/tests/integration.rs:5422`

`crates/rpptx/tests/integration.rs:5705`

`crates/rpptx/tests/integration.rs:6010`

The manifest now records and validates the approved movie timestamp, capture
method, extractor, zero tolerances, and classification. Those checks bind the
row labels, but they do not bind the PNG bytes to the claimed capture. The
generator writes each PNG hash into the manifest, and the gate compares the PNG
with that hash from the same unpinned manifest. It does not retain or validate
the exported movie, re-extract the frame, or compare the PNG with a separately
trusted expected hash or manifest hash. Replacing a PNG from another movie time
and updating its adjacent manifest hash therefore passes every new provenance
check. The provenance regression substitutes only text fields and never
substitutes image bytes plus their self-supplied hash. Pass 7 D2 remains open.

## Smells

None.

## Nitpicks

None.

## Pass 7 closure

- Pass 7 D1 is partial. Click and unclicked rows no longer share the same movie
  timestamp, and the pairwise matrix regression passes. D1 is the remaining
  full-slide state mismatch caused by taking the later frame from the same
  automatic timeline rather than an independent click capture.
- Pass 7 D2 is partial. Movie timestamp, capture-method fields, zero extraction
  tolerances, and exact classification are now machine-checked. D2 is the
  remaining missing trust boundary between those labels and the PNG bytes.

## Prior closure verification

- The 17-case table still binds each accepted case to an exact slide,
  slide-local timestamp, click count, and declared movie timestamp. Relabelled,
  substituted, duplicate, and unknown case inputs are rejected.
- Exact source-deck equality rejects a substituted source, and the source-built
  deck still round-trips through the F-213 model and evaluator.
- Parent group clips remain fixed in the targeting group's animated page space
  under nested-group motion and direct descendant spin.
- Invalid directional transitions and unsupported effects diagnose before
  terminal returns. A supplied outgoing index outside the deck returns
  `UnknownSlideIndex`.
- Targetless timing conditions retain the F-213 public projection while the
  approved target-presence cache distinguishes explicit unsupported targets.
  The duration-mutation rebuild regression passes.
- Compatibility-wrapped chart identity delegates its id and approved decoded
  name. Ordinary static rendering remains isolated from timeline execution.

## Approved OXML boundaries

The OXML public diff remains exactly the two user-approved methods:

- `ShapeTreeChild::non_visual_name(&self) -> Option<String>`
- `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) -> Option<bool>`

All concrete per-shape name helpers remain crate-private. No additional OXML
public method, type, module, field, or constant was added. Name caches remain
coherent across parsing, construction, clone, existing mutation, group
dispatch, and compatibility-wrapped chart selection. The condition-presence
cache remains namespace-aware, follows typed list indexes, and rebuilds through
ordinary parsing after duration mutation. Neither cache serializes or reparses
XML during timeline evaluation.

## Focused evidence

- The new click-movie separation, manifest row, provenance, classification,
  and source-substitution regressions passed.
- The source-built timeline model regression passed.
- The parent-group clip isolation, invalid terminal transition direction,
  outgoing-index bounds, and ordinary static-path regressions passed.
- The F-213 target projection, approved presence-cache, and
  compatibility-wrapped chart identity regressions passed.
- Optional oracle mode returned only because `manifest.tsv` is absent.
  Required mode failed closed at
  `crates/rpptx/tests/integration.rs:5946` for that exact missing manifest.
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

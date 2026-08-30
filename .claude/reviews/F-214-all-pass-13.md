# F-214, all, pass 13

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 5,614 additions and 31 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 12 review, the manual movie and rational sample-grid context, and
the two explicit OXML API approvals
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the fade oracle case still depends on the click and fill fixture

`crates/rpptx/tests/integration.rs:138`

`crates/rpptx/tests/integration.rs:5302`

`crates/rpptx/tests/integration.rs:5370`

`crates/rpptx/tests/integration.rs:6570`

`crates/rpptx/src/lib.rs:4616`

`crates/rpptx/src/lib.rs:4624`

The new external matrix removes the named click and fill cases, but its
`ordinary-transition` observation targets slide two. The gate therefore passes
slide one as the outgoing page. Slide one is still the click-only fixture with
the fill-remove marker, and the facade evaluates every outgoing page at
`u64::MAX` with `u32::MAX` clicks. The manifest binds only the incoming row's
zero click count and has no field that binds the outgoing PowerPoint click
state. Consequently the fade frame can pass or fail because of how the movie
played the click and fill behavior on slide one, even though those semantics
are meant to remain covered only by Rust tests. Use a clean outgoing slide for
the fade case or move the click fixture so it is not the predecessor of any
external transition sample.

## Smells

None.

## Nitpicks

None.

## Rational sample contract verification

- The shared table contains exactly nine named cases with exact signed sample
  values and positive timescales at `crates/rpptx/tests/integration.rs:132`.
  Its focused invariant requires every `(value, timescale)` pair to be unique,
  every external case to be automatic, and every rational time to map exactly
  to the declared integer Rust slide-local timestamp at
  `crates/rpptx/tests/integration.rs:5758`.
- The five slide-zero observations cover appear, wipe, opacity, parallel scale,
  spin and motion, and exit. The four compositor observations cover fade,
  morph, push, and zoom at `crates/rpptx/tests/integration.rs:133` and
  `crates/rpptx/tests/integration.rs:138`. D1 concerns isolation of the fade
  observation, not its exact rational mapping.
- Swift constructs the requested `CMTime` from the table's exact value and
  timescale with zero tolerance, captures `actualTime`, and prints both exact
  representations at `crates/rpptx/tests/integration.rs:106`. Rust requires
  requested and returned value and timescale fields to equal the approved pair
  exactly at `crates/rpptx/tests/integration.rs:5523`.
- Negative regressions reject relabelled coordinates, substituted local time,
  click count, slide, rational value, and rational timescale at
  `crates/rpptx/tests/integration.rs:5695`. They also reject a neighboring
  returned sample and replacement of the encoded rational by an equivalent
  millisecond request at `crates/rpptx/tests/integration.rs:5946`.
- Generator rows record both rational fields and are emitted only after exact
  extraction validation at `crates/rpptx/tests/integration.rs:6124` and
  `crates/rpptx/tests/integration.rs:6147`. The required gate uses the same
  twelve-field header, rejects duplicate case names, and requires every row to
  equal its complete approved tuple at `crates/rpptx/tests/integration.rs:6380`
  and `crates/rpptx/tests/integration.rs:6480`. Complete set equality rejects a
  missing or extra case at `crates/rpptx/tests/integration.rs:6659`.
- Gate-side re-extraction uses each row's exact rational fields, verifies the
  exact returned representation, and byte-compares the stored PNG with the
  fresh extraction at `crates/rpptx/tests/integration.rs:6519` and
  `crates/rpptx/tests/integration.rs:6530`.
- Verified 1920 by 1080 raw bytes are normalized only afterward. Geometry and
  SSIM continue to consume the deterministic 2001 by 1125 normalized oracle
  and unchanged literal-150-dpi Rust candidate at
  `crates/rpptx/tests/integration.rs:6551` and
  `crates/rpptx/tests/integration.rs:6579`.
- Click and fill boundaries remain directly asserted in Rust at local times 0,
  499, 500, and 501 ms at `crates/rpptx/tests/integration.rs:5646`. D1 is the
  remaining indirect dependency of the external fade case on that same slide.

## Prior closure verification

- Pass 12 normalization closure remains intact. Exact raw geometry,
  provenance, raw-byte identity, one-sided deterministic normalization,
  unchanged static bytes, comparison dimensions, geometry conversion, and SSIM
  routing remain at `crates/rpptx/tests/integration.rs:88`,
  `crates/rpptx/tests/integration.rs:5481`,
  `crates/rpptx/tests/integration.rs:5574`, and
  `crates/rpptx/tests/integration.rs:6004`.
- Sequence, parallel, click, alternative-condition, duration, fill, entrance,
  exit, emphasis, motion-origin, relative-path, finite-value, hold, remove, and
  endpoint semantics retain their prior fixes in
  `crates/rpptx-layout/src/timeline.rs:227`.
- Slide, layout, and master target scopes remain separate. Group ids retain
  lineage, group extents use their declared coordinate space, and parent clips
  remain isolated from nested-group and descendant animation at
  `crates/rpptx-layout/src/context.rs:2982` and
  `crates/rpptx-layout/src/context.rs:3175`.
- Wipe geometry, invalid transition directions, timestamped outgoing pages,
  finite composition, and bounded explicit-name morph matching retain their
  prior fixes in `crates/rpptx-render/src/timeline.rs:49` and
  `crates/rpptx-render/src/timeline.rs:259`.
- Ordinary static rendering remains independent of timeline evaluation at
  `crates/rpptx/tests/integration.rs:6350`. Compatibility-wrapped chart
  identity retains its decoded name and shape id at
  `crates/rpptx-oxml/tests/integration.rs:2016`.

## Approved OXML boundaries

The OXML public diff remains exactly the two user-approved methods:

- `ShapeTreeChild::non_visual_name(&self) -> Option<String>` at
  `crates/rpptx-oxml/src/shape_tree.rs:62`.
- `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) ->
  Option<bool>` at `crates/rpptx-oxml/src/timing.rs:278`.

All concrete per-shape name helpers remain crate-private. No additional OXML
public method, type, module, field, or constant was added. The name and
condition-presence caches retain their parse, construction, clone, mutation,
dispatch, and compatibility-selection invariants.

## Focused evidence

- All nine PowerPoint timeline oracle contract regressions passed, including
  the new unique-rational mapping and exact-representation rejection cases.
- Focused sequence and click evaluation, entrance and exit boundaries,
  four-mode transition composition, progress-zero morph, ordinary static path,
  condition-presence projection, and alternate-content identity regressions
  passed. Four initial namespaced unit invocations selected zero tests, so the
  four affected evaluator and compositor filters were rerun without `--exact`
  and each executed one passing test.
- Optional oracle mode returned only because `manifest.tsv` is absent. Required
  mode with the verified movie pin failed closed at
  `crates/rpptx/tests/integration.rs:6399` for that exact missing manifest.
- `git diff --check` passed. No broad test command ran. Focused `rpptx` tests
  emitted only the three unrelated existing F-221 dead-code warnings.

## External evidence status

The manually exported movie exists and its observed SHA-256 remains
`28514432f4aafae9d6c5ddd522d23e87458e08ec59ebf5555398a74e712fa83e`.
One stale partial PNG from the prior failed 17-case generation attempt remains,
but no approved nine-case artifact set or `manifest.tsv` exists. The required
PowerPoint geometry and SSIM differential has not run and is not passed. That
external evidence blocker is separate from D1.

## Explicit zero categories

No correctness defect outside D1 was found. No production panic, OOXML,
structure, or unapproved public-surface finding was found. No schema child-order
defect, namespace binding defect, retained raw-XML defect, reverse dependency,
new dependency, unapproved trait, generic, feature flag, crate, backend
animation variant, runtime oracle dependency, or committed binary fixture
defect was found. No public OXML surface beyond the two exact approvals was
added. No name-cache or condition-presence-cache invariant defect was found. No
ordinary static-path execution or identity-cache dependency was found. No
slide versus layout or master target-scope leak, group-lineage loss, target
mapping defect, or group-composition defect was found. No timing start, end,
click evaluation, hold, remove, transition algorithm, morph matching, geometry
interpolation, or endpoint defect was found. No rational uniqueness, exact-time
representation, local-time arithmetic, manifest substitution, duplicate case,
raw re-extraction, byte-identity, normalization, comparison geometry, or SSIM
routing defect outside D1 was found. The absent approved PNG and manifest
evidence means the external differential gate remains unpassed.

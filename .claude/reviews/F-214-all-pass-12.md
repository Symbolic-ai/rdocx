# F-214, all, pass 12

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 5,538 additions and 31 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 11 review, the manual-export and measured 150 dpi context, and the
two explicit OXML API approvals
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 11 closure verification

Pass 11 D1 is closed.

- Raw PowerPoint capture geometry remains independently fixed at 1920 by 1080,
  while the comparison geometry is separately fixed at the measured 2001 by
  1125 result of literal 150 dpi rendering at
  `crates/rpptx/tests/integration.rs:88`.
- The manifest must retain unique exact provenance for the zero-tolerance
  AVFoundation extraction and the one-sided bilinear normalization, including
  exact input, output, and DPI values at
  `crates/rpptx/tests/integration.rs:93` and
  `crates/rpptx/tests/integration.rs:5470`.
- The normalizer rejects input that is not exactly 1920 by 1080, places only the
  verified oracle PNG on the approved 960 by 540 point page, renders that copy
  at literal 150 dpi, and accepts only 2001 by 1125 output at
  `crates/rpptx/tests/integration.rs:5565`.
- The focused regression uses the approved source-built deck. It verifies the
  unchanged static path at 2001 by 1125, deterministic normalized bytes,
  rejection of non-raw input, and byte-identical static output before and after
  normalization at `crates/rpptx/tests/integration.rs:5937`.
- Gate-side raw validation remains ahead of normalization. Each manifest row
  must describe 1920 by 1080 raw geometry, and the stored PNG bytes must equal a
  fresh zero-tolerance exact-time extraction byte for byte at
  `crates/rpptx/tests/integration.rs:6431` and
  `crates/rpptx/tests/integration.rs:6465`.
- Only after those raw checks does the gate normalize an in-memory oracle copy
  at `crates/rpptx/tests/integration.rs:6475`. The Rust timeline candidate still
  renders directly at literal 150 dpi at
  `crates/rpptx/tests/integration.rs:6487` and
  `crates/rpptx/tests/integration.rs:6503`.
- Geometry and SSIM read the Rust candidate and normalized oracle paths. Both
  inputs must be exactly 2001 by 1125, and pixel error is converted to points
  using 150 dpi at `crates/rpptx/tests/integration.rs:6531` and
  `crates/rpptx/tests/integration.rs:6558`.

## Prior closure verification

- Exact integer-millisecond `CMTime` construction, zero requested tolerances,
  returned-time equality, checked Rust rational validation, and the 5,499,
  5,500, and 5,501 ms boundary regression remain intact at
  `crates/rpptx/tests/integration.rs:106`,
  `crates/rpptx/tests/integration.rs:5510`, and
  `crates/rpptx/tests/integration.rs:5876`.
- The shared 17-case matrix still binds each observation to its exact source
  slide, local timestamp, click count, and movie timestamp. Click observations
  remain isolated on their own source slide at
  `crates/rpptx/tests/integration.rs:5685` and
  `crates/rpptx/tests/integration.rs:5728`.
- The actual movie digest must equal an independent lowercase 64-character pin,
  and the manifest binds that digest to the exact source digest and three
  PowerPoint identity values at `crates/rpptx/tests/integration.rs:5485` and
  `crates/rpptx/tests/integration.rs:5541`.
- Sequence, parallel, click, alternative-condition, duration, fill, entrance,
  exit, emphasis, motion-origin, relative-path, position, finite-value, hold,
  remove, and endpoint semantics retain their prior fixes. Unsupported start
  and end targets remain distinguishable from targetless conditions at
  `crates/rpptx-layout/src/timeline.rs:364` and
  `crates/rpptx-layout/src/timeline.rs:499`.
- Slide, layout, and master target scopes remain separate. Group ids retain
  lineage, group extents use their declared coordinate space, and parent clips
  do not follow nested-group or descendant animation. The relevant construction
  remains centralized at `crates/rpptx-layout/src/context.rs:342` and timeline
  evaluation at `crates/rpptx-layout/src/timeline.rs:227`.
- Wipe geometry, ordinary transitions, invalid-direction diagnostics,
  timestamped outgoing pages, and bounded morph composition retain their prior
  fixes. Compatible morph still preserves outgoing content at progress zero at
  `crates/rpptx-render/src/timeline.rs:842`.
- Ordinary static rendering remains independent of timeline evaluation at
  `crates/rpptx/tests/integration.rs:6281`. Compatibility-wrapped chart identity
  retains its decoded name and shape id at
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
  source and case binding, isolated click capture, exact provenance,
  self-authentication rejection, exact existing-movie pinning, exact-time
  boundaries, and bounded deterministic non-mutating normalization. The
  normalization regression is at `crates/rpptx/tests/integration.rs:5937`.
- The focused sequence, entrance, exit, endpoint, parent-group clip, invalid
  transition direction, compatible morph, timestamped outgoing morph, ordinary
  static-path, condition projection, duration mutation, and alternate-content
  chart-identity regressions passed.
- Optional oracle mode returned only because `manifest.tsv` is absent. Required
  mode with the verified movie pin failed closed at
  `crates/rpptx/tests/integration.rs:6330` for that exact missing manifest.
- `git diff --check` passed. No broad test command ran. Focused `rpptx` tests
  emitted only the three unrelated existing F-221 dead-code warnings.

## External evidence status

The manually exported movie exists and its observed SHA-256 remains
`28514432f4aafae9d6c5ddd522d23e87458e08ec59ebf5555398a74e712fa83e`.
No final case PNGs or `manifest.tsv` exist, so raw-frame provenance, geometry,
and SSIM have not been exercised by the required PowerPoint differential. That
gate is not passed. The progress record's separate 50-deck result is not a
PowerPoint timeline differential and was not rerun during this review.

## Explicit zero categories

No correctness, contract, production panic, OOXML, test, structure, or public
surface finding was found. No schema child-order defect, namespace binding
defect, retained raw-XML defect, reverse dependency, new dependency, unapproved
trait, generic, feature flag, crate, backend animation variant, runtime oracle
dependency, or binary fixture defect was found. No public OXML surface beyond
the two exact approvals was added. No name-cache or condition-presence-cache
invariant defect was found. No ordinary static-path execution or identity-cache
dependency was found. No slide versus layout or master target-scope leak,
group-lineage loss, target mapping defect, or group-composition defect was
found. No timing start, end, click evaluation, hold, remove, transition, morph
matching, geometry interpolation, or endpoint defect was found. No movie
overwrite, pin-validation, source or build binding, raw-byte integrity,
normalization, comparison geometry, SSIM routing, or GUI-fallback defect was
found. The absent final PowerPoint PNG and manifest evidence means the external
differential gate remains unpassed.

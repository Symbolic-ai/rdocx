# F-214, all, pass 10

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 5,322 additions and 30 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 9 review, and the two explicit OXML API approvals
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 9 closure

- Pass 9 D1 is closed at `crates/rpptx/tests/integration.rs:108`. The Swift
  extractor constructs the requested time directly as integer milliseconds at
  timescale 1,000, so 5,499, 5,500, and 5,501 ms remain distinct exact
  rationals.
- The extractor captures the returned time and fails unless it is numerically
  equal to the request at `crates/rpptx/tests/integration.rs:110`.
- Rust independently parses all six reported integers, requires the exact
  request representation, and compares the returned rational with the approved
  millisecond timestamp through checked `i128` multiplication at
  `crates/rpptx/tests/integration.rs:5490`. This accepts equivalent positive
  timescales without division or overflow.
- Both the artifact generator and required differential gate apply that check
  at `crates/rpptx/tests/integration.rs:5890` and
  `crates/rpptx/tests/integration.rs:6266`. Their focused regression accepts
  exact equivalent rationals and rejects both former 600-timescale
  quantizations and a non-millisecond request representation.

## Prior closure verification

- The shared 17-case matrix still binds every case to one exact source slide,
  local timestamp, click count, and movie timestamp. Automatic observations
  remain on slide zero and click observations remain on the isolated click-only
  slide one.
- Source bytes, actual movie bytes, independent movie SHA-256, source SHA-256,
  PowerPoint version and both builds, case coordinates, capture provenance,
  PNG bytes, dimensions, geometry tolerance, SSIM threshold, and divergence
  classification remain fail-closed at their established boundaries.
- Sequence, parallel, click, alternative-condition, duration, fill, entrance,
  exit, emphasis, motion-origin, relative-path, position, finite-value, hold,
  remove, and endpoint semantics retain their prior fixes.
- Slide, layout, and master target scopes remain separate. Group ids retain
  lineage, group extents use their declared coordinate space, and parent clips
  do not follow nested-group or descendant animation.
- Wipe geometry, ordinary transitions, invalid directions, terminal
  diagnostics, outgoing-index validation, timestamped outgoing pages, and
  bounded morph composition retain their prior fixes. Morph matching remains
  limited to explicit `!!` names with compatible geometry and finite
  crossfade fallbacks.
- Unsupported start and end targets remain distinguishable from targetless
  conditions through the approved presence cache. Duration mutation rebuilds
  that cache, unsupported effect filters remain diagnostic, and supported
  siblings continue evaluating.
- Compatibility-wrapped chart identity retains its shape id and decoded name.
  Name caches remain coherent across parsing, construction, clone, existing
  mutation, group dispatch, and compatibility selection.
- Ordinary static rendering remains independent of timeline identity,
  evaluation, and composition. Resolver and outgoing-frame diagnostics remain
  preserved through the facade.

## Approved OXML boundaries

The OXML public diff remains exactly the two user-approved methods:

- `ShapeTreeChild::non_visual_name(&self) -> Option<String>`
- `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) -> Option<bool>`

All concrete per-shape name helpers remain crate-private. No additional OXML
public method, type, module, field, or constant was added. Neither approved
cache serializes or reparses XML during timeline evaluation.

## Focused evidence

- All seven PowerPoint timeline oracle contract regressions passed, including
  exact millisecond rational validation, isolated click state, source and case
  binding, provenance, independent movie binding, and stale-PNG rejection.
- The sequence and boundary evaluator tests, parent-group clip isolation,
  invalid transition direction, timestamped outgoing morph, outgoing-index
  bounds, and ordinary static-path regressions passed.
- The F-213 condition projection and mutation regressions and the
  compatibility-wrapped chart identity regression passed.
- Optional oracle mode returned only because `manifest.tsv` is absent.
  Required mode failed closed at
  `crates/rpptx/tests/integration.rs:6146` for that exact missing manifest.
- `git diff --check` passed. No broad test command ran. Focused `rpptx` tests
  emitted only the three unrelated existing F-221 dead-code warnings.

## External evidence blockers

The PowerPoint oracle directory has no `manifest.tsv`, retained movie, or PNG
frames. The independent movie SHA-256 environment pin is unset. GUI automation
did not produce the required artifact set. The required pinned PowerPoint
differential has not run and is not passed. The full 50-deck SSIM rider was
interrupted and remains incomplete and unclaimed. These absent external results
do not change the zero implementation-finding verdict.

## Explicit zero categories

No correctness, contract, panic, OOXML, test, or structure finding was found.
No production panic path, schema child-order defect, namespace binding defect,
retained raw-XML defect, reverse dependency, new dependency, unapproved trait,
generic, feature flag, crate, backend animation variant, runtime oracle
dependency, or binary fixture defect was found. No public OXML surface beyond
the two exact approvals was added. No name-cache or condition-presence-cache
invariant defect was found. No ordinary static-path execution or identity-cache
dependency was found. No slide versus layout or master target-scope leak,
group-lineage loss, target mapping defect, or group composition defect was
found. No timing start, end, click evaluation, hold, remove, transition, morph
matching, geometry interpolation, or endpoint defect was found. No oracle
source, case, provenance, movie, exact-time, PNG, geometry, or SSIM gate logic
defect was found. The missing external evidence means the differential gate is
not passed.

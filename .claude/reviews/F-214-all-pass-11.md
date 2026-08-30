# F-214, all, pass 11

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 5,404 additions and 30 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 10 review, the manual-export context, and the two explicit OXML API
approvals
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the new movie dimensions cannot satisfy the unchanged 150 dpi Rust gate

`crates/rpptx/tests/integration.rs:88`

`crates/rpptx/tests/integration.rs:5986`

`crates/rpptx/tests/integration.rs:6384`

`crates/rpptx/tests/integration.rs:6437`

`crates/oxml-pdf/src/raster.rs:282`

The shared constants now require every PowerPoint frame to be 1920 by 1080,
but the candidate is still rendered at the plan-required 150 dpi. The
deterministic 16:9 source slide is 12,192,000 by 6,858,000 EMU, or 13.333 by
7.5 inches. The raster backend therefore produces 2000 by 1125 pixels at 150
dpi. The manifest nevertheless records `dpi` as 150 beside 1920 by 1080 frame
rows, and the gate later requires the Rust dimensions to equal those row
dimensions. Every generated case will fail the dimension check before its
geometry or SSIM result can be accepted. A 1920 by 1080 raster corresponds to
144 dpi for this source, so the movie frames must be normalized to the declared
150 dpi or the candidate render, manifest DPI, geometry conversion, and
approved contract must use one other consistent resolution.

## Smells

None.

## Nitpicks

None.

## Pass 10 and prior closure verification

- Pass 10 remains clean. Exact integer-millisecond `CMTime` construction,
  returned-time equality, checked Rust rational validation, and the 5,499,
  5,500, and 5,501 ms boundary regression remain intact.
- The shared 17-case matrix still binds every case to one exact source slide,
  local timestamp, click count, and movie timestamp. Automatic observations
  remain on slide zero and click observations remain on the isolated click-only
  slide one.
- Source bytes, actual movie bytes, independent movie SHA-256, source SHA-256,
  PowerPoint version and both builds, case coordinates, capture provenance,
  PNG bytes, classification, and exact-time extraction remain fail-closed at
  their established boundaries. D1 is limited to the inconsistent raster
  geometry contract.
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
  Ordinary static rendering remains independent of timeline identity,
  evaluation, and composition.

## Existing movie reuse and GUI fallback

- An existing movie is accepted only when
  `RDOCX_PPTX_TIMELINE_ORACLE_MOVIE_SHA256` is a 64-character lowercase digest
  equal to the actual bytes at `crates/rpptx/tests/integration.rs:5478`.
- With no pin, an existing movie is rejected before PowerPoint is launched at
  `crates/rpptx/tests/integration.rs:5903`. With a supplied pin, a missing or
  mismatched movie fails closed and does not fall back to GUI export.
- The existing movie is hashed before extraction and again after all frame
  extraction at `crates/rpptx/tests/integration.rs:5977`. The observed retained
  movie still has SHA-256
  `28514432f4aafae9d6c5ddd522d23e87458e08ec59ebf5555398a74e712fa83e`.
- GUI export remains available only when neither an existing movie nor a pin is
  present. It verifies the pinned PowerPoint identities before launch and uses
  the shared movie geometry constants. No GUI command ran during this review.

## Approved OXML boundaries

The OXML public diff remains exactly the two user-approved methods:

- `ShapeTreeChild::non_visual_name(&self) -> Option<String>`
- `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) -> Option<bool>`

All concrete per-shape name helpers remain crate-private. No additional OXML
public method, type, module, field, or constant was added. Name and condition
presence caches retain their parse, construction, clone, mutation, dispatch,
and compatibility-selection invariants.

## Focused evidence

- All eight PowerPoint timeline oracle contract regressions passed, including
  the existing-movie pin and exact-time boundary tests.
- The sequence evaluator, parent-group clip isolation, invalid transition
  direction, timestamped outgoing morph, ordinary static-path, F-213 condition
  projection, and compatibility-wrapped chart identity regressions passed.
- Optional oracle mode returned only because `manifest.tsv` is absent.
  Required mode with the verified movie pin failed closed at
  `crates/rpptx/tests/integration.rs:6223` for that exact missing manifest.
- `git diff --check` passed. No broad test command ran. Focused `rpptx` tests
  emitted only the three unrelated existing F-221 dead-code warnings.

## External evidence status

The manually exported movie exists and matches its recorded independent
SHA-256. No case PNGs or `manifest.tsv` exist, so the PowerPoint frame geometry
and SSIM differential has not run and is not passed. The progress record states
that the separate 50-deck SSIM rider completed across 421 slides, with its
trend target false. That recorded corpus result is not a PowerPoint timeline
differential result and was not rerun during this review. These evidence facts
are separate from D1.

## Explicit zero categories

No correctness defect outside D1 was found. No contract, production panic,
OOXML, or structure finding was found. No schema child-order defect, namespace
binding defect, retained raw-XML defect, reverse dependency, new dependency,
unapproved trait, generic, feature flag, crate, backend animation variant,
runtime oracle dependency, or binary fixture defect was found. No public OXML
surface beyond the two exact approvals was added. No name-cache or
condition-presence-cache invariant defect was found. No ordinary static-path
execution or identity-cache dependency was found. No slide versus layout or
master target-scope leak, group-lineage loss, target mapping defect, or group
composition defect was found. No timing start, end, click evaluation, hold,
remove, transition, morph matching, geometry interpolation, or endpoint defect
was found. No movie overwrite, pin-validation, pre-extraction integrity,
post-extraction integrity, or GUI-fallback defect was found. D1 and the absent
PowerPoint PNG and manifest evidence mean the differential gate is not passed.

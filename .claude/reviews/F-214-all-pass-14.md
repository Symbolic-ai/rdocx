# F-214, all, pass 14

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 5,938 additions and 208 deletions, including both
approved untracked timeline modules, the approved plan and cited HLD sections,
progress notes, pass 13 review, the manual movie context, and the two explicit
OXML API approvals
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 13 closure verification

Pass 13 D1 is closed.

- The shared matrix now contains exactly ten unique rational samples. The new
  terminal observation binds slide one, `u64::MAX` elapsed time,
  `u32::MAX` clicks, and exact movie sample `4198/600`, and it immediately
  precedes the fade observation at `crates/rpptx/tests/integration.rs:132`.
- The binding helper accepts only that complete terminal tuple and the complete
  fade tuple in the required order at
  `crates/rpptx/tests/integration.rs:5489`. Focused negative cases reject a
  missing terminal observation and substituted click semantics at
  `crates/rpptx/tests/integration.rs:5919`. Complete case binding separately
  rejects substituted elapsed and click sentinels at
  `crates/rpptx/tests/integration.rs:5816`.
- The generator refuses to proceed unless the approved terminal-to-fade
  ordering exists, then emits both exact tuples from the shared matrix at
  `crates/rpptx/tests/integration.rs:6248` and
  `crates/rpptx/tests/integration.rs:6252`.
- The required gate starts with the outgoing state unbound and refuses to
  evaluate `ordinary-transition` until the terminal row has completed at
  `crates/rpptx/tests/integration.rs:6608` and
  `crates/rpptx/tests/integration.rs:6631`.
- Completion includes exact zero-tolerance rational re-extraction and raw PNG
  byte identity at `crates/rpptx/tests/integration.rs:6654`, followed by
  deterministic oracle normalization at
  `crates/rpptx/tests/integration.rs:6686`.
- The terminal row renders slide one at the exact MAX elapsed and click tuple
  without a compositor at `crates/rpptx/tests/integration.rs:6698`. Its
  normalized oracle and Rust candidate must both be 2001 by 1125, remain within
  one point geometry error, and reach SSIM 0.99 before the gate marks the fade
  outgoing state bound at `crates/rpptx/tests/integration.rs:6773` and
  `crates/rpptx/tests/integration.rs:6790`.
- A focused regression independently renders slide one's MAX/MAX terminal page
  and the slide-two fade at progress zero. Their literal-150-dpi PNG bytes must
  be equal at `crates/rpptx/tests/integration.rs:5715`.

The external fade case therefore no longer depends on an unverified PowerPoint
click and fill state. The 0, 499, 500, and 501 ms click and fill boundaries
remain Rust-only assertions at `crates/rpptx/tests/integration.rs:5680`.

## Rational-time and prior closure verification

- Every exact `(value, timescale)` pair remains unique. All ordinary movie
  samples map exactly to their declared integer slide-local time, while the
  terminal row is explicitly limited to its approved MAX/MAX sentinel tuple at
  `crates/rpptx/tests/integration.rs:5849`.
- Swift still constructs the requested `CMTime` from the exact rational fields
  with zero tolerance and returns both representations at
  `crates/rpptx/tests/integration.rs:106`. Rust requires exact requested and
  actual value and timescale equality at
  `crates/rpptx/tests/integration.rs:5552`, with negative representation cases
  at `crates/rpptx/tests/integration.rs:6071`.
- Raw 1920 by 1080 bytes, exact provenance, one-sided deterministic 2001 by
  1125 normalization, literal 150 dpi candidate rendering, geometry
  conversion, and SSIM routing retain the pass 12 closure at
  `crates/rpptx/tests/integration.rs:88`,
  `crates/rpptx/tests/integration.rs:5510`,
  `crates/rpptx/tests/integration.rs:5603`, and
  `crates/rpptx/tests/integration.rs:6129`.
- Sequence, parallel, click, alternative-condition, duration, fill, entrance,
  exit, emphasis, motion-origin, relative-path, finite-value, hold, remove, and
  endpoint semantics retain their prior fixes at
  `crates/rpptx-layout/src/timeline.rs:224`.
- Slide, layout, and master target scopes remain separate. Group ids retain
  lineage, group extents use their declared coordinate space, and parent clips
  remain isolated from nested-group and descendant animation at
  `crates/rpptx-layout/src/context.rs:2982` and
  `crates/rpptx-layout/src/context.rs:3175`.
- Wipe geometry, invalid transition directions, timestamped outgoing pages,
  finite composition, and bounded explicit-name morph matching retain their
  prior fixes at `crates/rpptx-render/src/timeline.rs:49` and
  `crates/rpptx-render/src/timeline.rs:259`.
- Ordinary static rendering remains independent of timeline evaluation at
  `crates/rpptx/tests/integration.rs:6478`. Compatibility-wrapped chart
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

- All eleven PowerPoint timeline oracle contract regressions passed, including
  terminal binding, absence and substitution rejection, progress-zero equality,
  unique rational mapping, exact representation, raw integrity, and bounded
  deterministic normalization.
- Focused sequence and click evaluation, entrance and exit boundaries,
  four-mode transition composition, progress-zero morph, ordinary static path,
  condition-presence projection, and alternate-content identity regressions
  passed.
- Optional oracle mode returned only because `manifest.tsv` is absent. Required
  mode with the verified movie pin failed closed at
  `crates/rpptx/tests/integration.rs:6527` for that exact missing manifest.
- `git diff --check` passed. No broad test command ran. Focused `rpptx` tests
  emitted only the three unrelated existing F-221 dead-code warnings.

## External evidence status

The manually exported movie exists and its observed SHA-256 remains
`28514432f4aafae9d6c5ddd522d23e87458e08ec59ebf5555398a74e712fa83e`.
One stale partial PNG from the prior failed 17-case generation attempt remains,
but the final ten approved PNGs and `manifest.tsv` do not exist. The required
PowerPoint geometry and SSIM differential has not run and is not passed. That
external evidence blocker is separate from the clean implementation verdict.

## Explicit zero categories

No correctness, contract, production panic, OOXML, test, structure, or public
surface finding was found. No schema child-order defect, namespace binding
defect, retained raw-XML defect, reverse dependency, new dependency, unapproved
trait, generic, feature flag, crate, backend animation variant, runtime oracle
dependency, or committed binary fixture defect was found. No public OXML
surface beyond the two exact approvals was added. No name-cache or
condition-presence-cache invariant defect was found. No ordinary static-path
execution or identity-cache dependency was found. No slide versus layout or
master target-scope leak, group-lineage loss, target mapping defect, or
group-composition defect was found. No timing start, end, click evaluation,
hold, remove, transition algorithm, morph matching, geometry interpolation, or
endpoint defect was found. No rational uniqueness, exact-time representation,
local-time arithmetic, terminal outgoing binding, manifest substitution,
duplicate case, raw re-extraction, byte-identity, normalization, comparison
geometry, or SSIM routing defect was found. The absent approved PNG and
manifest evidence means the external differential gate remains unpassed.

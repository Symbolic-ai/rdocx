# F-214, all, pass 9

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 5,234 additions and 30 deletions, including both approved
untracked timeline modules, the approved plan and cited HLD sections, progress
notes, pass 8 review, and the two explicit OXML API approvals
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, exact movie timestamps are quantized before frame extraction

`crates/rpptx/tests/integration.rs:108`

`crates/rpptx/tests/integration.rs:125`

`crates/rpptx/tests/integration.rs:6176`

The shared extractor converts integer milliseconds through a `CMTime`
timescale of 600 and discards `actualTime`. One tick at that timescale is about
1.667 ms, so the approved 5,499 and 5,501 ms boundary observations are not
representable. Running the exact expression used by the helper produces
5,498.333 ms for 5,499 and 5,500 ms for 5,501. The latter therefore collapses
the declared after-end row onto the 5,500 ms end row, even though the Rust
candidate deliberately evaluates different local states at 500 and 501 ms.
Zero requested-time tolerance constrains the already quantized `CMTime`, not
the approved integer timestamp. Because both generation and gate-side
re-extraction use this helper, PNG byte identity proves only that both paths
made the same quantized request. The differential cannot establish the exact
fill-remove boundary it claims to test.

## Smells

None.

## Nitpicks

None.

## Pass 8 closure

- Pass 8 D1 is closed. Automatic sequence observations remain on slide zero,
  while every click-count-one observation uses the click-only slide one. The
  source round-trip evaluates that separate slide and the shared 17-case table
  binds each case to its exact slide, local timestamp, click count, and movie
  timestamp.
- Pass 8 D2 is closed. The gate hashes the retained movie bytes, requires the
  same digest from the independent environment pin, binds the movie row to the
  exact deterministic source and all three PowerPoint identities, and requires
  each stored PNG to equal a fresh gate-side extraction byte for byte. The
  substitution regression rejects a movie plus adjacent self-supplied hash,
  wrong source, wrong PowerPoint version, and stale PNG bytes.

## Prior closure verification

- The evaluator keeps sequence, parallel, click, alternative-condition,
  duration, fill, entrance, exit, emphasis, motion-origin, relative-path,
  position, finite-value, hold, remove, and endpoint semantics deterministic.
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
  the cache, unsupported effect filters remain diagnostic, and supported
  siblings continue evaluating.
- Compatibility-wrapped chart identity retains its shape id and decoded name.
  Name caches remain coherent across parsing, construction, clone, existing
  mutation, group dispatch, and compatibility selection.
- Ordinary static rendering remains independent of timeline identity,
  evaluation, and composition. Resolver and outgoing-frame diagnostics remain
  preserved through the facade.
- The oracle continues to reject substituted source bytes, relabelled or
  duplicate cases, unsupported divergence classifications, invalid dimensions,
  excessive geometry error, and SSIM below 0.99. D1 is the one remaining defect
  in the new exact-time extraction chain.

## Approved OXML boundaries

The OXML public diff remains exactly the two user-approved methods:

- `ShapeTreeChild::non_visual_name(&self) -> Option<String>`
- `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) -> Option<bool>`

All concrete per-shape name helpers remain crate-private. No additional OXML
public method, type, module, field, or constant was added. Neither approved
cache serializes or reparses XML during timeline evaluation.

## Focused evidence

- All six focused PowerPoint timeline oracle contract regressions passed,
  including isolated click source state, source and case binding, provenance,
  independent movie binding, and stale-PNG rejection.
- The sequence and boundary evaluator tests, parent-group clip isolation,
  invalid transition direction, timestamped outgoing morph, outgoing-index
  bounds, and ordinary static-path regressions passed.
- The F-213 condition projection and mutation regression and the
  compatibility-wrapped chart identity regression passed.
- Optional oracle mode returned only because `manifest.tsv` is absent.
  Required mode failed closed at
  `crates/rpptx/tests/integration.rs:6062` for that exact missing manifest.
- `git diff --check` passed. No broad test command ran. Focused `rpptx` tests
  emitted only the three unrelated existing F-221 dead-code warnings.

## External evidence blockers

The PowerPoint oracle directory has no `manifest.tsv`, retained movie, or PNG
frames. The independent movie SHA-256 environment pin is unset. GUI automation
did not produce the required artifact set. The required pinned PowerPoint
differential has not run and is not passed. The full 50-deck SSIM rider was
interrupted and remains incomplete and unclaimed. These absent external results
are separate from D1.

## Explicit zero categories

No correctness defect outside D1 was found. No production panic path, schema
child-order defect, namespace binding defect, retained raw-XML defect, reverse
dependency, new dependency, unapproved trait, generic, feature flag, crate,
backend animation variant, runtime oracle dependency, or binary fixture defect
was found. No public OXML surface beyond the two exact approvals was added. No
name-cache or condition-presence-cache invariant defect was found. No ordinary
static-path execution or identity-cache dependency was found. No slide versus
layout or master target-scope leak, group-lineage loss, target mapping defect,
or group composition defect was found. No additional timing start, end, click
evaluation, hold, remove, transition, morph matching, geometry interpolation,
or endpoint defect was found. The external blockers and D1 mean the
differential gate is not passed.

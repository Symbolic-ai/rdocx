# F-214, all, pass 15

**Reviewed**: complete current worker diff against the F-214 base, 14
implementation files, 6,156 additions and 208 deletions, including both
approved untracked timeline modules, the approved plan and cited HLD sections,
progress notes, pass 14 review, fixed oracle diagnosis and temporary evidence,
and the two explicit OXML API approvals
**Verdict**: 0 defects, 0 smells, 0 nitpicks. The repository PowerPoint
differential gate remains unpassed because its ignored PNG and manifest set is
stale.

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Production fade and morph verification

- Ordinary fade now retains the outgoing page at full opacity and overlays the
  incoming page at transition progress. This is the source-over composition
  required by the fixed PowerPoint observation at
  `crates/rpptx-render/src/timeline.rs:91`. The exact-progress unit regression
  requires outgoing opacity 1.0 and incoming opacity 0.25 at
  `crates/rpptx-render/src/timeline.rs:672`.
- Compatible explicit-name morph retains the outgoing background at full
  opacity and overlays the incoming background at progress at
  `crates/rpptx-render/src/timeline.rs:294`. Its matched outgoing group remains
  opaque while the matched incoming group is multiplied by progress at
  `crates/rpptx-render/src/timeline.rs:367`. Progress zero therefore reproduces
  the outgoing pair exactly, as required at
  `crates/rpptx-render/src/timeline.rs:841`.
- Unmatched and incompatible morph shapes retain the planned independent
  fallback fades at `crates/rpptx-render/src/timeline.rs:307` and
  `crates/rpptx-render/src/timeline.rs:393`. Unsupported word and character
  morph options use the corrected ordinary source-over fallback and preserve a
  stable diagnostic at `crates/rpptx-render/src/timeline.rs:266`.
- The fixed fade artifact is pinned to its exact SHA-256, rendered at slide two
  and local 425 ms, normalized one-sided, and required to reach SSIM 0.99 at
  `crates/rpptx/tests/integration.rs:6311`. The focused fixed-artifact
  regression passed.

## Measurement and exact rational schedule

- Foreground measurement uses the opacity-invariant channel cutoff 240 at
  `crates/rpptx/tests/integration.rs:93`. The fixed raw appear and opacity
  hashes are pinned, cutoff 240 must preserve identical bounds, and the old
  cutoff 250 must expose different bounds at
  `crates/rpptx/tests/integration.rs:6231`.
- The shared generator and gate matrix now contains exactly nine cases at
  `crates/rpptx/tests/integration.rs:133`. It includes the five automatic
  evaluator observations, the slide-one terminal MAX/MAX outgoing observation,
  fade, calibrated morph `(3, 700, 0, 6336, 600)`, and calibrated push
  `(4, 264, 0, 8118, 600)`. Zoom is absent.
- The schedule regression requires unique encoded samples, locates morph within
  the corrected 10 to 11 second window, locates push within the corrected 13 to
  14 second window, and validates ordinary cases by exact rational arithmetic
  at `crates/rpptx/tests/integration.rs:5864`. Neighboring samples, the old
  `5742/600` morph sample, the old `6930/600` push sample, slide relabeling,
  timestamp substitution, timescale substitution, and outgoing sentinel
  substitution are rejected at `crates/rpptx/tests/integration.rs:5763`.
- Swift constructs the requested `CMTime` from the literal value and timescale,
  requests zero tolerance, and checks actual time equality at
  `crates/rpptx/tests/integration.rs:107`. Rust rejects equivalent but
  differently represented times and unequal actual times at
  `crates/rpptx/tests/integration.rs:6123`.
- The unchanged source makes external zoom unobservable. The approved-case
  query rejects zoom and the source's two zoom slides render byte-identically
  at `crates/rpptx/tests/integration.rs:5949`. Rust zoom direction and
  transformation coverage remains at `crates/rpptx-render/src/timeline.rs:710`
  and `crates/rpptx-render/src/timeline.rs:917`.

## Generator and gate contract

- Existing-movie ingestion requires the exact independent lowercase movie pin
  and refuses an unpinned overwrite at
  `crates/rpptx/tests/integration.rs:6397`. The generator iterates the same
  nine-case matrix, requires exact returned rational time, and emits source,
  movie, build, provenance, dimensions, classification, and per-frame hashes
  at `crates/rpptx/tests/integration.rs:6462`.
- The gate independently binds the source and movie to the exact hash and
  PowerPoint builds, rejects duplicate or substituted case tuples before
  comparison, and requires the outgoing terminal observation before fade at
  `crates/rpptx/tests/integration.rs:6751` and
  `crates/rpptx/tests/integration.rs:6814`.
- Every accepted row is re-extracted from the pinned movie at zero tolerance and
  must be byte-identical to its stored raw PNG before deterministic one-sided
  normalization at `crates/rpptx/tests/integration.rs:6860`. Candidate and
  normalized oracle dimensions must both be 2001 by 1125, geometry uses cutoff
  240 at literal 150 dpi, geometry error must be at most one point, and SSIM
  must be at least 0.99 at `crates/rpptx/tests/integration.rs:6936`. The final
  covered set must equal the exact shared nine-case matrix at
  `crates/rpptx/tests/integration.rs:7021`.
- The temporary out-of-tree corrected run against the unchanged pinned movie
  reports all nine geometry and SSIM rows passing, including fade SSIM
  0.999895, morph SSIM 0.998431, and push SSIM 0.997866 at
  `.claude/scratch/F-214-progress.md:731`. This is supporting diagnostic
  evidence only and is not the repository differential gate.

## Prior closures and approved OXML boundaries

- Sequence, parallel, click, alternative-condition, duration, fill, entrance,
  exit, emphasis, motion, finite-state, hold, remove, and endpoint semantics
  retain their prior fixes at `crates/rpptx-layout/src/timeline.rs:223` and
  `crates/rpptx-layout/src/timeline.rs:275`.
- Timestamped resolution remains additive. Slide, layout, and master identity
  scopes remain separate, group lineage is recovered from the correct source
  tree, and names remain attached to the same identity at
  `crates/rpptx-layout/src/context.rs:582`. Group-target clip conversion remains
  in declared group coordinates and maps back through each resolved shape at
  `crates/rpptx-layout/src/context.rs:3313`.
- Ordinary static rendering remains independent of timeline evaluation at
  `crates/rpptx/tests/integration.rs:6688`.
- The OXML public diff remains exactly the two user-approved methods:
  `ShapeTreeChild::non_visual_name(&self) -> Option<String>` at
  `crates/rpptx-oxml/src/shape_tree.rs:62`, and
  `CT_Timing::condition_has_explicit_target(node_id, end_condition, index) ->
  Option<bool>` at `crates/rpptx-oxml/src/timing.rs:278`.
- Shape-name parse and mutation caches remain synchronized at
  `crates/rpptx-oxml/src/shape_tree.rs:953` and
  `crates/rpptx-oxml/src/shape_tree.rs:1251`. Condition target-presence parsing
  remains namespace-aware and indexed by node, start versus end list, and list
  position at `crates/rpptx-oxml/src/timing.rs:1259`.

## Focused evidence

- Thirteen focused PowerPoint oracle contract regressions passed, including
  exact case binding, unique sample mapping, zoom exclusion, fixed opacity
  cutoff, exact extractor representation, raw identity helpers, provenance,
  source binding, normalization, and terminal fade binding.
- The fixed fade artifact regression passed. Focused fade, compatible morph,
  morph progress-zero, unsupported morph fallback, and zoom compositor
  regressions passed.
- Focused sequence and click evaluation, alternative start conditions,
  animation boundaries, ordinary static path, shape-name cache mutation,
  condition target presence, and compatibility-wrapped chart identity
  regressions passed.
- Required mode was invoked with the exact movie pin. It failed at the first
  gate-side AVFoundation re-extraction because this execution environment could
  not decode the movie, at the fail-closed assertion in
  `crates/rpptx/tests/integration.rs:6870`. No geometry or SSIM result from that
  invocation is claimed.
- `git diff --check` passed. No broad test command ran. Focused `rpptx` tests
  emitted only the three unrelated existing F-221 dead-code warnings.

## External evidence status

The ignored repository manifest still contains the superseded old morph and
push sample rows plus the excluded zoom row at
`corpus/pptx-timeline-oracle/manifest.tsv:26`. The reviewed source matrix and
gate reject those tuples at `crates/rpptx/tests/integration.rs:5805` and
`crates/rpptx/tests/integration.rs:6830`. The repository PNG and manifest set
has not been regenerated from the reviewed nine-case matrix. The required
PowerPoint geometry and SSIM differential is therefore unpassed. The temporary
out-of-tree metrics are not substituted for that gate, and the incomplete full
SSIM rider remains separately unclaimed.

## Explicit zero categories

No correctness, contract, production panic, OOXML, test, structure, or public
surface finding was found. No schema child-order defect, namespace binding
defect, retained raw-XML defect, reverse dependency, new dependency, unapproved
trait, generic, feature flag, crate, module, backend animation variant, runtime
oracle dependency, or committed binary fixture defect was found. No public
OXML surface beyond the two exact approvals was added. No name-cache or
condition-presence-cache invariant defect was found. No ordinary static-path
execution or identity-cache dependency was found. No slide versus layout or
master target-scope leak, group-lineage loss, target mapping defect, parent-clip
leak, or group-composition defect was found. No timing start, end, click
evaluation, hold, remove, transition algorithm, source-over fade, morph
matching, geometry interpolation, or endpoint defect was found. No rational
uniqueness, exact-time representation, calibrated local-time mapping, terminal
outgoing binding, manifest substitution, duplicate case, zoom-exclusion, raw
re-extraction, byte-identity, normalization, comparison geometry, cutoff, or
SSIM routing defect was found. The stale repository artifacts and failed local
movie decode mean the external differential gate remains unpassed.

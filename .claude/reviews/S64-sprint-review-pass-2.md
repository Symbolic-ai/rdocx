# S64 sprint review, pass 2

**Reviewed**: dirty `sprint/s64` at
`984d14b3518bab22e5a219064cdb5035f222c386` plus the seven-file pass-1
remediation against `0582da0a38886f5ceeb65ab9afcd0797f6fa14b0`, 80 files,
15,600 changed lines, crates: `oxml-chart`, `oxml-cli-support`, `oxml-core`,
`oxml-drawing`, `oxml-layout`, `oxml-media`, `oxml-opc`, `oxml-pdf`,
`oxml-sml`, `rdocx-wasm`, `rpptx-chart`, `rpptx-cli`, `rpptx-layout`,
`rpptx-oxml`, `rpptx-render`, `rpptx-wasm`, and `rpptx`
**Verdict**: 3 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, the recorded oracle is neither repeatable nor evidence for the signed no-repair deck

`crates/rpptx/tests/integration.rs:9001`

The ignored gate names one developer's ephemeral macOS temporary directories
and Downloads directory directly at `crates/rpptx/tests/integration.rs:9003`.
Those files happen to exist on the review machine and match the five embedded
hashes, but the test cannot be reproduced after those temporary files disappear
or on another checkout. It has no configurable oracle directory or checked
manifest.

The hashes also do not establish the claimed PowerPoint version, build, or
no-repair observation. The new test never calls the existing pinned-build check
at `crates/rpptx/tests/integration.rs:18271` and never consumes a recorded
no-repair observation. More importantly, it proves that PowerPoint received the
unsigned bytes at `crates/rpptx/tests/integration.rs:9044`, then constructs and
verifies a different signed deck only inside Rust at
`crates/rpptx/tests/integration.rs:9046`. That does not satisfy the one-deck
gate, which requires signatures to survive the same no-repair representative
round trip at `docs/hld/14-development-backlog.md:1896`. The HLD now exposes
the split by describing a signed source deck but binding PowerPoint evidence to
an unsigned source at `docs/hld/12-testing-strategy.md:865`.

The fix must use a portable, explicitly supplied test-only oracle location with
a pinned manifest that binds the exact signed source, PowerPoint version and
build, each output hash, and the performed no-repair observation. It must prove
that the deck opened and round-tripped by PowerPoint is the same signed,
macro-enabled representative deck whose semantic state is checked.

### B2, the visual gate can pass without checking Rust text order and its geometry sensitivity is synthetic

`crates/rpptx/tests/integration.rs:9130`

The static `text_order_matches` value is derived only from text extracted from
the PowerPoint PDF. It never examines Rust's rendered result. The animation
gate bypasses even that oracle-only check by passing literal `true` at
`crates/rpptx/tests/integration.rs:9211`. With the regional threshold set to
0.45 at `crates/rpptx/tests/integration.rs:8241`, a Rust text-order or glyph
content regression can therefore pass when the coarse ink bands remain similar.

The claimed geometry sensitivity does not mutate or rerun either raster. It
overwrites `geometry_error_px` in an already passing comparison and confirms
that seven is greater than six at `crates/rpptx/tests/integration.rs:9140`.
The text-order sensitivity similarly passes `false` directly at
`crates/rpptx/tests/integration.rs:9149`. These assertions test the final
Boolean expression, not whether the extraction and comparison pipeline detects
an isolated source or raster mutation. No corresponding animation mutation is
run.

The fix must derive exact expected and actual text order independently, use it
for static and all three movie samples, and apply isolated geometry and text
mutations to real inputs before recomputing the full predicate. Animation must
receive equivalent sensitivity coverage. The existing solid-raster mutation is
a useful real-input check and should remain.

### B3, adding `fonts` breaks the public frame type while the release promises no migration

`crates/rpptx/src/lib.rs:145`

`DeterministicTimelineFrame` was already a public struct with public fields.
Adding the required `fonts` field breaks downstream exhaustive struct literals
and patterns. That contradicts the release statement that the native additions
are additive and existing callers need no migration at `CHANGELOG.md:59`.

The field's narrower documentation is also inaccurate. Timeline preparation
resolves every slide through one font manager at `crates/rpptx/src/lib.rs:7536`,
then the frame returns `all_font_data()` at `crates/rpptx/src/lib.rs:3217`.
That function returns every loaded font at `crates/oxml-layout/src/font.rs:1679`,
not only programs referenced by glyph runs on the returned page as claimed at
`crates/rpptx/src/lib.rs:147`.

The fix must preserve the existing constructible frame shape or explicitly
declare and review a source-breaking 0.9 migration. Any new rendering access
must accurately describe whether it returns page-used fonts or the complete
prepared-presentation set.

## Should-fix

None. Count: 0.

## Nice-to-have

None. Count: 0.

## Pass-1 closure

Pass-1 S1 is closed. The generated summary now reports all 15 M21 stories done
and zero pending at `docs/sprints/BACKLOG.md:39`, and the overall totals agree
at `docs/sprints/BACKLOG.md:42`.

Pass-1 B1 is not closed because of B1 and B2 above. The source-built ordinary
test does combine comments, sections, SmartArt, exact audio, timeline,
signatures, macro-enabled package class, notes, and handout state at
`crates/rpptx/tests/integration.rs:8707`. It reopens and checks that state at
`crates/rpptx/tests/integration.rs:8842`, and the audio-poster exclusion is
bounded to its declared rectangle at `crates/rpptx/tests/integration.rs:8112`
with an explicit PowerPoint-versus-Rust pixel observation at
`crates/rpptx/tests/integration.rs:9156`. Notes and handout dimensions and
normalized geometry are checked at `crates/rpptx/tests/integration.rs:9233`.
Those are useful components, but the provenance, same-signed-deck, no-repair,
and sensitivity gaps prevent the combined milestone gate from holding.

## Milestone gate

The M21 gate at `docs/hld/14-development-backlog.md:1896` still does not hold.
The current implementation establishes a combined source fixture and
hash-pinned local artifacts, but it does not establish that the same signed
deck was opened without repair by the exact PowerPoint 16.104 build, and its
visual predicate does not independently compare Rust text order or exercise
real geometry sensitivity. Release and sprint closure remain blocked.

## Not found

- No GUI launch, printer invocation, tag, push, publication, or other external
  mutation was added by the remediation. The ignored gate invokes only
  read-only PDF and movie extraction helpers beginning at
  `crates/rpptx/tests/integration.rs:9111`.
- The user-provided manual PDFs remain test-only and are SHA-256 pinned at
  `crates/rpptx/tests/integration.rs:9035`. B1 concerns their path and provenance
  contract, not inclusion in production code.
- No unexplained hash expectation change was introduced. The HLD continues to
  require the unchanged 49-entry harness at
  `docs/hld/12-testing-strategy.md:862`.
- No new dependency, feature flag, crate-layering violation, HTML/PDF importer
  interaction defect, or unrelated production scope expansion was found in
  the seven-file remediation.

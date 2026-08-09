# F-116, all, pass 6

**Reviewed**: complete revised working tree diff, 3 files, 879 changed lines, comprising 870 insertions and 9 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the named four-viewer gate re-runs broken Keynote automation

`crates/rpptx/tests/integration.rs:222`

The ignored acceptance gate calls `assert_f116_keynote_acceptance` even though
the accepted Keynote result is user-confirmed human-action evidence. Direct
execution of this gate fails in that call. With Keynote closed, AppleScript
returns error `-600`. After Keynote is launched, the `open POSIX file` command
at `crates/rpptx/tests/integration.rs:4594` yields `missing value` and the script
returns error `-1700`. The user-confirmed evidence row remains valid, but the
named story gate cannot claim a clean four-viewer result while it always fails
before reaching LibreOffice and Google Slides. Make the Keynote portion use the
reviewed evidence consistently or make the executable automation complete
successfully.

## Smells

None.

## Nitpicks

None.

## Prior findings

- Pass-5 D1 is resolved at `crates/rpptx/tests/integration.rs:271`. The
  pending-state regression constructs an explicit
  `CrossViewerObservation::Pending` fixture, directly asserts that the clean
  validator rejects it, and no longer depends on a production evidence row
  remaining pending. The exact regression passes, and
  `cargo clippy -p rpptx --test integration -- -D warnings` is clean.
- Pass-3 D1 remains resolved at `crates/rpptx/tests/integration.rs:328` and
  `crates/rpptx/tests/integration.rs:4544`. Clean evidence rejects empty and
  whitespace-only export or close results.
- Pass-2 D1 remains resolved at `crates/rpptx/tests/integration.rs:360` and
  `crates/rpptx/tests/integration.rs:4514`. Clean evidence rejects missing or
  blank version, acceptance-date, and build metadata.
- Pass-1 D1 through D4 remain resolved. Evidence carries only observed slide
  counts, clean observations require every positive operation, ordinary save
  calls `Presentation::save`, and unignored writes use process-and-counter
  qualified temporary paths.

## Not found

- Correctness: zero findings. The generated candidate matches SHA-256
  `d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f`,
  produces exactly ten slides, validates, reopens, and preserves the asserted
  representative graph.
- Contract: no further findings beyond D1. The deck exercises the F-107 through
  F-115 write surfaces, including ordinary and slideshow saves, and all four
  viewer rows bind to the same artifact.
- Panics: zero findings. Changed Rust remains test-only code over a generated,
  trusted fixture. No production panic path changed.
- OOXML: zero findings. The ordinary and slideshow packages validate and
  reopen with the expected main content types, slide order, relationships, and
  media deduplication.
- Tests: no further findings beyond D1. The pending fixture makes its rejection
  regression non-vacuous. The exact pending and evidence-completeness tests
  pass, and the scoped warnings-as-errors Clippy gate is clean.
- Structure: zero findings. The diff adds no source module, test binary,
  production API, trait, generic, wrapper, feature, dependency, binary fixture,
  or durable generated deck.
- HLD consistency: zero findings. The feature matrix, viewer procedures,
  versions, builds, acceptance date, results, and artifact digest agree across
  the test constants, design plan, HLD 12, and progress record.
- External evidence and privacy: zero findings. Keynote remains explicitly
  identified as user-confirmed human-action evidence. Google Slides records the
  observed ten-slide and export results, acceptance date, and browser build
  without retaining the private imported-document URL or an account identifier.

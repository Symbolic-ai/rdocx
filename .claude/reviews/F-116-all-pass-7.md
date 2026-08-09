# F-116, all, pass 7

**Reviewed**: complete revised working tree diff, 3 files, 842 changed lines, comprising 833 insertions and 9 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the approved plan still requires Keynote automation

`.claude/plans/F-116-design.md:41`

The approved contract says to automate the locally scriptable viewers and its
Keynote step at line 46 requires an automated open, slide-count assertion,
warning observation, and close. The remediated gate deliberately does not run
Keynote automation. It validates the accepted user-confirmed evidence instead,
which now agrees with HLD 12 and the progress record. The executable behavior
is appropriate and the exact ignored gate passes, but the implementation still
contradicts its approved design plan. Update the plan to describe Keynote as
reviewed human-action evidence and reserve reruns for PowerPoint and
LibreOffice.

## Smells

None.

## Nitpicks

None.

## Prior findings

- Pass-6 D1 is resolved at `crates/rpptx/tests/integration.rs:218`. The named
  gate no longer invokes the broken Keynote script. It reruns PowerPoint and
  LibreOffice against the generated candidate, then validates all four clean
  evidence rows. The exact ignored gate passes in the application-enabled
  environment.
- Pass-5 D1 remains resolved at `crates/rpptx/tests/integration.rs:267`. The
  pending-state regression constructs an explicit pending fixture, directly
  asserts rejection, and is Clippy-clean.
- Pass-3 D1 and pass-2 D1 remain resolved. Clean evidence rejects blank
  operation results and missing or blank version, acceptance-date, and build
  metadata.
- Pass-1 D1 through D4 remain resolved. Evidence carries only observed slide
  counts, clean observations require every positive operation, ordinary save
  calls `Presentation::save`, and unignored writes use process-and-counter
  qualified temporary paths.

## Not found

- Correctness: zero findings. The candidate still matches SHA-256
  `d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f`,
  produces exactly ten slides, validates, reopens, and preserves the asserted
  representative graph.
- Contract: no further findings beyond D1. The deck exercises the F-107 through
  F-115 write surfaces, including ordinary and slideshow saves, and all viewer
  evidence rows bind to the same artifact.
- Panics: zero findings. Changed Rust remains test-only code over a generated,
  trusted fixture. No production panic path changed.
- OOXML: zero findings. The ordinary and slideshow packages validate and
  reopen with the expected main content types, slide order, relationships, and
  media deduplication.
- Tests: zero findings. The exact ignored gate and evidence-completeness test
  pass. The pending regression is non-vacuous, and the scoped
  warnings-as-errors Clippy gate is clean.
- Structure: zero findings. The diff adds no source module, test binary,
  production API, trait, generic, wrapper, feature, dependency, binary fixture,
  or durable generated deck.
- HLD consistency: zero findings. HLD 12 accurately says the ignored gate
  reruns PowerPoint and LibreOffice while retaining Keynote and Google Slides
  as reviewed human-action evidence.
- External evidence and privacy: zero findings. Keynote is explicitly
  user-confirmed. Google Slides records the observed ten-slide and export
  results, acceptance date, and browser build without retaining the private
  imported-document URL or an account identifier.

# F-116, all, pass 2

**Reviewed**: complete revised working tree diff, 3 files, 824 changed lines, comprising 818 insertions and 6 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, clean evidence accepts blank version and build metadata

`crates/rpptx/tests/integration.rs:4469`

`assert_clean_cross_viewer_evidence` rejects `None` for `version_or_date` and
`build`, but lines 4469 through 4477 accept `Some("")` and whitespace-only
values. After Keynote is observed clean, the Google row can therefore be
changed from pending to a structurally clean observation with an empty
acceptance date and browser build, and both the evidence test and ignored
viewer gate will pass. The design requires the exact service date and browser
build, not merely present option variants. Require trimmed nonempty metadata
and cover both blank cases in the negative schema regression.

## Smells

None.

## Nitpicks

None.

## Prior findings

- Pass-1 D1 is resolved at `crates/rpptx/tests/integration.rs:67` and
  `crates/rpptx/tests/integration.rs:101`. Pending observations now carry only
  a reason, so Keynote and Google Slides no longer record an unobserved count.
- Pass-1 D2 is resolved at `crates/rpptx/tests/integration.rs:265` and
  `crates/rpptx/tests/integration.rs:280`. A pending variant is rejected as
  clean, and the negative clean fixtures prove that open or import, ten
  observed slides, no repair or conversion error, and a close or export result
  are all required.
- Pass-1 D3 is resolved at `crates/rpptx/tests/integration.rs:181`.
  The ordinary package path now calls `Presentation::save`, reads the saved
  `.pptx`, and checks its main content type alongside the `.ppsx` path.
- Pass-1 D4 is resolved at `crates/rpptx/tests/integration.rs:138`,
  `crates/rpptx/tests/integration.rs:241`, and
  `crates/rpptx/tests/integration.rs:4451`. Every unignored file write now uses
  a process-and-atomic-counter-qualified path. Only the ignored external-viewer
  gate writes the stable reviewed candidate.

## Not found

- Correctness: no further findings. The ten-slide collection sequence and
  final title order remain coherent, and the reviewed candidate still hashes
  to `d36da6e8849eabd4487d2572baea19c3716ee7d0fe03aaa4714a28ce3c41de4f`.
- Contract: no further findings. The builder exercises the F-107 through F-115
  story surfaces in one package, including the ordinary and slideshow save
  APIs. Keynote and Google Slides remain honest pending human-action state
  rather than claimed successes.
- Panics: zero production findings. All changed Rust remains test-only code
  over generated trusted inputs. No library panic path changed.
- OOXML: zero findings. Ordinary and slideshow outputs check their distinct
  main content types, the generated graph validates cleanly, and the deck
  reopens through the facade with its final order and representative
  relationships intact.
- Tests: no further findings. Automatic candidate paths are race-free within
  and across test processes, the SHA is reproduced from the saved package, and
  the negative evidence regressions cover pending state and every required
  clean operation. The corrected PNG remains a valid test-only prerequisite.
- Structure: zero findings. The diff adds no file, source module, test binary,
  production API, trait, generic, forwarding wrapper, feature, or dependency.
- HLD and external evidence: zero findings. HLD 12 matches the exact feature
  matrix, pinned PowerPoint and LibreOffice evidence, and the incomplete
  Keynote and Google state. It does not promote either pending viewer to clean.

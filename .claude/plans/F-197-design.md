# F-197, Word SSIM harness

**Status**: completed
**Sprint**: S57
**Size**: L
**Depends on**: F-196

## Problem

The pinned Word corpus has no page-level visual comparison. Only Presentation
has an external render harness, in `scripts/pptx_ssim_harness.py:22`, and CI
only routes a `presentation-fidelity` job in `.github/workflows/ci.yml:27`.

The Word CLI already opens a package, performs deterministic layout, and emits
numbered PNGs through the production facade at
`crates/rdocx-cli/src/commands.rs:342`. The missing boundary is a fail-closed
orchestrator that compares every emitted page to a pinned external oracle and
records the trend without mistaking the trend threshold for a correctness
oracle.

## Spec reference

- `docs/hld/03-architecture.md`, "Three families, one workspace" and "The dependency rule".
- `docs/hld/08-rendering-spec.md`, "Performance".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy", "The render fidelity gate", and "What CI runs".
- `docs/hld/15-build-and-toolchain.md`, "Deterministic rendering" and "CI job matrix".
- `docs/hld/14-development-backlog.md`, "F-197, Word SSIM harness".
- `.claude/skills/differential-testing.md`, external render oracle rules.

## Approach

After F-196 establishes the exact manifest contract, add
`scripts/docx_ssim_harness.py`. For each verified document, invoke the existing
`rdocx-cli render` PNG path at 150 dpi in deterministic bundled-font mode.
Export the same document through an isolated, exact LibreOffice Writer
26.2.5.2 profile, then rasterise its PDF with exact Poppler 26.01.0 at 150 dpi.
Before Writer export, prepare an untracked accepted copy through the existing
`rdocx::Document::accept_all` API. The harness builds that one-function helper
locked and offline against the workspace, then reopens the copy and rejects any
remaining modeled revision. The pinned corpus remains unchanged.

Reuse the checked PNG decoder and the Presentation harness's deterministic
global-luminance SSIM calculation. Compare the union of page indices through
the larger Rust or oracle page count for each document. Composite unequal page
dimensions onto a shared white canvas sized to the maximum width and height.
When only one renderer produced a page at an index, compare it to a synthetic
blank white counterpart of the produced page's dimensions. Page-count and
dimension differences are scored fidelity outcomes, not orchestration errors.

Report every union page with its source dimensions, normalization action, and
SSIM. Report per-document Rust, oracle, and union page counts plus dimension
mismatch counts. Record minimum, median, maximum, coverage, and tool identity
in tab-separated results and JSON evidence.

Carry the Presentation trend reference of SSIM at least 0.95 on at least 80
percent of pages as a reviewed trend. A missed trend is visible evidence but
not the sole hard failure. Corpus mismatch, tool mismatch, renderer or oracle
failure, zero output, or a missing expected results or evidence artifact is a
hard failure. A focused sensitivity regression shifts a nonuniform page by one
pixel and proves the score moves. Focused union-coverage and normalization
regressions prove that unmatched pages and unequal dimensions cannot disappear
from the score set.

Add a path-filtered `word-fidelity` Ubuntu 24.04 CI job that installs the pinned
tools, fetches F-196, runs the harness, and retains evidence. Existing workflow
regressions prove the job and its aggregate-gate wiring fail closed.

## Rejected alternatives

- Add a new Rust render driver. The existing CLI already exercises the facade
  and avoids a second package interpretation path.
- Compare PDF bytes. PDF serialisation is not the visual contract.
- Reject page-count or dimension differences before scoring. Those differences
  are important fidelity evidence, and rejection hid 16 of the 18 union pages
  in the first reviewed calibration.
- Trust a fresh LibreOffice profile to select accepted view. Writer exported
  both deleted and inserted text from the legal fixture, so the harness must
  resolve revisions explicitly before it invokes the oracle.
- Make the trend threshold the only hard gate. LibreOffice is an imperfect
  external oracle, so completeness and unexplained regressions are the hard
  conditions.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| regression | harness self-tests | SSIM math, alpha compositing, union coverage, white-canvas normalization, tool pins, artifacts, and trend classification are exact |
| regression | `a_one_pixel_layout_shift_moves_word_ssim` | A visible page perturbation lowers the recorded score |
| regression | accepted-view preparation mutations | Removing `accept_all`, its reopen check, or the accepted-copy oracle input fails the harness tests |
| regression | union coverage and normalization mutations | Reverting union-index scoring, blank counterparts, or white-canvas compositing drops evidence or fails the test |
| regression | workflow contract tests | Word fidelity installs, fetches, checks, uploads evidence, and gates CI |
| differential | live full-corpus `--check` | Every page in the per-document union reports SSIM and page-count plus dimension evidence against the pinned LibreOffice oracle |

The **test gate** is regression. The harness reports per-page SSIM, and a
deliberate layout change moves it.

## HLD impact

- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/15-build-and-toolchain.md`

## Risk routing

- Layout and pagination. Read `docs/hld/08-rendering-spec.md`. Use bundled
  deterministic fonts for every Rust baseline and never record a system-font
  result.
- An external oracle comparison. Follow
  `.claude/skills/differential-testing.md`. Pin and record LibreOffice
  26.2.5.2 and Poppler 26.01.0, isolate the oracle profile, state the 150 dpi
  metric and tolerance, and reject tool-version drift.
- A new file. Read the structural rules in `CLAUDE.md` and obtain explicit
  approval for `scripts/docx_ssim_harness.py`. Add no production dependency,
  module, trait, generic, feature flag, or public API.

The consolidated sprint gate adds the harness self-tests, workflow regressions,
and one full required corpus comparison on the pinned environment.

## Hash harness

Expected to be unchanged. The story invokes existing deterministic output and
does not change rendering behaviour or the recorded sample baseline.

## Implementation checklist

- [x] Integrate F-196 and bind its exact manifest, count, and environment names.
- [x] Add the fail-closed SSIM harness and self-tests.
- [x] Use the existing CLI deterministic render path.
- [x] Pin and record the Writer and Poppler oracle environment.
- [x] Add union page coverage, white-canvas dimension normalization, trend, and perturbation regressions.
- [x] Add the Word fidelity CI job and workflow contract regressions.
- [x] Run a real corpus calibration and retain its reviewed evidence.
- [x] Run full verification and update exactly the listed HLD files.

## Open questions

None. The user approved the narrow in-sprint dependency interpretation and the
named new file. The user also approved the material revision after the first
calibration exposed page-count and one-pixel dimension differences. The
automated oracle is LibreOffice Writer 26.2.5.2 with Poppler 26.01.0, the
legal-revision view is accepted, and the 150 dpi 0.95 on 80 percent reference
remains advisory. Union-page scoring, shared white-canvas normalization, and
blank white counterparts are the reviewed fidelity contract.

# F-104, all aspects, pass 1

**Reviewed**: uncommitted worker diff, 27 files, 4,119 additions and 553
deletions
**Verdict**: 2 defects, 0 smells, 1 nitpick

## Defects

### D1, the revised acceptance contract is contradicted by authoritative inputs

`docs/hld/14-development-backlog.md:825`

The implemented harness treats 0.95 SSIM on 80 percent of slides as a trend,
but the backlog still names it as F-104's test gate. The same hard requirement
remains in `docs/sprints/CURRENT_SPRINT.md:54`, while the design plan still says
the harness enforces it at `.claude/plans/F-104-design.md:54`. The design plan's
HLD impact list at `.claude/plans/F-104-design.md:516` omits both
`docs/hld/02-scope-and-non-goals.md` and
`docs/hld/14-development-backlog.md`, even though both describe the changed
acceptance contract. Completion would therefore preserve mutually exclusive
definitions of done and violate the HLD update discipline. Revise the current
contract prose, add the affected HLD files to the impact list, and ensure the
integrator updates the active sprint definition before closure.

### D2, CI deletes the detailed trend evidence before the job ends

`scripts/pptx_ssim_harness.py:440`

The CI invocation supplies no output directory at
`.github/workflows/ci.yml:50`, so the harness writes its JSON and per-slide TSV
under a `TemporaryDirectory` and deletes that directory at
`scripts/pptx_ssim_harness.py:468`. Only the aggregate console summary survives.
That does not satisfy the revised contract that CI records full comparison
evidence, and it makes per-slide trend regressions unavailable after the job.
Give CI a stable job-local output directory and upload the evidence as an
artifact, or retain it through an equivalent durable mechanism.

## Smells

None.

## Nitpicks

- `crates/oxml-layout/src/line.rs:510`, the helper documentation says it slices
  stored glyphs and advances, but partial segments are now reshaped.

## Not found

Correctness beyond D1 and D2, panics on untrusted input, OOXML schema order,
namespace handling, unmodelled subtree preservation, dependency direction,
unsupported structural abstractions, and missing distinguishing renderer tests
produced no additional findings.

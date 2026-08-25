# F-197, all, pass 1

**Reviewed**: `work/f-197-codex` working tree, 7 files and 928 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, the Writer oracle does not enforce accepted revision view

`scripts/docx_ssim_harness.py:32`
`scripts/docx_ssim_harness.py:121`

The harness records `accepted` in evidence, but the LibreOffice invocation only
opens the original document through a fresh profile and exports it. It never
accepts tracked changes or configures an accepted revision view. The live legal
fixture export contains both the deleted `BH8810/2023/3` text and the inserted
`BH8810/2024/1` text on the same page, so the recorded identity is false and
the oracle is not comparing the requested view. Enforce acceptance before PDF
export without changing the pinned corpus, and add a regression that fails if
the accepted-view preparation is removed.

## Smells

None.

## Nitpicks

None.

## Not found

- **Union coverage and normalization**: all page indices through the larger
  page count are retained. Missing counterparts are synthesized as white pages,
  and unequal dimensions are composited top-left on an opaque shared canvas.
- **Hard and advisory outcomes**: corpus drift, tool drift, render failure, zero
  output, and missing artifacts fail closed. The 0.95 on 80 percent trend is
  recorded without becoming the sole hard gate.
- **CI and mutation coverage**: the filtered Word fidelity job installs the
  exact tools, fetches the exact corpus, runs the full check, uploads both
  artifacts, and participates in the aggregate result. Workflow mutations
  reject loss of those boundaries.
- **One-pixel sensitivity and scoring**: the focused perturbation changes the
  exact SSIM score, raw SSIM rejects dimension differences, and the harness
  tests prevent reverting union coverage, blank counterparts, or canvas
  normalization.
- **Panics, OOXML, API, and structure**: no production parser, OOXML ordering,
  namespace, preservation, dependency, public API, trait, generic, feature,
  crate, or module behavior changed. The approved test-infrastructure file
  imports the existing manifest, PNG decoder, and SSIM implementation directly.

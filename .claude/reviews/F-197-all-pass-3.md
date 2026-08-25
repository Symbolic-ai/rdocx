# F-197, all, pass 3

**Reviewed**: `work/f-197-codex` working tree, 7 files and 1,088 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- **Pass-2 trend source**: resolved. Classification at
  `scripts/docx_ssim_harness.py:374` consumes `COVERAGE_TARGET`, so JSON target
  evidence and the advisory verdict have one source.
- **Pass-1 accepted revision view**: resolved. The exact legal corpus document
  is accepted through the existing API before Writer export. Revised output is
  reopened and checked, while revision-free inputs remain byte-identical.
- **Coverage and normalization**: the larger page count defines the union. Each
  mismatch is scored on a shared white canvas, each absent page receives a
  white counterpart, and per-page original dimensions remain in the TSV.
- **Failure boundaries and trend**: exact corpus and tool drift, subprocess
  failure, timeout, zero pages, noncontiguous pages, and absent artifacts fail
  closed. The 0.95 on 80 percent trend remains advisory and fully reported.
- **Tests and CI**: sensitivity, acceptance, union, normalization, pins,
  artifacts, and trend regressions pass. Workflow mutations bind the filtered
  job, full check, both evidence artifacts, and aggregate result.
- **Panics, OOXML, API, dependencies, and structure**: no production code,
  package parser, OOXML sequence, namespace, preservation rule, public API,
  dependency, crate, module, trait, generic, wrapper, or feature changed. The
  sole approved new tracked file is the harness.

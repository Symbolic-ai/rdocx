# F-197, all, pass 2

**Reviewed**: `work/f-197-codex` working tree, 7 files and 1,088 changed lines
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, trend classification duplicates the coverage target

`scripts/docx_ssim_harness.py:35`
`scripts/docx_ssim_harness.py:374`

Evidence reports `COVERAGE_TARGET`, but `meets_coverage` independently embeds
the integer 80. A later reviewed target adjustment could therefore report one
target while classifying against another. Use the named constant in the
classification so the evidence and verdict cannot drift.

## Nitpicks

None.

## Not found

- **Pass-1 accepted revision view**: resolved. The oracle receives only an
  accepted copy prepared through the existing `Document::accept_all` API. The
  helper reopens revised output and rejects remaining modeled revisions, while
  documents without revisions are byte-copied unchanged. The legal oracle PDF
  contains inserted text and omits its deleted counterpart.
- **Union coverage and normalization**: all page indices through the larger
  count remain scored. Unequal dimensions use the shared white canvas, and
  unmatched pages receive a white counterpart without changing the hard gate.
- **Corpus and external tools**: the exact F-196 manifest, five-document count,
  immutable hashes, Writer build, Poppler version, 150 dpi, isolated profiles,
  and existing production render CLI are bound and fail closed.
- **Evidence and CI**: per-document page counts, dimension mismatches, unmatched
  pages, page-level dimensions and SSIM are retained in nonempty JSON and TSV.
  The filtered job installs, fetches, runs, uploads, and joins the aggregate
  result with mutation coverage for each boundary.
- **Panics, OOXML, API, and structure**: no production parser, OOXML ordering,
  namespace, preservation, dependency, public API, trait, generic, feature,
  crate, or module behavior changed. The acceptor is untracked test output,
  builds locked and offline, and uses the workspace target and existing API.

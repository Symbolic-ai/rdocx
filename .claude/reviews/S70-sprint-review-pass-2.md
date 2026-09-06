# S70 sprint review, pass 2

**Reviewed**: `sprint/s70` against
`59c47fb0877e2b1139f1d2c690bac57e1f8fb138`, 32 files, 4,479 changed
lines, crates: `rdocx`, `rdocx-layout`
**Verdict**: 0 blocking, 0 should-fix, 0 nice-to-have

## Blocking

None.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M23 end gate requires all five private references to be generated from a
blank public facade, reopen without repair, match required package semantics
and reviewed visual thresholds, produce identical DOCX bytes on repetition,
and report no unexplained preservation-only fallback. That end-of-milestone
gate is not yet due at this scheduled dependency-prefix boundary. F-242 and
F-243 through F-263 remain the explicit implementation path, so this review
does not claim the M23 gate is complete.

The completed prefix contracts do hold. F-241's public gate builds an offline
temporary consumer with exactly one public `rdocx` dependency and rejects raw
XML, base packages, and private-package aliases. Its required-private mode
passed anonymous P1 through P5 after exact identity, package graph, production
CLI projection, page geometry, and per-case SSIM checks. All 44 private
reference pages were visually inspected. The two Rust integration gates and
five mutation-focused harness self-tests passed, including the three defects
corrected after microscope pass 1.

F-X084's regressions prove that footnote and endnote text, insertion, and
deletion changes preserve at least 698 ordinary paragraph hits, rebuild at
most two paragraphs, and retain unaffected entries for the following
transaction. Warm and fresh layout plus provenance remain equal. Exact full
context still gates restart, table, header, footer, and note-page reuse.

Full integrated verification passed at
`dae856078f69792beb0920f3f795b21994bbb16e`. It included required-private P1
through P5, the public conformance path, the complete workspace suite, 107
workflow tests with two expected skips, warning-free rustdoc, README doctests,
22 package dry runs below 10 MiB, and the supply-chain gate. The hash harness
matched all 49 entries.

## Not found

- Interaction: the F-241 harness and F-X084 cache predicate have disjoint
  runtime ownership. Their shared testing HLD edits retain both contracts.
- Duplication: F-241 reuses the existing PNG decoder, SSIM implementation, and
  pinned-tool identity checks. It adds one conformance owner and no competing
  renderer or package parser path in production code.
- Layering: neither crate manifest changed, and no `oxml-*` dependency edge was
  added.
- Harness: all four completed AS_BUILT entries record the observed unchanged
  49-entry result.
- Gate: each completed-prefix claim has an executable regression or external
  evidence. The later M23 outcome is not claimed early.
- Docs: both Wave 2 plans changed exactly their listed HLD files, and the
  integrated testing strategy retains the Wave 1 and Wave 2 contracts.
- Dependencies: no Cargo dependency or feature changed.
- Surface: F-X084 changes private cache policy only. F-241 exercises existing
  public APIs and adds no library API.

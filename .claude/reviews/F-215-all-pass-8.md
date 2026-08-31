# F-215, all, pass 8

**Reviewed**: complete working diff against the F-215 worker base, 10 files,
4,726 additions and 62 deletions, plus the approved design, cited HLD sections,
progress notes, passes 1 through 7, and every default microscope aspect
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, duplicate-slide shape remapping omits valid ink timing targets

`crates/rpptx-oxml/src/shape_tree.rs:229`

`crates/rpptx-oxml/src/timing.rs:2580`

`crates/rpptx/src/lib.rs:1985`

Slide duplication assigns fresh values to modelled `p:cNvPr/@id` values, then
rewrites the shape references recognized by `rewrite_shape_ids`. The expanded
PresentationML reference classifier now covers `p:spTgt` and all four build
variants, but it still omits `p:inkTgt`. The timing parser already recognizes
that element as a valid timing target whose unsupported projection remains raw.
Its unqualified `spid` references an ink shape in the same slide shape-id
domain. A duplicated slide can therefore give that ink shape a fresh id while
leaving the retained `p:inkTgt/@spid` at the producer value. The copied timing
target then names no shape or a different shape that reused the old id.

## Smells

None.

## Nitpicks

None.

## Pass-7 follow-up

- D1 is fixed for `p:bldOleChart`. The shape-reference classifier requires the
  PresentationML namespace and includes the complete set of build variants,
  including `bldOleChart` (`crates/rpptx-oxml/src/shape_tree.rs:229`). It also
  requires the exact unqualified `spid` attribute
  (`crates/rpptx-oxml/src/shape_tree.rs:239`).
- The focused facade regression duplicates a slide through the production
  package path, proves the real `p:bldOleChart/@spid` follows the fresh shape
  id, and retains both a foreign element lookalike and a qualified attribute
  lookalike byte for byte (`crates/rpptx/tests/integration.rs:3613`).

## Not found

- Correctness beyond D1: structural timing-id allocation, inverse trim
  offsets, dual-source linked precedence, shape-scoped standard and Office
  source replacement, shared relationship retention, metadata-compatible
  deduplication, and candidate-only part pruning remain correctly implemented.
- Contract beyond D1: `oxml-media` owns signature and MIME checks and documents
  the boundary. Additions require a validated poster, linked targets remain
  exact and external without fetching, unsupported embedded bytes remain
  packaged and extractable, and replacement preserves geometry and timing.
- Panics: the added production indexing and `expect` sites are dominated by
  checked slide indices, parser-established roots, or fixed local
  construction.
- OOXML beyond D1: timing-id discovery and media removal use namespace-aware
  structural ownership, picture lookup and replacement use exact schema paths,
  new timing children follow schema order, and trim lexemes plus unrelated raw
  relationship attributes remain preserved.
- Tests beyond D1: the pass-7 remediation regression passes. The complete
  `rpptx-oxml` integration suite passes 143 tests, and the complete `rpptx`
  integration suite passes 132 tests with eight expected oracle ignores. No
  duplicate-slide regression covers a retained `p:inkTgt`.
- Structure and scope: no new trait, generic, feature, crate, module, file,
  dependency, forwarding wrapper, or builder was introduced. The native public
  surface remains within the approved pre-1.0 crates, and `rpptx-layout` only
  diagnoses the new retained media timing variants.

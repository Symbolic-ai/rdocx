# F-215, all, pass 9

**Reviewed**: complete working diff against the F-215 worker base, 10 files,
4,729 additions and 62 deletions, plus the approved design, cited HLD sections,
progress notes, passes 1 through 8, and every default microscope aspect
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass-8 follow-up

- D1 is fixed. The shape-reference classifier requires the exact
  PresentationML namespace and now includes `inkTgt` alongside `spTgt` and all
  four standard build variants
  (`crates/rpptx-oxml/src/shape_tree.rs:229`). It rewrites only the exact
  unqualified `spid` attribute (`crates/rpptx-oxml/src/shape_tree.rs:239`).
- Slide duplication invokes that classifier after relationship rewriting and
  before reparsing the duplicated slide (`crates/rpptx/src/lib.rs:1985`). The
  focused facade regression proves that a real `p:inkTgt/@spid` and
  `p:bldOleChart/@spid` follow the fresh `p:cNvPr/@id`, while foreign element
  lookalikes and qualified attribute lookalikes remain byte-exact
  (`crates/rpptx/tests/integration.rs:3613`).
- The shape-reference domain audit is complete. Standard timing targets in the
  remapped slide-shape domain are `spTgt` and `inkTgt`, and the standard build
  list contains `bldP`, `bldDgm`, `bldGraphic`, and `bldOleChart`. All six are
  classified (`crates/rpptx-oxml/src/shape_tree.rs:229`). `sldTgt` has no shape
  id and `sndTgt` owns a relationship id, as recorded in the implementation
  audit (`.claude/scratch/F-215-progress.md:35`). Remaining `ST_ShapeID` uses
  such as nested legacy-diagram `subSp` do not refer to the slide
  `p:cNvPr/@id` map and must remain unchanged.

## Not found

- Correctness: no new defect was found in structural timing-id allocation,
  inverse trim offsets, dual-source linked precedence, shape-scoped standard
  and Office source replacement, shared relationship retention,
  metadata-compatible deduplication, candidate-only part pruning, or complete
  duplicate-slide shape-reference rewriting.
- Contract: `oxml-media` owns format-neutral signature and MIME checks and
  documents the boundary. Additions require a validated poster, linked targets
  remain exact and external without fetching, unsupported embedded bytes
  remain packaged and extractable, and replacement preserves geometry and
  unrelated timing.
- Panics: the added production indexing and `expect` sites remain dominated by
  checked slide indices, parser-established roots, or fixed local
  construction.
- OOXML: timing-id discovery and media removal use namespace-aware structural
  ownership, picture lookup and replacement use exact schema paths, new timing
  children follow schema order, and trim lexemes plus unrelated raw XML and
  relationship attributes remain preserved.
- Tests: the pass-8 shape-target regression passes, as do seven focused
  low-level timing regressions and the format-neutral media signature test. The
  previously recorded complete affected suites and routed gates remain green.
- Structure and scope: no new trait, generic, feature, crate, module, file,
  dependency, forwarding wrapper, or builder was introduced. The native public
  surface remains within the approved pre-1.0 crates, and `rpptx-layout` only
  diagnoses the retained media timing variants.

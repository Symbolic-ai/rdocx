# F-215, all, pass 7

**Reviewed**: complete working diff against the F-215 worker base, 10 files,
4,683 additions and 62 deletions, plus the approved design, cited HLD sections,
progress notes, passes 1 through 6, and every default microscope aspect
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, duplicate-slide shape remapping omits valid OLE chart build targets

`crates/rpptx-oxml/src/shape_tree.rs:229`

`crates/rpptx/src/lib.rs:1985`

Slide duplication assigns fresh values to every modelled `p:cNvPr/@id` and now
rewrites PresentationML shape references for media timing and three build
variants. The reference classifier omits `p:bldOleChart`, whose `spid`
attribute belongs to the same slide shape-id domain as `p:bldP`, `p:bldDgm`,
and `p:bldGraphic`. A duplicated slide containing an OLE chart build therefore
keeps the producer shape id after its chart shape receives a fresh id. The
copied build points at no shape or at a different shape that reused that id,
so the duplicate-slide remapping is incomplete.

## Smells

None.

## Nitpicks

None.

## Pass-6 follow-up

- D1 is fixed for structural timing-id ownership. Allocation walks the direct
  PresentationML timing list, every schema timing-node kind, each kind's exact
  common-node slot, and nested child and sub timing lists
  (`crates/rpptx-oxml/src/timing.rs:515`,
  `crates/rpptx-oxml/src/timing.rs:584`, and
  `crates/rpptx-oxml/src/timing.rs:603`). Foreign wrappers remain outside that
  traversal, while the unsupported `p:animClr` regression allocates after its
  owned id (`crates/rpptx-oxml/tests/integration.rs:242`).
- D2 is fixed. Standard replacement recognizes only the direct
  `p:pic/p:nvPicPr/p:nvPr/a:audioFile` or `a:videoFile` slot and splices only
  its relationships-namespace `link` value
  (`crates/rpptx-oxml/src/picture.rs:1066` and
  `crates/rpptx-oxml/src/picture.rs:1128`). The package regression retains the
  unrelated raw relationship reference, its old relationship record, and its
  payload (`crates/rpptx/tests/integration.rs:423`).
- D3 is fixed. Nonnumeric `playFrom` and `seek` bodies fall back to the exact
  `MediaCommandKind::Other` spelling
  (`crates/rpptx-oxml/src/timing.rs:1728`), and the focused test retains its
  complete command XML byte for byte
  (`crates/rpptx-oxml/tests/integration.rs:269`).
- D4 is fixed. Known MP3, RIFF WAVE, and ISO base media MIME names compare
  case-insensitively (`crates/oxml-media/src/lib.rs:128` and
  `crates/oxml-media/src/lib.rs:145`). The facade continues to reject a known
  signature mismatch before staging package state
  (`crates/rpptx/src/lib.rs:2854`), including the uppercase regression
  (`crates/rpptx/tests/integration.rs:1193`).

## Not found

- Correctness beyond D1: inverse trim offsets, dual-source linked precedence,
  shape-scoped standard and Office source replacement, shared relationship
  retention, metadata-compatible deduplication, and candidate-only part
  pruning remain correctly implemented.
- Contract beyond D1: `oxml-media` owns signature and MIME checks and documents
  the boundary. Additions require a validated poster, linked targets remain
  exact and external without fetching, unsupported embedded bytes remain
  packaged and extractable, and replacement preserves geometry and timing.
- Panics: the added production indexing and `expect` sites are dominated by
  checked slide indices, parser-established roots, or fixed local
  construction.
- OOXML beyond D1: timing-id discovery uses namespace-aware schema ownership,
  timing removal stops at unmodelled wrappers, picture lookup and replacement
  use exact schema paths, new timing children follow schema order, and trim
  lexemes and unrelated raw relationship attributes remain preserved.
- Tests beyond D1: the four focused pass-6 regressions pass, as do the broader
  media-focused low-level tests and the existing shape-id remapping regression.
  The remaining build variant is not represented in that remapping test.
- Structure and scope: no new trait, generic, feature, crate, module, file,
  dependency, forwarding wrapper, or builder was introduced. The native public
  surface remains within the approved pre-1.0 crates, and `rpptx-layout` only
  diagnoses the new retained media timing variants.

# F-173, all, pass 4

**Reviewed**: the remediated working tree on `work/f-173-codex`, 21 tracked
feature files plus one approved new file, 2,635 additions and 256 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: zero findings. Descriptionless inline Image and Group leaves now
  become artifact `MarkedContent` at
  `crates/rdocx-layout/src/paginator.rs:2279`. Informative Figure wrappers stay
  intact at `crates/rdocx-layout/src/paginator.rs:2257`, retain their source
  alternate text, and receive their Figure node before pagination. The focused
  image and group regression at `crates/rdocx-layout/src/paginator.rs:2805`
  covers the corrected artifact path.
- End-to-end ownership: zero findings. The real Word fixture follows
  `L -> LI -> LBody` recursively and requires an actual depth-three list at
  `crates/rdocx/tests/integration_test.rs:3128`. For every page, it checks the
  unique deterministic `/StructParents` key, resolves the ParentTree array,
  requires contiguous page-local stream MCIDs, and matches every array slot to
  exactly one owner MCR with the same `/Pg` and `/MCID` at
  `crates/rdocx/tests/integration_test.rs:3063`. Set and length equality reject
  missing, duplicate, swapped, cross-page, or extra MCRs. The test also follows
  both `TH` nodes to their paragraph children and requires repeated owned paint
  at `crates/rdocx/tests/integration_test.rs:3143`.
- Reading order: zero findings. The structure builder attaches an anchored
  Figure after its source paragraph as a sibling, while inline Figures remain
  children at `crates/rdocx-layout/src/engine.rs:3094`. This separates Word
  reading order from the earlier behind-document paint MCID. Contentless nodes
  retain semantic child order, and valid mixed-content nodes preserve their
  document-order `/K` sequence.
- Malformed public input: zero findings. Graph validation rejects non-contiguous
  or missing ids, bad roots, multiple parents, cycles, unreachable nodes, and
  unknown marked ids at `crates/oxml-pdf/src/structure.rs:259`. When preparation
  is rejected, semantic containers recurse without BDC or MCIDs, while explicit
  artifacts remain artifact operators at `crates/oxml-pdf/src/writer.rs:1019`.
- Contract: zero findings. `PositionedElement::MarkedContent` remains the sole
  positioned semantic ownership carrier. Exact leaf wrapping, recursive
  walkers, empty-carrier omission, deterministic structure allocation,
  page-local MCIDs, ParentTree arrays, `/MarkInfo`, `/Lang (und)`, alternate
  text, table column scope, and conditional PDF/UA metadata match the approved
  design and HLD.
- Panics: zero findings. Production indexing and `expect` sites added by the
  feature are protected by validated graph or sequential-id invariants. The
  public malformed-graph path degrades without recursion through a rejected
  graph.
- OOXML: zero findings. The diff changes no OOXML parser or serializer, schema
  child order, namespace handling, or unmodelled-subtree preservation path.
- Tests: zero findings. The regression gate now proves all six heading roles,
  real three-level list topology, multipage repeated header ownership, exact
  MCID and MCR ownership, artifact handling, figure alternate text, contentless
  order, raster equality, and malformed-graph fallback. The focused pass-4
  tests and the recorded full workspace rerun are green.
- Structure: zero findings. The sole new module is the approved
  `crates/oxml-pdf/src/structure.rs`. No new trait, generic parameter, crate,
  feature flag, forwarding wrapper, or parallel carrier was introduced.
- Public compatibility: zero findings. Existing `InlineItem` and `LineItem`
  Image and Group variants retain exactly their old required fields at
  `crates/oxml-layout/src/line.rs:87` and
  `crates/oxml-layout/src/line.rs:175`. Informative drawings use additive Figure
  variants on the already non-exhaustive enums, and direct construction tests
  preserve the old source form.
- Untagged output: zero findings. Presentation still produces
  `structure: None` at `crates/rpptx-render/src/lib.rs:3340`. The untagged writer
  path gains no accessibility catalog entry or fallback title.
- Hash and resources: zero findings. Relative to the claim base, the 49-entry
  baseline changes exactly the approved 14 `pdf/pages` and `pdf/bytes` entries.
  All seven page-one PNGs, PDF resource fingerprints, and OOXML entries remain
  unchanged. The narrow metadata-stream exclusion does not remove metadata from
  the complete PDF byte digest.
- External oracle: zero findings. Six generated PDFs are tagged and declare
  PDF/UA. `feature_showcase.pdf` remains tagged but omits that declaration. The
  recorded pinned veraPDF 1.30.2 evidence has the in-code fixture and six
  declaring samples passing `ua1`, while the forced showcase check fails on its
  pre-existing `.notdef` glyph as expected.

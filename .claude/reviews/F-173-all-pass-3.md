# F-173, all, pass 3

**Reviewed**: the remediated working tree on `work/f-173-codex`, 21 tracked
feature files plus one approved new file, 2,419 additions and 255 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, descriptionless inline drawings enter paragraph semantics instead of artifact content

`crates/rdocx-layout/src/engine.rs:3543`
`crates/rdocx-layout/src/paginator.rs:2279`

An inline image or chart with a missing, blank, or whitespace-only
`wp:docPr/@descr` remains the ordinary `Image` or `Group` item. Pagination then
wraps every such item with the paragraph structure id. The drawing therefore
receives a semantic MCID below `P` and enters the reading order without a
`Figure` node or alternate text. The equivalent anchored drawing keeps
`structure_id: None` and becomes an artifact. The approved contract says
decorative drawing operations are artifacts, so the inline and anchored paths
currently disagree for the same descriptionless source case.

### D2, the multipage regression does not prove list topology or exact page ownership

`crates/rdocx/tests/integration_test.rs:2957`
`crates/rdocx/tests/integration_test.rs:2979`
`crates/rdocx/tests/integration_test.rs:3000`

The real `Document` fixture is valuable, but its assertions are weaker than the
contract recorded in the HLD. Three or more `/S /L` substrings also pass when
the lists are siblings, so the test does not prove a true three-level
`L -> LI -> L -> LI -> L` chain. Each page is checked only for the presence of
`/StructParents`, without checking distinct keys or following those keys into
the ParentTree arrays. Finally, equality between the total number of stream
MCIDs and total number of MCR dictionaries does not prove that every page-local
MCID occurs once at the matching ParentTree slot and once in an MCR with the
correct `/Pg` and `/MCID`. Duplicate, swapped, or cross-page ownership can
therefore keep the regression green. Repeated `TH` content is followed to its
paragraph, but those MCRs are counted rather than individually reconciled with
their page-local marked content.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness beyond D1: rejected or malformed public structure graphs emit no
  structure tree and unwrap semantic containers without emitting orphan MCIDs.
  Explicit artifact containers remain `/Artifact BMC`. Behind-document figures
  are siblings after their source paragraph in the semantic tree, independent
  of their earlier paint position. Contentless semantic nodes retain source
  order.
- Contract beyond D1 and D2: `MarkedContent` remains the sole positioned
  ownership carrier. Exact leaf wrappers, recursive walkers, page-local MCID
  allocation, deterministic structure references, ParentTree serialization,
  `/MarkInfo`, alternate text, and conditional PDF/UA metadata have no further
  finding.
- Panics: zero findings. The new production `expect` is guarded by the
  non-zero sequential structure-id invariant, and validated graphs prevent the
  former cyclic traversal case.
- OOXML: zero findings. No parser or serializer path changes, schema child
  order changes, namespace changes, or unmodelled-subtree loss were introduced.
- Tests beyond D2: zero findings. Focused coverage exercises malformed-graph
  fallback, contentless order, heading roles, table scope, figure alternate
  text, artifact operators, `.notdef` claim suppression, raster equality, and
  unchanged source construction of the existing Image and Group variants.
- Structure: zero findings. The only new module is the approved
  `crates/oxml-pdf/src/structure.rs`. No trait, generic parameter, crate,
  feature flag, forwarding wrapper, or parallel semantic carrier was added.
- Public compatibility: the existing `InlineItem` and `LineItem` Image and
  Group variants retain their old required fields. Informative content uses
  additive Figure variants on the already non-exhaustive enums.
- Untagged Presentation behavior: Presentation still produces
  `structure: None`, and the untagged metadata path does not gain the fallback
  title or accessibility catalog entries.
- Hash and resource invariants: the baseline changes exactly the approved 14
  `pdf/pages` and `pdf/bytes` entries. All seven PNG, OOXML, and PDF resource
  entries are unchanged. The resource fingerprint excludes only metadata
  streams, while the full byte digest still covers them.
- External oracle honesty: six generated PDFs contain the PDF/UA declaration,
  while `feature_showcase.pdf` remains tagged and contains no declaration. The
  recorded pinned veraPDF 1.30.2 evidence reports the six declaring samples and
  the in-code fixture passing `ua1`, with the forced showcase check failing on
  its pre-existing `.notdef` glyph as expected.

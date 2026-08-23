# F-173, all, pass 2

**Reviewed**: the remediated working tree on `work/f-173-codex`, 18 tracked
implementation files plus one approved new file, 1,967 additions and 170
deletions
**Verdict**: 3 defects, 1 smell, 0 nitpicks

## Defects

### D1, rejected structure graphs still emit orphan MCIDs

`crates/oxml-pdf/src/writer.rs:1016`

`PreparedStructure::new` now rejects cycles, missing nodes, duplicate parents,
and unknown marked ids, but `write_pdf` turns that rejection into
`tagged = None`. The content emitter still handles every
`MarkedContent { structure: Some(_) }` as `/Span <</MCID n>> BDC` when its
prepared structure is absent. The page then has no `/StructParents`, catalog
`/StructTreeRoot`, or ParentTree entry for those MCIDs. A malformed public graph
is therefore not cleanly rejected or degraded. It leaves unowned semantic
markers in an otherwise untagged PDF. The new validation test calls only
`PreparedStructure::new`, so it does not observe the writer fallback.

### D2, behind-document figures use paint order as reading order

`crates/rdocx-layout/src/paginator.rs:1053`

A `behindDoc` drawing is deliberately moved before all body elements in the
page content stream. `PreparedStructure::ordered_kids` at
`crates/oxml-pdf/src/structure.rs:212` then sorts the paragraph's own marked
text and Figure children by those MCIDs. For a paragraph whose source text
precedes an informative anchored drawing, the early paint MCID moves the Figure
before the text in `/K`. The structure builder also appends every anchored
Figure after scanning all line items at
`crates/rdocx-layout/src/engine.rs:3121`, so the original run boundary is no
longer available to repair this. Visual z-order is not Word source order, and
the resulting reading order violates the approved document-order contract.

### D3, the approved regression gate is still not exercised end to end

`crates/oxml-pdf/src/writer.rs:2065`

The named heading and nested-list regression still constructs a synthetic
writer tree with only `H1`, one `L`, and one `LI`, then checks unordered PDF
substrings. It has no `H2` through `H6`, nested list, Word input, or ordering
assertion. The table regression at line 2099 still uses one page and does not
exercise repeated headers or prove exclusive content ownership. The internal
Word structure test at `crates/rdocx-layout/src/engine.rs:8491` proves a
pre-pagination node sequence, but never renders that Word result to PDF or
checks MCIDs and ParentTree links. The pass-1 fixes added useful focused tests,
yet the approved test gate still would not fail for a broken Word-to-PDF handoff
or the behind-document ordering defect above.

## Smells

### S1, semantic propagation expands existing public enum variants

`crates/oxml-layout/src/line.rs:87`

`alternate_text` and `structure_id` were added as required fields to the
existing public `InlineItem::Image`, `InlineItem::Group`, `LineItem::Image`, and
`LineItem::Group` struct variants. `#[non_exhaustive]` protects downstream
matches, but it does not let an existing downstream constructor omit newly
required fields. The risk record describes the published semantic surface as
additive, while these changes are source-breaking for Rust callers that build
line items directly. Either keep propagation additive or record and gate the
intentional pre-1.0 source break explicitly.

## Nitpicks

None.

## Not found

- Complex-table conformance: every `TH` now carries column scope. The current
  generated complex table no longer produces the prior veraPDF table finding.
- PDF/UA claim honesty: the six generated samples that declare PDF/UA all pass
  pinned veraPDF 1.30.2 profile `ua1`. `feature_showcase.pdf` remains tagged
  with a structure tree and `/Lang (und)`, but has no PDF/UA identification
  metadata because its shown text contains glyph zero.
- Inline chart alternate text: chart groups now retain the source description,
  receive a Figure id, and lower through the same figure path as images.
- Contentless container order: structure nodes with no own occurrences now
  retain their semantic child order, and a focused regression covers an empty
  node before a visible one.
- Untagged output: metadata without a title no longer gains the tagged fallback
  title. The untagged Presentation structure remains absent.
- Language overclaim: tagged output now uses the honest undetermined language
  tag rather than claiming English for every document.
- Graph validation itself: the iterative validator rejects missing roots,
  non-document roots, non-contiguous ids, multiple parents, cycles, unreachable
  nodes, and marked ids absent from the tree without recursive traversal.
- OOXML and dependency structure: no parser or serializer changed, no
  format-neutral crate gained a format dependency, and no trait, generic,
  feature flag, crate, or unapproved module was introduced.
- Semantic carrier ownership: `MarkedContent` remains the sole positioned
  ownership carrier. No page sidecar or parallel carrier was added.
- Hash and visual invariants: the baseline still contains exactly the approved
  14 `pdf/pages` and `pdf/bytes` changes. Page-one pixels, OOXML parts, and
  resource entries remain unchanged, and metadata remains covered by the byte
  digest.

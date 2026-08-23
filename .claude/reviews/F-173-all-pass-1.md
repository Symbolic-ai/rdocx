# F-173, all, pass 1

**Reviewed**: the working tree on `work/f-173-codex`, 18 tracked files plus
one approved new file, 1,669 additions and 182 deletions
**Verdict**: 7 defects, 1 smell, 0 nitpicks

## Defects

### D1, complex tables do not expose usable header associations

`crates/oxml-pdf/src/structure.rs:108`

Every table header cell is emitted as `TH`, but no header attributes are
written. The generated `samples/feature_showcase.pdf` fails veraPDF 1.30.2
profile `ua1` at ISO 14289-1 clause 7.5 because a data cell's header cannot be
determined algorithmically. The simple synthetic table passes only because its
single row needs no disambiguation. A merged or otherwise complex table is
therefore tagged but not PDF/UA conforming.

### D2, the writer declares PDF/UA for content that is not conforming

`crates/oxml-pdf/src/writer.rs:558`

Every tagged Word result receives the PDF/UA-1 identification metadata without
validating the actual document. The generated `samples/feature_showcase.pdf`
fails veraPDF 1.30.2 profile `ua1` at ISO 14289-1 clause 7.21.8 because six text
operators reference the `.notdef` glyph. This produces a file that declares
PDF/UA-1 while violating it. The oracle regression cannot catch this because
its fixture contains only a rectangle and no font, list marker, image, or table
content.

### D3, inline chart descriptions never reach a Figure node

`crates/rdocx-layout/src/engine.rs:3509`

The inline chart branch lowers directly to `InlineItem::Group` and drops
`inline.description`. Only the image branch at line 3523 carries alternate text
and a structure id. An inline Word chart with a `docPr` description is therefore
owned by its paragraph rather than a `Figure`, and the source description never
becomes `/Alt`. Anchored charts do not have this problem because their drawing
carrier retains the description.

### D4, contentless semantic nodes are moved out of source order

`crates/oxml-pdf/src/structure.rs:194`

`ordered_kids` sorts every logical child by its first MCID and assigns
`(usize::MAX, i32::MAX)` to a child with no marked-content occurrence. Empty
paragraphs intentionally have no MCID, so an empty paragraph before a visible
paragraph is moved after it in the document `/K` array. The same swap can occur
between empty and non-empty table cells. This contradicts the document-order
tree contract and turns the required empty-carrier omission into a reading-order
regression.

### D5, the untagged writer path is no longer byte compatible

`crates/oxml-pdf/src/writer.rs:533`

The document-info branch now writes `/Title (Untitled document)` whenever
metadata exists but its title is absent, even when `tagged` is `None`. Before
this change an untagged layout with author, subject, keywords, or creator but no
title omitted `/Title`. This changes existing Presentation and direct
`oxml-pdf` output despite the HLD statement that a result with no structure
retains the byte-compatible untagged path. The Presentation regression checks
only that `layout.structure` is `None`, so it cannot detect the PDF change.

### D6, the document language is hard-coded to English

`crates/oxml-pdf/src/writer.rs:519`

Every tagged PDF receives `/Lang (en-US)` regardless of the Word source. A
French, Arabic, or mixed-language document is therefore tagged with incorrect
natural-language metadata. Presence is enough for the current oracle fixture,
but assistive technology receives the wrong language and the semantic output is
not faithful to the source.

### D7, the regression gate does not prove the approved contract

`crates/oxml-pdf/src/writer.rs:2045`

The heading and nested-list test constructs no Word input, includes only `H1`,
contains only one list level, and checks unordered PDF substrings. The table
test at line 2080 has one page and checks only role names, so it does not test a
repeated header, MCID ownership, ParentTree consistency, or the complex table
case that fails veraPDF. The figure test at line 2111 contains no artifact and
does not exercise Word alternate-text propagation. The golden test at
`crates/oxml-pdf/src/lib.rs:91` checks raster equality but not its named
page-resource invariant. These tests would remain green if source nesting,
document order, repeated headers, exclusive ownership, or artifact marking
regressed, so the approved regression gate has not been implemented.

## Smells

### S1, the public structure graph has no validation boundary

`crates/oxml-pdf/src/structure.rs:223`

`DocumentStructure` has public nodes and child ids, while `first_occurrence`
recurses through them without cycle detection. A caller can construct a cycle,
duplicate ids, a missing root, or a marked-content id absent from the node map.
Those states can cause unbounded recursion or a catalog and ParentTree with
dangling or missing entries. The in-tree Word producer is currently acyclic,
but the newly published carrier makes these invalid states reachable by Rust
callers.

## Nitpicks

None.

## Not found

- OOXML parser and serializer defects: no OOXML parsing or serialization path
  changed in this diff.
- Dependency-boundary violations: `oxml-*` remains independent of Word and
  Presentation crates.
- Unapproved structure: the only new module is the explicitly approved
  `crates/oxml-pdf/src/structure.rs`. No new trait, generic parameter, crate, or
  feature flag was introduced.
- Parallel semantic ownership: `MarkedContent` remains the sole positioned
  ownership carrier. No page sidecar or second carrier was added.
- Valid-tree MCID accounting: for the in-tree acyclic producer, semantic MCIDs
  are page-local and ParentTree arrays follow their numeric order. Artifact
  wrappers do not consume MCIDs, and empty no-glyph carriers are omitted.
- Hash and visual invariants: the baseline records exactly the approved 14
  `pdf/pages` and `pdf/bytes` changes. The metadata exclusion is limited to
  `/Type /Metadata`, while `pdf/bytes` still covers it, and page-one PNG and
  resource entries remain unchanged.

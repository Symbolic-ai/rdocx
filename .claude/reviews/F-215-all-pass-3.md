# F-215, all, pass 3

**Reviewed**: complete working diff against the F-215 worker base, 10 files,
3,124 additions and 9 deletions, plus the approved design, cited HLD sections,
progress notes, pass-1 and pass-2 findings, and every default microscope aspect
**Verdict**: 3 defects, 1 smell, 0 nitpicks

## Defects

### D1, replacement does not change the Office media source attribute

`crates/rpptx/src/lib.rs:2461`

`crates/rpptx-oxml/src/picture.rs:209`

`crates/rpptx-oxml/src/picture.rs:214`

`replace_media` discards the new embedded or linked source variant returned by
`add_media_relationships`, then the picture helper rewrites only relationship
values. Replacing an embedded source with a linked source therefore leaves
`p14:media@r:embed` pointing at an external relationship. The inverse leaves
`p14:media@r:link` pointing at an internal relationship. On a producer element
that carries both attributes, rewriting the selected linked value also keeps
linked precedence even when the requested replacement is embedded. The add
path chooses the attribute from the source kind, but replacement must make the
same modelled change while preserving unrelated raw XML.

### D2, removing dual-source media leaves its embedded relationship and part

`crates/rpptx-oxml/src/picture.rs:1023`

`crates/rpptx-oxml/src/picture.rs:1025`

`crates/rpptx/src/lib.rs:2552`

Pass 2 correctly made `r:link` take precedence when both Office media
attributes are present, but the projection retains only that winning
relationship id. `remove_media` consequently treats the selected link, the
standard relationship, and the poster as owned. It never considers the
coexisting `r:embed` relationship. Removing a valid dual-source picture leaves
that now-unreferenced relationship in the slide scope, which keeps its embedded
payload reachable and prevents the owned candidate from being pruned.

### D3, standard media lookup can select a relationship from retained nested XML

`crates/rpptx-oxml/src/picture.rs:689`

`crates/rpptx-oxml/src/picture.rs:721`

`crates/rpptx-oxml/src/picture.rs:729`

The cached standard media relationship is found by recursively searching the
complete `p:nvPicPr` subtree for the first DrawingML `audioFile` or `videoFile`.
The schema-owned attachment is a direct child of `p:nvPr`. A retained extension
under an earlier non-visual child can legally carry arbitrary payload and can
therefore contain the same expanded element name. In that case the recursive
search caches the extension's relationship instead of the direct attachment.
Replacement or removal then rewrites or deletes the unrelated relationship and
leaves the actual standard media relationship unchanged.

## Smells

### S1, the timing projection exposes permanently empty trim fields

`crates/rpptx-oxml/src/timing.rs:191`

`crates/rpptx-oxml/src/timing.rs:1323`

`CommonMediaNode` still publicly exposes `trim_start_ms` and `trim_end_ms`, but
the only constructor assigns both fields `None`. Pass 1 established that trim
belongs to `p14:media/p14:trim` on the picture, and pass 2 made timing insertion
truthful by removing its trim arguments. Keeping a second public trim
representation that can never contain a value leaves contradictory ownership
in the published API and invites callers to read a field that cannot describe
the media picture.

## Nitpicks

None.

## Pass-2 follow-up

- D1 is fixed for direct parsing and facade authorship. Independent inverse
  offsets such as `st="875.25"` and `end="125.5"` parse, retain their lexical
  XML, project checked rounded values, and survive add, save, and reopen.
- D2 is fixed for inspection and untouched round-trip. When both `r:embed` and
  `r:link` are present, the linked relationship wins and the original dual
  attributes remain byte-retained. The mutation ownership gap is pass-3 D2.
- D3 is fixed for `CT_Timing::add_media`. The two unused trim parameters were
  removed and the facade authors trim only through `CT_Picture`.

## Not found

- Correctness beyond the findings: trigger classification keeps timing
  ancestry, replacement is shape-scoped, shared relationships survive while
  referenced, and explicit deduplication compares bytes plus OPC metadata.
- Contract beyond the findings: additions require a validated poster, linked
  targets remain exact and external without fetching, and unsupported codec
  bytes remain packaged, extractable, and diagnosable.
- Panics: added production indexing and `expect` sites are dominated by
  validated slide indices, fixed local construction, or parser-established
  roots.
- OOXML beyond the findings: namespace aliases are resolved by URI, new timing
  lists precede later schema children, foreign same-local-name timing lists are
  not selected, and unmodelled serialization sources remain retained.
- Tests beyond the missing mutation cases identified by D1 through D3: the
  pinned-deck gate asserts exact media bytes, relationship types and targets,
  content types, poster ownership, playback settings, and unsupported metadata
  against producer expectations.
- Structure beyond S1: no new trait, generic, feature, crate, module, file,
  dependency, forwarding wrapper, or builder was introduced. Media sniffing
  remains in the dependency-free `oxml-media` leaf, and `rpptx-layout` changes
  only its exhaustive diagnostic matches.

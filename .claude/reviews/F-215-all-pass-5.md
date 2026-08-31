# F-215, all, pass 5

**Reviewed**: complete working diff against the F-215 worker base, 10 files,
3,877 additions and 9 deletions, plus the approved design, cited HLD sections,
progress notes, passes 1 through 4, and every default microscope aspect
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, nested foreign time nodes can supply media playback settings

`crates/rpptx-oxml/src/timing.rs:1306`

`crates/rpptx-oxml/src/timing.rs:1343`

The typed common-media parser finds the direct PresentationML `p:cTn`, but it
then obtains `repeatCount` and `display` through a second local-name-only scan
of the complete retained subtree. If an earlier unmodelled child contains
`<x:cTn repeatCount="indefinite" display="1"/>`, that foreign nested element
supplies both values instead of the direct schema child. Inspection therefore
reports the media as looped or visible after stopping even when its actual
`p:cTn` requests neither setting. The lookup must share the direct,
namespace-aware ownership boundary already used for the common time node.

### D2, removal deletes media-shaped XML from unmodelled wrappers

`crates/rpptx-oxml/src/timing.rs:739`

`crates/rpptx-oxml/src/timing.rs:760`

The removal traversal recursively enters every captured child and treats any
PresentationML `audio`, `video`, or `cmd` descendant targeting the selected
shape as owned. A producer extension such as an unmodelled `x:payload` that
contains a retained `p:cmd` is therefore drained by `remove_media`, even though
that command is outside a schema-owned timing-list or child-list position and
never becomes a typed timing node. Removal must follow the modelled timing-tree
slots and leave media-shaped content inside raw producer subtrees byte-exact.

### D3, foreign time nodes can block otherwise valid media insertion

`crates/rpptx-oxml/src/timing.rs:521`

`crates/rpptx-oxml/src/timing.rs:525`

Timing-id allocation scans every element whose local name is `cTn` without
checking its namespace or whether it occupies a modelled timing position. An
unmodelled `<x:cTn id="producer"/>` makes `add_media` fail while parsing an id
that is not part of the PresentationML id domain. A foreign numeric maximum can
instead cause false exhaustion. Unrelated retained XML must not influence or
prevent allocation of new PresentationML timing ids.

## Smells

None.

## Nitpicks

None.

## Pass-4 follow-up

- D1 is fixed. Replacement selects a prefix from an existing source attribute
  that resolves to the relationships namespace, rather than emitting a literal
  `r:` prefix (`crates/rpptx-oxml/src/picture.rs:1156`). The adversarial test
  retains the local `r` shadow and writes the replacement through `rel`
  (`crates/rpptx-oxml/tests/integration.rs:46`).
- D2 is fixed. Parsing accepts only the direct Office media extension under the
  exact extension URI (`crates/rpptx-oxml/src/picture.rs:866`), and replacement
  requires the complete `p:pic/p:nvPicPr/p:nvPr/p:extLst/p:ext` path
  (`crates/rpptx-oxml/src/picture.rs:1099`). The adversarial nested media
  subtree remains byte-retained through both source-direction replacements
  (`crates/rpptx-oxml/tests/integration.rs:41`).
- D3 is fixed. Pictures without an Office media relationship bypass the Office
  extension rewrite (`crates/rpptx-oxml/src/picture.rs:223`), while facade
  replacement stages the Microsoft relationship only when the original picture
  had one (`crates/rpptx/src/lib.rs:2438`). The package-level regression covers
  linked and embedded replacement without creating `p14:media` or a Microsoft
  media relationship (`crates/rpptx/tests/integration.rs:423`).

## Not found

- Correctness beyond D1 through D3: inverse trim offsets, dual-source linked
  precedence, cross-source replacement, shared relationship retention,
  metadata-compatible deduplication, and candidate-only part pruning remain
  correctly implemented.
- Contract beyond the findings: `oxml-media` owns signature and MIME checks and
  documents that boundary. Additions require a validated poster, linked targets
  remain exact and external without fetching, and unsupported embedded bytes
  remain packaged, extractable, and diagnosable.
- Panics: added production indexing and `expect` sites are dominated by checked
  slide indices, parser-established roots, or fixed local construction.
- OOXML beyond the findings: the picture extension lookup and replacement use
  namespace-aware exact structural ownership, new timing nodes retain schema
  order, trim lexemes remain raw, and unrelated picture extension content is
  preserved.
- Tests beyond the three timing gaps: pass-4 regressions directly cover prefix
  shadows, unrelated nested Office-media lookalikes, and standard-only
  replacement in both source directions. The corpus gate continues to compare
  exact producer expectations.
- Structure and scope: no new trait, generic, feature, crate, module, file,
  dependency, forwarding wrapper, or builder was introduced. The public change
  remains within the approved pre-1.0 crates, and `rpptx-layout` only diagnoses
  the new retained timing variants.

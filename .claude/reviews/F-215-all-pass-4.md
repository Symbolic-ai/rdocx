# F-215, all, pass 4

**Reviewed**: complete working diff against the F-215 worker base, 10 files,
3,613 additions and 9 deletions, plus the approved design, cited HLD sections,
progress notes, passes 1 through 3, and every default microscope aspect
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, replacement can bind its new relationship attribute to a shadow namespace

`crates/rpptx-oxml/src/picture.rs:1099`

`crates/rpptx-oxml/src/picture.rs:1166`

The replacement path correctly identifies existing `embed` and `link`
attributes by namespace URI, but it always inserts the new attribute with the
literal `r:` prefix. A producer `p14:media` can locally bind `r` to an unrelated
namespace and use another prefix such as `rel` for the relationships namespace.
Replacement then removes the valid `rel:embed` or `rel:link`, retains the local
`xmlns:r` shadow, and inserts an `r:` attribute in the wrong namespace. The
saved Office media extension no longer carries a relationship source, while
the newly allocated Microsoft media relationship remains unused.

### D2, Office media discovery and replacement accept any nested p14 media element

`crates/rpptx-oxml/src/picture.rs:769`

`crates/rpptx-oxml/src/picture.rs:867`

`crates/rpptx-oxml/src/picture.rs:881`

`crates/rpptx-oxml/src/picture.rs:1030`

The picture parser sends every direct `p:nvPr` child into a recursive search
that selects the first expanded-name `p14:media` at any depth. It does not
require the schema-owned `p:extLst/p:ext` position or the Office media extension
URI. The replacement traversal has the same any-descendant rule once it enters
`p:nvPr`. An earlier producer extension can therefore retain a nested
`p14:media` payload that is not this picture's media attachment. Inspection
projects that unrelated relationship and trim metadata, and replacement edits
it while leaving the actual Office media extension unchanged.

### D3, inspected standard-only media cannot be replaced

`crates/rpptx-oxml/src/picture.rs:834`

`crates/rpptx-oxml/src/picture.rs:1079`

`crates/rpptx/src/lib.rs:2493`

The reader deliberately falls back to the direct standard `a:audioFile` or
`a:videoFile` relationship when a picture has no Office 2010 media extension,
so the facade exposes valid standard-only media through `media`, `extract_media`,
and `remove_media`. `replace_media` nevertheless calls the Office extension
rewriter unconditionally, and that helper returns a missing-element error at
EOF. A media object accepted by inspection therefore cannot use the promised
replacement operation, even though its standard relationship is already a
modelled editable field. The staged failure remains atomic, but the operation
is unavailable for this valid input shape.

## Smells

None.

## Nitpicks

None.

## Pass-3 follow-up

- D1 is fixed for ordinary and dual-source pictures. Replacement now receives
  the new source variant, removes both old Office source attributes, writes the
  matching new attribute, and retains trim plus unrelated raw XML.
- D2 is fixed for relationship ownership. The picture exposes both Office
  relationship ids, replacement and removal consider both, retained slide
  references keep shared relationships, and candidate parts are pruned only
  after their last package relationship disappears.
- D3 is fixed for the standard attachment. One namespace-aware direct
  `p:nvPr` traversal now supplies the standard relationship cache, while a
  nested DrawingML same-name element remains raw and untouched.
- S1 is fixed. `CommonMediaNode` no longer exposes the two permanently empty
  trim fields, and picture-owned trim remains available through `CT_Picture`.

## Not found

- Correctness beyond the findings: inverse trim offsets, ancestry-owned
  triggers, shape-scoped relationship replacement, shared relationship
  retention, metadata-compatible deduplication, and candidate-only part pruning
  are implemented and covered.
- Contract beyond the findings: additions require a validated poster, linked
  targets remain exact and external without fetching, and unsupported codec
  bytes remain packaged, extractable, and diagnosable.
- Panics: added production indexing and `expect` sites are dominated by
  validated slide indices, fixed local construction, or parser-established
  roots.
- OOXML beyond the findings: timing insertion is namespace-aware and follows
  schema child order, dual Office attributes use linked precedence, and direct
  inverse trim XML retains its lexical serialization source.
- Tests beyond the three gaps: the focused remediation regressions prove
  cross-source replacement in both directions, dual-source cleanup with a
  shared payload, direct standard relationship ownership, and the removed
  timing trim surface. The pinned-deck gate continues to compare explicit
  producer expectations.
- Structure: no new trait, generic, feature, crate, module, file, dependency,
  forwarding wrapper, or builder was introduced. Format-neutral sniffing stays
  in `oxml-media`, and the layout change remains an exhaustive diagnostic match.

# F-219, correctness, pass 2

**Reviewed**: the complete updated working-tree implementation diff, seven files with 2,559 added lines and 11 removed lines, plus the 87-line pass-1 review
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, relIds remapping loses inherited namespace context
`crates/rpptx-oxml/src/diagram.rs:135`

`DiagramRelationshipIds::remap` runs `rewrite_exact_rel_ids` over the captured
`dgm:relIds` bytes alone. The graphic-frame parser passes inherited namespace
bindings at `crates/rpptx-oxml/src/graphic_frame.rs:670`, but the model does not
retain those bindings. When `r` or another relationship-prefix alias is
declared only on the slide or graphic ancestor, the raw subtree is not
self-contained and the rewriter cannot recognize its relationship attributes.
The public fields are updated while `to_xml()` still contains the old ids. The
remap regression at `crates/rpptx-oxml/tests/integration.rs:254` declares its
relationship alias on `relIds` itself, so it does not cover the inherited case
used by the graphic-frame path. Pass-1 D4 is therefore only partially fixed.

### D2, data-model schema order is not enforced for retained known children
`crates/rpptx-oxml/src/diagram.rs:213`

The data-model parser recognizes only `dgm:ptLst` and `dgm:cxnLst` for sequence
tracking. Known later children such as `dgm:bg`, `dgm:whole`, and `dgm:extLst`
fall into the current raw boundary. A package with `dgm:bg` between the point
and connection lists is accepted, and a node-text edit writes it back before
`dgm:cxnLst` at `crates/rpptx-oxml/src/diagram.rs:273`. The structural reparse
accepts the same invalid order. The writer therefore does not guarantee the
declared `xsd:sequence`, which can make PowerPoint reject the edited part.

### D3, the approved cross-deck transfer work is absent
`crates/rpptx/src/lib.rs:1930`

The complete diagram graph-copy helper is called only from
`duplicate_slide_in_place`, for a notes scope and the slide scope again at
`crates/rpptx/src/lib.rs:2001`. No cross-presentation transfer path calls it,
and the regression at `crates/rpptx/tests/integration.rs:242` duplicates a
slide inside one `Presentation`. The approved checklist requires both slide
duplication and transfer to remap all five diagram relationships without
cross-scope aliasing. The transfer half of that contract has no implementation
or test.

### D4, diagram root parsers accept incomplete or trailing XML
`crates/rpptx-oxml/src/diagram.rs:104`

`DiagramRelationshipIds::from_xml` returns as soon as it sees a start tag, so
an unclosed `dgm:relIds` element is reported as parsed. The shared root parser
likewise returns immediately at the matching end tag at
`crates/rpptx-oxml/src/diagram.rs:739`, without reading to EOF. Data, layout,
style, colour, and drawing parts therefore accept a second root or non-space
content after the expected root. Such parts become `DiagramPart::Parsed`
instead of `Invalid`, and the facade's serialize-and-reparse mutation check
does not prove a complete well-formed part.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: pass-1 D1's Microsoft `diagramDrawing` relationship URI, producing-scope ownership, drawing id projection, and duplicate-slide graph remap are corrected.
- Preservation: pass-1 D2's list attributes, foreign children, comments, processing instructions, and direct text events now survive a dirty data-model write.
- OOXML namespaces: pass-1 D3's element and attribute projections are namespace-aware, and foreign same-local-name drawing shapes are ignored.
- Contract scopes: pass-1 D5's slide, layout, and master inspection paths now resolve independently. The remaining transfer omission is D3 above.
- Fixed-prefix shadows: conflicting `dgm`, `a`, and `r` bindings are checked at the root, point-list, point, and connection-list writer scopes before canonical output.
- Panics: no new reachable panic, unchecked arithmetic, or externally controlled unsafe index was found.
- Structure: no unjustified trait, dynamic dispatch, dependency, feature, crate, wrapper, or test binary was added.
- Tests: focused SmartArt tests pass. Corpus-dependent cases skipped in this review worktree because its ignored corpus directory is absent, while the worker progress record reports the required-corpus run passed before handoff.

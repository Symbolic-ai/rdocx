# F-219, correctness, pass 1

**Reviewed**: the complete working-tree diff, seven files with 1,972 added lines and 9 removed lines, including the untracked diagram module
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, cached diagram drawings use the wrong relationship type and owner
`crates/oxml-opc/src/relationship.rs:67`

The implementation declares `diagramDrawing` in the Open XML 2006 relationship
namespace, then `Presentation::smart_art` searches for that relationship under
the diagram data part at `crates/rpptx/src/lib.rs:2293`. PowerPoint SmartArt
packages use
`http://schemas.microsoft.com/office/2007/relationships/diagramDrawing` in the
producing slide relationship scope. The data part carries that slide-local id
in `dsp:dataModelExt/@relId`. As a result, real corpus SmartArt returns no
`SmartArtInfo::drawing`, and slide duplication leaves the real cached drawing
outside the copied diagram graph. The source-built fixture encodes the same
incorrect data-part ownership at `crates/rpptx/tests/integration.rs:425`, so the
regression passes without exercising the package shape produced by PowerPoint.

### D2, a node-text edit drops unmodelled data-model content
`crates/rpptx-oxml/src/diagram.rs:218`

Once `CT_DiagramData` becomes dirty, the writer creates fresh `dgm:ptLst` and
`dgm:cxnLst` containers and emits only the typed points and connections. The
parsers at `crates/rpptx-oxml/src/diagram.rs:481` and
`crates/rpptx-oxml/src/diagram.rs:538` discard list attributes and every foreign
or unsupported child inside those containers. The shared child collector also
ignores comments, processing instructions, and text events at
`crates/rpptx-oxml/src/diagram.rs:629`. A supported node-text edit therefore
removes producer extensions and direct XML events from the same data part,
contrary to the required ordered raw capture at every typed boundary.

### D3, typed diagram projections are not namespace-aware
`crates/rpptx-oxml/src/diagram.rs:719`

Layout, style, and colour descendants are selected by local name alone, and
the attribute helper at `crates/rpptx-oxml/src/diagram.rs:930` likewise accepts
qualified foreign attributes as schema attributes. A producer extension such
as `x:alg`, `x:styleLbl`, or `x:modelId` can therefore alter the public layout
family, style, colour, point, or connection projection. The cached drawing
root compounds this by accepting `drawing` in any namespace at
`crates/rpptx-oxml/src/diagram.rs:469`. These values feed F-220 rendering, so a
foreign same-local-name extension would acquire diagram semantics instead of
remaining opaque.

### D4, relationship-id remapping destroys the preserved relIds subtree
`crates/rpptx-oxml/src/diagram.rs:132`

`DiagramRelationshipIds::remap` replaces the retained raw subtree with a newly
constructed empty element. The constructor at
`crates/rpptx-oxml/src/diagram.rs:813` emits only the four modelled ids, drops
all producer attributes and children, and writes an invented `r:draw`
attribute when `drawing` is present. The DrawingML `dgm:relIds` element does not
own the cached drawing id. That id is carried by the data-model extension in
the producing relationship scope. Calling this approved remapping API can both
lose unmodelled XML and serialize a non-schema relationship attribute.

### D5, layout and master producing scopes are not inspected
`crates/rpptx/src/lib.rs:2253`

`Presentation::smart_art` walks only the selected slide's shape tree and
resolves every id against `record.part_name`. It never walks a slide layout or
master shape tree, despite the contract requiring equal ids in slide, layout,
and master scopes to resolve independently. The test adds decoy relationships
to a layout and an otherwise unreferenced master at
`crates/rpptx/tests/integration.rs:136`, but then inspects only the two slide
fixtures at `crates/rpptx/tests/integration.rs:155`. It therefore does not
exercise either advertised scope. SmartArt inherited from a layout or master
is absent from the typed facade needed by F-220.

## Smells

None.

## Nitpicks

None.

## Not found

- Panics: no new reachable panic or unchecked indexing defect was found in the implementation paths reviewed.
- Structure: the approved concrete diagram module and five-instantiation `DiagramPart<T>` generic satisfy the repository structural rules. No unjustified trait, dynamic dispatch, feature, crate, or test binary was added.
- Atomicity: the facade stages node mutation and slide duplication before publishing state. No additional atomicity defect was found beyond the relationship-graph defects above.

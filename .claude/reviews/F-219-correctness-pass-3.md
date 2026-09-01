# F-219, correctness, pass 3

**Reviewed**: the complete updated working-tree implementation and plan diff, nine files with 2,885 added lines and 34 removed lines, plus the pass-1 and pass-2 reviews
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, cross-deck slide transfer leaves ordinary internal dependencies behind
`crates/rpptx/src/lib.rs:3366`

`transfer_slide_from` is a public operation for copying a slide from any other
presentation, but `duplicate_relationship_scope` copies only images and the
five diagram relationship types. Every other internal relationship keeps the
source package's resolved part name without copying that part into the
destination. A source slide containing a chart and its embedded workbook,
embedded audio or video, an OLE object, or another internal owned part will
therefore point at an absent destination part. If the destination happens to
have the same part name, the transferred slide silently aliases unrelated
destination content instead. Notes relationships take the same path, and the
arbitrary first-layout substitution at `crates/rpptx/src/lib.rs:1858` can also
change inherited slide content. The SmartArt transfer fixture starts from the
same package shape and checks only the five diagram types at
`crates/rpptx/tests/integration.rs:403`, so it does not exercise a source-only
ordinary dependency.

### D2, dirty diagram text treats foreign namespace children as DrawingML
`crates/rpptx-oxml/src/diagram.rs:649`

The diagram point parser hands the captured `dgm:t` subtree to
`CT_TextBody::from_xml_as` without namespace bindings or a fixed-prefix safety
check. That parser selects `bodyPr`, `lstStyle`, paragraphs, and their
descendants by local name alone at
`crates/oxml-drawing/src/text/mod.rs:111`. A producer subtree such as
`<dgm:t xmlns:a="urn:producer"><a:bodyPr/><a:p/></dgm:t>` is therefore accepted
as typed DrawingML. After `set_node_text`, serialization writes those elements
with the canonical DrawingML `a` namespace, changing their meaning and losing
the producer namespace rather than rejecting the unsafe rewrite. The existing
fixed-prefix checks cover the data-model root, point list, point, and connection
list, but not this nested typed text boundary.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: pass-2 D1's inherited relationship namespace remap now retains ancestor bindings and updates the raw `dgm:relIds` bytes.
- OOXML order: pass-2 D2's `ptLst`, `cxnLst`, `bg`, `whole`, and `extLst` sequence is now tracked, duplicate and out-of-order known children are rejected, and dirty output uses schema order.
- XML completeness: pass-2 D4 is corrected. All shared diagram root parsers and `DiagramRelationshipIds` now validate the complete document through EOF.
- SmartArt graph ownership: pass-2 D3's cross-package path now creates fresh data, layout, quick-style, colour, and drawing parts, remaps the cached drawing id, and deduplicates drawing-owned image bytes without diagram-part aliasing. The broader transfer defect is D1 above.
- Prior fixes: pass-1 relationship URI and ownership, raw-boundary preservation, namespace-aware diagram projections, exact relationship remapping, and slide, layout, and master inspection remain corrected.
- Panics: no new reachable panic, unchecked arithmetic, or externally controlled unsafe index was found.
- Structure: no unjustified trait, dynamic dispatch, dependency, feature, crate, wrapper, or test binary was added.
- Tests: `cargo test -p rpptx-oxml` passed 161 tests. `cargo test -p rpptx` passed 180 tests with 8 ignored. Both crate checks and `git diff --check` passed. Corpus-dependent focused cases skip in this isolated worktree because its corpus directory is absent, while the worker progress record reports the required corpus run passed before handoff.

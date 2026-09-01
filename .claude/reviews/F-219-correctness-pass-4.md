# F-219, correctness, pass 4

**Reviewed**: the complete updated working-tree implementation and plan diff, nine files with 3,138 added lines and 34 removed lines, plus the pass-1 through pass-3 reviews
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, transfer preflight does not inspect nested diagram dependencies
`crates/rpptx/src/lib.rs:3356`

The bounded preflight examines only relationships owned directly by the source
slide. Once an allowed diagram relationship is followed,
`duplicate_diagram_part_graph` still leaves any internal relationship that is
neither an image nor another diagram type at its source part name at
`crates/rpptx/src/lib.rs:3510`. A diagram drawing or producer extension with an
internal chart, OLE, media, or custom dependency therefore passes preflight. If
the destination has the same part name, the transferred graph silently aliases
destination content. If it does not, the copied relationship is dangling. The
rejection regression adds its chart directly to the slide at
`crates/rpptx/tests/integration.rs:488`, so it does not prove the plan's promise
to reject every unsupported internal dependency in the complete owned graph.

### D2, transfer does not reconcile placeholders with the selected layout
`crates/rpptx/src/lib.rs:2088`

Cross-deck transfer retargets the slide-layout relationship and rewrites shape
ids, but it does not inspect or rewrite `p:ph/@idx`. A valid source slide whose
body placeholder uses index 7 can therefore be attached to a destination layout
whose corresponding body placeholder uses index 3 while retaining index 7.
That silently severs geometry and formatting inheritance because the placeholder
index is the layout join key. The cited HLD requires a distinct index-allocation
path for copying slides between presentations at
`docs/hld/06-presentationml-model.md:561`. The transfer fixture contains only a
graphic frame at `crates/rpptx/tests/integration.rs:706`, so it cannot detect
this loss.

### D3, an edited diagram text root drops unmodelled attributes
`crates/rpptx-oxml/src/diagram.rs:425`

The point parser retains the original `dgm:t` bytes only for namespace safety,
then `write_xml_as` creates a fresh root with no source attributes. A supported
text payload such as `<dgm:t x:keep="producer">` with ordinary DrawingML
children passes `validate_diagram_text_namespaces`, but `set_node_text` writes
`<dgm:t>` and drops `x:keep`. This violates the approved preservation contract
for unsupported attributes at typed boundaries. The new shadow regression at
`crates/rpptx-oxml/tests/integration.rs:239` covers an unsafe nested `a`
binding, but not a safe typed text root carrying an unrelated producer
attribute.

## Smells

None.

## Nitpicks

None.

## Not found

- Direct pass-3 transfer remediation: the API now requires and validates an explicit destination layout index. Notes, comments, and unsupported relationships directly owned by the slide reject before staging, and the tested failures leave destination bytes unchanged.
- Diagram collision handling: all five directly owned SmartArt parts receive collision-free destination names, the cached drawing id is remapped, and drawing-owned images deduplicate by bytes.
- Direct pass-3 namespace remediation: producer-owned `a` and `r` shadows and foreign same-local-name text descendants can no longer be interpreted as DrawingML. Unsafe text remains opaque and byte-exact during another point mutation.
- Prior correctness: inherited relationship namespace remap, complete data-model sequence enforcement, full root validation through EOF, producing-scope resolution, exact raw relationship remap, and slide, layout, and master inspection remain corrected.
- OOXML preservation: point-list and connection-list attributes, raw direct events, background, whole, extension-list, unsupported algorithm, style, colour, and drawing content remain retained. The remaining text-root attribute loss is D3 above.
- Panics: no new reachable panic, unchecked arithmetic, or externally controlled unsafe index was found.
- Structure: no unjustified trait, dynamic dispatch, dependency, feature, crate, wrapper, or test binary was added.
- Tests: `cargo test -p rpptx-oxml` passed 162 tests. `cargo test -p rpptx` passed 181 tests with 8 ignored. Both changed-crate checks and `git diff --check` passed. Corpus-dependent focused cases skip in this isolated worktree because its corpus directory is absent, while the worker progress record reports the pinned 50-deck run and the full verification gate passed before handoff.

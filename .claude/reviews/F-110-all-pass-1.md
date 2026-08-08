# F-110, all, pass 1

**Reviewed**: uncommitted working diff against `HEAD`, 5 files, 546 additions and 2 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, appending after a trailing raw child can violate shape-tree schema order
`crates/rpptx/src/lib.rs:1104`
`crates/rpptx/src/lib.rs:1128`
`crates/rpptx/src/lib.rs:1152`
`crates/rpptx/src/lib.rs:1165`

All four constructors push directly into `tree.children` without reconciling
the tree's `OrderedRawChildren`. A parsed final `p:extLst` is stored at the old
trailing boundary. After the push, `write_group` emits that boundary before
the newly appended child, placing a shape after schema-final `p:extLst` and
making PowerPoint repair the deck. The required per-constructor raw-subtree
preservation coverage is also absent, so the current tests exercise only trees
without this state.

### D2, tree-wide id allocation ignores ids in opaque shape-tree content
`crates/rpptx-oxml/src/shape_tree.rs:77`

`ShapeIdAllocator::scan` collects only typed children, nested typed groups,
and the selected fallback projection. It does not inspect preserved raw
shape-tree members or opaque `AlternateContent` choices for `cNvPr` ids. A
valid deck with an occupied low id in one of those preserved branches can
therefore receive the same id from any new constructor. The facade validation
also walks only typed children, so `four_appended_shapes_have_unique_ids_and_reopen`
does not detect this collision before PowerPoint sees it.

### D3, arbitrary preset strings are serialized as schema values
`crates/rpptx/src/lib.rs:1121`

`add_shape` accepts every string and `CT_PresetGeometry2D::new` stores it
without checking the complete generated ECMA preset lookup. An input such as
`notAStandardPreset` therefore returns `Ok`, writes an invalid
`a:prstGeom/@prst` value, and can produce a repair prompt or an unsupported
shape. Keeping a string API does not require accepting values outside
`ST_ShapeType`.

### D4, the reopen test does not assert preserved geometry
`crates/rpptx/tests/integration.rs:109`

The approved integration test requires save and reopen to preserve both kind
and geometry. Its post-reopen assertions check only the four kinds and the
cardinality of their ids. Dropping or replacing the textbox, preset-shape, or
connector transforms can leave this test green, so it does not prove the
geometry half of its contract.

## Smells

None.

## Nitpicks

None.

## Not found

No additional correctness, contract, OOXML, or test findings. Panics and
overflow produced no findings. Connector normalization uses `abs_diff` plus a
checked conversion at the `i64` extremes, and one-axis zero extents remain
accepted. The post-push `last_mut` borrows and the fixed-shell group `expect`
are justified by local invariants.

Structure produced no findings. No new module, trait, generic parameter,
feature, dependency, forwarding wrapper, or erased concrete type was added.

The textbox contains its required paragraph. Ordinary shape, connector, and
group shells include their required children in fixed-prefix schema order.
The affected Rust crate suites passed. The ignored native helper was invoked
explicitly and passed against pinned PowerPoint 16.104 build
16.104.25121423.

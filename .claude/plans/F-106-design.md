# F-106, ShapeIdAllocator and MediaStore

**Status**: completed
**Sprint**: S26
**Size**: M
**Depends on**: F-070, F-036

## Problem

The typed shape tree exposes recursive children at
`crates/rpptx-oxml/src/shape_tree.rs:30`, but each shape's `p:cNvPr/@id` remains
inside private raw non-visual XML. There is no tree-wide allocator that can
avoid collisions across groups and selected `mc:AlternateContent` fallbacks.
The `rpptx` facade also owns package bytes without an insertion-time media
index, so future picture and copy operations cannot reuse an existing part for
identical bytes.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Shapes".
- `docs/hld/03-architecture.md`, "The dependency rule" and "Why these seams".
- `docs/hld/04-opc-and-packaging.md`, "Part naming" and "Media".
- `docs/hld/06-presentationml-model.md`, "The shape tree", "Preservation
  strategy", and "Adding a slide".
- `docs/hld/08-rendering-spec.md`, "The renderer's input".
- `docs/hld/14-development-backlog.md`, "F-106, ShapeIdAllocator and
  MediaStore".

## Approach

Project each concrete shape-tree child's non-visual drawing id without changing
the raw XML used to serialize its `p:cNvPr`. Keep malformed producer absence
observable as `None`, which also gives the `AlternateContent` container a total
result because the container itself has no single drawing id. Add one
normalized accessor on `ShapeTreeChild`, then add the concrete allocator in the
existing shape-tree module:

```rust
impl ShapeTreeChild {
    pub fn non_visual_id(&self) -> Option<u32>;
}

pub struct ShapeIdAllocator { /* occupied ids and next candidate */ }

impl ShapeIdAllocator {
    pub fn scan(tree: &CT_ShapeTree) -> Self;
    pub fn allocate(&mut self) -> u32;
}
```

`scan` visits every concrete child, nested group, and selected fallback child.
It ignores an absent malformed producer id while preserving that source XML for
later validation. Allocation starts at 2, skips every occupied id, records each
returned id, and never assigns the shape-tree root id 1. The concrete struct is
justified by the current allocator story and by F-107's slide-placeholder
synthesis. No trait or generic is added.

Add a private concrete `MediaStore` to the existing `rpptx` facade file. It is
built from `/ppt/media/` parts whenever a presentation is opened. It indexes
bytes by the existing stable `oxml_layout::MediaId`, keeps a byte comparison in
each hash bucket to make collisions safe, and retains the part name plus
content type. Its insertion method uses `oxml_media::resolve` and
`MediaNamer::scan` to add a new `/ppt/media/imageN.ext` part only when no
byte-identical entry exists. `Presentation` owns the store so all later write
operations share one deck-wide deduplication boundary.

## Rejected alternatives

- Allocate from immediate children only. Nested groups and fallback shapes
  would still collide.
- Replace each `p:cNvPr` subtree during parsing. The id is read-only in this
  story, so raw serialization remains the safer preservation source.
- Trust the compact `MediaId` as collision-free. Its documented FNV-1a value is
  a renderer key, so the store also compares bytes.
- Add a media-store module or trait. One facade-owned implementation in the
  existing file is enough today.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `shape_id_allocator_scans_nested_groups_and_alternate_content` | IDs in the root, nested groups, and selected fallback are all occupied before allocation |
| unit | `shape_id_allocator_starts_at_two_and_skips_sparse_ids` | The tree root keeps id 1 and returned ids are fresh even with gaps and duplicates in input |
| round-trip | `typed_non_visual_ids_preserve_original_shape_xml` | Prefix-tolerant parsing exposes ids while fixed-prefix writing preserves unmodelled non-visual content and schema order |
| unit | `equal_media_bytes_inserted_twice_reuse_one_part` | Two insertions return the same part and create exactly one `/ppt/media/` entry |
| regression | `media_store_compares_bytes_inside_a_hash_bucket` | A forced key collision does not alias different bytes |
| unit | `media_store_allocates_after_the_highest_existing_suffix` | Sparse existing names never collide and content type follows sniffed bytes |

The backlog test gate is named explicitly: ids are unique across nested groups
and `AlternateContent`, and the same image inserted twice creates one part.

## HLD impact

None. The existing HLD already defines tree-wide allocation, id 1 reservation,
content-addressed media reuse, collision limits, sniffing, and part naming.

## Risk routing

- Parser or serialiser: read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add alternate-prefix, fixed-prefix,
  schema-order, and raw non-visual subtree preservation checks.
- Crate dependency graph and new uses across families: read
  `docs/hld/03-architecture.md`. Run `cargo tree -p rpptx --edges normal` and
  prove no `oxml-*` crate gains an `rpptx-*` dependency.
- Public API of unpublished crates: read `docs/hld/10-bindings-spec.md` and the
  structural rules in `CLAUDE.md`. State that there is no released semver
  impact. Keep `MediaStore` private and expose only the allocator API needed by
  the current F-107 consumer.

## Hash harness

Expected to be unchanged. These unpublished PresentationML write primitives do
not affect Word rendering output.

## Implementation checklist

- [x] Type and expose non-visual ids without replacing preserved raw XML.
- [x] Implement recursive tree scanning and collision-free allocation from 2.
- [x] Add the facade-owned collision-safe `MediaStore`.
- [x] Scan existing media and allocate new names after the greatest suffix.
- [x] Add focused allocator, round-trip, deduplication, and collision tests.
- [x] Prove the dependency graph still points from the facade to shared leaf
  crates.

## Open questions

None. The plan keeps the store private until a public picture API consumes it,
and reuses the existing `MediaId`, image resolver, and media namer rather than
creating a second content identity. The normalized id accessor returns `None`
for the `AlternateContent` container and for malformed missing producer ids.
The allocator still visits every selected fallback child.

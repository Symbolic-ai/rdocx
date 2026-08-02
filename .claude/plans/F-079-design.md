# F-079, The rpptx read facade

**Status**: approved
**Sprint**: S19
**Size**: L
**Depends on**: F-069, F-070

## Problem

The workspace has the typed PresentationML roots and shape tree, but it has no
public `rpptx` facade. The workspace member list contains `rpptx-oxml` but no
`rpptx` crate at `Cargo.toml:3`, while `crates/rpptx-oxml/src/lib.rs:3` exposes
only the schema-level modules. Callers must currently open `OpcPackage`, resolve
the presentation and slide relationships, parse each root, and join notes to
slides themselves.

The lower-level model already preserves document order and the six shape-tree
members at `crates/rpptx-oxml/src/shape_tree.rs:24`, exposes text bodies on
ordinary shapes at `crates/rpptx-oxml/src/shape_tree.rs:169`, and extracts
speaker notes at `crates/rpptx-oxml/src/notes_parts.rs:108`. What is missing is
one read surface that owns those parts, resolves them without fixed filenames,
and presents safe borrowed handles.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Scope" and "API compatibility".
- `docs/hld/03-architecture.md`, "Three families, one workspace", "The
  dependency rule", "Versioning", and "Crate-level conventions".
- `docs/hld/04-opc-and-packaging.md`, "The package" and "Deterministic output".
- `docs/hld/06-presentationml-model.md`, "Parts", "presentation.xml", "Notes
  parts", "The shape tree", and "Preservation strategy".
- `docs/hld/12-testing-strategy.md`, "The deck corpus" and "Binding tests".
- `docs/hld/14-development-backlog.md`, "F-079, The rpptx read facade".
- `docs/hld/15-build-and-toolchain.md`, "Publishing".

## Approach

Create the unpublished `crates/rpptx` facade crate at version `0.0.0` with
`publish = false`. Add it to the explicit workspace member and dependency lists.
Its production dependencies are `oxml-opc`, `rpptx-oxml`, and `thiserror`. The
dependency direction remains from `rpptx` toward the lower layers.

Keep the complete implementation in `src/lib.rs`. `Presentation` owns the OPC
package, the parsed presentation root, and the ordered slide records resolved
from `p:sldIdLst` relationship ids. Each slide record owns its `CT_Slide`, part
name, producer slide id, and optional relationship-resolved `CT_NotesSlide`.
Missing parts, wrong relationship types, external slide targets, duplicate
notes-slide relationships, and malformed XML return the crate's concrete
`Error` enum.

The public read surface is:

```rust
pub struct Presentation { /* private fields */ }

impl Presentation {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self>;
    pub fn from_bytes(bytes: &[u8]) -> Result<Self>;
    pub fn to_bytes(&self) -> Result<Vec<u8>>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn slide(&self, index: usize) -> Option<SlideRef<'_>>;
    pub fn slides(&self) -> impl ExactSizeIterator<Item = SlideRef<'_>>;
}

#[derive(Clone, Copy)]
pub struct SlideRef<'a> { /* borrowed slide record */ }

impl SlideRef<'_> {
    pub fn id(&self) -> u32;
    pub fn name(&self) -> Option<&str>;
    pub fn shape(&self, index: usize) -> Option<ShapeRef<'_>>;
    pub fn shapes(&self) -> impl ExactSizeIterator<Item = ShapeRef<'_>>;
    pub fn text(&self) -> String;
    pub fn notes_text(&self) -> Option<String>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShapeKind {
    Shape,
    Picture,
    GraphicFrame,
    Group,
    Connector,
    AlternateContent,
}

#[derive(Clone, Copy)]
pub struct ShapeRef<'a> { /* borrowed ShapeTreeChild */ }

impl ShapeRef<'_> {
    pub fn kind(&self) -> ShapeKind;
    pub fn text(&self) -> Option<String>;
    pub fn child_count(&self) -> usize;
    pub fn child(&self, index: usize) -> Option<ShapeRef<'_>>;
    pub fn children(&self) -> impl ExactSizeIterator<Item = ShapeRef<'_>>;
}
```

`shapes()` is the immediate z-order view. Group children and the selected
`mc:AlternateContent` fallback are available through `children()`. Indexed
access always returns `Option`. Shape text is the ordinary shape text body or,
for a table frame, row-major cell text joined with tabs and newlines. Slide text
walks those visible children recursively in z-order and joins nonempty shape
text with newlines. `notes_text()` distinguishes a missing notes slide from an
empty notes body.

`to_bytes()` clones the owned package, writes the parsed presentation, ordered
slides, and notes slides back to their relationship-resolved part names, then
uses the deterministic OPC writer. No public mutation surface is added in this
read story.

Add `examples/dump_deck.rs`. It emits a stable tab-separated record for each
deck, slide, recursive shape path, normalized `ShapeKind`, shape text, slide
text, and notes text. Escaping makes tabs, newlines, carriage returns, and
backslashes unambiguous. Add one `tests/integration.rs` binary. Its differential
test invokes python-pptx 1.0.2 through `uv run --with python-pptx==1.0.2`, checks
the resolved version, computes the same normalized records for all 50 pinned
decks, and compares those records to the facade. The oracle remains test-only.

## Rejected alternatives

- Expose `CT_*` values directly. That makes callers repeat relationship joins
  and leaks the schema layer through the facade.
- Add a trait for shape text. The six variants already have one concrete
  dispatch point and there is no second implementer today.
- Add one wrapper type per shape variant. `ShapeRef` already normalizes the six
  existing variants and extra forwarding wrappers would increase lookup sites.
- Compare producer-specific python-pptx enum values and names. The implemented
  schema model supports six structural OOXML kinds, so the oracle is normalized
  to that supported surface instead of claiming finer shape classification.
- Add a committed binary fixture. The fetched corpus is the one approved binary
  fixture exception.
- Publish `rpptx`. PowerPoint development crates remain at version 0.0.0 with
  publication disabled until development is complete.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `rpptx_is_an_unpublished_workspace_member` | The new crate is version 0.0.0, has publication disabled, and is an explicit workspace member |
| integration | `presentation_resolves_ordered_slides_and_notes` | Slide order follows `p:sldIdLst`, nonstandard part names resolve through relationships, and notes attach to their source slide |
| integration | `indexed_read_access_is_total` | Empty and out-of-range slide, shape, group, and fallback indices return `None` |
| integration | `shape_refs_cover_the_typed_shape_tree` | All six kinds, recursive groups, selected fallback children, shape text, table text, slide text, and missing versus empty notes are observable |
| negative | `broken_presentation_graph_returns_contextual_errors` | Missing parts, wrong relationship types, external targets, duplicate notes links, and malformed roots return errors without panics |
| round-trip | `facade_bytes_reopen_with_the_same_read_model` | A code-built deck opens, serialises, reopens, and has the same ordered read surface while opaque parts remain unchanged |
| differential | `dump_deck_matches_python_pptx_1_0_2_for_the_corpus` | The Rust and pinned python-pptx normalized slide, shape, table-text, and notes records match for all 50 decks |

The backlog test gate is named explicitly: a `dump_deck` example printing every
slide's shapes and text matches python-pptx's output on the corpus.

## HLD impact

- `docs/hld/06-presentationml-model.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser. Recheck relationship-resolved parts, fixed write
  prefixes, schema order, and byte preservation for unmodelled subtrees. Run
  the focused round-trip tests with `RDOCX_PPTX_CORPUS_REQUIRED=1`.
- Crate dependency graph and a new `use` across families. Run
  `cargo tree -p rpptx --edges normal` and prove no `oxml-*` crate gains an
  `rpptx-*` dependency.
- A new crate, module, or file. Obtain explicit approval for `crates/rpptx`,
  `crates/rpptx/Cargo.toml`, `crates/rpptx/src/lib.rs`,
  `crates/rpptx/examples/dump_deck.rs`, and
  `crates/rpptx/tests/integration.rs` before implementation. The crate uses no
  source module beyond `lib.rs`.
- An external oracle comparison. Pin python-pptx to 1.0.2 in the executable
  test command, assert the resolved version, and keep it out of crate source
  dependencies.
- Version strings. Inspect the root manifest, new crate manifest, and
  `Cargo.lock` diff. Confirm version 0.0.0, `publish = false`, no README version
  change, no release action, and no tag.

## Hash harness

Expected to be unchanged. The facade and corpus-only differential output do not
participate in the 28 Word rendering hashes.

## Implementation checklist

- [ ] Add the unpublished `rpptx` workspace member and one-file facade.
- [ ] Resolve presentation, slide, and notes parts through OPC relationships.
- [ ] Add safe slide and shape reference handles with recursive text access.
- [ ] Flush facade-owned modelled parts through deterministic `to_bytes()`.
- [ ] Add the `dump_deck` example and the single integration test entrypoint.
- [ ] Run the focused, oracle, dependency-tree, full verification, prose, and
  hash checks.

## Open questions

None. The user approved creation of the unpublished `crates/rpptx` crate and
the four listed files. The user also approved the normalized read contract,
including immediate z-order iteration, explicit child iteration for groups and
alternate fallbacks, table-cell text, recursive slide text, and optional notes
text.

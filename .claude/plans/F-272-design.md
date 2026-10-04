# F-272, Rich footnote authoring

**Status**: completed
**Sprint**: S86
**Size**: L
**Depends on**: F-253, F-254, F-255

## Problem

`Document::add_footnote` in `crates/rdocx/src/document.rs` only appends plain
text. The typed note model in `crates/rdocx-oxml/src/footnotes.rs` stores only
paragraphs, and its flush path rejects edits beyond additive notes. Existing
package-backed story edits can author rich content within an existing note but
cannot create, reorder or remove notes as a coherent lifecycle.

## Spec reference

- `docs/hld/03-architecture.md`, "Container-neutral Word story editing" and
  the footnote and endnote layout streams.
- `docs/hld/04-opc-and-packaging.md`, "The package", related story parts and
  retained XML.
- `docs/hld/08-rendering-spec.md`, footnote placement and continuation.
- `docs/hld/14-development-backlog.md`, "F-272, Rich footnote authoring".

## Approach

Retain `add_footnote(text)` for compatibility. Add fallible facade operations:
`create_footnote(&mut self, reference: &ContentLocation, text: &str) -> Result<i32>`,
`footnote_story(&self, id: i32) -> Result<Option<StoryId>>`,
`move_footnote_before(&mut self, id: i32, before_id: i32) -> Result<()>`, and
`remove_footnote(&mut self, id: i32) -> Result<()>`. Creation appends a reference
to the selected direct body paragraph and allocates the first available normal
footnote ID. Removal clears every matching body reference in the same staged
transaction. Invalid locations, IDs and package changes publish nothing.
Reuse `StoryId` and the common content, picture and link editing operations for
its rich body. Extend the existing comment anchoring path to related note
paragraphs so rich footnotes can author annotations as the story requires.
Stage mutations against the package note story, validate the
whole candidate, then reopen and publish. Preserve separators, unknown children,
producer namespaces and independent part-local relationships. Numbering follows
the existing reference-driven footnote stream. F-274 owns custom markers,
separators and restart policy.

## Rejected alternatives

- Rewriting an edited note through the current paragraph-only `CT_Footnote`
  would discard tables, controls and unsupported children.
- Silently leaving dangling body references after removal produces a corrupt
  document.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `rich_footnotes_match_word_after_create_edit_reorder_and_remove` | **Test gate.** Numbering, page placement, continuation and round-trip structure match a pinned Word oracle. |
| integration | `rich_footnotes_keep_part_scoped_relationships` | Pictures and links resolve from the footnotes part after save and reopen. |
| regression | `footnote_removal_clears_references_atomically` | Removal clears matching body references, preserves other notes and rolls back on invalid input. |
| round-trip | `rich_footnote_edit_preserves_unmodelled_children` | Separator and unknown note XML survive byte for byte. |

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and
  `06-presentationml-model.md`. Verify schema order, prefix-tolerant read,
  fixed-prefix write and byte-for-byte unknown subtree retention.
- **Layout, pagination, line breaking, text shaping**. Read
  `docs/hld/08-rendering-spec.md`. Use deterministic fonts for every rendering
  baseline and declare any intentional baseline delta.
- **Public API of a published crate**. Read `docs/hld/10-bindings-spec.md`,
  state additive semver impact, run `cargo publish --dry-run` and the `.crate`
  size assertion.
- **External oracle comparison**. Read
  `.claude/skills/differential-testing.md`. Pin and record the Word oracle
  version and compare structure separately from rendering.

## Hash harness

Expected unchanged unless the approved implementation fixes existing note
rendering. Any changed output must be attributed before updating a baseline.

The scoped implementation keeps all 49 recorded hash entries unchanged.
Microsoft Word for Mac 16.113.2 build 16.113.26092012 opened the generated
rich-footnote DOCX and exported a three-page PDF. The first page displayed
body reference labels 1 and 2. The long first note continued across all three
pages, and the short second note appeared on page three. Deterministic rdocx
PDF output matches those recorded page, label, and continuation observations.

## Implementation checklist

- [x] Establish stable note identity and atomic creation and removal with body references.
- [x] Implement staged rich note reorder and package-backed mutation.
- [x] Reuse common story content and part-scoped asset operations, and support comment anchoring in note paragraphs.
- [x] Run the pinned differential gate and focused preservation tests.
- [x] Run scoped verification and obtain a zero-finding microscope review.

## Open questions

None. Removal clears all references to the selected note atomically. Reorder
changes note element order while preserving IDs, and displayed numbering stays
reference driven.

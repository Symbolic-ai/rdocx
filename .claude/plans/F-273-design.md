# F-273, Rich endnote authoring

**Status**: completed
**Sprint**: S86
**Size**: L
**Depends on**: F-272

## Problem

Existing endnotes are discovered as package stories but cannot be created,
reordered or removed through the facade. `Paragraph::add_footnote_ref` has no
endnote equivalent. F-272 establishes a package-backed rich note lifecycle,
which endnotes need with their own identifiers and placement stream.

## Spec reference

- `docs/hld/03-architecture.md`, footnote and endnote streams and
  "Container-neutral Word story editing".
- `docs/hld/08-rendering-spec.md`, endnote document-end placement.
- `docs/hld/14-development-backlog.md`, "F-273, Rich endnote authoring".

## Approach

Extend F-272's concrete note lifecycle with
`create_endnote(&mut self, reference: &ContentLocation, text: &str) -> Result<i32>`,
`endnote_story(&self, id: i32) -> Result<Option<StoryId>>`,
`move_endnote_before(&mut self, id: i32, before_id: i32) -> Result<()>`, and
`remove_endnote(&mut self, id: i32) -> Result<()>`. Reuse its
common story content, relationship and validation code where both actual
implementations exist. Allocate normal endnote IDs only against endnotes and
keep existing footnote IDs untouched. Stage and reopen every mutation. Preserve
endnote separators and unknown XML. Verify references across section boundaries
and document-end placement under the existing policy. F-274 owns section-end
policy, custom markers, separator authoring and restart rules.

## Rejected alternatives

- Putting endnotes in the footnote part would couple their ID and relationship
  namespaces.
- Duplicating F-272's package traversal would create two paths for the same
  rich content grammar.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `mixed_rich_notes_match_word_at_section_and_document_end_boundaries` | **Test gate.** Word confirms independent footnote and endnote occurrence order across sections. The test asserts the documented current-policy divergence in endnote page placement and number format. |
| integration | `rich_endnotes_reopen_with_part_scoped_relationships` | Rich content, drawings and links survive all lifecycle operations. |
| regression | `endnote_removal_preserves_footnotes_with_the_same_id` | Removing an endnote and its references leaves an equal-numbered footnote unchanged. |
| round-trip | `endnote_edit_preserves_unmodelled_children` | Separator and unknown XML survive byte for byte. |

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
  `docs/hld/08-rendering-spec.md`. Use deterministic fonts for any baseline.
- **Public API of a published crate**. Read `docs/hld/10-bindings-spec.md`,
  state additive semver impact, run `cargo publish --dry-run` and the `.crate`
  size assertion.
- **External oracle comparison**. Read
  `.claude/skills/differential-testing.md`. Pin and record the Word oracle
  version.

## Hash harness

Expected unchanged. Any rendering delta must be attributed to the story.

## Implementation checklist

- [x] Consume F-272's completed note lifecycle without changing its contract.
- [x] Add endnote creation and reference insertion with independent IDs.
- [x] Cover rich edits, reorder, removal and part-local relationships.
- [x] Run the pinned differential gate and scoped verification.
- [x] Obtain a zero-finding microscope review.

## Open questions

None. The section boundary case tests references in different sections under
the current document-end placement policy. Policy authoring belongs to F-274.

## Pinned Word comparison

Microsoft Word for Mac 16.113.2 build 16.113.26092012 opened the exact
generated `/private/tmp/f273-note-oracle.docx`. Its accessibility view showed
two pages. The first section displayed footnote `1` and endnote `i`. The
second displayed footnote `2` and endnote `ii`. Both endnote bodies followed
the second section's body text on page two. The deterministic renderer displays
the same independent occurrence order as `1/1` and `2/2` but places the
endnotes on a third page. `docs/hld/08-rendering-spec.md` specifies the current
fresh-page and decimal-label behavior. F-274 owns the placement and number
format policy difference.

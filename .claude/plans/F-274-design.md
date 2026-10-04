# F-274, Note separators, markers, and restart policy

**Status**: completed
**Sprint**: S89
**Size**: L
**Depends on**: F-269, F-272, F-273

## Problem

The two note families can be created and edited, but their visible labels are
global decimal counters and their separator lines are hard-coded in layout.
`crates/rdocx-layout/src/engine.rs:7552` assigns global labels, while
`crates/rdocx-layout/src/paginator.rs:1855` draws fixed rules and its endnote
path starts a fresh page. Existing `CT_NoteProperties` fields in
`crates/rdocx-oxml/src/document.rs:334` have no complete public policy path.

## Spec reference

- `docs/hld/02-scope-and-non-goals.md`, "Modern DOCX capability matrix", rows DOCX-036 and DOCX-041.
- `docs/hld/03-architecture.md`, "What stays put" and "Facade conventions", note streams and staged note edits.
- `docs/hld/04-opc-and-packaging.md`, "The package", relationship-resolved note ownership.
- `docs/hld/08-rendering-spec.md`, "The renderer's input", note marker and placement policy.
- `docs/hld/12-testing-strategy.md`, "The Word corpus", note differential gate.
- `docs/hld/14-development-backlog.md`, "F-274, Note separators, markers, and restart policy".

## Approach

Expose checked document and section note policy setters on `Document` and
`SectionMut` using concrete note-family, number-format, restart and placement
enums. Preserve the existing note property source slots and unknown children.
Add checked APIs to create or replace separator, continuation-separator and
continuation-notice records in the relationship-resolved note parts. Cover
document defaults, section overrides and the explicit absence of an override.
Carry optional custom mark
text through both the body reference and note marker without changing the
normal note ID namespace. A marker's source of truth is the reference and its
note record, not the note part's physical order.

Compute an ordinal per note family from first body occurrence under the
effective document and section policy. Restart by section or page as selected,
then format the ordinal once for both body and note layouts. Keep custom marks
out of the numeric stream. Place footnotes at the configured page or section
boundary and endnotes at the configured section or document boundary. Render
authored separator and continuation content using the note story rather than a
synthetic line. Preserve the family-specific special-record ID namespace.
Stage every package mutation and publish only after save and reopen.

## Rejected alternatives

- A second counter in the paginator would let body and note markers disagree.
- Treating separator records as normal notes would expose them as public stories and consume normal IDs.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| differential | `note_policies_match_pinned_word_markers_and_page_placement` | **Test gate.** Both families cover document default and section override, each supported format, start, restart, custom mark and placement, including note carryover and continuation notice, without changing unrelated section numbering. The pinned Word for Mac 16.113.2 fixture ignores `beneathText` in both valid document and explicit final-section `w:footnotePr` forms and places the note at page bottom. Assert that observed Word placement separately from the native renderer's `beneathText` placement, which follows the OOXML policy. |
| round-trip | `note_policy_preserves_unknown_note_and_section_xml` | Alias-prefixed inputs parse, changed modeled children write in schema order, and unmodeled children retain exact bytes. |
| regression | `body_and_note_markers_share_formatted_labels` | Reordering note elements and repeated references do not change first-occurrence labels. |
| regression | `invalid_note_policy_rolls_back_atomically` | Invalid format, placement or separator mutation leaves complete package bytes unchanged. |

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/08-rendering-spec.md`
- `docs/hld/12-testing-strategy.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**. Read `docs/hld/04-opc-and-packaging.md` and `06-presentationml-model.md`. Check schema order, prefix tolerance, fixed-prefix changed output and byte-exact unknown subtree preservation.
- **Layout and pagination**. Read `docs/hld/08-rendering-spec.md`. Run deterministic-font render comparisons and state any intentional hash delta.
- **Public API of a published crate**. Read `docs/hld/10-bindings-spec.md`. State additive pre-1.0 semver impact, run `cargo publish --dry-run` and assert the `.crate` size.
- **External oracle comparison**. Read `.claude/skills/differential-testing.md`. Pin and record the exact Word build.

## Hash harness

Expected unchanged for current fixtures. A changed note fixture needs an attributed, reviewed delta before baseline movement.

## Implementation checklist

- [x] Author checked document and section policies and separator records in existing note and facade files.
- [x] Carry custom reference marks through typed run and note XML.
- [x] Resolve policy and one formatted label stream for body and note layout.
- [x] Place both note families and render their authored separators.
- [x] Run pinned differential, round-trip, focused and scoped verification checks.
- [x] Obtain a zero-finding microscope review.

## Open questions

None. Use the OOXML policy vocabulary already represented by `CT_NoteProperties`. The pinned Word fixture fixes observable placement at section and document boundaries. The valid `/private/tmp/f274-beneath-text-oracle.docx` carries both document and final-section `w:pos w:val="beneathText"`. Word for Mac 16.113.2 placed its footnote at page bottom, while the deterministic native renderer placed it beneath the body. This observed divergence does not suppress native support for the OOXML placement value and is recorded in `docs/hld/08-rendering-spec.md`.

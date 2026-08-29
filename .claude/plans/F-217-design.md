# F-217, Presentation collaboration and navigation model

**Status**: approved
**Sprint**: S59
**Size**: L
**Depends on**: none

## Problem

The presentation model still preserves `p14:sectionLst` and comment payloads
as opaque XML in `docs/hld/06-presentationml-model.md`, "Preservation
strategy". Callers therefore cannot inspect or mutate comment threads,
section membership, or their package relationships without bypassing the
facade.

Header and footer visibility is already typed for slide layouts and masters in
`crates/rpptx-oxml/src/slide_parts.rs`, but the notes-master parser retains
`p:hf` as raw XML in `crates/rpptx-oxml/src/notes_parts.rs`. The facade also
does not own the handout master. Slide numbers, dates, footers, notes headers,
and handout settings cannot yet be changed through one relationship-safe model.

## Spec reference

- ECMA-376 Part 1, PresentationML comments, comment authors, presentation
  sections, header and footer settings, notes masters, and handout masters.
- Microsoft Office 2021 PresentationML extensions, modern comments and
  threaded replies.
- `docs/hld/02-scope-and-non-goals.md`, "Explicitly not in v1".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  and "Package integrity".
- `docs/hld/06-presentationml-model.md`, "Parts", "Public facade",
  "Placeholders", "Preservation strategy", and "Validation".
- `docs/hld/10-bindings-spec.md`, "The chosen design" and the native-only
  additive API policy.
- `docs/hld/12-testing-strategy.md`, "Test taxonomy" and the PresentationML
  corpus round-trip gate.
- `docs/hld/14-development-backlog.md`, "F-217, Presentation collaboration and
  navigation model".

## Approach

Add one `rpptx-oxml` comments module for the independent modern comment-author
and per-slide comment roots. It will expose concrete `CommentAuthor`,
`Comment`, and `CommentReply` values with ordered raw sidecars. It will parse
by expanded name, write fixed modelled prefixes in schema order, and preserve
unsupported anchors, properties, attributes, and children byte for byte.
Legacy ISO comment parts remain opaque package content.

Type `p14:sectionLst` in `CT_Presentation`, including ordered sections, stable
section ids, and slide-id membership. Section setters reject duplicate ids and
unknown slide ids before mutation. Moving a slide keeps its producer slide id,
and removing a slide removes that id from section membership.

Reuse `CT_HeaderFooter` for slide-number, date, footer, and header visibility.
Type the existing notes-master `p:hf`, add a concrete handout-master root, and
resolve its presentation relationship without assuming a conventional part
name. The handout scope is the master root and its header/footer settings, not
print options in `presProps.xml`.

Extend `Presentation` with concrete borrowed comment, reply, section, notes,
and handout accessors plus ordered mutation methods. Authors, comments,
replies, sections, ids, and RFC 3339 timestamps are caller-supplied and
validated. Package parts, relationships, content types, slide comment
references, and typed roots are staged and committed only after serialization
and reopen succeed. The API is native Rust only and adds no trait, generic,
builder, feature, crate, or production dependency.

The additive public surface is:

```rust
Presentation::comment_authors(&self) -> &[CommentAuthor];
Presentation::add_comment_author(&mut self, author: CommentAuthor) -> Result<()>;
Presentation::comments(&self, slide_index: usize) -> Option<&[Comment]>;
Presentation::add_comment(&mut self, slide_index: usize, comment: Comment) -> Result<()>;
Presentation::reply_to_comment(
    &mut self,
    slide_index: usize,
    comment_id: &str,
    reply: CommentReply,
) -> Result<()>;
Presentation::move_comment(
    &mut self,
    slide_index: usize,
    from: usize,
    to: usize,
) -> Result<()>;
Presentation::move_reply(
    &mut self,
    slide_index: usize,
    comment_id: &str,
    from: usize,
    to: usize,
) -> Result<()>;
Presentation::sections(&self) -> &[Section];
Presentation::set_sections(&mut self, sections: Vec<Section>) -> Result<()>;
Presentation::notes_header_footer_mut(&mut self) -> Option<&mut CT_HeaderFooter>;
Presentation::handout_header_footer_mut(&mut self) -> Option<&mut CT_HeaderFooter>;
```

## Rejected alternatives

- Modelling both legacy and modern comments would add a second mutation model
  when replies exist only on the modern boundary required by this story.
- Folding two independent comment root serializers into the already large
  presentation-root module would increase the number of unrelated cases a
  reader must consider.
- Allocating ids or timestamps from ambient randomness or time would add
  nondeterminism and dependencies that no current consumer needs.
- Modelling `presProps.xml` print options would expand handout scope beyond the
  stated header and footer acceptance contract.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| round-trip | `modern_comments_replies_sections_and_handout_settings_survive_ordered_mutation_save_and_reopen` | An in-code noncanonical package survives author, comment, reply, section, slide-order, and header/footer mutation with relationship targets, content types, ordering, and raw siblings intact. |
| round-trip | `modern_comments_and_replies_preserve_order_and_unmodelled_xml` | Prefix-tolerant parsing, fixed-prefix schema-order writing, reparsed equality, and byte-exact opaque children. |
| round-trip | `sections_notes_and_handout_settings_write_in_schema_order` | Section membership plus notes and handout `p:hf` values retain order and raw boundaries. |
| regression | `invalid_collaboration_graph_does_not_mutate_the_presentation` | Duplicate ids, unknown authors, bad section membership, external or wrong-type relationships, and occupied conventional part names fail atomically. |

The test gate is **round-trip**. Every collaboration and navigation object
survives reordering, mutation, save, and reopen with its relationships intact.
All fixtures are constructed in the existing integration binaries.

## HLD impact

- `docs/hld/02-scope-and-non-goals.md`
- `docs/hld/04-opc-and-packaging.md`
- `docs/hld/06-presentationml-model.md`
- `docs/hld/10-bindings-spec.md`
- `docs/hld/12-testing-strategy.md`

## Risk routing

- Any parser or serialiser: re-read `docs/hld/04-opc-and-packaging.md` and
  `docs/hld/06-presentationml-model.md`. Add namespace-alias, fixed-prefix,
  schema-order, structural reparse, relationship, and byte-exact unmodelled
  subtree preservation checks.
- Public API of published crates: state the additive pre-1.0 semver impact.
  Run the workspace publish dry run for `rpptx-oxml` and `rpptx`, and assert
  both archives stay below 10 MiB.
- New module or file: obtain explicit approval for
  `crates/rpptx-oxml/src/comments.rs`. This is the one new module. No new
  trait, generic, crate, feature, or dependency is introduced.

## Hash harness

Expected unchanged. This changes package metadata and preserved PresentationML
parts only. Generated document, PDF, and PNG samples must remain byte-identical.

## Implementation checklist

- [ ] Add modern comment-author and comment relationship and content-type constants.
- [ ] Add the approved comment root module with ordered raw preservation.
- [ ] Type section and handout identifiers in the presentation root.
- [ ] Type notes-master and handout-master header and footer settings.
- [ ] Resolve and own collaboration, notes-master, and handout-master package roots.
- [ ] Add atomic ordered facade mutations and relationship validation.
- [ ] Add the four in-code tests to the two existing integration binaries.
- [ ] Run focused `oxml-opc`, `rpptx-oxml`, and `rpptx` checks plus every routed rider.

## Open questions

None. Modern threaded comments, opaque legacy comment preservation, bounded
handout settings, caller-supplied ids and timestamps, and the single
`crates/rpptx-oxml/src/comments.rs` module are approved.

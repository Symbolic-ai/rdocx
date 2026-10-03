# F-271, Uniform rich header and footer editing

**Status**: approved
**Sprint**: S86
**Size**: L
**Depends on**: F-252, F-253, F-254, F-255

## Problem

The facade already creates, links, inherits, unlinks and replaces section
stories in `crates/rdocx/src/document.rs`. Its rich-content regression in
`crates/rdocx/tests/regression_test.rs` covers one default header, while the
six-variant regression uses only plain markers. The combination of rich content,
all variants and part-scoped relationships has no acceptance coverage.

## Spec reference

- `docs/hld/03-architecture.md`, "Per-section header and footer ownership" and
  "Container-neutral Word story editing".
- `docs/hld/04-opc-and-packaging.md`, "The package", related story parts and
  relationship scope.
- `docs/hld/14-development-backlog.md`, "F-271, Uniform rich header and footer editing".

## Approach

Exercise the existing `SectionStory`, `StoryId`, `ContentFragment` and story
mutation APIs across header and footer, each with default, first and even
variants. Author paragraphs containing fields, links, drawings and annotations,
plus a table and a block content control. Save and reopen each variant, verify
its own relationship target, then exercise inherited and unlinked variants.
Repair demonstrated gaps in the common story path without adding a parallel
header or footer content model. Preserve valid producer content and attributes
that the facade does not edit.

## Rejected alternatives

- A header-specific rich-content API duplicates the common story surface and
  gives the same edit two paths.
- Declaring the current one-header test sufficient leaves five variants and
  relationship collisions untested.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| integration | `rich_content_reopens_in_every_header_and_footer_variant` | **Test gate.** All six variants reopen with their ordered rich subtree and own image and link relationships. |
| regression | `rich_section_stories_survive_reopen_replace_and_unlink` | Existing inheritance and unlink behavior remains valid. |
| round-trip | `header_footer_unmodelled_content_survives_rich_edit` | Unknown XML remains byte preserving at its source slot. |

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/14-development-backlog.md`

## Risk routing

- **Any parser or serialiser**, if a source fix changes one. Read
  `docs/hld/04-opc-and-packaging.md` and `06-presentationml-model.md`. Check
  schema child order, prefix-tolerant read, fixed-prefix write and byte-for-byte
  retention of an unmodelled subtree.
- **Public API of a published crate**, if an API gap requires an addition. Read
  `docs/hld/10-bindings-spec.md`, state the additive semver impact, run
  `cargo publish --dry-run` and the package size assertion.

## Hash harness

Expected unchanged. No sample or renderer behavior is planned to change.

## Implementation checklist

- [ ] Extend the existing integration entrypoint to exercise all six variants.
- [ ] Repair any confirmed common story edit or relationship defect.
- [ ] Run focused integration and round-trip checks, then scoped verification.
- [ ] Obtain a zero-finding microscope review.

## Open questions

None. Annotations mean existing comment and range content in the common story
model. Clone behavior may omit anchors where the existing clone contract says so.

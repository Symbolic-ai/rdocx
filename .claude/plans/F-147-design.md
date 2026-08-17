# F-147, Comment model and part

**Status**: approved
**Sprint**: S46
**Size**: M
**Depends on**: none

## Problem

`Document::from_package` loads styles, numbering, and footnotes through typed
relationships but has no comments relationship or typed comments part
(`crates/rdocx/src/document.rs:156`). Paragraph parsing sends comment range
markers to insertion-aware raw XML, so anchors survive but cannot be queried
(`crates/rdocx-oxml/src/text.rs:428`). The package therefore preserves an
existing comment without exposing its body or validating its anchors.

## Spec reference

- `docs/hld/14-development-backlog.md`, "F-147, Comment model and part".
- `docs/hld/03-architecture.md`, "What stays put".
- `docs/hld/04-opc-and-packaging.md`, "Relationship types", "Part naming",
  and "Package integrity".
- `docs/hld/12-testing-strategy.md`, "Test taxonomy".

## Approach

Add a focused `rdocx-oxml` comments model with `CT_Comment`, `CT_Comments`, and
typed comment range markers. `CT_Comment` will retain the required numeric id,
optional author metadata, ordered body paragraphs, root namespace context, and
insertion-aware raw XML for unmodelled attributes and children. Read is
prefix-tolerant and write uses fixed Word prefixes in schema order.

Extend paragraph content ordering so `w:commentRangeStart`,
`w:commentRangeEnd`, and the run-level `w:commentReference` are typed without
normalising any neighbouring raw producer XML. Add comment part state to
`Document`, resolve it through the document relationship on open, and write it
back to the resolved target with its content type. An absent part remains
absent until a later API story creates a comment.

Proposed new source file:

```text
crates/rdocx-oxml/src/comments.rs
```

No new trait, generic parameter, crate, or feature flag is introduced.

## Rejected alternatives

- Keep comments as raw package bytes. This cannot expose `CT_Comment` or
  correlate body anchors with part entries.
- Rebuild every paragraph child from a flat enum. That broadens this story and
  risks normalising unrelated hyperlinks, fields, and producer extensions.
- Assume `/word/comments.xml`. The repository resolves typed parts through
  relationships so noncanonical producer layouts keep working.

## Test plan

| Category | Test | Asserts |
|---|---|---|
| unit | `comments_accept_aliases_and_write_fixed_prefixes_in_schema_order` | Aliased WordprocessingML parses and the typed writer emits valid ordered `w:comment` content |
| round-trip | `three_comments_and_cross_paragraph_anchors_round_trip_byte_identically` | Three comments, including one spanning two paragraphs, retain every anchor position and every unmodelled byte |
| round-trip | `comments_part_uses_its_existing_relationship_target` | A noncanonical comments part target is updated in place without an orphan conventional part |
| regression | `saving_without_comments_does_not_manufacture_a_comments_part` | An absent comments part, relationship, and override remain absent |

The **test gate**, from the backlog, is round-trip. A document with three
comments, one spanning two paragraphs, reloads with every anchor in the same
place and saves byte-identical.

Tests join existing crate-local `#[cfg(test)]` modules and the existing rdocx
integration binary. No new test binary is added.

## HLD impact

- `docs/hld/03-architecture.md`
- `docs/hld/04-opc-and-packaging.md`

Record `rdocx-oxml` ownership of the typed comments model and the relationship
resolved, package-preserving comments part contract.

## Risk routing

- Any parser or serialiser. Read HLD 04 and HLD 06. Add alias, fixed-prefix,
  schema-order, malformed-id, structural round-trip, and byte-preservation
  coverage for both the comments part and body anchors.
- Public API of a published crate. Read HLD 10 and the structural rules. The
  low-level model is additive. Run the affected package dry-run and archive
  size assertion.
- A new module or file. The focused comments module needs explicit approval
  before implementation and must reduce the number of parser cases a reader
  follows in `text.rs` and `document.rs`.

## Hash harness

Expected unchanged across all 49 entries. Existing generated samples contain
no comments, so any delta is unrelated and blocks the sprint.

## Implementation checklist

- [ ] Add the typed comment part parser and writer with ordered raw retention.
- [ ] Type the three comment anchor forms without moving neighbouring XML.
- [ ] Resolve, load, and flush the existing comments relationship target.
- [ ] Add the round-trip, relationship-target, and absence tests.
- [ ] Run focused checks plus the declared packaging rider.
- [ ] Update exactly HLD 03 and HLD 04 at completion.

## Open questions

None. The focused `crates/rdocx-oxml/src/comments.rs` module is approved.

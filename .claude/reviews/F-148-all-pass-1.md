# F-148, all aspects, pass 1

**Reviewed**: working tree against `HEAD`, 10 files and 1,589 changed lines
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, the comment reference is emitted after the paragraph instead of at the range end
`crates/rdocx/src/comments.rs:169`

`add_comment` records the range end at `range.end.run_index`, but always appends
the `w:commentReference` run to the paragraph. A range that ends before later
runs therefore serializes the reference after those unrelated runs instead of
immediately after `w:commentRangeEnd`. The visible reference position no longer
matches the public half-open range contract.

### D2, removing a comment deletes unrelated empty runs
`crates/rdocx/src/comments.rs:615`

After removing matching `w:commentReference` content, the cleanup marks every
empty run without properties or preserved XML for deletion. An empty run that
predated the comment is removed even though it is not one of the selected
comment's three anchors. This violates the removal contract and changes
unrelated paragraph content.

### D3, resolving preserved unknown metadata can produce a duplicate attribute
`crates/rdocx-oxml/src/comments_extended.rs:158`

An unrecognised `w15:done` value is intentionally retained in
`extra_attributes`. `resolve_comment` later sets the typed `done` field, and the
writer emits both the typed `w15:done` and the preserved attribute. XML forbids
two attributes with the same expanded name, so mutating a producer value such
as `w15:done="producer-value"` creates an invalid comments-extended part.

### D4, resolving a reply does not resolve its thread
`crates/rdocx/src/comments.rs:292`

`resolve_comment` writes `done` on the extension entry for the supplied comment
id. When the id names a reply, the parent linkage is ignored and only the reply
entry changes. The approved contract is to mark a thread resolved, so the
operation must follow `paraIdParent` to the root entry before updating state.

### D5, paragraph-id fallbacks are evaluated even when an id already exists
`crates/rdocx/src/comments.rs:195`

Both reply and resolve use `Option::unwrap_or` with a fallible paragraph-id
allocation expression. `unwrap_or` evaluates that expression eagerly. An
existing paragraph id therefore still pays for a full collision scan and the
operation can incorrectly return the allocator's exhaustion error even though
no fallback id is needed.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML child-order, namespace binding, test-gate,
or structural findings were found. The approved two-module and additive public
API shape remains sufficient.

# F-112, all, pass 2

**Reviewed**: uncommitted working diff against `HEAD`, 4 files, 551 additions and 2 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Pass 1 resolution

D1 is resolved. `CT_TextParagraph::properties_mut` detects the absent-property
case before inserting `a:pPr`. It rebuilds the raw boundary collection by
moving boundary 0 to boundary 1 and leaving every boundary from 1 through the
post-end-property boundary unchanged. Children are copied in original boundary
and document order, so only their relationship to the newly inserted property
changes.

`TextParagraphMut::set_properties` always uses `properties_mut`.
`TextParagraphMut::set_bullet` retains an existing property object, while its
absent-property and nonempty-bullet path also uses `properties_mut`. Clearing an
absent bullet remains a no-op and does not invent an empty `a:pPr`.

The preservation regression now uses a complete `mc:AlternateContent` run
substitution at the old boundary 0. It asserts `a:pPr`, the exact compatibility
subtree, and the following typed field and break in that order. It then reparses
and serializes the slide and repeats the same exact-byte and order checks.

## Not found

Correctness produced no additional findings. Text-body construction,
replacement, clearing, and append retain at least one paragraph. Replacement
and append semantics remain distinct. Existing body properties, list style,
first-paragraph state, end properties, and placeholder metadata survive frame
replacement as approved.

Raw-boundary handling produced no findings. Body replacement collapses removed
paragraph boundaries after the one surviving paragraph without changing raw
child order. Paragraph replacement keeps the before-property, after-run, and
post-end-property regions separate. Paragraph and run append move only the old
trailing boundary. Property insertion now moves only the former pre-property
region. Untouched fields, breaks, and raw MC bytes remain ordered and complete.

Text value handling produced no findings. New and replaced values use the
existing escaped text writer, which emits `xml:space="preserve"` for leading or
trailing whitespace. Direct run replacement keeps existing properties, raw
children, and source whitespace intent.

Contract produced no findings. `ShapeMut::set_text` handles both absent and
existing ordinary-shape bodies, preserves placeholder identity, and returns a
contextual error for unsupported kinds. The minimal newly created body has
canonical body properties, an optional absent list style, and one required
paragraph. Paragraph, bullet, character-property, and font setters take the
approved typed values by ownership.

Panics produced no findings. Invalid shape and paragraph indices are total.
The post-insertion `expect` and appended-run `unreachable` are protected by
immediate local invariants.

Borrow handling produced no findings. Frame, paragraph, and run handles remain
tied to the mutable borrow that created them. Each append returns the exact
newly inserted item, and no structural mutation can invalidate a live nested
handle.

OOXML produced no findings. Required paragraph cardinality, fixed prefixes,
paragraph and run choice order, end properties, bullet components, and font
properties use the existing schema-order writers. The repaired MC case
reparses with `a:pPr` ahead of the substituted run content.

Tests produced no findings. The focused raw-preservation and schema-order test,
the deterministic placeholder render test, and the paragraph field and break
round-trip test passed. The render path uses
`layout_presentation_deterministic`, which excludes discovered system fonts and
records no system-font baseline. `cargo fmt --all --check` and
`git diff --check` passed.

Structure produced no findings. The behavior-bearing `properties_mut` accessor
keeps boundary repair beside the paragraph model. No new module, trait, generic
parameter, feature, dependency, forwarding wrapper, or erased concrete type
was added.

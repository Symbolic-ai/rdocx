# F-217, all, pass 1

**Reviewed**: uncommitted working tree diff, 10 files, 2,846 changed lines, with 2,833 additions and 13 deletions
**Verdict**: 7 defects, 0 smells, 0 nitpicks

## Defects

### D1, duplicating a commented slide aliases its per-slide comment part
`crates/rpptx/src/lib.rs:1594`

The duplicate record clones `source.comments` while the duplicated relationship
scope keeps the original internal comment target. Both slides therefore expose
the same comment part and the same globally validated comment ids. Saving and
reopening the result fails duplicate-id validation. Removing either slide also
deletes the shared part at `crates/rpptx/src/lib.rs:1398`, leaving the other
slide's relationship dangling. A slide with comments triggers this through the
existing public `duplicate_slide` method.

### D2, section discovery ignores the required extension URI and parent shape
`crates/rpptx-oxml/src/presentation.rs:689`

The scanner claims the first descendant named `p14:sectionLst` anywhere inside
the presentation extension list. It never verifies that the direct parent is a
`p:ext` whose `uri` is `{521415D9-36F7-43E2-AB2F-B90AF26B5E84}`. A producer
payload with the same expanded name under another extension is therefore
exposed as typed sections and rewritten by `set_sections`, instead of remaining
opaque. This violates both the section extension contract and unmodelled XML
preservation.

### D3, a self-closing slide extension list becomes a duplicate extension list
`crates/rpptx-oxml/src/slide_parts.rs:563`

`find_extension_list_closing` ignores `Event::Empty`, so a valid existing
`<p:extLst/>` is reported as absent. `ensure_modern_comment_relationship` then
appends a second root `p:extLst` at
`crates/rpptx-oxml/src/slide_parts.rs:426`. The slide parser accepts both and the
comment mutation succeeds, but the emitted slide violates the root
`xsd:sequence` by containing two extension lists.

### D4, arbitrary comment status values are accepted and written
`crates/rpptx-oxml/src/comments.rs:468`

The parser stores any `status` string without checking the `active`, `resolved`,
or `closed` enumeration, and the public `Comment` and `CommentReply` fields can
also be assigned any string. The writer emits that value at
`crates/rpptx-oxml/src/comments.rs:815`. Setting `status` to `"bogus"` before
`add_comment` or `reply_to_comment` survives candidate reopen and commits schema
invalid modern comment XML, despite the plan requiring caller-supplied comment
values to be validated.

### D5, a harmless local `p188` shadow breaks fixed-prefix serialization
`crates/rpptx-oxml/src/comments.rs:464`

Comment and reply namespace declarations are retained as ordinary raw
attributes without rejecting conflicts with writer-owned prefixes. For example,
an aliased `<q:cm>` may legally carry an otherwise unused
`xmlns:p188="urn:producer"`. The writer changes the element name to `p188:cm`
and then replays that declaration at `crates/rpptx-oxml/src/comments.rs:729`, so
the modelled element is emitted in `urn:producer`. Any ordered mutation then
fails the required reopen instead of supporting prefix-tolerant input.

### D6, direct comments and processing instructions in typed lists are dropped
`crates/rpptx-oxml/src/comments.rs:862`

The ordered-list parser records only start and empty element events. Direct XML
comments, processing instructions, and text nodes fall through the wildcard arm
and disappear. The author-shell parser has the same behavior at
`crates/rpptx-oxml/src/comments.rs:883`. A producer comment or processing
instruction between authors, comments, replies, or inside an author is lost on
the first dirty serialization, contrary to the approved byte-exact raw sidecar
contract.

### D7, unsupported comment attributes are not preserved byte for byte
`crates/rpptx-oxml/src/comments.rs:470`

Unsupported attributes are decoded into `(String, String)` pairs and later
recreated through `push_attribute` at
`crates/rpptx-oxml/src/comments.rs:1073`. Entity spelling, quote choice, and
other producer lexical form are therefore canonicalized. An attribute such as
`x:flag='a&#x20;b'` does not survive a comment reorder byte-identically. The
round-trip test checks reparsed equality and selected child substrings, so it
does not exercise the design plan's byte-exact unsupported-attribute promise.

## Smells

None.

## Nitpicks

None.

## Not found

Panics and structure produced no findings. The only new module was explicitly
approved, and the diff adds no trait, generic, crate, feature, builder, wrapper,
or production dependency.

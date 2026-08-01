# S17 sprint review, pass 1

**Reviewed**: `sprint/s17` against `bdb89af`, 37 files, 6,486 changed
lines, crates: `oxml-drawing`, `rpptx-oxml`
**Verdict**: 1 blocking, 1 should-fix, 0 nice-to-have

## Blocking

### B1, empty elements leak namespace scopes into later siblings

`crates/rpptx-oxml/src/graphic_frame.rs:615`

`referenced_namespaces` combines `Event::Start` and `Event::Empty`, then uses
`BytesStart::is_empty()` to decide whether to pop the namespace scope. That
method tests whether the start-tag byte buffer is empty, which is false for
every named element. An empty element therefore leaves its declarations on
`local_scopes`. If it locally shadows an inherited prefix and a later sibling
uses that inherited prefix, the later use appears locally bound and lines 673
through 677 omit the required ancestor binding. A typed table then serialises
the preserved sibling with an unbound prefix. This violates the namespace
completion contract in `docs/hld/05-drawingml-model.md`. The fix must pop every
`Event::Empty` scope and add a regression with an empty local shadow followed
by a sibling that uses the inherited binding.

## Should-fix

### S1, shape and picture paths duplicate the placeholder application parser

`crates/rpptx-oxml/src/shape_tree.rs:483`

`crates/rpptx-oxml/src/picture.rs:540`

Both modules contain the same `p:nvPr` parser and the same placeholder-child
capture helper. This is the sprint-scope duplication check's exact failure
mode, and a namespace or preservation correction can now land in one path but
not the other. Consolidate the concrete parser in an existing module and keep
both callers on that one implementation.

## Nice-to-have

None.

## Milestone gate

The M8 end gate is: "all 50 corpus decks round-trip, and every one opens in
PowerPoint without a repair prompt."

S17 does not complete M8, so the manual PowerPoint half is not yet due and was
not claimed. The S17 slice holds. The required matching, crop, URI dispatch,
merged-cell, and all-corpus structural tests passed across the pinned 50-deck
corpus. The integrated full gate passed, including all 28 unchanged hashes.

## Not found

No additional findings in interaction, layering, harness attribution, sprint
gate coverage, HLD alignment, dependency consumers, or unrequested public API
surface. No Cargo manifest changed, and the integrated dependency tree retains
the documented direction.

# F-X038, all aspects, pass 2

**Reviewed**: remediated uncommitted working diff, 6 files and 1,676 changed lines, with 1,583 insertions and 93 deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, equal font-set lengths do not prove cold-order equality
`crates/oxml-layout/src/font.rs:382`

`every_loaded_font_is_current` compares only the number of historical and
current fonts. Prime an engine with paragraphs using families A then B, then
reorder those paragraphs to B then A. Cached font traces make
`layout_fonts == [B, A]`, while persistent `fonts` remains `[A, B]`. Both have
length two, so the engine skips canonicalization and returns persistent ids and
font-table order. A cold engine assigns ids and returns fonts in `[B, A]`
order. Warm and cold results therefore still differ when the active set is
unchanged but its first-resolution order changes. The remediation test replaces
one family and exercises a stale-set length mismatch, not this same-set reorder.

### D2, the loaded-face ceiling silently substitutes the wrong family
`crates/oxml-layout/src/font.rs:649`

After 256 distinct faces are loaded, a request that resolves to a new database
face is replaced by the first already-loaded face with matching bold and
italic flags. A document that legitimately uses a 257th installed family is
therefore shaped and rendered with unrelated bytes. Bounds may evict historical
cache state between layouts, but they cannot change the current document's
font semantics. The current bound test requests missing aliases that all map to
the same bundled face, so it never reaches this production fallback with 257
distinct faces.

### D3, transactional staging is unbounded until the whole layout finishes
`crates/rdocx-layout/src/engine.rs:813`

Every newly built safe paragraph is pushed into `pending_paragraph_cache`
without applying the entry or aggregate byte ceilings. Eviction happens only
after the complete document succeeds. A large cold document can therefore
retain a duplicate block and key for every safe paragraph during layout, then
discard nearly all of them while publishing the final 256 entries or 16 MiB.
This violates the required bounded-memory behavior precisely on the large
documents the story targets. Transactional rollback needs a bounded staging
queue with the same production eviction policy.

### D4, the global font trace grows with every resolution and retains capacity
`crates/oxml-layout/src/font.rs:885`

`record_layout_font` appends one event for every resolution call in the entire
document. `begin_layout` uses `clear`, so the persistent engine retains the
largest trace allocation across later edits. The trace is needed only while
capturing one candidate paragraph, yet unsafe paragraphs, headers, tables,
notes, and fields also accumulate events. This is another unbounded
process-lifetime allocation outside every declared cache ceiling.

### D5, debug text is not a byte bound for the owned paragraph key
`crates/rdocx-layout/src/engine.rs:1016`

The remediation now counts line and reflow buffers, but the cloned `CT_P` key
is still estimated by formatted debug output. That is not a conservative size
bound. For example, each safe run owns an inline `Option<CT_RPr>` whose value
type contains many `Option<String>` and vector fields. When the option is
`None`, the debug representation contributes only the word `None`, while each
run element still occupies the full enum and struct storage in the cloned
`runs` allocation. A paragraph with many empty or tiny runs can therefore
retain more than 16 MiB while its recorded key bytes remain below the ceiling.
The new boundary test enlarges only a reflow advance vector and does not cover
key-heavy paragraphs.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 1 D2 is closed by staging new entries until the complete layout succeeds
and discarding them on a later error. Pass 1 D3 and D4 are closed by the
isolated caller-font entrypoint and the shared document-owned engine for
tracked normal layouts. Coverage fallback and miss collections are now bounded
and deduplicated. Raw run content and AlternateContent drawings bypass reuse.
The invalidation regression now exercises warm and cold engines, the diagnostic
test uses a non-empty diagnostic and a late failure, and the TTC test loads two
real collection indices through the production face path. Production shaping,
file-byte, and reflow insertion paths now have focused byte-ceiling tests.

No new findings were found in transactional rollback correctness, poison
recovery, system, deterministic, and caller-font database isolation,
diagnostic ordering, result-local source rebinding, `Document: Send + Sync`,
OOXML ordering or preservation, panic sites, or the repository's structural
rules.

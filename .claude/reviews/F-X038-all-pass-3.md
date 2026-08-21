# F-X038, all aspects, pass 3

**Reviewed**: pass-2 remediated uncommitted working diff, 6 files and 2,097 changed lines, with 2,003 insertions and 94 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, reflow parameter buffers remain outside the paragraph byte ceiling
`crates/rdocx-layout/src/engine.rs:1208`

`paragraph_cache_entry_bytes` now accounts for the fixed `ParagraphReflow`
value, its `items` allocation, and each item's owned payload. It does not count
the allocations owned by `reflow.params`. In production that value contains a
second converted `tab_stops` vector for the paragraph. A cache-safe paragraph
with enough tab stops, in a document that preserves reflow because another
paragraph has a wrapping drawing, can therefore retain more than the 16 MiB
ceiling while its recorded bytes remain below the limit. The prefix and suffix
vectors are empty when the block is staged today, but they are also owned
buffers omitted by this structural calculation. The focused reflow boundary
test uses default parameters and enlarges only a text advance vector, so it
does not exercise the missing allocation.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 2 D1 is closed by exact loaded-font membership, order, and contiguous-id
comparison before the fast path. Same-set family reordering now canonicalizes
to the same font table and glyph ids as a cold engine.

Pass 2 D2 is closed for output correctness. More than 256 distinct active
faces resolve to their requested families, and inactive historical faces are
pruned only after result font data is canonicalized and paragraph entries are
filtered against the current trace. Failed attempts cannot leave a paragraph
entry referring to a removed face.

Pass 2 D3 and D4 are closed. Pending paragraph publication applies both
production ceilings before transaction commit. Font traces exist only for one
cache candidate, overflow bypasses publication, and both completed trace and
next-layout allocations release excess capacity.

Pass 2 D5 is otherwise closed for the paragraph key, lines, reflow items,
diagnostics, borders, and font traces. Raw and revision property payloads and
AlternateContent drawings bypass reuse.

No new findings were found in exact cache identities, successful-only
publication, diagnostics replay and ordering, context invalidation, system,
deterministic, and caller-font isolation, TTC face paths, process and shaping
cache bounds, poison recovery, `Document: Send + Sync`, F-X037 result-local
source rebinding, panic sites, OOXML preservation, or the repository's
structural rules.

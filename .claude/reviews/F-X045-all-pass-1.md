# F-X045, all, pass 1

**Reviewed**: complete working tree diff against `7f317e7`, 4 files, 1,071 changed lines, comprising 982 insertions and 89 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, retained-byte accounting does not measure the complete owned key

`crates/rdocx-layout/src/engine.rs:2138`

The entry-size calculation substitutes the length of a temporary `Debug`
rendering for retained capacity and adds only selected inner capacities. It
does not count the capacities of the section reference vectors, section
columns, the section change subtree, the part paragraph vector, the watermark
projection stored in the key, or the outer raw-XML and namespace vectors. A
cacheable typed part or section can therefore carry large spare capacity while
the recorded `bytes` remains below 4 MiB, allowing the queue to retain more
than its declared ceiling. The bounds test expands an output advances vector,
so it does not exercise this key-capacity path. Account every allocation owned
by the key and add a structural oversized-key regression.

### D2, the safety predicate discards every opaque subtree before deciding reuse

`crates/rdocx-layout/src/engine.rs:1656`

The predicate clears all paragraph and run raw XML, not only a recognized VML
watermark projection. A header containing an ordinary opaque `w:pict`, embedded
object, or other unmodeled run child is consequently treated as cache-safe as
long as its typed text is ordinary. That contradicts the approved contract to
bypass variants whose traversal state is not fully represented. Permit only
the exact supported watermark raw subtree needed by this story and reject all
other preserved producer XML, with a focused bypass regression.

### D3, the named variant regression does not exercise inheritance

`crates/rdocx-layout/src/engine.rs:4977`

`cacheable_header_footer_input` puts every default, first, and even reference
directly on the one final section, and `safe_header_footer_variants_reuse_exactly`
uses only that fixture. It therefore proves six direct variants but never an
inherited header or footer cache hit, despite the design contract and HLD gate
requiring inherited coverage. The pre-existing F-X042 PDF test protects output
selection, not this cache's hit and replay behavior. Add a multi-section case
whose later section receives inherited references and prove exact warm hits and
cold equality.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: no further findings. Exact typed equality authorizes hits, story
  kind and variant are part of the key, diagnostics and font traces replay, and
  source nodes rebind to the current result.
- Contract: no further findings. Media, full section state, styles, numbering,
  notes, theme, revision view, fonts, provenance mode, and the remaining
  reusable context invalidate reuse.
- Panics: zero production findings. The serializer `expect` follows an explicit
  successful result check, cache source ids are statically valid, and retained
  byte arithmetic otherwise saturates.
- OOXML: zero findings. The implementation does not alter parse or write order,
  and canonical header and footer serialization remains part of exact identity.
- Tests: no further findings. Direct default, first, even, header, footer,
  watermark, invalidation, transaction, queue bounds, warm-cold layout, PDF,
  and unchanged-hash behavior are covered.
- Structure: zero findings. The diff adds no trait, generic, crate, module,
  feature, dependency, forwarding wrapper, or new test binary.

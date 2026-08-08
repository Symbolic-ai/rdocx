# F-106, all, pass 1

**Reviewed**: working-tree diff against
`e4d59ae8edeb899bb96a3748c0e180b788120c3f`, 9 files, 372 insertions and
25 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, non-visual id projection can select an extension element

`crates/rpptx-oxml/src/namespace.rs:113`

The helper scans every descendant and accepts `cNvPr` by local name alone. A
captured PresentationML `p:cNvPr` whose required root `id` is absent can contain
an extension descendant such as `x:cNvPr id="9"`, causing the accessor to
return 9 instead of `None`. Group and graphic-frame accessors also scan opaque
children with this helper, so a foreign `x:cNvPr` before the real
PresentationML child can shadow the actual shape id. This contradicts the
plan's malformed-absence result and can make the allocator reserve the wrong
ids.

### D2, a conflicting existing default leaves new media with the wrong type

`crates/rpptx/src/lib.rs:165`

Insertion correctly sniffs the bytes, but it registers the result only through
`add_default`. That method preserves an existing mapping. If a loaded package
maps `png` to a different content type, the new `.png` part remains governed by
that conflicting default instead of the sniffed `image/png` type. The story's
content-type contract therefore fails on an existing package with a stale or
producer-specific default. A part override can preserve the existing default
while typing the new part correctly.

### D3, reuse among duplicate existing media parts is nondeterministic

`crates/rpptx/src/lib.rs:115`

The initial index is populated by iterating `OpcPackage::parts`, which is a
`HashMap`. If two existing media parts contain identical bytes, their bucket
order varies with the map seed, and the later lookup at line 157 returns the
first arbitrary entry. A future insertion of those bytes can therefore target
different existing part names across identical runs. Sort existing part names
before indexing so the deduplication result is deterministic.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML child-order, preservation, test-gate,
dependency-direction, public-API, or structural issues were found.

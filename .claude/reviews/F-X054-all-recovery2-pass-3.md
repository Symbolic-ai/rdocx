# F-X054, all, second recovery pass 3

**Reviewed**: uncommitted working diff, 15 files, 3,319 changed lines with
3,250 additions and 69 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, A Word namespace alias used by retained XML is discarded

`crates/rdocx/src/document.rs:873`

Nested owner discovery removes every declaration whose value is the
WordprocessingML namespace before it determines whether a retained raw marker
depends on that declaration. For example, a modeled `q:p` can declare
`xmlns:x` as the WordprocessingML namespace and retain the unmodeled direct
child `<x:producer/>`. The paragraph serializer does not retain the local
declaration, owner replay sees no declaration to restore, and even an unchanged
save writes `<x:producer/>` with `x` unbound. Reopen then changes the public
unsupported namespace fact from the WordprocessingML URI to `None`. A Word
alias used only by typed content may be excluded, but an alias used by retained
raw content must participate in marker identity and replay.

### D2, A descendant shadow is mistaken for use of the owner declaration

`crates/rdocx/src/document.rs:541`

The dependency check excludes a prefix only when it is declared on the same
event. It does not account for declarations inherited from an intermediate raw
ancestor. For example, a modeled paragraph can declare
`xmlns:wp="urn:owner"` and retain
`<x:wrapper xmlns:x="urn:x" xmlns:wp="urn:child"><wp:producer/></x:wrapper>`.
The producer is bound by the wrapper's declaration, so no retained event uses
the paragraph's `wp` binding. The marker collector still attributes that use to
the paragraph declaration. The fixed-prefix guard then rejects every otherwise
safe edit with a shadowed `wp` error. Dependency-only guarding must track which
scope supplied each resolved binding, not only whether the current event
declares the prefix.

## Smells

None.

## Nitpicks

None.

## Not found

The implicit `xml` binding now resolves consistently in public unsupported
facts and logical snapshots. Empty and expanded raw events both remain marker
identity evidence. Exact marker matching is one-to-one, byte sensitive, and
cardinality preserving. Used foreign fixed-prefix bindings still fail closed,
while the direct unused fixed-prefix regression succeeds.

All earlier recorded triggers were rechecked. No additional findings were
found in body, cell, paragraph, hyperlink, or run item ordering, exact exposed
raw bytes, public enum exhaustiveness, drawing and field projections, legacy
flattened accessors, producer-defined numbering round trips, layout and
exporter marker suppression, fail-closed ordinary or deleted text decoding,
Python error classification, OOXML child order, panic safety, public API
documentation, dependency structure, test naming, or the repository
structural rules. The complete 166-test `rdocx` regression binary and all 278
`rdocx-oxml` unit tests plus its documentation test passed during this review.

# F-X054, all, third recovery pass 1

**Reviewed**: uncommitted working diff, 15 files, 3,476 changed lines with
3,407 additions and 69 deletions
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, A raw-dependent Word alias makes typed siblings impossible to correlate

`crates/rdocx/src/document.rs:641`

The logical-owner snapshot normalizes every element and attribute using a
retained declaration prefix to the same owner-declaration token. It does not
limit that normalization to the retained raw marker events that made the
declaration necessary. For example, a paragraph can declare `x` as the Word
namespace and contain both retained `<x:producer/>` and modeled
`<x:r><x:t>typed</x:t></x:r>`. The original snapshot normalizes the producer,
run, and text names through the `x` token. Canonical serialization changes the
modeled run and text to `w`, so the candidate snapshot resolves them to the
Word namespace instead. The structures no longer match and a safe edit such as
appending an unrelated paragraph fails with `cannot identify retained p nested
namespace owner after mutation`. The new regression uses `q` for its modeled
siblings, so it does not exercise a raw-dependent alias that is also used by
typed content.

### D2, An identical intermediate shadow still counts as an owner dependency

`crates/rdocx/src/document.rs:600`

Dependency collection decides that an event uses an owner declaration by
comparing the active namespace URI with the owner declaration value. It does
not retain which scope supplied that value. If a paragraph declares
`xmlns:wp="urn:foreign"` and its retained raw wrapper independently redeclares
the same `xmlns:wp="urn:foreign"` before `<wp:producer/>`, the descendant is
self-contained and does not depend on the paragraph binding. The URI equality
still marks `wp` as owner-dependent. The fixed-prefix guard then rejects an
otherwise safe modified save with a shadowed `wp` error. The new regression
uses different owner and child URIs, which proves value-changing shadowing but
not an identical redeclaration. Scope provenance, rather than URI equality,
must decide whether the owner declaration is required.

## Smells

None.

## Nitpicks

None.

## Not found

The two prior recovery defects are fixed for their exact new regression
inputs. A Word alias used by raw XML replays when modeled siblings use another
Word prefix. An intermediate raw ancestor with a different namespace value
shadows the owner binding, while direct retained use of a foreign fixed prefix
still fails closed.

All 168 `rdocx` regression tests passed. The focused producer-numbering and
undecodable-text tests passed, along with the ordered-reader save and reopen
gate. Prose checking and `git diff --check` passed.

No additional findings were found in body, cell, paragraph, hyperlink, or run
item ordering, complete typed variant projection, exact raw bytes, modeled
unsupported facts, namespace classification outside the two cases above,
producer-defined numbering preservation, layout and exporter marker
suppression, fail-closed ordinary or deleted text decoding, Python error
classification, legacy flattened accessors, public enum exhaustiveness,
OOXML child order, panic safety, public documentation, dependency structure,
test naming, or the repository structural rules.

# F-X054, all, pass 1

**Reviewed**: uncommitted working diff, 13 files, 1,360 changed lines with
1,310 additions and 50 deletions
**Verdict**: 5 defects, 0 smells, 0 nitpicks

## Defects

### D1, Run items drop raw children before run properties

`crates/rdocx/src/run.rs:568`

When a parsed run has properties, the first requested raw boundary is one.
A raw child that precedes `w:rPr` has stored boundary zero, so it is never
selected by the loop. For example, a run containing an unsupported child,
then `w:rPr`, then `w:t` silently omits that unsupported child from
`RunRef::items()`. This violates the exact direct-child and preserved-boundary
contract even though serialization still retains the raw child.

### D2, Conventional-prefix fallback misclassifies shadowed namespaces

`crates/rdocx/src/document.rs:149`

The fallback assigns Word, relationship, and markup-compatibility identities
from the spelling of the prefix rather than its in-scope binding. A document
can bind `q` to WordprocessingML and shadow `w` with `urn:foreign`. An
unsupported `<w:producer/>` body child then reports the WordprocessingML URI
instead of `urn:foreign`. This contradicts the namespace-aware alias contract
and can make foreign content look modeled.

### D3, Body-local inherited namespace declarations are not available

`crates/rdocx/src/document.rs:1334`

Raw body facts receive only declarations retained from the document element.
A valid `<q:body xmlns:x="urn:body"><x:producer/></q:body>` input captures the
child as `<x:producer/>`, but `UnsupportedXmlRef::namespace_uri()` returns
`None` because the body declaration is neither local to the raw subtree nor in
`document.extra_namespaces`. The contract requires resolution through
in-scope inherited declarations, not only root declarations.

### D4, An empty default namespace declaration panics

`crates/rdocx/src/document.rs:235`

`raw_attribute_value` calls `slice::windows` with the namespace value length.
For valid namespace undeclaration such as `<producer xmlns=""/>`, that length
is zero and `windows(0)` panics. Calling the public `namespace_uri()` accessor
on untrusted document content must return a fact or `None`, not panic.

### D5, Child-content detection ignores visible CDATA and entity references

`crates/rdocx/src/document.rs:267`

The event loop treats only `Event::Text` as visible text. Quick XML reports
CDATA as `Event::CData` and entity references as `Event::GeneralRef`, so
`<x:item><![CDATA[visible]]></x:item>` and `<x:item>&amp;</x:item>` both report
`has_child_content() == false`. That contradicts the method's visible-text
contract and leaves the required child-content fact incomplete.

## Smells

None.

## Nitpicks

None.

## Not found

No additional defects were found in producer-defined numbering preservation,
fail-closed ordinary or deleted text decoding, Python error classification,
cell ordering, paragraph and hyperlink sidecar ordering, field projection,
legacy flattened accessors, public enum exhaustiveness, dependency structure,
schema order, or the focused gate commands that were run. No smells or
nitpicks were found.

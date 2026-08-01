# F-068, correctness, pass 1

**Reviewed**: working tree against the claimed base, 6 files, 1,029 insertions
and 19 deletions
**Verdict**: 3 defects, 1 smell, 0 nitpicks

## Defects

### D1, local-name matching models extension elements as PresentationML

`crates/rpptx-oxml/src/presentation.rs:137`

The root and child dispatch use only the local name and never verify the
namespace URI. An extension document rooted at `x:presentation`, or an
unsupported `x:sldIdLst` child inside a real presentation, is accepted and
rewritten as typed PresentationML. This violates the any-prefix contract,
which changes the prefix but not the namespace, and it drops the required raw
preservation for same-local-name extension children.

### D2, any qualified id attribute is consumed as the relationship id

`crates/rpptx-oxml/src/presentation.rs:547`

The identifier parser treats every qualified attribute whose local name is
`id` as the OPC relationship id. A producer extension such as `vendor:id`
therefore replaces `r:id`, or causes a duplicate error when both are present,
instead of surviving as an unsupported attribute. Attribute matching must use
the relationships namespace URI with any prefix.

### D3, canonical prefix declarations can change preserved raw XML semantics

`crates/rpptx-oxml/src/presentation.rs:778`

The parser always removes root declarations named `xmlns:p`, `xmlns:a`, and
`xmlns:r`, while the writer always rebinds those prefixes to the canonical
namespaces. If a valid alternate-prefix document uses one of those lexical
prefixes for a producer extension, a preserved raw child or qualified raw
attribute is silently rebound on output. The fixed-prefix writer must retain
the namespace meaning of raw XML, either by rejecting conflicting bindings or
by localising the original binding on preserved raw content.

## Smells

### S1, presentation child capture has thirteen mutable output parameters

`crates/rpptx-oxml/src/presentation.rs:305`

The clippy exception hides a parser-state function whose fields must remain in
lockstep with the final root construction. Group the in-flight presentation
state so a new field or schema slot has one ownership point and cannot be
omitted from one of the duplicated call sites.

## Nitpicks

None.

## Not found

No other correctness, contract, panic-path, OOXML ordering or preservation,
test-strength, or structural findings.

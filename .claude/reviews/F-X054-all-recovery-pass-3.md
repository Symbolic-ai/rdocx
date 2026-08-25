# F-X054, all, recovery pass 3

**Reviewed**: uncommitted working diff, 15 files, 2,424 changed lines with
2,355 additions and 69 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Owner insertion moves a retained namespace declaration to the wrong element

`crates/rdocx/src/document.rs:516`

Nested declarations are associated with only an element kind and its original
global occurrence number. Replay then applies that occurrence number to the
new serialized tree. A supported mutation can insert another owner before the
recorded owner. For example, open a document whose first paragraph declares
`xmlns:x="urn:producer"` for a retained `<x:producer/>` child, then call
`insert_paragraph(0, "new")`. Replay puts the declaration on the new paragraph
at occurrence zero. The original paragraph moves to occurrence one and keeps
the raw child bytes without their declaration, so save emits an unbound prefix
and changes the expanded name after reopen.

The recovery gate at `crates/rdocx/tests/regression_test.rs:2194` mutates only
by appending a paragraph. It therefore leaves every recorded occurrence stable
and does not exercise insertion or removal before a declaration-owning
paragraph, table, cell, content control, hyperlink, or run. This violates the
contract that ordinary nested declarations are replayed on their corresponding
modeled owner after modification.

## Smells

None.

## Nitpicks

None.

## Not found

All original pass 1 through pass 3 triggers and both direct recovery pass 2
triggers were rechecked. Raw run children before properties, namespace-aware
prefix classification, body-local inherited scope, empty default namespace
undeclarations, CDATA and entity child-content facts, decoded local and root
namespace URIs, complete public item variants, recursive save and reopen
snapshots, distinct root and body scopes, fixed serializer-prefix collisions,
parser-derived names, empty modeled controls, and numeric whitespace
references remain fixed for their recorded inputs.

No additional findings were found in unchanged unsafe-scope byte preservation,
modified unsafe-scope rejection, arbitrary declaration replay when owner
positions remain stable, exact retained raw subtree bytes, cell, paragraph,
hyperlink, or run item order, producer-defined numbering preservation, layout
and exporter marker suppression, fail-closed ordinary or deleted text
decoding, Python error classification, legacy flattened accessors, public enum
exhaustiveness, OOXML child order, panic safety, API documentation, dependency
structure, or the repository structural rules. The complete 157-test `rdocx`
regression binary passed during this review. No smells or nitpicks were found.

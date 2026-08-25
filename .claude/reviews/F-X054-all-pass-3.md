# F-X054, all, pass 3

**Reviewed**: uncommitted working diff, 13 files, 1,706 changed lines with
1,654 additions and 52 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, Body namespace promotion collapses distinct XML scopes

`crates/rdocx/src/document.rs:1121`

The save path copies declarations that were in scope on `w:body` onto the
document root. It skips `xmlns:w`, `xmlns:r`, and `xmlns:mc`, while it replaces
an existing root declaration for every other prefix. Neither choice preserves
a body-local shadow. For example, an aliased Word document can declare
`xmlns:w="urn:foreign"` on its body and retain `<w:producer/>` as unsupported
content. Before save the new fact correctly reports `urn:foreign`. Save skips
that binding, emits the canonical Word binding for `w` at the root, and writes
the raw subtree unchanged. Reopen then reports the WordprocessingML namespace
instead. An ordinary prefix collision has the converse failure: replacing a
root `xmlns:x="urn:root"` with the body's `xmlns:x="urn:body"` changes the
namespace of preserved root-scope XML such as `background_xml`.

The shadowed-prefix regression at
`crates/rdocx/tests/regression_test.rs:1864` checks only the pre-save fact, and
the body-local save-reopen regression at
`crates/rdocx/tests/regression_test.rs:1877` uses a prefix with no conflicting
root binding. The recursive round-trip fixture therefore remains green without
exercising either collision. This violates namespace-aware classification, raw
subtree preservation, and save-reopen equality.

## Smells

None.

## Nitpicks

None.

## Not found

All pass-1 D1 through D5 triggers and the direct pass-2 D2 through D4 triggers
were rechecked and remain fixed. Pass-2 D1 is fixed only for non-colliding
body-local namespace declarations, with the remaining collision defect
reported above.

No additional correctness, contract, panic-safety, OOXML namespace or child
order, raw-preservation, public-variant coverage, gate-reality, round-trip,
structure, dependency, or API-compatibility findings were found. Producer
numbering preservation, fail-closed ordinary and deleted text decoding, Python
error classification, complete direct item ordering, legacy flattened
accessors, non-exhaustive open item enums, and the focused gates were also
rechecked without another finding.

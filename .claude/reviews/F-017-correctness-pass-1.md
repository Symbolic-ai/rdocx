# F-017, correctness, pass 1

**Reviewed**: working diff, 4 files, 920 additions and 5 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, root parts accept missing, wrong, or unclosed roots

`crates/oxml-core/src/app_properties.rs:193`

`crates/oxml-core/src/custom_properties.rs:59`

Both event loops treat end of file as successful completion without tracking
whether the expected `Properties` root was opened and closed. The application
parser also captures a wrong root as an unknown child. Empty input, a document
rooted at another element, or a truncated `Properties` element can therefore
produce an apparently valid model whose serialization has a different
structure. Require exactly one expected root and its closing event, with a
regression test for malformed root parts.

## Smells

None.

## Nitpicks

None.

## Not found

No additional contract, panic, OOXML ordering, namespace, preservation, test,
or structural findings.

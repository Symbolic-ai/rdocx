# F-027, correctness, pass 1

**Reviewed**: working tree diff, 8 files and 199 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, conflicting defaults can defeat sniffed package metadata

`crates/rdocx/src/document.rs:728`

`ContentTypes::add_default` keeps an existing mapping. When an opened package
already maps the sniffed extension to a different content type, storing a new
image leaves that conflicting default in force. For example, a package with
`jpeg` mapped to `application/octet-stream` stores the new `.jpeg` part but
reports the wrong content type. This violates the plan's canonical metadata
contract. Register an override for the new part when its effective content type
conflicts, and cover that loaded-package case.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings.

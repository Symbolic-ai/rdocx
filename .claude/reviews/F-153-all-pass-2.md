# F-153, all aspects, pass 2

**Reviewed**: remediated working tree implementation, 3 files and 1,331 added lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D4, a nested datastoreItem lookalike is accepted as the properties root

`crates/rdocx/src/content_control.rs:618`

The namespace and attribute checks are now correct, but the parser accepts the
first `ds:datastoreItem` at any depth. A different root can wrap a nested
lookalike and still bind the custom XML item. The properties contract requires
`ds:datastoreItem` to be the document element, so lookup must inspect only the
first element after declarations, comments, and processing instructions.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-1 findings D1 through D3 and S1 through S2 are closed. No additional
control ownership, exact-value replacement, atomicity, XPath-boundary,
namespace-shadowing, schema-order, byte-preservation, panic, arithmetic,
dependency, or structural issue was found.

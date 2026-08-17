# F-153, all aspects, pass 1

**Reviewed**: working tree implementation, 3 files and 1,331 added lines
**Verdict**: 3 defects, 2 smells, 0 nitpicks

## Defects

### D1, an outer update reads and mutates nested controls as its own value

`crates/rdocx/src/content_control.rs:187`

Both summary text collection and display replacement recurse through nested
content controls. An outer control therefore reports the concatenated value of
its nested controls. Updating only the outer control also clears or rewrites a
nested control that had no matching tag or alias. Nested controls are separate
items in the ordered facade collection and must remain unrelated unless their
own key selects them.

### D2, non-text run content survives a value replacement

`crates/rdocx/src/content_control.rs:420`

Display replacement changes only `RunContent::Text`. A plain-text control whose
old display contains a tab or hard break retains that old display content after
the method reports success, so its visible value is not the requested value.

### D3, datastore item lookup accepts an item id from the wrong namespace

`crates/rdocx/src/content_control.rs:673`

The properties root is checked against the custom XML namespace, but its
`itemID` attribute is matched by local name only. A producer extension attribute
with the same local name can therefore be treated as `ds:itemID` and bind the
wrong datastore. Attribute namespace resolution must be checked too.

## Smells

### S1, the direct alias mutation method has no gate

`crates/rdocx/src/content_control.rs:105`

The regressions exercise alias lookup and map fallback, but never call
`set_content_control_value_by_alias`. Replacing that public method with a no-op
would leave every current test green.

### S2, the invalid-binding regression does not prove rollback after a valid edit

`crates/rdocx/tests/regression_test.rs:168`

Each case contains only one invalid bound control. The contract specifically
stages a group so a later failure cannot expose an earlier display or part
change. The test should put a valid selected binding before the invalid one and
then compare the complete package bytes.

## Nitpicks

None.

## Not found

No additional contract drift, unchecked indexing, arithmetic overflow,
schema-order violation, prefix-resolution defect, custom XML byte-loss path,
unjustified trait, generic, wrapper, dependency, feature flag, crate, module, or
public type was found.

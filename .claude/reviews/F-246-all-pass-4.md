# F-246, all, pass 4

**Reviewed**: remediated working-tree diff excluding review records, 20 files, 2,838 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, changing a scalar value drops its unmodeled metadata
`crates/rdocx-oxml/src/styles.rs:1230`

The raw scalar element is replayed only while its modeled value is unchanged.
Changing a name, link, priority, or toggle emits a fresh element containing
only the modeled `w:val`. Producer attributes, local namespace declarations,
comments, and processing instructions on that same element disappear even
though the update did not target them. The scalar writer needs to replace the
modeled value while retaining the rest of the captured element.

### D2, adding one conditional region during an update removes the others
`crates/rdocx/src/document.rs:3628`

An empty conditional-region list preserves the existing list, but any authored
region makes the complete existing list disappear. Calling
`conditional_table_style` to add or update one region therefore removes every
unmentioned region without using the explicit clear operation. A matching
region also loses any unmentioned paragraph, table, or cell property group.
Update semantics need to preserve unmentioned regions and merge the supplied
property groups into a matching region.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 3 default paragraph chain and table-look overlay are correct. Graph
validation, atomic publication, reference scanning, schema order, base table
resolution, panic safety, public API scope, tests, and repository structure
produced no additional findings.

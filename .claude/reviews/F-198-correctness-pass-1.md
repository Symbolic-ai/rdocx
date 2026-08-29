# F-198, correctness, pass 1

**Reviewed**: working-tree diff, 21 files, 1,127 additions and 21 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, explicit nonempty `w:lang` syntax is not typed
`crates/rdocx-oxml/src/properties.rs:1126`

The typed `w:lang` branch handles only an XML empty event. The equivalent
`<w:lang w:val="en-US"></w:lang>` form reaches the generic start-element arm
and remains raw, so `CT_RPr::language` is empty and automatic hyphenation is
silently disabled for that run. Complete `w:lang` typing must accept both XML
surface forms while preserving the existing raw fallback for foreign content.

## Smells

None.

## Nitpicks

None.

## Not found

Contract, panics, OOXML namespace and child order, tests, and structure produced
no other findings. Dependency direction, generated source provenance, regional
language mapping, paragraph suppression, deterministic evidence, and package
scope match the approved plan.

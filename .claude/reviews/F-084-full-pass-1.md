# F-084, full, pass 1

**Reviewed**: working-tree implementation diff, 8 files, 874 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, opaque effect DAGs bypass replacement and placeholder checks

`crates/rpptx-layout/src/style.rs:94`

The resolver only treats the modelled `CT_EffectList` field as an explicit
effect and only scans that modelled list for `phClr`. A shape whose `p:spPr`
contains a preserved `a:effectDag` therefore retains the referenced theme
effect instead of replacing it atomically. If that opaque DAG contains
`a:schemeClr val="phClr"`, resolution also returns success instead of
`ResolveError::UnresolvedPlaceholderColor`. The same missing check affects an
opaque effect DAG selected from a theme `a:effectStyle`.

## Smells

None.

## Nitpicks

None.

## Not found

No other correctness, contract, panic, OOXML, test, or structure findings.
The numeric index policies, reference-colour transform ordering, typed style
child order, raw style preservation, public boxing remediation, and named gate
match the revised design contract.

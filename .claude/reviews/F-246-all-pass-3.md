# F-246, all, pass 3

**Reviewed**: remediated working-tree diff excluding review records, 20 files, 2,749 changed lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, the default paragraph style skips its inheritance chain
`crates/rdocx/src/style.rs:605`

When no explicit paragraph style is present, the facade merges only the
default style's own paragraph properties. It does not walk that style's
`basedOn` chain as the explicit-style path does. The layout copy has the same
logic at `crates/rdocx-layout/src/style_resolver.rs:122`. A default style that
inherits indentation, spacing, or pagination properties from a base therefore
renders and resolves without those inherited values. This contradicts the
base-first default-style contract.

### D2, inherited table-look fields are replaced as one value
`crates/rdocx-layout/src/table.rs:684`

Table width, borders, and margins overlay field by field, but a derived
`w:tblLook` replaces the complete inherited look. If a base style enables a
first-row treatment and a derived style only changes horizontal banding, the
derived partial look erases the inherited first-row flag. Table-look
attributes need the same non-None base-first overlay used by style updates.

## Smells

None.

## Nitpicks

None.

## Not found

Correctness outside the two inheritance gaps, contract scope, panic safety,
OOXML namespace and schema ordering, preservation tests, atomic mutation,
repository structure, and the unchanged 49-entry hash baseline produced no
additional findings.

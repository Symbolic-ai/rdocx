# S14 sprint review, pass 1

**Reviewed**: `sprint/s14` against `597eb7f69df79760c45b9fbb3c2a8678ba915dca`, 33 files, 8,101 changed lines, crates: `oxml-drawing`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, Valid line and shape-property attributes are discarded

`crates/oxml-drawing/src/line.rs:398`
`crates/oxml-drawing/src/line.rs:513`
`crates/oxml-drawing/src/shape_props.rs:122`
`crates/oxml-drawing/src/shape_props.rs:230`

`CT_LineProperties::from_start` retains only `w` and `cap`, and its writer
reconstructs only those two attributes. A valid line carrying `cmpd` or `algn`
therefore loses those values on its first round trip. `CT_ShapeProperties`
discards the root start element entirely and always creates a fresh `a:spPr`,
so a valid `bwMode` is lost in the same way. This breaks the sprint goal to
preserve unmodelled XML at its schema boundary and prevents the eventual
corpus `a:spPr` gate from holding. The fix must retain unmodelled root
attributes, or model every schema attribute, and add focused round-trip
regressions for both elements.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The S14 definition requires the line, effect, shape-property, style-reference,
and text vocabularies to preserve unmodelled XML at its schema boundary while
writing children in schema order. Child-order, text, bullet, list-style, dash,
effect, and style-reference evidence passes, and the 28-entry hash harness is
unchanged. The gate does not yet hold because B1 demonstrates valid root
attributes that are not preserved.

## Not found

No additional interaction, duplication, layering, harness, documentation,
dependency, or public-surface finding was found. `oxml-drawing` retains only
the permitted downward dependency on `oxml-core`, no workspace dependency or
manifest changed, and released Word crate paths remain untouched.

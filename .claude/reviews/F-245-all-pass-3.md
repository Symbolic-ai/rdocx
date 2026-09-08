# F-245, all, pass 3

**Reviewed**: working-tree implementation diff excluding review records, 17 files, 2,072 additions and 40 deletions
**Verdict**: 4 defects, 0 smells, 0 nitpicks

## Defects

### D1, fixed-prefix serialization can duplicate a producer namespace binding

`crates/rdocx-oxml/src/font_table.rs:649`

The pass 2 alias fix retains any noncanonical value bound to `w`, `r`, or
`rdocx`, while `to_xml` unconditionally writes those same three prefix names at
lines 220 through 222. A valid input such as `q:fonts` with `xmlns:q` bound to
Word and `xmlns:w` bound to a producer namespace serializes two `xmlns:w`
attributes after mutation. The output is not well-formed XML. Conflicting
fixed-prefix bindings must be rejected safely or remapped before raw replay.

### D2, the public font-key validator accepts non-OOXML lexical forms

`crates/rdocx/src/document.rs:11965`

`font_key_bytes` removes braces and every hyphen before checking only 32 hex
digits. It therefore accepts an unbraced value or hyphens at arbitrary
positions, then `embed_font` writes that original invalid value into
`w:fontKey`. The repository's other OOXML GUID validators require the exact
38-byte braced form with hyphens at positions 9, 14, 19, and 24. The public API
must reject a key that cannot be emitted as the promised valid OOXML GUID.

### D3, public descriptive font values can violate schema enumerations

`crates/rdocx/src/document.rs:8607`

`set_font` validates only a nonempty name and an empty embedded-font vector.
It accepts any `family` or `pitch` string, while the corresponding Word values
are closed schema enumerations. Inputs such as `family="unknown"` or
`pitch="wide"` are committed and later serialized into invalid font-table XML.
The authoring boundary must validate these two modeled values before staging.

### D4, the differential gate does not check theme-color behavior

`crates/rdocx/tests/integration_test.rs:477`

The approved test contract names both effective fonts and theme colors. The
test now has real Word font-table structure and a pinned LibreOffice pass, but
its theme assertions cover only major and minor typeface names. The rendered
run has no theme-color reference, so the pixel comparison also cannot fail if
authored theme colors are lost or ignored. A source-encoded Word color
projection and an assertion over the authored or reopened theme are still
required.

## Smells

None.

## Nitpicks

None.

## Not found

Pass 2 relationship ownership and package-wide reference safety are memories
of the actual package graph now, and their focused regressions exercise both
failure modes. Aliased nonconflicting prefixes and explicit-element comments,
processing instructions, and whitespace survive the modeled edit. No new
panic or arithmetic issue was found in those paths. The implementation still
uses the one approved schema module and introduces no unjustified trait,
generic parameter, feature flag, crate, or forwarding wrapper.

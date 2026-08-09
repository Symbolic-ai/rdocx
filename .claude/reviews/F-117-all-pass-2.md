# F-117, all, pass 2

**Reviewed**: working-tree implementation diff, 9 files, 1,100 added lines and
5 removed lines. This includes all 1,034 untracked lines in
`crates/oxml-sml/Cargo.toml`, `crates/oxml-sml/README.md`, and
`crates/oxml-sml/src/lib.rs`. The 58-line pass 1 review record and the progress
notes were inspected separately.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, SpreadsheetML string encoding still changes or malforms valid inputs

`crates/oxml-sml/src/lib.rs:303`

The shared encoder leaves tab and line-feed characters unchanged, then uses
that result for the `formatCode` attribute at line 517. XML attribute-value
normalization changes those characters to spaces, so a caller's number format
does not round-trip as promised. The encoder also emits U+FFFE and U+FFFF
unchanged even though XML 1.0 forbids both scalars. A header, text value,
worksheet name, or number format containing either scalar therefore produces
malformed XML instead of a contextual error. Extend the boundary validation
and cover tab, line feed, U+FFFE, and U+FFFF in the exact encoding tests.

### D2, the accepted input space can panic while interning shared strings

`crates/oxml-sml/src/lib.rs:438`

The constructor limits each column independently, but the workbook may contain
up to 16,384 text columns with 1,048,575 values each. That permits more than
`u32::MAX` distinct shared strings. `SharedStrings::intern` asserts that the
column limit bounds the string count and panics once the next index no longer
fits `u32`. The same accepted space can write an `sst` `count` greater than the
SpreadsheetML unsigned-integer boundary at line 450. Serialization is a
fallible public operation, so reject an unrepresentable shared-string table or
return a contextual error instead of panicking or emitting an invalid count.

## Smells

None.

## Nitpicks

None.

## Not found

The pass 1 row-limit defect is fixed at the accepted and first-rejected
boundaries. The SHA-bound viewer candidate matches the recorded digest, and
the ignored gate now checks pinned Excel and LibreOffice versions, worksheet
content, repair or conversion failure, and the converted package. No additional
findings were found in contract scope, OOXML schema child order, namespaces,
package relationships, deterministic allocation, dependency layering, tests,
or structure. Focused tests and strict Clippy passed, and the normal dependency
tree contains only `oxml-opc`, `quick-xml`, `thiserror`, and their external
dependencies.

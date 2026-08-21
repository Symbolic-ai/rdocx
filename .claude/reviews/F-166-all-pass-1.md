# F-166, all, pass 1

**Reviewed**: Uncommitted working diff, 3 files and 385 changed lines, with 383
additions and 2 deletions
**Verdict**: 3 defects, 2 smells, 0 nitpicks

## Defects

### D1, Section mode duplicates document-wide body identities

`crates/rdocx/src/field.rs:277`

Each candidate body is appended without remapping identities that must remain
unique across the combined document. A two-record template containing one
bookmark therefore produces two starts and two ends with the same id and two
bookmarks with the same name. The facade reports that result as malformed and a
later `REF` update cannot resolve the duplicated target. Drawing and content
control identities have the same cloning hazard. The returned document is not a
valid independent section for each record when the source body carries these
ordinary elements.

### D2, Nested header and footer fields bypass the variation rejection

`crates/rdocx/src/field.rs:86`

`crates/rdocx/src/field.rs:255`

Header and footer evaluation visits only the direct paragraphs projected by
`CT_HdrFtr`. Valid tables and content controls remain raw in that model. A
`MERGEFIELD` inside a header table is consequently left at its stored display
for every candidate. The raw part bytes compare equal, so section mode succeeds
for records with different values instead of rejecting the record-varying
non-body field required by the approved contract.

### D3, Relationship-resolved footnotes can be merged into the wrong part

`crates/rdocx/src/field.rs:246`

`crates/rdocx/src/document.rs:713`

The candidate round trip flushes typed footnotes to the conventional
`/word/footnotes.xml` path even when the existing `FOOTNOTES` relationship
targets another valid package part. The relationship continues to point at the
unchanged producer part, so separate mode returns the stored footnote display
and adds an unrelated conventional part. Section mode then compares the
unchanged relationship target and can accept records whose footnote merge
values differ. This violates both non-body rejection and package preservation.

## Smells

### S1, Staging clone logic now has two exact copies

`crates/rdocx/src/document.rs:427`

`crates/rdocx/src/document.rs:2608`

`clone_for_staging` duplicates `clone_for_template` field for field. Any later
addition to `Document` must now be kept synchronized across two staging helpers,
and the names make callers inspect both to discover that they are identical.
One shared staging clone keeps this construct local and satisfies the structural
rule against adding another place to look.

### S2, The round-trip test does not prove byte preservation

`crates/rdocx/tests/regression_test.rs:3586`

The test counts namespace and attribute substrings, so it still passes if an
unmodelled producer subtree is reordered, truncated, or otherwise changed while
those two tokens survive. The parser and serializer risk rider requires the raw
subtrees to remain byte-identical, so the assertion does not protect the stated
contract.

## Nitpicks

None.

## Not found

Panics: the empty-record guard makes the new indexing and subtraction safe, and
no untrusted-input unwrap or unchecked arithmetic was found in the new paths.

Schema order and fixed write prefixes: no additional finding beyond the
document-wide identity defect. The inserted paragraph section properties use
the existing ordered serializers, and the final body section properties remain
schema-final.

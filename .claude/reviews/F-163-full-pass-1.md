# F-163, full, pass 1

**Reviewed**: working tree against `9cefee4`, 10 files, 770 changed lines
**Verdict**: 0 defects, 1 smell, 0 nitpicks

## Defects

None.

## Smells

### S1, template-specific table discovery has no facade regression

`crates/rdocx/src/document.rs:5754`

The scalar source-coverage test exercises the body, headers, footers, a text
box, and chart values, but it does not place a scalar tag in a table. F-163 adds
new recursive table source discovery before delegating to the existing mapper.
A regression in that discovery can therefore leave table tags unresolved while
the complete F-163 test set remains green. Add one table-cell tag to this test
and include it in the expected count.

## Nitpicks

None.

## Not found

Correctness, contract scope, panic paths, byte-safe indexing, OOXML schema
order, namespace handling, unmodelled subtree preservation, dependency
direction, public API scope, and structural indirection produced no findings.

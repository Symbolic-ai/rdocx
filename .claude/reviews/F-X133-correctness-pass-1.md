# F-X133, correctness, pass 1

**Reviewed**: working diff, 4 files, 104 changed lines
**Verdict**: 1 defect, 1 smell, 0 nitpicks

## Defects

### D1, plan describes a plain save but the gate rewrites the part
`.claude/plans/F-X133-design.md:28`

The test at `crates/rdocx/tests/regression_test.rs:17847` calls `try_replace_text` before serialization. A plain save may retain the original package bytes, so the plan claims a path the gate does not exercise.

## Smells

### S1, shadow case checks a foreign subtree instead of retained typed attributes
`crates/rdocx/tests/regression_test.rs:17869`

The local `xmlns:w="urn:producer"` changes `w:r` into a foreign element. Preserving that raw subtree is useful, but it does not test how `push_root_attribute_record` handles a shadow on a modeled paragraph or run. The test should use a canonical element with a shadowed attribute prefix if this assertion is intended as the retained-attribute guard.

## Nitpicks

None.

## Not found

No panics, schema-order regressions, unmodelled-child loss, or new structural indirection in this diff. The named gate fails when the existing canonical `w` skip is removed.

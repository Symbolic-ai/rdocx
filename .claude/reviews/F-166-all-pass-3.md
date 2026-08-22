# F-166, all, pass 3

**Reviewed**: Uncommitted working diff, 4 files and 1,328 changed lines, with
1,285 additions and 43 deletions
**Verdict**: 3 defects, 0 smells, 0 nitpicks

## Defects

### D1, Bookmark references in preserved body XML are not remapped

`crates/rdocx/src/field.rs:607`

`crates/rdocx/src/field.rs:929`

`crates/rdocx/src/field.rs:968`

Identity discovery scans the complete serialized body, including preserved raw
subtrees, but reference reservation and remapping traverse only typed content.
Both top-level and content-control `RawXml` are skipped. A bookmark inside a
preserved `w:customXml` subtree is therefore renamed in the second record while
its raw `REF`, `PAGEREF`, or hyperlink target retains the old name. The
reference becomes unresolved. An intentionally unresolved raw target such as
`MailMerge1` can also be captured by a generated bookmark name. Pass-2 D1 is
only resolved for references represented by the typed projection.

### D2, Escaped bookmark names do not share the remap key used by typed references

`crates/rdocx/src/field.rs:672`

`crates/rdocx/src/field.rs:696`

`crates/rdocx/src/field.rs:1006`

The identity scanner stores raw attribute bytes without XML entity decoding.
For a valid bookmark name serialized as `A&amp;B`, the remap key is
`A&amp;B`, while the typed `REF` or `PAGEREF` target is `A&B`. The bookmark
is renamed in later records, but the reference lookup misses and leaves the old
target behind. The new raw merge-field scanner decodes text and attribute
entities correctly, but body identity remapping does not.

### D3, New scanners select attributes by local name only

`crates/rdocx/src/field.rs:451`

`crates/rdocx/src/field.rs:696`

`crates/rdocx/src/field.rs:855`

Element namespaces are resolved, but attributes are selected solely by their
local name. An ignorable foreign attribute such as `x:instr` placed before the
real `w:instr` on `w:fldSimple` is treated as the field instruction. Sectioned
merge can consequently reject a stable story or accept a record-varying one.
The same issue affects identity collection and editing. A foreign `x:id` or
`x:name` placed before the WordprocessingML attribute is collected and edited,
leaving the real bookmark, content-control, or drawing identity unchanged and
duplicated across records.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-2 D2 is resolved. The raw story scanner uses namespace-aware event
resolution, so field prefixes declared on intermediate tables or controls stay
in scope.

Pass-2 D3 is resolved for the approved merge path. Clean, relationship-resolved
footnote parts remain source-backed, field edits patch the original bytes, and
the regression test proves that an unmodelled table and a nonconventional part
target survive.

Pass-2 D4 is resolved. Ordinary `evaluate_fields` and `update_fields` again use
their original typed header and footer traversal, while nested raw dependency
scanning is confined to sectioned mail merge.

Panics: none found. Empty records are rejected before candidate indexing, ID
allocation is checked, and the new XML scanners use bounded parsing.

OOXML child order and fixed output prefixes: no additional finding beyond D1
through D3. Section properties remain schema-final and the raw story scanner
does not reserialize producer content.

Tests: no independent test-harness defect found. `cargo test -p rdocx --test
regression_test` passed all 89 tests. The uncovered raw-reference, escaped-name,
and foreign-attribute cases are the triggers documented in D1 through D3.

Structure and scope: no finding. Although the diff is 1,328 lines, the added
helpers are concrete and local to the two mail-merge operations. No new trait,
generic, module, forwarding wrapper, feature flag, or speculative abstraction
was introduced.

Verification also passed `cargo check -p rdocx --all-targets`, `git diff
--check`, and `python3 scripts/prose_check.py`.

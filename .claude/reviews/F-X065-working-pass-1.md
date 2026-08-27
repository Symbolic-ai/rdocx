# F-X065, working, pass 1

**Reviewed**: working diff against canonical claim Base
`d55e85b8a5e8f3d47f87ecae3cc29a4a7062dbf4`, 13 files, 297 insertions and
26 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, preserved grid XML loses ancestor-only namespace bindings

`crates/rdocx-oxml/src/table.rs:717`

The new grid parser stores `tblGridChange` and foreign direct children with
`capture_element` or `capture_empty_element` alone at lines 717, 719, 733, and
735. Those captures contain only the selected element, so a prefix declared on
an ancestor `tbl` or `tblGrid` is absent from the stored bytes. Serialization
then places the raw bytes below canonical `w:tbl` and `w:tblGrid` owners at
lines 781 through 785. A table-local Word alias such as the tested `q` binding,
or a table-local foreign `ext` binding, therefore becomes unbound after save
and reopen unless the same prefix happens to be declared independently on the
document root. The existing namespace test checks the captured byte slices at
`crates/rdocx-oxml/src/table.rs:1959` and
`crates/rdocx-oxml/src/table.rs:1965`, but does not serialize and reparse those
ancestor-bound alias and foreign cases. This violates the required exact raw
preservation and package round-trip behavior. The table parser already uses
the repository's owner-binding machinery for other raw table subtrees, so the
grid owner and child captures need equivalent binding retention plus a save and
reopen regression.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness beyond D1: canonical modeled columns and one historical change
  follow the approved projection, and a second modeled change fails closed.
- Contract: the additive `TableRef::has_grid_change()` surface reports only
  presence, and the exact PR 56 source SHA remains recorded in the plan and
  progress evidence without external mutation.
- Panics and errors: no new untrusted-input indexing, slicing, unchecked
  arithmetic, `unwrap`, or `expect` occurs in production code.
- OOXML beyond D1: modeled active columns serialize before the historical
  change, foreign same-local names do not enter the modeled projection, and
  Word attributes are selected through the current namespace scope.
- Layout: the historical bytes have no layout consumer, and the focused test
  proves active widths remain authoritative.
- Public compatibility: the additive facade method is documented, every
  repository literal was updated, and the intentional pre-1.0 exhaustive
  `CT_TblGrid` literal impact is stated in the binding specification.
- Tests beyond D1: duplicate rejection, schema order, public facade package
  save and reopen, active-layout isolation, deterministic fonts, the pinned
  corpus, package dry runs, and the unchanged hash gate are represented in the
  implementation evidence.
- HLD scope: exactly the five plan-listed HLD files changed, and they describe
  current behavior rather than feature history.
- Structure: no new module, test binary, trait, generic parameter, wrapper,
  dependency, or forwarding-only API was introduced.

# F-203, all, pass 3

**Reviewed**: the complete uncommitted working diff against `HEAD`, 4 files,
194 additions and 38 deletions. The review rechecked both pass-2 defects, the
revised plan, its cited HLD contract, all six planned regressions, and the
changed-crate all-targets check.
**Verdict**: 1 defect, 2 smells, 0 nitpicks

## Defects

### D1, the cell-property writer encodes the wrong OOXML child order
`crates/rdocx-oxml/src/table.rs:1143`

`CT_TcPr` requires `w:textDirection`, then `w:tcFitText`, then `w:vAlign`, but
the writer emits `w:vAlign` before `w:textDirection`. The remediated boundary
table encodes the same reversed order. A schema-valid input containing
`w:textDirection`, a foreign same-local-name child, and then `w:vAlign` is
therefore rewritten as `w:vAlign`, `w:textDirection`, and the foreign child.
This both violates `xsd:sequence` and fails the plan's same-slot preservation
contract. The new regression at `crates/rdocx-oxml/src/table.rs:2072` uses the
already reversed order and consequently asserts an invalid sequence instead
of detecting the defect.

## Smells

### S1, the remediation leaves forwarding-only parser wrappers
`crates/rdocx-oxml/src/table.rs:969`
`crates/rdocx-oxml/src/table.rs:1265`

Both existing prefix-aware methods now do nothing except forward to the new
owner-binding variants with an empty slice, and each has only its public
wrapper as a caller. This is the forwarding-only indirection prohibited by the
repository structural rules. The public wrappers can call the concrete
owner-binding parsers directly.

### S2, the content-control cell path has no regression coverage
`crates/rdocx-oxml/src/content_control.rs:571`

The diff adds a second cell-owner binding propagation path for a `w:tc` inside
`w:sdtContent`, but none of the six F-203 tests exercises it. Reverting only
this hunk leaves every planned test green while again producing an unbound
producer prefix for a content-control-wrapped cell. The regression gate does
not protect all behavior introduced by the remediation.

## Nitpicks

None.

## Not found

Pass-2 D1 is fixed for the direct-row and content-control parser paths, and
pass-2 D2 is fixed by namespace-aware boundary classification. Panics,
arithmetic hazards, typed attribute namespace recognition, owner-local binding
injection, numbering boundary 5 ordering, HLD/API accounting, and additional
structural issues produced no findings.

## Checks

- `cargo test -p rdocx-oxml foreign_cell_width_remains_raw_and_unmodelled`, passed.
- `cargo test -p rdocx-oxml aliased_cell_width_uses_in_scope_word_bindings`, passed.
- `cargo test -p rdocx-oxml cell_property_preserves_child_binding_declared_on_owner`, passed.
- `cargo test -p rdocx-oxml cell_property_preserves_child_binding_declared_on_cell`, passed.
- `cargo test -p rdocx-oxml foreign_same_name_after_later_property_keeps_current_boundary`, passed.
- `cargo test -p rdocx-oxml level_raw_is_lgl_stays_before_suffix`, passed.
- `cargo check -p rdocx-oxml --all-targets`, passed.

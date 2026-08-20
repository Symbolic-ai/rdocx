# F-203, all, pass 4

**Reviewed**: the complete uncommitted working diff against `HEAD`, 4 files,
232 additions and 45 deletions. The review rechecked every pass-3 finding, the
revised plan and cited HLD contract, all seven planned regressions, formatting,
and the changed-crate all-targets check.
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, unmodelled Word cell properties do not receive absolute schema slots
`crates/rdocx-oxml/src/table.rs:1192`

The boundary table still falls back to the current source boundary for standard
WordprocessingML children other than `w:tcFitText`. Consequently `w:hMerge`,
`w:tcMar`, `w:hideMark`, `w:headers`, `w:cellIns`, `w:cellDel`, `w:cellMerge`,
and `w:tcPrChange` are not attached to their known `CT_TcPr` schema positions.
For example, parsing a cell with only preserved `w:tcMar` stores it at boundary
0. If a caller then sets typed `grid_span`, serialization emits `w:tcMar`
before `w:gridSpan`, although the schema requires `w:tcMar` after `w:noWrap`.
This violates both `xsd:sequence` and the plan's schema-slot sidecar contract.
The current schema-order regression covers only the specially mapped
`w:tcFitText`, so the mutation case remains unprotected.

## Smells

None.

## Nitpicks

None.

## Not found

Pass-3 D1 is fixed by emitting `w:textDirection`, preserved `w:tcFitText`, and
`w:vAlign` in schema order. Pass-3 S1 is fixed by removing the two
forwarding-only parser layers. Pass-3 S2 is fixed by the content-control cell
binding regression. Namespace identity, owner and cell binding propagation,
panics, arithmetic hazards, numbering boundary 5 ordering, HLD/API accounting,
and additional structural issues produced no findings.

## Checks

- `cargo test -p rdocx-oxml foreign_cell_width_remains_raw_and_unmodelled`, passed.
- `cargo test -p rdocx-oxml aliased_cell_width_uses_in_scope_word_bindings`, passed.
- `cargo test -p rdocx-oxml cell_property_preserves_child_binding_declared_on_owner`, passed.
- `cargo test -p rdocx-oxml cell_property_preserves_child_binding_declared_on_cell`, passed.
- `cargo test -p rdocx-oxml foreign_same_name_after_later_property_keeps_current_boundary`, passed.
- `cargo test -p rdocx-oxml content_control_cell_preserves_child_binding_declared_on_cell`, passed.
- `cargo test -p rdocx-oxml level_raw_is_lgl_stays_before_suffix`, passed.
- `cargo check -p rdocx-oxml --all-targets`, passed.
- `cargo fmt --all --check`, passed.
- `git diff --check`, passed.

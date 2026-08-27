# F-X065, working, pass 2

**Reviewed**: complete working diff against canonical claim Base
`d55e85b8a5e8f3d47f87ecae3cc29a4a7062dbf4`, 14 tracked modified files,
378 insertions and 28 deletions, plus the pass-1 review record
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass-1 D1: historical changes and foreign direct grid children pass their
  complete in-scope bindings to the established raw normalizer at
  `crates/rdocx-oxml/src/table.rs:727` and
  `crates/rdocx-oxml/src/table.rs:751`. Foreign same-local `tblGrid` elements
  receive the same treatment at `crates/rdocx-oxml/src/table.rs:1759` and
  `crates/rdocx-oxml/src/table.rs:1781`. Required ancestor-only aliases are
  therefore copied onto each preserved subtree before its owner is
  canonicalized.
- Canonical preservation: `preserved_table_raw_bindings` excludes only the
  canonical `w` to WordprocessingML binding at
  `crates/rdocx-oxml/src/table.rs:67`. The package root already owns that
  binding, so ordinary canonical historical bytes remain unchanged. Foreign,
  aliased Word, default-namespace, and shadowed bindings remain available to
  the raw normalizer.
- Self-contained serialization and repeated round trip: the remediation
  regression declares `q` and `ext` only on ancestors at
  `crates/rdocx-oxml/src/table.rs:2022`, then requires both declarations in the
  serialized result, reparses the modeled active grid and historical change,
  retains both foreign grid projections, and compares the complete second
  serialization byte for byte at `crates/rdocx-oxml/src/table.rs:2034`.
- Namespace contract: active `tblGrid`, `gridCol`, width attributes, and
  `tblGridChange` remain selected through the current WordprocessingML scope.
  Foreign same-local elements stay outside the modeled projection, including
  self-closing and nonempty foreign grids.
- Duplicate and schema-order behavior: the second modeled historical change
  still returns an explicit error at
  `crates/rdocx-oxml/src/table.rs:2050`. The writer continues to emit all
  active columns before the preserved history.
- Layout isolation: layout reads only `CT_TblGrid::columns`. The conflicting
  historical-width regression remains exact at
  `crates/rdocx-layout/src/table.rs:895`.
- Public compatibility: `TableRef::has_grid_change()` remains the only new
  facade method, and all repository `CT_TblGrid` literals initialize the
  intentional pre-1.0 preservation fields. The helper visibility change in
  `crates/rdocx-oxml/src/numbering.rs:587` is crate-private.
- Panics and errors: no new production `unwrap`, `expect`, unchecked index,
  slice, or arithmetic path was introduced. XML capture and binding
  normalization propagate errors through the existing `Result` contract.
- Package and tests: canonical facade save and reopen still compares the exact
  historical bytes at `crates/rdocx/src/table.rs:818`. The remediated OXML
  suite, deterministic layout suite, unchanged 49-entry hash gate, and patched
  public package dry runs are recorded green.
- HLD and attribution: exactly the five plan-listed HLD files describe the
  current parser, package, facade, and layout contracts. PR 56 and exact source
  SHA `8b79c4cd0452defafe0a58e86b332c98e7fe52d7` remain recorded without
  external mutation.
- Structure: no new file, module, dependency, trait, generic, public type, or
  forwarding-only wrapper was introduced.

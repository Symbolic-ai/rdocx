# F-055, all aspects, pass 1

**Reviewed**: uncommitted worker diff, 6 files, 1294 additions and 42 deletions
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, explicit empty transform pairs are not modelled

`crates/oxml-drawing/src/color.rs:447`

Every known transform received as an XML start event is stored as raw XML.
That preserves unexpected nested content, but it also treats a schema-valid
explicit empty pair such as `<a:tint val="65000"></a:tint>` as unmodelled.
The transform is then absent from `transforms()` and from colour resolution.
Recognise an immediate matching end event as the known transform while keeping
any nonempty known element raw and byte-preserved.

## Smells

None.

## Nitpicks

None.

## Not found

- Contract: the implementation covers all 28 transforms, ordered resolution,
  the approved static oracle table, and the exact HLD impact file.
- Panics: production parsing and resolution add no input-triggered panic path.
- OOXML: local-name input and fixed-prefix output are correct, raw unknown
  siblings retain their slots, and the remaining empty-pair defect is cited
  above.
- Tests: the 40-row PowerPoint table is independent of the Rust formulas and
  focused tests cover order, alpha, gamma conversion, and raw children.
- Structure: the diff adds no trait, generic parameter, wrapper, crate, module,
  file, or feature flag. The OPC dependency remains development-only.

# F-246, all, pass 6

**Reviewed**: verified working-tree diff excluding review records, 20 files, 3,110 changed lines
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, canonical style scalars gain redundant namespace declarations
`crates/rdocx-oxml/src/styles.rs:984`

Every captured scalar is made self-contained against all inherited bindings,
including the canonical `w` and `r` bindings that the style-part writer always
provides at its root. Reopening and saving a fresh document therefore adds a
local `xmlns:w` declaration to each scalar and changes package bytes. This
breaks repeated-save determinism, same-class signature preservation, and the
public authoring conformance gate. Self-contained capture must omit bindings
that the fixed style writer guarantees while retaining aliases and default or
producer namespaces that are not guaranteed.

## Smells

None.

## Nitpicks

None.

## Not found

The five earlier microscope remediations remain correct. The formal suite
failure is isolated to redundant canonical namespace materialization. Style
semantics, graph validation, update preservation, layout resolution, and the
49-entry hash baseline produced no additional findings.

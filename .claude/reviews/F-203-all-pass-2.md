# F-203, all, pass 2

**Reviewed**: the complete uncommitted working diff against `HEAD`, 3 files,
83 additions and 19 deletions. The review also followed the revised plan into
the cited HLD sections and the existing table-property and numbering paths that
the diff relies on.
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, bindings inherited from the cell are not retained
`crates/rdocx-oxml/src/table.rs:1280`

The new path passes only namespace declarations local to `w:tcPr` into
`raw_with_external_bindings`. A producer cell such as
`<w:tc xmlns:ext="urn:producer"><w:tcPr><ext:property/></w:tcPr>...</w:tc>`
therefore gives the preserved child an empty owner-binding list. Serialization
rewrites both `w:tc` and `w:tcPr` without the cell's declaration, then emits the
raw `<ext:property/>`, leaving `ext` unbound. The sidecar does not preserve the
producer subtree as namespace-complete XML when its binding is inherited from
the immediately enclosing cell.

### D2, foreign same-local-name children still change schema boundaries
`crates/rdocx-oxml/src/table.rs:1177`

`tc_pr_raw_boundary` still classifies names by local name alone. For example,
a validly preserved `<ext:tcW>` that originally follows `<w:vAlign>` is assigned
boundary 1 and also regresses the current boundary from 8 to 2. The writer then
moves that foreign child ahead of the typed properties instead of emitting it
at its original schema slot. The existing regression places `ext:tcW` at
boundary 1, so it cannot expose this collision. Namespace identity must guard
schema-boundary advancement as well as typed projection.

## Smells

None.

## Nitpicks

None.

## Not found

Panics, arithmetic hazards, typed attribute namespace recognition, owner-local
binding injection, numbering boundary 5 ordering, HLD/API accounting, and
structural indirection produced no additional findings.

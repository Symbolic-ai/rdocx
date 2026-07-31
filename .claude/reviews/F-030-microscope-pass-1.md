# F-030, all, pass 1

**Reviewed**: working tree against `fa89cbb`, 4 files, 995 added lines
**Verdict**: 2 defects, 0 smells, 0 nitpicks

## Defects

### D1, copied tests do not use deterministic font mode
`crates/oxml-layout/src/line.rs:786`

The copied tests construct `FontManager::new()`, which discovers host system
fonts when the default feature is enabled. The same constructor is used again
at lines 795, 804, 818, and 939. This violates the plan's explicit requirement
to run every copied line test in deterministic font mode. These tests must use
the deterministic constructor so their font environment is identical on every
machine.

### D2, the tab regression never exercises leader shaping
`crates/oxml-layout/src/line.rs:983`

`tab_stops_use_point_positions_and_owned_leaders` calls only
`resolve_tab_width` and asserts the selected leader character. It never reaches
`inline_to_line_item` or `shape_leader`, so it does not prove the test plan's
required leader glyph behavior. A regression that drops the shaped leader,
uses the wrong font, or produces no glyphs would leave this test green.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: the owned type substitutions, explicit spacing modes, and wrap
  guard match the approved contract. Forced line, page, and column breaks
  remain effective with wrapping disabled.
- Contract: no staged type retains twips or the stringly `line_rule` field.
  Released `rdocx-layout` source, its manifest, and its consumers are unchanged.
- Panics: no new production panic path was found in the F-030 substitutions.
  The copied indexing paths retain their existing font-shaping invariants.
- OOXML: not applicable. The story adds no parser, serializer, namespace, raw
  subtree, or schema-order behavior.
- Structure: the new concrete types and `line.rs` module are explicitly
  authorized. No trait, generic parameter, forwarding wrapper, feature flag,
  or extra dependency beyond `unicode-linebreak` was introduced.
- Dependency isolation: the staged module contains no `rdocx-*` or `rpptx*`
  reference, and its only new manifest dependency is the approved
  `unicode-linebreak`.

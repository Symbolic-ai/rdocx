# F-030, all, pass 2

**Reviewed**: working tree against `fa89cbb`, 4 files, 1,003 added lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Pass 1 D1 is resolved. The shared test helper at
  `crates/oxml-layout/src/line.rs:784` constructs the deterministic bundled-font
  manager, and every copied line-breaking test uses it.
- Pass 1 D2 is resolved. The tab regression at
  `crates/oxml-layout/src/line.rs:971` resolves a bundled font, exercises
  `inline_to_line_item`, and asserts a non-empty shaped dot leader with the
  expected point width.
- Correctness: the owned type substitutions, explicit spacing modes, and wrap
  guard match the approved contract. Forced line, page, and column breaks
  remain effective with wrapping disabled.
- Contract: the staged API contains the approved `TabStop`, `Align`, `TabAlign`,
  `TabLeader`, `Underline`, and `LineSpacing` types. No staged type retains
  twips or the stringly `line_rule` field. Released `rdocx-layout` source, its
  manifest, and its consumers remain unchanged.
- Panics: no new production panic path was found in the F-030 substitutions.
  The copied indexing paths retain their existing font-shaping invariants.
- OOXML: not applicable. The story adds no parser, serializer, namespace, raw
  subtree, or schema-order behavior.
- Tests: the 11 copied cases use the owned types. The added tests cover every
  spacing mode, compatibility-default wrapping, all three explicit break
  variants with wrapping disabled, point-valued tab stops, and actual leader
  shaping in deterministic font mode.
- Structure: the concrete types and `line.rs` module are explicitly authorized.
  No trait, generic parameter, forwarding wrapper, feature flag, or extra
  dependency beyond `unicode-linebreak` was introduced.
- Dependency isolation: the staged module contains no `rdocx-*` or `rpptx*`
  reference, and its only new manifest dependency is the approved
  `unicode-linebreak`.

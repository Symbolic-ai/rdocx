# S20 sprint review, pass 1

**Reviewed**: `sprint/s20` against `31a0249d50a767f43c99eb53af0436143825d56d`, 34 files, 3,397 changed lines, crates: `oxml-drawing`, `rpptx-oxml`, `rpptx-layout`
**Verdict**: 1 blocking, 0 should-fix, 0 nice-to-have

## Blocking

### B1, opaque effect detection ignores XML namespaces

`crates/oxml-drawing/src/effect.rs:398`

The raw effect scanners identify `schemeClr` and `effectDag` by local name
alone. A preserved extension such as `x:effectDag` is therefore treated as the
DrawingML choice and suppresses a referenced modelled effect. Likewise, an
extension `x:schemeClr val="phClr"` is reported as an unresolved DrawingML
placeholder colour. This conflicts with the sprint's preservation contract and
can reject or change a valid shape merely because a producer used the same
local name in another namespace. The fix must require the DrawingML namespace
for both names and add regressions for foreign-namespace `effectDag` and
`schemeClr` elements.

## Should-fix

None.

## Nice-to-have

None.

## Milestone gate

The M9 end gate at `docs/hld/14-development-backlog.md:653` is: "the contract
is frozen and published to the render track."

S20 does not complete M9, so this gate is not yet due and is not claimed. The
S20 slice gates were exercised by the named placeholder, transform, seven-step
text, format-scheme, and typeface tests. The full workspace gate, normal-stack
50-deck structural corpus test, exact 40-case colour table, and all 28 unchanged
hashes also passed before sprint review. F-086 through F-088 remain pending and
own the flattener, frozen `ResolvedSlide`, and final differential evidence.

## Not found

- Interaction: apart from B1, the placeholder chain, body cascade, text cache,
  format overlay, and typeface lookup compose without conflicting precedence.
- Duplication: each resolver concern has one implementation and no duplicate
  placeholder, merge, matrix, substitution, or font helper was added.
- Layering: `rpptx-layout` depends downward on `rpptx-oxml` and
  `oxml-drawing`. No `oxml-*` crate gained an `rpptx-*` dependency.
- Harness: every S20 AS_BUILT entry records the observed unchanged 28-entry
  result.
- Gate: every S20 story has a focused non-vacuous test, and the integrated risk
  riders passed.
- Docs: the approved HLD impact files describe the typed parser surfaces,
  inheritance precedence, cache boundary, format references, and font tokens.
- Deps: the new unpublished `rpptx-layout` crate has only its two named model
  consumers and adds no third-party dependency.
- Surface: the public resolver, effective-value, typed shape-property, and
  typed shape-style APIs are required by the approved F-081 through F-085
  contracts. All PowerPoint crates remain `0.0.0` and unpublished.

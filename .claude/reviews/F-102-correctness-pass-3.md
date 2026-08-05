# F-102, correctness, pass 3

**Reviewed**: fully remediated uncommitted F-102 working diff, 11 files, 2,958 changed lines
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- `correctness`: table style precedence, explicit text overrides, option-gated
  corners, right-to-left unequal widths, merge ownership, far-edge border
  sourcing, and physical border conflict ordering have distinguishing tests.
- `contract`: the parser, resolver, renderer, diagnostics, tests, plan, and five
  HLD updates match the approved F-102 scope.
- `panics`: production parsing and lowering guard dimensions and spans, use
  bounded lookups for covered merge cells, and return typed errors for malformed
  XML or concrete values.
- `ooxml`: alternate-prefix reads, fixed-prefix writes, schema-order guards,
  exact raw-subtree preservation, producer wrappers, and the pinned table-style
  corpus round-trip are covered.
- `tests`: the six named plan gates are present. The base implementation lacked
  `CT_TableStyleList` and dropped `ResolvedContent::Table`, so the parser,
  resolver, and renderer gates fail when reverted.
- `structure`: no new source file, module, trait, generic parameter, crate,
  feature flag, or forwarding wrapper was added. The private cell-position value
  replaces repeated argument groups across three cascade functions.
- `pass-1`: all four defects and the alias smell have distinguishing green
  regressions.
- `pass-2`: corners now require both matching option flags, and unsupported cell
  and border fills record a stable deduplicated diagnostic.

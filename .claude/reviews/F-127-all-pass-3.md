# F-127, all, pass 3

**Reviewed**: final remediated working diff from claim base `b1a4abd`, 4
files, 725 changed lines, with 619 additions and 106 deletions
**Verdict**: 0 defects, 0 smells, 0 nitpicks

## Defects

None.

## Smells

None.

## Nitpicks

None.

## Not found

- Correctness: pass-2 D1 is fixed. Direct scheme colours select the mapped
  theme slot, apply that slot's transforms first, then apply the series
  transforms. The combined path preserves alpha transform order.
- Contract: direct fill and line precedence, transparent `a:noFill`, the
  mapped six-accent cycle, geometry colours, marker colours, and legend
  swatches match the approved design.
- Panics: no new reachable panic, unchecked slice, or untrusted arithmetic
  issue was found.
- OOXML: no schema-order, namespace, whitespace, or raw-preservation issue was
  found.
- Tests: all 80 `rpptx-chart` unit tests passed. The exact-colour test covers a
  mapped accent with theme transforms followed by series transforms and
  series alpha. The direct-colour regression covers unrelated theme slots,
  transparent area fill, and partial-alpha area fill.
- Structure: no unjustified trait, generic, wrapper, module, file, or dynamic
  dispatch was introduced.

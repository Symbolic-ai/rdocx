# F-060, all aspects, pass 1

**Reviewed**: the 3-file working diff against
`35f6b4229173a89e4555ee002f1fe31cc9b63020`, 1,400 lines across the design
contract, module export, and fill implementation
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, pattern wrapper raw children are dropped when no known colour parses

`crates/oxml-drawing/src/fill.rs:561`

The parser records unknown children inside `a:fgClr` and `a:bgClr` even when
there is no recognised colour choice, but the writer emits each wrapper only
when its recognised colour is present. An input such as
`<a:fgClr><x:extension/></a:fgClr>` therefore loses the entire preserved
subtree. This contradicts the approved byte-for-byte nested-extension
preservation contract.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in contract scope, panic safety, schema ordering,
namespace handling, gradient stop order, relationship isolation, tests, or
structure.

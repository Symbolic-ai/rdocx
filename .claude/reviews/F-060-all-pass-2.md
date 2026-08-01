# F-060, all aspects, pass 2

**Reviewed**: the post-remediation 4-file working diff against
`35f6b4229173a89e4555ee002f1fe31cc9b63020`, including the pass 1 review and
the extension-only pattern regression
**Verdict**: 1 defect, 0 smells, 0 nitpicks

## Defects

### D1, nested extensions in modelled leaf elements are discarded

`crates/oxml-drawing/src/fill.rs:369`

The parser consumes an explicit nonempty `a:lin` element with
`capture_element` and then discards the captured bytes. The same pattern is
used for `a:tileRect`, `a:srcRect`, `a:fillToRect`, and `a:fillRect`. An unknown
extension nested in one of these modelled leaf elements therefore disappears
on write. The approved contract requires nested extensions to survive byte for
byte at their schema boundary.

## Smells

None.

## Nitpicks

None.

## Not found

No additional findings in contract scope, panic safety, schema ordering,
namespace handling, gradient stop order, relationship isolation, tests, or
structure.
